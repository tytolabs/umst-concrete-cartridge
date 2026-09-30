// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Maps [`PhysicsPipelineReport`](super::PhysicsPipelineReport) into manifold [`PhysicalResult`](umst_manifold::core::traits::PhysicalResult).
//!
//! This is **control-policy**, not a thermodynamic identity:
//! - `free_energy[..,0]` = Jennings tensor strength headline (MPa).
//! - `free_energy[..,1]` = YODEL yield stress (Pa) after θ calibration ([`crate::calibration_fit`]).
//! - `dissipation[..,0]` = tensor-route hydration degree α (`physics::hydration`).
//! - `safety_margin[..,0]` = homogeneous admissibility margin using the **same** α and effective w/c.
//! - `cost[..,0]` = sustainability embodied carbon (kg CO₂-eq / m³).
//!
//! Differentiable potentials live in manifold adapters; this module is pure assembly (no duplicate CD).

use burn::tensor::{backend::Backend, Data, Shape, Tensor};
use umst_manifold::core::tensors::MaterialCompositionTensor;
use umst_manifold::core::traits::PhysicalResult;

use crate::calibration::Profile;
use crate::calibration_fit::calibrated_tau0_pa;
use crate::homogeneous::mix_row_from_scalar_spec;
use crate::mix_layout::{fractions_from_mix_row, mix_tensor_from_layout};
use crate::pipeline::report::PhysicsPipelineReport;

/// Scalar recipe for topology nominal-mix when an explicit design is known (avoids regime midpoint).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Decouples `core` from `facade::MixSpec` while sharing mix_layout semantics.
#[derive(Debug, Clone, Copy)]
pub struct TopologyNominalMix {
    pub w_c: f32,
    pub superplasticiser_pct: f32,
    pub fly_ash_pct: f32,
    pub silica_fume_pct: f32,
    pub aggregate_volume_fraction: f32,
    pub target_age_hours: f32,
    pub temperature_k: f32,
}

impl From<&crate::facade::MixSpec> for TopologyNominalMix {
    fn from(s: &crate::facade::MixSpec) -> Self {
        Self {
            w_c: s.w_c.value(),
            superplasticiser_pct: s.superplasticiser_pct,
            fly_ash_pct: s.fly_ash_pct,
            silica_fume_pct: s.silica_fume_pct,
            aggregate_volume_fraction: s.aggregate_volume_fraction,
            target_age_hours: s.target_age_hours,
            temperature_k: s.temperature_k.value(),
        }
    }
}

/// Single-run nominal-mix [`PhysicsPipelineReport`] for topology (avoids duplicate `run_full_physics_pipeline` calls).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: When `nominal` is set, uses caller recipe instead of regime midpoint.
#[must_use]
pub fn topology_pipeline_report<B: Backend<FloatElem = f32>>(
    profile: &Profile,
    device: &B::Device,
    nominal: Option<TopologyNominalMix>,
) -> PhysicsPipelineReport {
    let mix: MaterialCompositionTensor<B> = match nominal {
        Some(n) => mix_tensor_from_topology_nominal::<B>(profile, n, device),
        None => nominal_mix_tensor_for_topology::<B>(profile, device),
    };
    super::run_full_physics_pipeline::<B>(profile, &mix)
}

fn mix_tensor_from_topology_nominal<B: Backend<FloatElem = f32>>(
    profile: &Profile,
    n: TopologyNominalMix,
    device: &B::Device,
) -> MaterialCompositionTensor<B> {
    let row = mix_row_from_scalar_spec(
        profile,
        n.w_c,
        n.superplasticiser_pct,
        n.fly_ash_pct,
        n.silica_fume_pct,
        n.aggregate_volume_fraction,
        n.target_age_hours,
        n.temperature_k,
    );
    let layout = fractions_from_mix_row(&row, n.aggregate_volume_fraction);
    mix_tensor_from_layout(&layout, device)
}

#[must_use]
pub(crate) fn topology_pipeline_headlines_from_report(
    report: &PhysicsPipelineReport,
) -> (f32, f32, f32, f32, f32, f32) {
    let s = &report.summary;
    (
        s.strength_jennings_mpa,
        s.rheology_yield_stress_pa,
        s.sustainability_gwp_kg_co2_m3,
        s.effective_water_cement_ratio,
        s.hydration_alpha,
        s.fracture_toughness_k_ic_mpa_sqrt_m,
    )
}

/// Single source of truth for [`PhysicalResult`] assembly from a pipeline report + profile cues.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Encodes CLI tensor summary contract; see module-level policy mapping.
#[must_use]
pub fn physical_result_from_report<B: Backend<FloatElem = f32>>(
    profile: &Profile,
    report: &PhysicsPipelineReport,
    device: &B::Device,
) -> PhysicalResult<B> {
    let fc = report.summary.strength_jennings_mpa;
    let tau = calibrated_tau0_pa(
        report.summary.rheology_yield_stress_pa,
        profile.rheology_calibration.as_ref(),
    );
    let alpha = report.summary.hydration_alpha;
    let gwp = report.summary.sustainability_gwp_kg_co2_m3;
    let w_c_eff = report.summary.effective_water_cement_ratio;

    let margin = crate::homogeneous::safety_margin(profile, w_c_eff, alpha);

    let free_energy =
        Tensor::<B, 2>::from_data(Data::new(vec![fc, tau], Shape::new([1, 2])), device);
    let dissipation = Tensor::<B, 2>::from_data(Data::new(vec![alpha], Shape::new([1, 1])), device);
    let safety_margin_tensor =
        Tensor::<B, 2>::from_data(Data::new(vec![margin], Shape::new([1, 1])), device);
    let cost = Tensor::<B, 2>::from_data(Data::new(vec![gwp], Shape::new([1, 1])), device);

    let damage = Tensor::<B, 2>::zeros([1, 1], device);

    PhysicalResult {
        free_energy,
        dissipation,
        safety_margin: safety_margin_tensor,
        cost,
        damage,
        temperature_delta: None,
    }
}

/// Build [`MixTensor`] from an explicit [`crate::facade::MixSpec`] (caller recipe, not regime midpoint).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Shared layout path for predict and topology when recipe is known.
#[must_use]
pub fn nominal_mix_tensor_for_mix_spec<B: Backend<FloatElem = f32>>(
    profile: &Profile,
    spec: &crate::facade::MixSpec,
    device: &B::Device,
) -> MaterialCompositionTensor<B> {
    let w_c = spec.w_c.value();
    let sp_pct = spec.superplasticiser_pct;
    let fly_pct = spec.fly_ash_pct;
    let silica_pct = spec.silica_fume_pct;
    let phi = spec.aggregate_volume_fraction;
    let age_h = spec.target_age_hours;
    let temp_k = spec.temperature_k.value();
    let row = mix_row_from_scalar_spec(
        profile, w_c, sp_pct, fly_pct, silica_pct, phi, age_h, temp_k,
    );
    let layout = fractions_from_mix_row(&row, phi);
    mix_tensor_from_layout(&layout, device)
}

/// Regime-centered nominal [`MixTensor`] when topology receives only a [`UnifiedMaterialStateTensor`](umst_manifold::core::tensors::UnifiedMaterialStateTensor)
/// (no explicit recipe). Uses profile `[regime]` midpoints and conservative SCM splits from optional caps.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Deterministic surrogate mix so staged tensor engines match `compute_all` semantics.
#[must_use]
pub fn nominal_mix_tensor_for_topology<B: Backend<FloatElem = f32>>(
    profile: &Profile,
    device: &B::Device,
) -> MaterialCompositionTensor<B> {
    let w_c = (((profile.regime.w_c_min + profile.regime.w_c_max) * 0.5) as f32).clamp(0.10, 1.0);
    let age_h =
        (((profile.regime.age_hours_min + profile.regime.age_hours_max) * 0.5) as f32).max(0.0);
    let temp_k = (((profile.regime.temperature_k_min + profile.regime.temperature_k_max) * 0.5)
        as f32)
        .max(1.0);
    let fly_pct = profile
        .regime
        .fly_ash_pct_max
        .map(|x| ((x * 0.5) as f32).clamp(0.0, 75.0))
        .unwrap_or(0.0);
    let silica_pct = profile
        .regime
        .silica_fume_pct_max
        .map(|x| ((x * 0.5) as f32).clamp(0.0, 75.0))
        .unwrap_or(0.0);
    let phi = 0.65_f32;
    let sp_pct = 0.5_f32;
    let row = mix_row_from_scalar_spec(
        profile, w_c, sp_pct, fly_pct, silica_pct, phi, age_h, temp_k,
    );
    let layout = fractions_from_mix_row(&row, phi);
    mix_tensor_from_layout(&layout, device)
}

/// Headline scalars from [`super::run_full_physics_pipeline`] on [`nominal_mix_tensor_for_topology`], aligned with
/// [`physical_result_from_report`] (`free_energy` / `cost` / admissibility wires).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Single SSOT with Jennings/YODEL/GWP tensor pipeline used by bulk predict.
#[must_use]
pub fn topology_pipeline_headlines<B: Backend<FloatElem = f32>>(
    profile: &Profile,
    device: &B::Device,
) -> (f32, f32, f32, f32, f32, f32) {
    topology_pipeline_headlines_from_report(&topology_pipeline_report::<B>(profile, device, None))
}
