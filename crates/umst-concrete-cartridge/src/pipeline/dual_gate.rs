// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Track A dual gate: lazy witness composition **W_print ∧ W_thermo** (equal weight).
//!
//! Each leg is a morphism from proposed mix → admissible or reject. Composition short-circuits
//! only at the verdict boundary (both legs evaluated; pass iff both invertible on the admissible
//! subcategory). Thermodynamic leg delegates CD to manifold `catalog_id`
//! [`CD_TRANSITION_CATALOG_ID`] — no duplicate CD math in cartridge.
//!
//! Witness ladder (proxy-loop scope): printability is a literature surrogate below R1; thermodynamic
//! leg is R1 (`umst.gate.cd_transition`) via [`super::canonical_gate`] when `manifest-bridge` is on.
//! See `umst-manifold/docs/GOD_GRADE_WITNESS_LADDER.md` (release witness profile; legacy filename).

use crate::calibration::Profile;
use crate::facade::MixSpec;
use crate::pipeline::canonical_gate::{thermodynamic_admissible, thermodynamic_verdict, ThermoReject};
use crate::pipeline::PhysicsPipelineSummary;
#[cfg(feature = "virtual-proxies")]
use crate::proxies::{virtual_extrusion, virtual_stack};

/// Manifold gate registry slug for Clausius–Duhem transition (GateUnificationSpec SSOT).
/// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
pub const CD_TRANSITION_CATALOG_ID: &str = "umst.gate.cd_transition";

/// formal_anchor: literature://roussel-2018-buildability-window
/// formal_status: Literature
/// formal_citation: "Roussel (2018) Cem. Concr. Res. 112, 76 — printable τ₀ band"
/// formal_form: "τ₀ ∈ [180, 360] Pa extrusion window"
pub const PRINTABLE_TAU_LO: f32 = 180.0;
/// formal_anchor: literature://roussel-2018-buildability-window
/// formal_status: Literature
/// formal_citation: "Roussel (2018) Cem. Concr. Res. 112, 76 — printable τ₀ band"
/// formal_form: "τ₀ ∈ [180, 360] Pa extrusion window"
pub const PRINTABLE_TAU_HI: f32 = 360.0;

/// Track A composite gate — equal-weight AND of printability ⊗ thermodynamic legs.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Algebraic verdict carrier (MP3.3); bool wire via deprecated accessors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CastGateVerdict {
    Admissible,
    RejectPrintability(PrintabilityReject),
    RejectThermodynamic(ThermoReject),
    RejectBoth {
        printability: PrintabilityReject,
        thermodynamic: ThermoReject,
    },
}

/// Printability leg reject reasons (Roussel τ₀ band + extrudability floor).
/// formal_anchor: literature://roussel-2018-buildability-window
/// formal_status: Literature
/// formal_citation: "Roussel (2018) Cem. Concr. Res. 112, 76 — printable τ₀ band"
/// formal_form: "τ₀ ∈ [180, 360] Pa AND extrudability ≥ 0.35"
/// formal_anchor_rationale: Structured reject carrier for printability leg (MP3.3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrintabilityReject {
    TauBelowBand {
        tau_pa: f32,
        lo: f32,
        hi: f32,
    },
    TauAboveBand {
        tau_pa: f32,
        lo: f32,
        hi: f32,
    },
    ExtrudabilityLow {
        extr: f32,
        min: f32,
    },
    NonFiniteExtrudability,
    #[cfg(feature = "virtual-proxies")]
    VirtualProxyFail {
        stack: f32,
        extr: f32,
    },
}

/// Type alias preserving Track A / CLI import path during one-release shim window.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Deprecated bool-shim alias for [`CastGateVerdict`] (MP3.3).
pub type DualGateVerdict = CastGateVerdict;

impl CastGateVerdict {
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Equal-weight AND of printability and thermodynamic legs.
    #[must_use]
    #[deprecated(note = "use match CastGateVerdict; removed MP3.6")]
    pub fn passes(&self) -> bool {
        matches!(self, Self::Admissible)
    }

    #[must_use]
    #[deprecated(note = "use match CastGateVerdict; removed MP3.6")]
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Deprecated bool shim — printability leg pass bit.
    pub fn printability_ok(&self) -> bool {
        !matches!(
            self,
            Self::RejectPrintability(_) | Self::RejectBoth { .. }
        )
    }

    #[must_use]
    #[deprecated(note = "use match CastGateVerdict; removed MP3.6")]
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Deprecated bool shim — thermodynamic leg pass bit.
    pub fn thermodynamic_ok(&self) -> bool {
        !matches!(
            self,
            Self::RejectThermodynamic(_) | Self::RejectBoth { .. }
        )
    }

    #[must_use]
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Algebraic admissibility predicate on [`CastGateVerdict`].
    pub fn is_admissible(self) -> bool {
        matches!(self, Self::Admissible)
    }
}

/// formal_anchor: literature://roussel-2018-buildability-window
/// formal_status: Literature
/// formal_citation: "Roussel (2018) Cem. Concr. Res. 112, 76 — printable τ₀ band"
/// formal_form: "τ₀ band AND extrudability ≥ 0.35"
/// formal_envelope: "tests/printability.rs"
#[must_use]
pub fn printability_window_ok(tau_y_pa: f32, extrudability: f32) -> bool {
    printability_leg_scalars(tau_y_pa, extrudability).is_ok()
}

/// Printability leg on scalar τ₀ and extrudability (no virtual proxies).
/// formal_anchor: literature://roussel-2018-buildability-window
/// formal_status: Literature
/// formal_citation: "Roussel (2018) Cem. Concr. Res. 112, 76 — printable τ₀ band"
/// formal_form: "τ₀ ∈ [180, 360] Pa AND extrudability ≥ 0.35"
/// formal_anchor_rationale: Enum leg evaluator; bool shim via [`printability_window_ok`].
#[must_use]
pub fn printability_leg_scalars(tau_y_pa: f32, extrudability: f32) -> Result<(), PrintabilityReject> {
    if !extrudability.is_finite() {
        return Err(PrintabilityReject::NonFiniteExtrudability);
    }
    if extrudability < 0.35 {
        return Err(PrintabilityReject::ExtrudabilityLow {
            extr: extrudability,
            min: 0.35,
        });
    }
    if tau_y_pa < PRINTABLE_TAU_LO {
        return Err(PrintabilityReject::TauBelowBand {
            tau_pa: tau_y_pa,
            lo: PRINTABLE_TAU_LO,
            hi: PRINTABLE_TAU_HI,
        });
    }
    if tau_y_pa > PRINTABLE_TAU_HI {
        return Err(PrintabilityReject::TauAboveBand {
            tau_pa: tau_y_pa,
            lo: PRINTABLE_TAU_LO,
            hi: PRINTABLE_TAU_HI,
        });
    }
    Ok(())
}

/// Printability leg from pipeline summary scalars (± virtual proxies).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Summary-scalar wrapper over [`printability_leg_scalars`].
#[must_use]
pub fn printability_leg(summary: &PhysicsPipelineSummary) -> Result<(), PrintabilityReject> {
    printability_leg_scalars(
        summary.rheology_yield_stress_pa,
        summary.printability_extrudability,
    )?;

    #[cfg(feature = "virtual-proxies")]
    {
        let stack = virtual_stack::virtual_stack_score_in_band(summary.rheology_yield_stress_pa);
        let extr = virtual_extrusion::virtual_extrusion_score(
            summary.rheology_yield_stress_pa,
            summary.printability_extrudability,
        );
        if stack <= 0.2 || extr <= 0.35 {
            return Err(PrintabilityReject::VirtualProxyFail { stack, extr });
        }
    }

    Ok(())
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Summary-scalar wrapper over [`printability_window_ok`].
#[must_use]
pub fn printability_from_summary(summary: &PhysicsPipelineSummary) -> bool {
    printability_leg(summary).is_ok()
}

/// Printability leg augmented by virtual-lab proxy scores (feature `virtual-proxies`).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Lazy AND of summary band + Roussel stack/extrusion surrogates.
#[cfg(feature = "virtual-proxies")]
#[must_use]
pub fn printability_with_virtual_proxies(summary: &PhysicsPipelineSummary) -> bool {
    printability_leg(summary).is_ok()
}

/// Thermodynamic leg: canonical composed gate (regime envelope + manifold CD); Phase 0d routing.
/// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
#[must_use]
pub fn thermodynamic_ok(profile: &Profile, spec: &MixSpec) -> bool {
    thermodynamic_admissible(profile, spec)
}

/// Thermodynamic leg verdict — maps manifold [`ThermoReject`] at cartridge boundary.
/// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
/// formal_anchor_rationale: Enum leg evaluator; bool shim via [`thermodynamic_ok`].
#[must_use]
pub fn thermodynamic_leg(profile: &Profile, spec: &MixSpec) -> Result<(), ThermoReject> {
    thermodynamic_verdict(profile, spec)
}

/// Evaluate printability (± virtual proxies) AND thermodynamic gate with equal weight.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Track A composite gate; legs carry individual anchors.
#[must_use]
pub fn evaluate_dual_gate(
    profile: &Profile,
    spec: &MixSpec,
    summary: &PhysicsPipelineSummary,
) -> CastGateVerdict {
    let print_res = printability_leg(summary);
    let thermo_res = thermodynamic_leg(profile, spec);

    match (print_res, thermo_res) {
        (Ok(()), Ok(())) => CastGateVerdict::Admissible,
        (Err(p), Ok(())) => CastGateVerdict::RejectPrintability(p),
        (Ok(()), Err(t)) => CastGateVerdict::RejectThermodynamic(t),
        (Err(p), Err(t)) => CastGateVerdict::RejectBoth {
            printability: p,
            thermodynamic: t,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use umst_manifold::gate::verdict::GateRejectReason;

    #[test]
    fn printability_band_matches_bool_shim() {
        let cases = [
            (250.0_f32, 0.5_f32, true),
            (100.0, 0.5, false),
            (400.0, 0.5, false),
            (250.0, 0.2, false),
            (250.0, f32::NAN, false),
        ];
        for (tau, extr, expected) in cases {
            assert_eq!(
                printability_window_ok(tau, extr),
                expected,
                "tau={tau} extr={extr}"
            );
            assert_eq!(printability_leg_scalars(tau, extr).is_ok(), expected);
        }
    }

    #[test]
    fn cast_gate_verdict_bool_shim_roundtrip() {
        let admissible = CastGateVerdict::Admissible;
        #[allow(deprecated)]
        {
            assert!(admissible.passes());
            assert!(admissible.printability_ok());
            assert!(admissible.thermodynamic_ok());
        }

        let print_reject = CastGateVerdict::RejectPrintability(
            PrintabilityReject::TauBelowBand {
                tau_pa: 100.0,
                lo: PRINTABLE_TAU_LO,
                hi: PRINTABLE_TAU_HI,
            },
        );
        #[allow(deprecated)]
        {
            assert!(!print_reject.passes());
            assert!(!print_reject.printability_ok());
            assert!(print_reject.thermodynamic_ok());
        }

        let thermo_reject =
            CastGateVerdict::RejectThermodynamic(ThermoReject(GateRejectReason::RegimeEnvelope));
        #[allow(deprecated)]
        {
            assert!(!thermo_reject.passes());
            assert!(thermo_reject.printability_ok());
            assert!(!thermo_reject.thermodynamic_ok());
        }

        let both = CastGateVerdict::RejectBoth {
            printability: PrintabilityReject::ExtrudabilityLow { extr: 0.1, min: 0.35 },
            thermodynamic: ThermoReject(GateRejectReason::MassViolation),
        };
        #[allow(deprecated)]
        {
            assert!(!both.passes());
            assert!(!both.printability_ok());
            assert!(!both.thermodynamic_ok());
        }
    }
}
