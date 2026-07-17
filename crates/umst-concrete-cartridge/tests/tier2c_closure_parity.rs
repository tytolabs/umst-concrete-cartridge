// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! W9 Tier-2c closure parity: cartridge SSOT vs kernel injection paths (byte-equiv).

use umst_concrete_cartridge::{
    cement_reaction_extent_kinetics_spec, CementMaterialParams, CEMENT_DEFAULT_S_INTRINSIC_MPA,
    CEMENT_REACTION_ENTHALPY_J_PER_KG,
};
use umst_manifold::core::{
    MaterialTransitionParams, ReactionExtentKineticsSpec, SubstrateMaterialParams,
};
use umst_manifold::gate::{
    thermodynamic_transition_admissible, transition_outcome, AdmissibilityVerdict,
    ThermodynamicGate, ThermodynamicStateSnapshot, TransitionFilter, TRANSITION_TOLERANCE,
};
use umst_manifold::physics::solvers::ReactionExtentKinetics;

/// Injects cartridge SSOT scalars through the substrate-neutral trait slot (not `CementMaterialParams`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SsotInjectedParams;

impl MaterialTransitionParams for SsotInjectedParams {
    fn reaction_enthalpy_j_per_kg(&self) -> f64 {
        CEMENT_REACTION_ENTHALPY_J_PER_KG
    }

    fn default_intrinsic_strength_mpa(&self) -> f64 {
        CEMENT_DEFAULT_S_INTRINSIC_MPA
    }

    fn reaction_extent_kinetics_spec(&self) -> ReactionExtentKineticsSpec {
        cement_reaction_extent_kinetics_spec()
    }
}

#[test]
fn gate_byte_equiv_cement_vs_ssot_injected_substrate() {
    let w_c = 0.5;
    let alpha_old = 0.3;
    let alpha_new = 0.5;
    let temp = 293.0;
    let dt = 3600.0;

    let cement = CementMaterialParams;
    let injected = SsotInjectedParams;

    let old_a = ThermodynamicStateSnapshot::from_mix_with_params(w_c, alpha_old, temp, &cement);
    let new_a = ThermodynamicStateSnapshot::from_mix_with_params(w_c, alpha_new, temp, &cement);
    let old_b = ThermodynamicStateSnapshot::from_mix_with_params(w_c, alpha_old, temp, &injected);
    let new_b = ThermodynamicStateSnapshot::from_mix_with_params(w_c, alpha_new, temp, &injected);

    let mut filter = TransitionFilter::new();
    let out_a = transition_outcome(&old_a, &new_a, dt, TRANSITION_TOLERANCE);
    let out_b = transition_outcome(&old_b, &new_b, dt, TRANSITION_TOLERANCE);
    // Telemetry path must agree with pure evaluator.
    let out_a_filter = filter.check_transition(&old_a, &new_a, dt);
    assert_eq!(out_a.is_accepted(), out_a_filter.is_accepted());
    assert!((out_a.dissipation - out_a_filter.dissipation).abs() < 1e-12);

    assert_eq!(out_a.is_accepted(), out_b.is_accepted());
    assert!((out_a.dissipation - out_b.dissipation).abs() < 1e-12);
}

#[test]
fn thmc_kinetics_parity_cement_params_vs_ssot_fn() {
    let from_fn = ReactionExtentKinetics::from(cement_reaction_extent_kinetics_spec());
    let from_cement =
        ReactionExtentKinetics::from(CementMaterialParams.reaction_extent_kinetics_spec());
    assert_eq!(
        from_fn.arrhenius_prefactor_s,
        from_cement.arrhenius_prefactor_s
    );
    assert_eq!(
        from_fn.activation_energy_j_per_mol,
        from_cement.activation_energy_j_per_mol
    );
    assert_eq!(
        from_fn.exothermic_k_per_alpha_rate,
        from_cement.exothermic_k_per_alpha_rate
    );
}

#[test]
fn substrate_contrast_zero_enthalpy() {
    let substrate = SubstrateMaterialParams;
    assert_eq!(substrate.reaction_enthalpy_j_per_kg(), 0.0);
    assert_eq!(substrate.default_intrinsic_strength_mpa(), 0.0);
}

#[test]
fn forward_hydration_admissible_thermodynamic_gate() {
    let mut gate = ThermodynamicGate::new();
    let params = CementMaterialParams;
    let old =
        umst_manifold::gate::ThermodynamicState::from_mix_with_params(0.5, 0.3, 293.0, &params);
    let new =
        umst_manifold::gate::ThermodynamicState::from_mix_with_params(0.5, 0.5, 293.0, &params);
    let r = gate.check_transition(&old, &new, 3600.0);
    assert!(r.is_accepted());
    assert!(r.dissipation > 0.0);
}

#[test]
fn admissible_forward_reaction_extent() {
    let mut filter = TransitionFilter::new();
    let params = CementMaterialParams;
    let old = ThermodynamicStateSnapshot::from_mix_with_params(0.5, 0.3, 293.0, &params);
    let new = ThermodynamicStateSnapshot::from_mix_with_params(0.5, 0.5, 293.0, &params);
    assert!(new.free_energy < old.free_energy);
    let r = filter.check_transition(&old, &new, 3600.0);
    assert!(r.is_accepted());
    assert!(r.dissipation > 0.0);
    assert_eq!(r.verdict(), AdmissibilityVerdict::Accepted);
}

#[test]
fn reject_reverse_reaction_extent() {
    let mut filter = TransitionFilter::new();
    let params = CementMaterialParams;
    let old = ThermodynamicStateSnapshot::from_mix_with_params(0.5, 0.7, 293.0, &params);
    let new = ThermodynamicStateSnapshot::from_mix_with_params(0.5, 0.3, 293.0, &params);
    let r = filter.check_transition(&old, &new, 3600.0);
    assert!(!r.is_accepted());
    assert!(r.dissipation < 0.0);
}

#[test]
fn strength_monotonicity_rejects_strength_drop() {
    let mut filter = TransitionFilter::new();
    let mut old = ThermodynamicStateSnapshot::new_idle();
    old.strength = 30.0;
    old.reaction_extent = 0.5;
    let mut new = ThermodynamicStateSnapshot::new_idle();
    new.strength = 25.0;
    new.reaction_extent = 0.5;
    let r = filter.check_transition(&old, &new, 1.0);
    assert!(!r.is_accepted());
}

#[test]
fn pure_gate_matches_filter_forward_reaction_extent() {
    let params = CementMaterialParams;
    let old = ThermodynamicStateSnapshot::from_mix_with_params(0.5, 0.3, 293.0, &params);
    let new = ThermodynamicStateSnapshot::from_mix_with_params(0.5, 0.5, 293.0, &params);
    assert!(thermodynamic_transition_admissible(
        old.density,
        old.free_energy,
        old.reaction_extent,
        old.strength,
        new.density,
        new.free_energy,
        new.reaction_extent,
        new.strength,
        CEMENT_DEFAULT_S_INTRINSIC_MPA,
        3600.0,
    ));
}

#[test]
fn pure_gate_rejects_reverse_reaction_extent() {
    let params = CementMaterialParams;
    let old = ThermodynamicStateSnapshot::from_mix_with_params(0.5, 0.7, 293.0, &params);
    let new = ThermodynamicStateSnapshot::from_mix_with_params(0.5, 0.3, 293.0, &params);
    assert!(!thermodynamic_transition_admissible(
        old.density,
        old.free_energy,
        old.reaction_extent,
        old.strength,
        new.density,
        new.free_energy,
        new.reaction_extent,
        new.strength,
        CEMENT_DEFAULT_S_INTRINSIC_MPA,
        3600.0,
    ));
}

#[test]
fn dissipation_matches_rho_q_alpha_dot() {
    let mut filter = TransitionFilter::new();
    let w_c = 0.45;
    let alpha_old = 0.4;
    let alpha_new = 0.6;
    let dt = 7.0 * 86400.0;
    let params = CementMaterialParams;
    let old = ThermodynamicStateSnapshot::from_mix_with_params(w_c, alpha_old, 293.0, &params);
    let new = ThermodynamicStateSnapshot::from_mix_with_params(w_c, alpha_new, 293.0, &params);
    let r = filter.check_transition(&old, &new, dt);

    let rho = (old.density + new.density) / 2.0;
    let alpha_dot = (alpha_new - alpha_old) / dt;
    let expected = rho * CEMENT_REACTION_ENTHALPY_J_PER_KG * alpha_dot;
    let rel_err = ((r.dissipation - expected) / expected).abs();
    assert!(
        rel_err < 1e-10,
        "got {} expected {}",
        r.dissipation,
        expected
    );
}
