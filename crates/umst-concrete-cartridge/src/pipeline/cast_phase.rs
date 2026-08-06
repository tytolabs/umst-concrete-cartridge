// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! FP Manifesto §3 — collapsed-pipeline cast lifecycle classifier (MP3.1).
//!
//! [`CastPhase`] is the 0-D projection of manifold [`MaterialPhase`](umst_manifold::core::MaterialPhase):
//! hydration degree `α` alone routes macroscopic phase; printable τ₀ band is a **gate** on `Fluid`
//! (see [`super::dual_gate`]), not a phase discriminator.
//!
//! Schedule: `archived/residuals/misc-outputs-tmp/fp_concrete_dual_gate_adt_plan.md` MP3.1.

use serde::{Deserialize, Serialize};
use umst_manifold::core::MaterialPhaseKind;

/// Macroscopic cast phase for the collapsed tensor pipeline (singleton spatial axes).
///
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: 0-D mirror of manifold `MaterialPhase` variants; thresholds from `Profile::cast_lifecycle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CastPhase {
    /// α < α_set — rheology + printability only.
    Fluid,
    /// α_set ≤ α < α_hard — chemo-thermal + transport + shrinkage.
    Setting,
    /// α ≥ α_hard — mechanics + fracture + creep + durability.
    Solid,
}

/// Classifier inputs — pure scalars from the pipeline head.
///
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: `yield_stress_pa` and `age_days` reserved for MP3.2 audit; classifier uses `α` only.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CastPhaseInputs {
    pub hydration_alpha: f32,
    pub yield_stress_pa: f32,
    pub age_days: f32,
}

/// Profile-owned hydration thresholds (`[cast_lifecycle]` in bundled TOML).
///
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Defaults α_set=0.15, α_hard=0.85 per `fp_material_phase_adt_plan.md`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct CastLifecycleThresholds {
    #[serde(default = "default_alpha_set")]
    pub alpha_set: f32,
    #[serde(default = "default_alpha_hard")]
    pub alpha_hard: f32,
}

const fn default_alpha_set() -> f32 {
    0.15
}

const fn default_alpha_hard() -> f32 {
    0.85
}

impl Default for CastLifecycleThresholds {
    fn default() -> Self {
        Self {
            alpha_set: default_alpha_set(),
            alpha_hard: default_alpha_hard(),
        }
    }
}

/// Classify cast lifecycle from hydration degree and profile thresholds.
///
/// τ₀ and age are **not** used here — printability is evaluated on `Fluid` via the dual gate.
///
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Pure total function; no orchestrator or gate side effects (MP3.1).
#[must_use]
pub fn classify_cast_phase(
    inputs: &CastPhaseInputs,
    thresholds: &CastLifecycleThresholds,
) -> CastPhase {
    match inputs.hydration_alpha {
        a if a < thresholds.alpha_set => CastPhase::Fluid,
        a if a < thresholds.alpha_hard => CastPhase::Setting,
        _ => CastPhase::Solid,
    }
}

impl CastPhase {
    /// Project to manifold [`MaterialPhaseKind`] for cross-crate routing vocabulary.
    ///
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Keeps cartridge phase tags aligned with manifold MP1 SSOT.
    #[inline]
    #[must_use]
    pub fn material_phase_kind(self) -> MaterialPhaseKind {
        match self {
            Self::Fluid => MaterialPhaseKind::Fluid,
            Self::Setting => MaterialPhaseKind::Setting,
            Self::Solid => MaterialPhaseKind::Solid,
        }
    }
}

/// Stage eligibility matrix for MP3.2 orchestrator phase router.
///
/// `optional` cells in the plan table are treated as **run** to preserve scalar parity pins.
///
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Pure total function; locked table in `fp_concrete_dual_gate_adt_plan.md`.
#[must_use]
pub fn stage_eligible(stage_id: &'static str, phase: CastPhase) -> bool {
    use CastPhase::Fluid;
    match stage_id {
        "hydration_degree"
        | "packing_density"
        | "porosity_capillary_bulk"
        | "sustainability"
        | "cost_linear_dot" => true,
        "strength_jennings" => !matches!(phase, Fluid),
        "colloidal_dlvo" => true,
        "rheology_yodel" | "rheology_chateau_ovarlez" => !matches!(phase, CastPhase::Solid),
        "thermo_heat_rate_proxy" => matches!(phase, CastPhase::Setting | CastPhase::Solid),
        "transport_chloride" => !matches!(phase, Fluid),
        "printability" => matches!(phase, Fluid),
        "itz" => !matches!(phase, Fluid),
        "chemo_water" => !matches!(phase, Fluid),
        "fracture" => matches!(phase, CastPhase::Solid),
        "nano_enhancement_baseline" => !matches!(phase, Fluid),
        "creep" => matches!(phase, CastPhase::Solid),
        "set_time" => !matches!(phase, CastPhase::Solid),
        "shrinkage" => !matches!(phase, Fluid),
        "freeze_thaw" => matches!(phase, CastPhase::Solid),
        "self_heal" => matches!(phase, CastPhase::Solid),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THRESH: CastLifecycleThresholds = CastLifecycleThresholds {
        alpha_set: 0.15,
        alpha_hard: 0.85,
    };

    fn inputs(alpha: f32) -> CastPhaseInputs {
        CastPhaseInputs {
            hydration_alpha: alpha,
            yield_stress_pa: 250.0,
            age_days: 0.5,
        }
    }

    #[test]
    fn cast_phase_classifier_golden_table_default_thresholds() {
        let cases: &[(f32, CastPhase)] = &[
            (0.0, CastPhase::Fluid),
            (0.149_999, CastPhase::Fluid),
            (0.15, CastPhase::Setting),
            (0.5, CastPhase::Setting),
            (0.849_999, CastPhase::Setting),
            (0.85, CastPhase::Solid),
            (1.0, CastPhase::Solid),
        ];

        for &(alpha, expected) in cases {
            let got = classify_cast_phase(&inputs(alpha), &THRESH);
            assert_eq!(
                got, expected,
                "α={alpha}: expected {expected:?}, got {got:?}"
            );
        }
    }

    #[test]
    fn cast_phase_classifier_custom_thresholds() {
        let custom = CastLifecycleThresholds {
            alpha_set: 0.2,
            alpha_hard: 0.9,
        };
        assert_eq!(
            classify_cast_phase(&inputs(0.19), &custom),
            CastPhase::Fluid
        );
        assert_eq!(
            classify_cast_phase(&inputs(0.2), &custom),
            CastPhase::Setting
        );
        assert_eq!(
            classify_cast_phase(&inputs(0.89), &custom),
            CastPhase::Setting
        );
        assert_eq!(
            classify_cast_phase(&inputs(0.9), &custom),
            CastPhase::Solid
        );
    }

    #[test]
    fn cast_phase_kind_aligns_with_manifold_material_phase_kind() {
        assert_eq!(
            CastPhase::Fluid.material_phase_kind(),
            MaterialPhaseKind::Fluid
        );
        assert_eq!(
            CastPhase::Setting.material_phase_kind(),
            MaterialPhaseKind::Setting
        );
        assert_eq!(
            CastPhase::Solid.material_phase_kind(),
            MaterialPhaseKind::Solid
        );
    }

    #[test]
    fn cast_phase_inputs_tau_and_age_do_not_affect_classifier() {
        let base = inputs(0.1);
        let mut varied = base;
        varied.yield_stress_pa = 999.0;
        varied.age_days = 365.0;
        assert_eq!(
            classify_cast_phase(&base, &THRESH),
            classify_cast_phase(&varied, &THRESH)
        );
    }

    #[test]
    fn cast_lifecycle_thresholds_default() {
        let d = CastLifecycleThresholds::default();
        assert!((d.alpha_set - 0.15).abs() < f32::EPSILON);
        assert!((d.alpha_hard - 0.85).abs() < f32::EPSILON);
    }

    #[test]
    fn bundled_profiles_carry_cast_lifecycle_defaults() {
        use crate::calibration::{Profile, BUNDLED_PROFILE_IDS};

        for id in BUNDLED_PROFILE_IDS {
            let profile = Profile::load_bundled(id).expect("bundled profile loads");
            let expected = CastLifecycleThresholds::default();
            assert_eq!(
                profile.cast_lifecycle, expected,
                "profile `{id}` cast_lifecycle mismatch"
            );
        }
    }
}
