// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! B2/B3 scalar delegate for [`super::orchestrator::run_full_physics_pipeline`].
//!
//! Card `g_spawn_i_orch_2054` — begin reroute of inelastic/fracture tail + B3 porosity audit
//! toward atom strict APIs. Burn tensor engines in `physics/*` are **retained** until S6 purge.
//!
//! formal_anchor: NONE
//! formal_status: NONE
//! formal_anchor_rationale: Scalar routing shim only; ψ/𝒟 compose digest unchanged.

use umst_cartridge_concrete::capillary_porosity_from_chem;
use umst_cartridge_solid_inelastic::{
    fracture_energy_gc_j_m2, try_autogenous_shrinkage_microstrain, try_creep_compliance,
    try_fracture_energy_gc_j_m2, try_fracture_phase_ledger, AutogenousShrinkageInput,
    CreepComplianceInput,
};

/// Orchestrator creep stage ambient RH pin — matches legacy `CreepEngine` call site.
pub const ORCHESTRATOR_AMBIENT_RH: f64 = 0.55;

/// Orchestrator creep stage load age [days] — matches legacy tensor path.
pub const ORCHESTRATOR_T_LOAD_DAYS: f64 = 7.0;

/// Collapsed mix scalars extracted from orchestrator layout — B2/B3 domain mirror.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrchestratorMixScalars {
    /// Effective water/cement ratio [—].
    pub w_c_eff: f64,
    /// Hydration degree α ∈ [0, 1].
    pub hydration_alpha: f64,
    /// Jennings compressive strength [MPa] when strength stage executed.
    pub fc_mpa: f64,
    /// Cement content [kg/m³].
    pub cement_kg_m3: f64,
    /// SCM mass fraction [—].
    pub scm_mass_fraction: f64,
    /// Mix age [days].
    pub age_days: f64,
}

impl OrchestratorMixScalars {
    /// Creep compliance input — B2 `creep.rs` domain.
    #[must_use]
    pub fn creep_input(self) -> CreepComplianceInput {
        CreepComplianceInput {
            compressive_strength_mpa: self.fc_mpa.max(1.0),
            wc_ratio: self.w_c_eff,
            ambient_rh: ORCHESTRATOR_AMBIENT_RH,
            t_load_days: ORCHESTRATOR_T_LOAD_DAYS,
            t_current_days: self.age_days.max(0.1),
        }
    }

    /// Autogenous shrinkage input — B2 `shrinkage.rs` domain.
    #[must_use]
    pub fn shrinkage_input(self) -> AutogenousShrinkageInput {
        AutogenousShrinkageInput {
            wc_ratio: self.w_c_eff,
            degree_hydration: self.hydration_alpha,
            cement_content_kg: self.cement_kg_m3.max(1.0),
            scm_ratio: self.scm_mass_fraction,
        }
    }
}

/// B2 scalar creep compliance [1/GPa] — strict path with monolith-compatible fallback `None`.
#[must_use]
pub fn try_creep_compliance_orchestrator(mix: OrchestratorMixScalars) -> Option<f32> {
    let compliance = try_creep_compliance(mix.creep_input()).ok()?;
    if compliance.is_finite() {
        Some(compliance as f32)
    } else {
        None
    }
}

/// B2 scalar autogenous shrinkage [µε] — strict path; report uses magnitude via `.max(0.0)`.
#[must_use]
pub fn try_autogenous_shrinkage_orchestrator(mix: OrchestratorMixScalars) -> Option<f32> {
    let microstrain = try_autogenous_shrinkage_microstrain(mix.shrinkage_input()).ok()?;
    if microstrain.is_finite() {
        Some(microstrain as f32)
    } else {
        None
    }
}

/// B2 scalar fracture energy `G_c` [J/m²] from profile `s_intrinsic`.
///
/// Strict [`try_fracture_energy_gc_j_m2`] with B2 saturating fallback — no monolith hot path.
#[must_use]
pub fn fracture_energy_gc_j_m2_orchestrator(s_intrinsic: f64) -> f32 {
    let gc = try_fracture_energy_gc_j_m2(s_intrinsic)
        .unwrap_or_else(|_| fracture_energy_gc_j_m2(s_intrinsic));
    gc as f32
}

/// B2 fracture tail given B1-supplied `e_eff` scalar and profile `s_intrinsic`.
///
/// `e_eff_gpa` must be in the same numeric scale as the legacy tensor path (GPa-class).
#[must_use]
pub fn try_fracture_k_ic_orchestrator(e_eff_gpa: f64, s_intrinsic: f64) -> Option<f32> {
    let ledger = try_fracture_phase_ledger(e_eff_gpa, s_intrinsic).ok()?;
    let k_ic = ledger.fracture_toughness_k_ic;
    if k_ic.is_finite() && k_ic > 0.0 {
        Some(k_ic as f32)
    } else {
        None
    }
}

/// B3 capillary porosity φ_c audit scalar — `umst-chem` SSOT via consumer capwrap.
///
/// **Not wired to summary yet** — shadow path for orchestrator B3 begin slice; tensor
/// `compute_capillary_porosity` remains report SSOT until parity witness lands.
#[must_use]
pub fn capillary_porosity_b3_audit(w_c_eff: f64, hydration_alpha: f64) -> f64 {
    capillary_porosity_from_chem(w_c_eff, hydration_alpha)
}

#[cfg(test)]
mod tests {
    use super::*;
    use umst_cartridge_solid_inelastic::{fracture_energy_gc_j_m2, UCI_D1_S_INTRINSIC_REF};

    fn orchestrator_pin_mix() -> OrchestratorMixScalars {
        OrchestratorMixScalars {
            w_c_eff: 0.45,
            hydration_alpha: 0.7,
            fc_mpa: 40.0,
            cement_kg_m3: 350.0,
            scm_mass_fraction: 0.1,
            age_days: 28.0,
        }
    }

    #[test]
    fn creep_delegate_finite_at_orchestrator_pin() {
        let compliance = try_creep_compliance_orchestrator(orchestrator_pin_mix())
            .expect("orchestrator creep pin");
        assert!(compliance.is_finite());
        assert!(compliance > 0.0);
    }

    #[test]
    fn shrinkage_delegate_finite_at_orchestrator_pin() {
        let shrink = try_autogenous_shrinkage_orchestrator(orchestrator_pin_mix())
            .expect("orchestrator shrinkage pin");
        assert!(shrink.is_finite());
    }

    #[test]
    fn fracture_delegate_matches_b2_ledger_at_orchestrator_pin() {
        let e_eff = 30.0_f64;
        let k_ic = try_fracture_k_ic_orchestrator(e_eff, UCI_D1_S_INTRINSIC_REF)
            .expect("orchestrator fracture pin");
        let gc = fracture_energy_gc_j_m2(UCI_D1_S_INTRINSIC_REF);
        let expected = (e_eff * gc).sqrt() as f32;
        assert!((k_ic - expected).abs() / expected.max(1e-6) < 1e-5);
    }

    #[test]
    fn gc_orchestrator_matches_monolith_oracle_at_uci_d1() {
        use crate::calibration::Profile;
        use crate::physics::fracture_material::fracture_energy_gc_j_per_m2_from_profile;

        let profile = Profile::load_bundled("uci_d1").expect("bundled uci_d1");
        let bridge = fracture_energy_gc_j_m2_orchestrator(profile.powers.s_intrinsic);
        let oracle = fracture_energy_gc_j_per_m2_from_profile(&profile);
        assert!(
            (bridge - oracle).abs() / oracle.max(1e-6) < 1e-6,
            "G_c bridge drift: bridge={bridge} oracle={oracle}"
        );
    }

    #[test]
    fn gc_orchestrator_matches_b2_scalar_scaled_profile() {
        let s = 90.0_f64;
        let bridge = fracture_energy_gc_j_m2_orchestrator(s);
        let b2 = fracture_energy_gc_j_m2(s) as f32;
        assert!((bridge - b2).abs() / b2.max(1e-6) < 1e-6);
    }

    #[test]
    fn b3_capillary_porosity_audit_admissible_at_orchestrator_pin() {
        let mix = orchestrator_pin_mix();
        let phi = capillary_porosity_b3_audit(mix.w_c_eff, mix.hydration_alpha);
        assert!(phi.is_finite());
        assert!((0.0..=1.0).contains(&phi));
    }

    /// FP §6 — delegate inputs idempotent on repeated evaluation.
    #[test]
    fn orchestrator_mix_inputs_idempotent() {
        let mix = orchestrator_pin_mix();
        assert_eq!(mix.creep_input(), mix.creep_input());
        assert_eq!(mix.shrinkage_input(), mix.shrinkage_input());
    }
}
