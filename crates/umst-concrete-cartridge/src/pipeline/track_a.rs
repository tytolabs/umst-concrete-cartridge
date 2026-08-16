// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Track A coordinate-descent optimiser: dual-gated mix proposal (printability AND thermodynamic).
//!
//! Search is iterative (bisection), but each step is a pure score + gate evaluation composed with
//! [`crate::pipeline::dual_gate::evaluate_dual_gate`]. Thermodynamic leg stays on R1
//! (`umst.gate.cd_transition`); printability is literature surrogate below R1.

use burn::tensor::{backend::Backend, Data, Shape, Tensor};
use burn_ndarray::NdArrayDevice;
use serde::Serialize;

use crate::calibration::Profile;
use crate::calibration_fit::calibrated_tau0_pa;
use crate::facade::{
    predict_with_options, FacadeBackend, MixSpec, PredictOptions, WaterCementRatio,
};
use crate::mix_layout;
use crate::physics::printability::PrintabilityEngine;
use crate::pipeline::dual_gate::{evaluate_dual_gate, CastGateVerdict};
use crate::pipeline::physical_summary::nominal_mix_tensor_for_mix_spec;
use crate::pipeline::{
    run_full_physics_pipeline, PhysicsPipelineSummary, PRINTABLE_TAU_HI, PRINTABLE_TAU_LO,
};

/// Serializable next-mix proposal for the experiment loop (`proposed_next_mix.json`).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Wire envelope for Track A CLI; physics claims live on nested gate fields.
#[derive(Debug, Clone, Serialize)]
pub struct ProposedNextMix {
    pub schema_version: &'static str,
    pub calibration_profile: String,
    pub base_mix: MixSpecWireOut,
    pub proposed_mix: MixSpecWireOut,
    pub dual_gate: DualGateWire,
    pub objective: String,
    pub steps: usize,
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Mix JSON mirror without newtype wrappers for serde output.
#[derive(Debug, Clone, Serialize)]
pub struct MixSpecWireOut {
    pub w_c: f64,
    pub temperature_k: f64,
    pub superplasticiser_pct: f64,
    pub silica_fume_pct: f64,
    pub fly_ash_pct: f64,
    pub aggregate_volume_fraction: f64,
    pub target_age_hours: f64,
    pub profile_name: String,
}

/// v1 JSON wire block for `proposed_next_mix.v1` sidecar.
///
/// Bool fields (`printability_ok`, `thermodynamic_ok`, `passes`) are **wire-compat
/// keys** for operator scripts and CLI contract tests — not MP3.6 Rust shim debt
/// (bool shims closed @ MP3.6; values from [`CastGateVerdict`] leg-pass helpers).
/// Prefer [`Self::is_printability_ok`], [`Self::is_thermodynamic_ok`], [`Self::is_admissible`].
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Dual-gate audit block for proposed mix JSON sidecar.
#[allow(missing_docs)] // Legacy bool mirrors — prefer accessor predicates (P26).
#[derive(Debug, Clone, Serialize)]
pub struct DualGateWire {
    /// Legacy v1 wire mirror — prefer [`Self::is_printability_ok`].
    #[deprecated(
        since = "0.2.0",
        note = "use DualGateWire::is_printability_ok() — v1 JSON key unchanged"
    )]
    pub printability_ok: bool,
    /// Legacy v1 wire mirror — prefer [`Self::is_thermodynamic_ok`].
    #[deprecated(
        since = "0.2.0",
        note = "use DualGateWire::is_thermodynamic_ok() — v1 JSON key unchanged"
    )]
    pub thermodynamic_ok: bool,
    /// Legacy v1 wire mirror — prefer [`Self::is_admissible`].
    #[deprecated(
        since = "0.2.0",
        note = "use DualGateWire::is_admissible() — v1 JSON key `passes` unchanged"
    )]
    pub passes: bool,
    pub yield_stress_pa: f64,
    pub printability_extrudability: f64,
}

impl DualGateWire {
    /// Printability leg pass — mirrors [`CastGateVerdict::printability_leg_pass`].
    #[must_use]
    pub fn is_printability_ok(&self) -> bool {
        #[allow(deprecated)]
        {
            self.printability_ok
        }
    }

    /// Thermodynamic leg pass — mirrors [`CastGateVerdict::thermodynamic_leg_pass`].
    #[must_use]
    pub fn is_thermodynamic_ok(&self) -> bool {
        #[allow(deprecated)]
        {
            self.thermodynamic_ok
        }
    }

    /// Composite admissibility — mirrors [`CastGateVerdict::is_admissible`].
    #[must_use]
    pub fn is_admissible(&self) -> bool {
        #[allow(deprecated)]
        {
            self.passes
        }
    }

    /// Build v1 wire-stable bool block from [`CastGateVerdict`] leg-pass helpers.
    ///
    /// Wire-compat only: JSON keys are frozen for `proposed_next_mix.v1`; intentional
    /// serde surface, not residual MP3.6 bool-shim debt.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: SSOT for dual-gate sidecar bools from enum algebra.
    #[must_use]
    #[allow(deprecated)]
    pub fn from_verdict(verdict: &CastGateVerdict, summary: &PhysicsPipelineSummary) -> Self {
        Self {
            printability_ok: verdict.printability_leg_pass(),
            thermodynamic_ok: verdict.thermodynamic_leg_pass(),
            passes: verdict.is_admissible(),
            yield_stress_pa: f64::from(summary.rheology_yield_stress_pa),
            printability_extrudability: f64::from(summary.printability_extrudability),
        }
    }
}

impl From<&MixSpec> for MixSpecWireOut {
    fn from(s: &MixSpec) -> Self {
        Self {
            w_c: f64::from(s.w_c.value()),
            temperature_k: f64::from(s.temperature_k.value()),
            superplasticiser_pct: f64::from(s.superplasticiser_pct),
            silica_fume_pct: f64::from(s.silica_fume_pct),
            fly_ash_pct: f64::from(s.fly_ash_pct),
            aggregate_volume_fraction: f64::from(s.aggregate_volume_fraction),
            target_age_hours: f64::from(s.target_age_hours),
            profile_name: s.profile_name.clone(),
        }
    }
}

/// Effective τ₀ after optional profile θ calibration.
/// formal_anchor: empirical://datasets/printability-rheology-yield-proxy.v1.csv
/// formal_status: Empirical
/// formal_dataset: "Tyto mortar rheology θ bias (calibration_fit)"
/// formal_citation: "In-house Tyto mortar yield proxy calibration"
/// formal_envelope: "tests/calibration_tyto_mortar.rs"
#[must_use]
pub fn calibrated_yield_stress_pa(profile: &Profile, summary: &PhysicsPipelineSummary) -> f32 {
    calibrated_tau0_pa(
        summary.rheology_yield_stress_pa,
        profile.rheology_calibration.as_ref(),
    )
}

/// Pipeline summary with calibrated τ₀ for gate evaluation.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Applies θ bias before dual-gate printability leg.
#[must_use]
pub fn summary_with_calibrated_tau(
    profile: &Profile,
    summary: &PhysicsPipelineSummary,
) -> PhysicsPipelineSummary {
    let tau = calibrated_yield_stress_pa(profile, summary);
    PhysicsPipelineSummary {
        rheology_yield_stress_pa: tau,
        printability_extrudability: extrudability_from_tau_pa(tau),
        printability_buildability: buildability_from_tau_pa(tau),
        ..summary.clone()
    }
}

fn t4_scalar(v: f32, device: &<FacadeBackend as Backend>::Device) -> Tensor<FacadeBackend, 4> {
    mix_layout::collapsed_rank4_from_rank2_scalar(
        Tensor::from_data(Data::new(vec![v], Shape::new([1, 1])), device),
        device,
    )
}

fn min_f32_rank4(t: Tensor<FacadeBackend, 4>) -> f32 {
    t.into_data()
        .value
        .into_iter()
        .fold(f32::INFINITY, f32::min)
}

#[must_use]
fn extrudability_from_tau_pa(tau_pa: f32) -> f32 {
    let dev = NdArrayDevice::default();
    let tau_for_print = tau_pa.max(50.0_f32);
    let pump_pa = (tau_for_print * 0.85).max(45.0);
    let extr = PrintabilityEngine::<FacadeBackend>::compute_extrudability(
        t4_scalar(tau_for_print, &dev),
        t4_scalar(pump_pa, &dev),
        16.0,
        120.0,
    );
    min_f32_rank4(extr)
}

#[must_use]
fn buildability_from_tau_pa(tau_pa: f32) -> f32 {
    let dev = NdArrayDevice::default();
    let tau_for_print = tau_pa.max(50.0_f32);
    let build = PrintabilityEngine::<FacadeBackend>::compute_buildability(
        t4_scalar(tau_for_print, &dev),
        t4_scalar(80.0, &dev),
        80.0,
    );
    min_f32_rank4(build)
}

/// Run pipeline + dual gate for a candidate mix.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Track A scoring helper; gate semantics from `dual_gate`.
#[must_use]
pub fn evaluate_mix_dual_gate(
    profile: &Profile,
    spec: &MixSpec,
) -> (PhysicsPipelineSummary, CastGateVerdict) {
    let device = NdArrayDevice::default();
    let mix = nominal_mix_tensor_for_mix_spec::<FacadeBackend>(profile, spec, &device);
    let report = run_full_physics_pipeline::<FacadeBackend>(profile, &mix);
    let summary = summary_with_calibrated_tau(profile, &report.summary);
    let verdict = evaluate_dual_gate(profile, spec, &summary);
    (summary, verdict)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Track A optimise targets mirrored from CLI `OptimizeField`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrackAObjective {
    YieldStressPa(f32),
    Extrudability(f32),
    PrintableWindow,
}

#[derive(Debug, Clone, Copy)]
enum SearchAxis {
    WaterCement,
    Superplasticiser,
}

#[derive(Debug, Clone, Copy)]
struct SearchBounds {
    lo: f32,
    hi: f32,
}

#[derive(Debug, Clone)]
struct SearchState {
    mix: MixSpec,
    summary: PhysicsPipelineSummary,
    verdict: CastGateVerdict,
    score: f32,
}

/// Coordinate descent over `w_c` then `superplasticiser_pct` to satisfy the objective.
/// formal_anchor: empirical://datasets/cli-optimize-wc-bisection.v1.csv
/// formal_status: Empirical
/// formal_dataset: "Track A w/c–SP bisection search grid"
/// formal_citation: "Proxy-loop coordinate descent envelope (CLI tests)"
/// formal_envelope: "crates/umst-cli/tests/proxy_loop_optimize.rs"
#[must_use]
pub fn coordinate_descent_optimize(
    profile: &Profile,
    base: &MixSpec,
    objective: TrackAObjective,
    steps: usize,
) -> (MixSpec, PhysicsPipelineSummary, CastGateVerdict) {
    let steps = steps.max(4);
    let (summary, verdict) = evaluate_mix_dual_gate(profile, base);
    let score = objective_score(objective, &summary, &verdict);
    let mut state = SearchState {
        mix: base.clone(),
        summary,
        verdict,
        score,
    };

    let w_lo = profile.regime.w_c_min as f32;
    let w_hi = profile.regime.w_c_max as f32;

    for axis in [SearchAxis::WaterCement, SearchAxis::Superplasticiser] {
        let bounds = axis_bounds(axis, w_lo, w_hi);
        state = bisect_axis(profile, objective, axis, bounds, steps, state);
    }

    (state.mix, state.summary, state.verdict)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Pure bisection envelope for one search axis.
fn bisect_axis(
    profile: &Profile,
    objective: TrackAObjective,
    axis: SearchAxis,
    mut bounds: SearchBounds,
    steps: usize,
    mut best: SearchState,
) -> SearchState {
    for _ in 0..steps {
        let mid = (bounds.lo + bounds.hi) * 0.5;
        let cand = mix_with_axis(&best.mix, axis, mid);
        let (summary, verdict) = evaluate_mix_dual_gate(profile, &cand);
        let score = objective_score(objective, &summary, &verdict);

        bounds = update_bisection_bounds(objective, &summary, bounds, mid);

        if candidate_preferred(objective, score, &verdict, best.score, &best.verdict) {
            best = SearchState {
                mix: cand,
                summary,
                verdict,
                score,
            };
        }

        if objective == TrackAObjective::PrintableWindow
            && matches!(verdict, CastGateVerdict::Admissible)
        {
            break;
        }
    }
    best
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Declarative candidate preference for coordinate descent.
#[must_use]
fn candidate_preferred(
    objective: TrackAObjective,
    score: f32,
    verdict: &CastGateVerdict,
    best_score: f32,
    best_verdict: &CastGateVerdict,
) -> bool {
    let score_improves = score < best_score;
    let gate_improves = match objective {
        TrackAObjective::PrintableWindow => {
            matches!(verdict, CastGateVerdict::Admissible)
                && !matches!(best_verdict, CastGateVerdict::Admissible)
        }
        _ => score_improves,
    };
    let admissible = matches!(verdict, CastGateVerdict::Admissible);
    gate_improves || (score_improves && admissible)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Pure bisection bracket update from pipeline summary scalars.
#[must_use]
fn update_bisection_bounds(
    objective: TrackAObjective,
    summary: &PhysicsPipelineSummary,
    bounds: SearchBounds,
    mid: f32,
) -> SearchBounds {
    match objective {
        TrackAObjective::YieldStressPa(target) => {
            if summary.rheology_yield_stress_pa > target {
                SearchBounds {
                    lo: mid,
                    hi: bounds.hi,
                }
            } else {
                SearchBounds {
                    lo: bounds.lo,
                    hi: mid,
                }
            }
        }
        TrackAObjective::Extrudability(target) => {
            if summary.printability_extrudability > target {
                SearchBounds {
                    lo: bounds.lo,
                    hi: mid,
                }
            } else {
                SearchBounds {
                    lo: mid,
                    hi: bounds.hi,
                }
            }
        }
        TrackAObjective::PrintableWindow => {
            if summary.rheology_yield_stress_pa > PRINTABLE_TAU_HI {
                SearchBounds {
                    lo: mid,
                    hi: bounds.hi,
                }
            } else {
                SearchBounds {
                    lo: bounds.lo,
                    hi: mid,
                }
            }
        }
    }
}

#[must_use]
fn axis_bounds(axis: SearchAxis, w_lo: f32, w_hi: f32) -> SearchBounds {
    match axis {
        SearchAxis::WaterCement => SearchBounds { lo: w_lo, hi: w_hi },
        SearchAxis::Superplasticiser => SearchBounds { lo: 0.0, hi: 2.0 },
    }
}

#[must_use]
fn objective_score(
    objective: TrackAObjective,
    summary: &PhysicsPipelineSummary,
    verdict: &CastGateVerdict,
) -> f32 {
    match objective {
        TrackAObjective::YieldStressPa(t) => (summary.rheology_yield_stress_pa - t).abs(),
        TrackAObjective::Extrudability(t) => (summary.printability_extrudability - t).abs(),
        TrackAObjective::PrintableWindow => match verdict {
            CastGateVerdict::Admissible => 0.0,
            _ => 1.0 + (summary.rheology_yield_stress_pa - PRINTABLE_TAU_LO).abs(),
        },
    }
}

#[must_use]
fn mix_with_axis(base: &MixSpec, axis: SearchAxis, value: f32) -> MixSpec {
    match axis {
        SearchAxis::WaterCement => MixSpec {
            w_c: WaterCementRatio::try_from(f64::from(value.clamp(0.10, 1.0))).unwrap_or(base.w_c),
            ..base.clone()
        },
        SearchAxis::Superplasticiser => MixSpec {
            superplasticiser_pct: value.clamp(0.0, 5.0),
            ..base.clone()
        },
    }
}

/// Build `proposed_next_mix.json` payload.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: JSON sidecar assembly for Track A CLI.
#[must_use]
pub fn proposed_next_mix_json(
    profile: &Profile,
    base: &MixSpec,
    proposed: &MixSpec,
    summary: &PhysicsPipelineSummary,
    verdict: &CastGateVerdict,
    objective: &str,
    steps: usize,
) -> ProposedNextMix {
    ProposedNextMix {
        schema_version: "proposed_next_mix.v1",
        calibration_profile: profile.bundle_id.clone(),
        base_mix: MixSpecWireOut::from(base),
        proposed_mix: MixSpecWireOut::from(proposed),
        dual_gate: DualGateWire::from_verdict(verdict, summary),
        objective: objective.to_string(),
        steps,
    }
}

/// Thermodynamic leg only (manifest CD when `manifest-bridge` enabled).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Thin wrapper for comparison example Track A path.
#[must_use]
pub fn thermodynamic_gate_ok(profile: &Profile, spec: &MixSpec) -> bool {
    let opts = PredictOptions {
        compare_homogeneous: false,
        ..PredictOptions::default()
    };
    predict_with_options(profile, spec, opts).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::dual_gate::{CastGateVerdict, PrintabilityReject};
    use crate::pipeline::ThermoReject;
    use umst_manifold::gate::verdict::GateRejectReason;

    fn stub_summary(tau_pa: f32, extr: f32) -> PhysicsPipelineSummary {
        PhysicsPipelineSummary {
            effective_water_cement_ratio: 0.45,
            hydration_alpha: 0.1,
            porosity_capillary: 0.15,
            strength_jennings_mpa: 20.0,
            rheology_yield_stress_pa: tau_pa,
            thermo_adiabatic_rise_proxy_c: 5.0,
            chloride_diffusivity_m2_s: 1e-12,
            printability_buildability: 0.5,
            printability_extrudability: extr,
            rheology_plastic_viscosity_pa_s: 50.0,
            itz_thickness_microns: 30.0,
            fracture_toughness_k_ic_mpa_sqrt_m: 1.0,
            sustainability_gwp_kg_co2_m3: 300.0,
            sustainability_cost_usd_per_m3: 100.0,
            dlvo_potential_kt_minimum: 0.9,
            shrinkage_microstrain_proxy: 200.0,
            freeze_thaw_durability_factor: 0.8,
            creep_compliance_1_over_gpa: 1e-2,
        }
    }

    #[test]
    fn dual_gate_wire_accessors_match_verdict_legs() {
        let summary = stub_summary(250.0, 0.5);
        let cases = [
            CastGateVerdict::Admissible,
            CastGateVerdict::RejectPrintability(PrintabilityReject::TauBelowBand {
                tau_pa: 100.0,
                lo: PRINTABLE_TAU_LO,
                hi: PRINTABLE_TAU_HI,
            }),
            CastGateVerdict::RejectThermodynamic(ThermoReject(GateRejectReason::RegimeEnvelope)),
            CastGateVerdict::RejectBoth {
                printability: PrintabilityReject::ExtrudabilityLow {
                    extr: 0.1,
                    min: 0.35,
                },
                thermodynamic: ThermoReject(GateRejectReason::MassViolation),
            },
        ];
        for verdict in cases {
            let wire = DualGateWire::from_verdict(&verdict, &summary);
            assert_eq!(wire.is_printability_ok(), verdict.printability_leg_pass());
            assert_eq!(wire.is_thermodynamic_ok(), verdict.thermodynamic_leg_pass());
            assert_eq!(wire.is_admissible(), verdict.is_admissible());
        }
    }
}
