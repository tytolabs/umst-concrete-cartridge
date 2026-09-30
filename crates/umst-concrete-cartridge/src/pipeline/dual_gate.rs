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
//! leg is R1 (`umst.gate.cd_transition`) via `predict_with_options` when `manifest-bridge` is on.
//! See `umst-manifold/docs/GOD_GRADE_WITNESS_LADDER.md` (release witness profile; legacy filename).

use crate::calibration::Profile;
use crate::facade::{predict_with_options, MixSpec, PredictOptions};
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

/// Result of dual-gate evaluation (equal-weight AND).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Composite verdict; legs documented on helper fns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DualGateVerdict {
    pub printability_ok: bool,
    pub thermodynamic_ok: bool,
}

impl DualGateVerdict {
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Equal-weight AND of printability and thermodynamic legs.
    #[must_use]
    pub fn passes(&self) -> bool {
        self.printability_ok && self.thermodynamic_ok
    }
}

/// formal_anchor: literature://roussel-2018-buildability-window
/// formal_status: Literature
/// formal_citation: "Roussel (2018) Cem. Concr. Res. 112, 76 — printable τ₀ band"
/// formal_form: "τ₀ band AND extrudability ≥ 0.35"
/// formal_envelope: "tests/printability.rs"
#[must_use]
pub fn printability_window_ok(tau_y_pa: f32, extrudability: f32) -> bool {
    let in_band = (PRINTABLE_TAU_LO..=PRINTABLE_TAU_HI).contains(&tau_y_pa);
    let extr_ok = extrudability.is_finite() && extrudability >= 0.35;
    in_band && extr_ok
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Summary-scalar wrapper over [`printability_window_ok`].
#[must_use]
pub fn printability_from_summary(summary: &PhysicsPipelineSummary) -> bool {
    printability_window_ok(
        summary.rheology_yield_stress_pa,
        summary.printability_extrudability,
    )
}

/// Printability leg augmented by virtual-lab proxy scores (feature `virtual-proxies`).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Lazy AND of summary band + Roussel stack/extrusion surrogates.
#[cfg(feature = "virtual-proxies")]
#[must_use]
pub fn printability_with_virtual_proxies(summary: &PhysicsPipelineSummary) -> bool {
    let stack = virtual_stack::virtual_stack_score_in_band(summary.rheology_yield_stress_pa);
    let extr = virtual_extrusion::virtual_extrusion_score(
        summary.rheology_yield_stress_pa,
        summary.printability_extrudability,
    );
    printability_from_summary(summary) && stack > 0.2 && extr > 0.35
}

/// Thermodynamic leg: predict path must succeed (manifest CD when `manifest-bridge` on).
/// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
#[must_use]
pub fn thermodynamic_ok(profile: &Profile, spec: &MixSpec) -> bool {
    let opts = PredictOptions {
        compare_homogeneous: true,
    };
    predict_with_options(profile, spec, opts).is_ok()
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
) -> DualGateVerdict {
    #[cfg(feature = "virtual-proxies")]
    let print_ok = printability_with_virtual_proxies(summary);
    #[cfg(not(feature = "virtual-proxies"))]
    let print_ok = printability_from_summary(summary);

    DualGateVerdict {
        printability_ok: print_ok,
        thermodynamic_ok: thermodynamic_ok(profile, spec),
    }
}
