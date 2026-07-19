// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! B1 scalar delegate for [`super::orchestrator::run_full_physics_pipeline`] fracture `E_eff` head.
//!
//! Card `g_spawn_i_b16_mt_carve_0721` — reroute Mori–Tanaka homogenization off monolith
//! `FractureEngine::compute_effective_modulus_mt` toward B1 `umst-cartridge-continuum`.
//! Burn tensor engine in `physics/fracture.rs` is **retained** until operator S7 purge batch.
//!
//! formal_anchor: literature://Ulm-Coussy-2003-micromechanics
//! formal_status: Literature
//! formal_anchor_rationale: Faithful scalar lift; B2 fracture tail consumes GPa-class `e_eff` only.

use umst_cartridge_continuum::{
    compute_effective_modulus_mt_gpa, ThreePhaseCompositeCoeffs,
};

/// B1 three-phase Voigt–Reuss `E_eff` [GPa] — orchestrator fracture stage pin @ `fc_mpa`.
///
/// Coefficients mirror legacy `pipeline/orchestrator.rs` fracture tensors:
/// `e_paste = (fc·5).max(1)` GPa · `v_agg = 0.65` · `v_itz = 0.06` · `E_agg = 60` · `E_itz = 20` GPa.
#[must_use]
pub fn compute_effective_modulus_mt_orchestrator(fc_mpa: f64) -> f64 {
    let coeffs = ThreePhaseCompositeCoeffs::orchestrator_pin_from_fc_mpa(fc_mpa);
    compute_effective_modulus_mt_gpa(coeffs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b1_e_eff_finite_positive_at_orchestrator_pin() {
        let e_eff = compute_effective_modulus_mt_orchestrator(40.0);
        assert!(e_eff.is_finite() && e_eff > 0.0);
    }

    /// FP §6 — homogenization idempotent on repeated evaluation.
    #[test]
    fn b1_e_eff_idempotent_on_repeated_calls() {
        let e0 = compute_effective_modulus_mt_orchestrator(30.0);
        let e1 = compute_effective_modulus_mt_orchestrator(30.0);
        assert_eq!(e0, e1);
    }

    #[test]
    fn b1_e_eff_sweep_monotonic_positive() {
        for fc in [6.0, 20.0, 40.0, 60.0] {
            let e_eff = compute_effective_modulus_mt_orchestrator(fc);
            assert!(e_eff.is_finite() && e_eff > 0.0, "fc={fc}");
        }
    }
}
