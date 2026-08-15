// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Cluster A–H boundary parity — adapter delegates match pre-lift literals and
//! `umst-chem` SSOT where lifted. Cluster H: H-01, H-02, H-03, and H-07 lifted.

use umst_cartridge_concrete::dissipation_modulus_eta_from_s_intrinsic;
use umst_chem::{
    csh_gel_modulus_scales, csh_ld_volume_fraction, csh_paste_bulk_modulus_voigt_gpa,
    csh_youngs_moduli_gpa, desiccation_params, dlvo_params, gel_space_ratio,
    hydration_degree_calibrated as chem_hydration_degree_calibrated,
    jennings_capillary_porosity_clamped, jennings_compressive_strength,
    kinetics::ReactionExtentKineticsSpec as ChemKineticsSpec,
    powers::powers_capillary_water_volume, powers_capillary_porosity, powers_gel_volume,
    set_time_activation_energy_j_per_mol, ultimate_degree_of_hydration, vinet_pressure_gpa,
    voigt_bulk_modulus_gpa, HydrationKineticsBundle, PowersIntrinsicStrength, SpeciesId,
    BOLTZMANN_J_PER_K, CEMENT_VOLUME_PER_WC, CRITICAL_WC, CSH_HD_SCALE_OF_BULK,
    CSH_LD_FRAC_INTERCEPT, CSH_LD_FRAC_SLOPE, CSH_LD_SCALE_OF_BULK, CSH_VOLUME_FACTOR,
    DEBYE_PREFACTOR_NM, DESICCATION_RH_DROP_SCALE, DIELECTRIC_WATER, DLVO_COLLAPSE_SEPARATION_NM,
    DLVO_REFERENCE_TEMPERATURE_K, GAS_CONSTANT_J_PER_MOL_K, HAMAKER_CEMENT_WATER_J,
    JENNINGS_STRENGTH_EXPONENT_DEFAULT, KELVIN_CAPILLARY_SCALE_MPA, NANO_HEALING_BOOST_PER_DOSAGE,
    NANO_SSA_REF_M2_PER_G, NUCLEATION_BETA_MIN_PER_DECADE, OPC_REACTION_ENTHALPY_J_PER_KG,
    POWERS_GEL_VOLUME_FACTOR, POWERS_NON_EVAP_WATER_COEFF, POWERS_PASTE_DENOMINATOR_OFFSET,
    POZZOLANIC_ALPHA, VACUUM_PERMITTIVITY,
};
use umst_concrete_cartridge::chem_adapter::{
    cartridge_default_intrinsic_strength_mpa, cement_reaction_enthalpy_j_per_kg,
    cement_reaction_extent_kinetics_spec, cement_volume_per_wc_f32,
    chemo_diffusion_weight_scale_f32, clinker_bulk_modulus_ambient_gpa_f32,
    clinker_vinet_params_f32, critical_wc, critical_wc_f32, csh_hd_scale_of_bulk_f32,
    csh_ld_frac_intercept_subtrahend_f32, csh_ld_frac_slope_f32, csh_ld_scale_of_bulk_f32,
    csh_ld_volume_fraction_f32, csh_volume_factor_f32, csh_youngs_moduli_from_k0_f32,
    desiccation_rh_drop_scale, desiccation_rh_drop_scale_f32,
    dissipation_modulus_eta_from_s_intrinsic_mpa, dlvo_boltzmann_f32, dlvo_boltzmann_j_per_k,
    dlvo_collapse_separation_f32, dlvo_collapse_separation_nm, dlvo_debye_prefactor_f32,
    dlvo_debye_prefactor_nm, dlvo_dielectric_water, dlvo_dielectric_water_f32, dlvo_hamaker_f32,
    dlvo_hamaker_j, dlvo_reference_temperature_f32, dlvo_reference_temperature_k,
    dlvo_vacuum_permittivity, dlvo_vacuum_permittivity_f32, e_to_fc_stiffness_bridge_f32,
    gas_constant_f32, gas_constant_j_per_mol_k, gel_space_ratio_f32,
    hydration_activation_over_r_f32, hydration_alpha_max_opc_f32,
    hydration_alpha_max_scm_slope_f32, hydration_degree_calibrated, hydration_k_ref_f32,
    hydration_scm_rate_slope_f32, hydration_t_ref_k_f32, jennings_capillary_porosity_clamped_f32,
    jennings_compressive_strength_f32, jennings_strength_exponent_default,
    kelvin_capillary_scale_mpa, kelvin_capillary_scale_mpa_f32, nano_cartridge_calibration,
    nano_deferred_kinetics_pins, nano_healing_boost_per_dosage_f32, nano_inventory_disposition,
    nano_nucleation_beta_min_per_decade_f32, nano_optimal_dosage_pct_f32,
    nano_pore_refinement_delta_f32, nano_pozzolanic_alpha_f32, nano_ssa_ref_m2_per_g_f32,
    nano_strength_gamma_f32, paste_bulk_modulus_voigt_from_wc_gpa, powers_capillary_porosity_f32,
    powers_capillary_water_volume_f32, powers_compressive_strength_f32, powers_gel_volume_f32,
    powers_gel_volume_factor_f32, powers_non_evap_water_coeff_f32,
    powers_paste_denominator_offset_f32, reaction_gibbs_opc_hydration_joules,
    set_time_activation_energy_f32, ultimate_degree_of_hydration_f32, vinet_pressure_gpa_f32,
    voigt_bulk_modulus_gpa_f32, ClinkerPhaseTag, NanoChemLiftDisposition,
    ADIABATIC_TEMP_RISE_PER_ALPHA, CHEM_AFFINITY_EXPONENT, CLUSTER_H_INVENTORY_MANIFEST,
    THERMO_REF_RATE,
};
use umst_concrete_cartridge::{
    calibration::{ModelKind, Profile},
    cement_reaction_extent_kinetics_spec as material_transition_kinetics_spec,
    physics::optical::paste_bulk_modulus_voigt_from_wc_gpa as optical_paste_bulk_modulus_voigt_from_wc_gpa,
    CEMENT_REACTION_ENTHALPY_J_PER_KG,
};

const EPS: f64 = 1e-12;
const EPS_F32: f64 = 1e-5;
/// f32 cast slack for Vinet V₀ rows — A-01 AliteM3 364.2 Å³/f.u. round-trips at ~1.22e-5.
/// Cross-ref: `physics::clinker_eos::tests::VINET_F32_ABS_TOL` · prep §4.2 f32 boundary.
const VINET_F32_ABS_TOL: f64 = 2e-5;

#[test]
fn cluster_a_stoichiometry_coefficients_match_chem_ssot() {
    assert!((f64::from(powers_gel_volume_factor_f32()) - POWERS_GEL_VOLUME_FACTOR).abs() < EPS_F32);
    assert!(
        (f64::from(powers_non_evap_water_coeff_f32()) - POWERS_NON_EVAP_WATER_COEFF).abs()
            < EPS_F32
    );
    assert!(
        (f64::from(powers_paste_denominator_offset_f32()) - POWERS_PASTE_DENOMINATOR_OFFSET).abs()
            < EPS_F32
    );
}

#[test]
fn cluster_a_gel_volume_matches_homogeneous_literal() {
    let alpha = 0.55_f32;
    let expected = POWERS_GEL_VOLUME_FACTOR * f64::from(alpha);
    assert!((f64::from(powers_gel_volume_f32(alpha)) - expected).abs() < EPS_F32);
    assert!(
        (f64::from(powers_gel_volume_f32(alpha)) - powers_gel_volume(f64::from(alpha))).abs()
            < EPS_F32
    );
}

#[test]
fn cluster_a_capillary_water_volume_matches_homogeneous_literal() {
    let wc = 0.45_f32;
    let alpha = 0.55_f32;
    let expected = f64::from(wc) - POWERS_NON_EVAP_WATER_COEFF * f64::from(alpha);
    assert!((f64::from(powers_capillary_water_volume_f32(wc, alpha)) - expected).abs() < EPS_F32);
    assert!(
        (f64::from(powers_capillary_water_volume_f32(wc, alpha))
            - powers_capillary_water_volume(f64::from(wc), f64::from(alpha)))
        .abs()
            < EPS_F32
    );
}

#[test]
fn cluster_a_capillary_porosity_matches_homogeneous_literal() {
    let wc = 0.45_f32;
    let alpha = 0.55_f32;
    let expected = powers_capillary_porosity(f64::from(alpha), f64::from(wc));
    let adapter = powers_capillary_porosity_f32(wc, alpha);
    assert!((f64::from(adapter) - expected).abs() < EPS_F32);
    let legacy = ((wc - 0.36 * alpha) / (wc + 0.32)).clamp(0.0, 1.0);
    assert!((f64::from(adapter) - f64::from(legacy)).abs() < EPS_F32);
}

#[test]
fn cluster_a_ultimate_doh_matches_mills_closure() {
    let wc = 0.45_f32;
    let expected = 1.031 * f64::from(wc) / (0.194 + f64::from(wc));
    assert!((f64::from(ultimate_degree_of_hydration_f32(wc)) - expected).abs() < EPS_F32);
    assert!(
        (f64::from(ultimate_degree_of_hydration_f32(wc))
            - ultimate_degree_of_hydration(f64::from(wc)))
        .abs()
            < EPS_F32
    );
}

#[test]
fn cluster_a_gel_space_ratio_matches_powers_closure() {
    let wc = 0.45_f32;
    let alpha = 0.55_f32;
    let expected = gel_space_ratio(f64::from(alpha), f64::from(wc));
    assert!((f64::from(gel_space_ratio_f32(wc, alpha)) - expected).abs() < EPS_F32);
}

#[test]
fn cluster_a_powers_strength_matches_gel_space_cube() {
    let wc = 0.45_f32;
    let alpha = 0.55_f32;
    let voids = 0.02_f32;
    let s_intrinsic = 234.0_f32;
    let x = gel_space_ratio_f32(wc, alpha);
    let expected = s_intrinsic * x.powi(3);
    assert!(
        (powers_compressive_strength_f32(wc, alpha, voids, s_intrinsic) - expected).abs()
            < EPS_F32 as f32
    );
}

#[test]
fn cluster_a_cartridge_intrinsic_strength_matches_chem_ssot() {
    assert!(
        (cartridge_default_intrinsic_strength_mpa()
            - PowersIntrinsicStrength::tabulated().cartridge_default_mpa)
            .abs()
            < EPS
    );
}

#[test]
fn cluster_b_bundle_scalars_match_chem_ssot() {
    let bundle = HydrationKineticsBundle::opc_default();
    assert!((f64::from(hydration_alpha_max_opc_f32()) - bundle.alpha_max_opc).abs() < EPS_F32);
    assert!(
        (f64::from(hydration_alpha_max_scm_slope_f32()) - bundle.alpha_max_scm_slope).abs()
            < EPS_F32
    );
    assert!((f64::from(hydration_k_ref_f32()) - bundle.k_ref).abs() < EPS_F32);
    assert!(
        (f64::from(hydration_activation_over_r_f32()) - bundle.activation_over_r).abs() < EPS_F32
    );
    assert!((f64::from(hydration_scm_rate_slope_f32()) - bundle.scm_rate_slope).abs() < EPS_F32);
    assert!((f64::from(hydration_t_ref_k_f32()) - bundle.t_ref_k).abs() < EPS_F32);
}

#[test]
fn cluster_b_alpha_max_matches_formulas_literal() {
    let scm = 0.25_f32;
    let expected = 0.95 - 0.15 * f64::from(scm);
    assert!(
        (HydrationKineticsBundle::opc_default().alpha_max(f64::from(scm)) - expected).abs()
            < EPS_F32
    );
    let via_adapter = f64::from(hydration_alpha_max_opc_f32())
        - f64::from(scm) * f64::from(hydration_alpha_max_scm_slope_f32());
    assert!((via_adapter - expected).abs() < EPS_F32);
}

#[test]
fn cluster_b_scm_factor_matches_formulas_literal() {
    let scm = 0.25_f32;
    let expected = 1.0 - 0.4 * f64::from(scm);
    assert!(
        (HydrationKineticsBundle::opc_default().scm_factor(f64::from(scm)) - expected).abs()
            < EPS_F32
    );
    let via_adapter = 1.0 - f64::from(scm) * f64::from(hydration_scm_rate_slope_f32());
    assert!((via_adapter - expected).abs() < EPS_F32);
}

#[test]
fn cluster_b_set_time_activation_energy_matches_chem_ssot() {
    assert!((f64::from(set_time_activation_energy_f32()) - 40_000.0).abs() < EPS_F32);
    assert!((set_time_activation_energy_j_per_mol() - 40_000.0).abs() < f64::EPSILON);
}

#[test]
fn cluster_b_hydration_degree_calibrated_matches_chem_closure() {
    let age = 7.0_f32;
    let temp = 23.0_f32;
    let scm = 0.1_f32;
    let mult = 1.0_f32;
    let bundle = HydrationKineticsBundle::opc_default();
    let chem = chem_hydration_degree_calibrated(
        f64::from(age),
        f64::from(temp),
        f64::from(scm),
        f64::from(mult),
        &bundle,
    );
    let adapter = hydration_degree_calibrated(age, temp, scm, mult);
    assert!((f64::from(adapter) - chem).abs() < EPS_F32);
    assert!((0.0..=1.0).contains(&chem));
}

#[test]
fn cluster_b_formulas_path_delegates_through_adapter() {
    use umst_concrete_cartridge::formulas;
    let age = 28.0_f32;
    let temp = 20.0_f32;
    let scm = 0.0_f32;
    let mult = 1.0_f32;
    assert!(
        (formulas::hydration_degree_calibrated(age, temp, scm, mult)
            - hydration_degree_calibrated(age, temp, scm, mult))
        .abs()
            < EPS_F32 as f32
    );
}

#[test]
fn cluster_c_enthalpy_matches_chem_ssot() {
    assert!((cement_reaction_enthalpy_j_per_kg() - OPC_REACTION_ENTHALPY_J_PER_KG).abs() < EPS);
    assert!((CEMENT_REACTION_ENTHALPY_J_PER_KG - 450.0).abs() < EPS);
}

#[test]
fn cluster_c_reaction_gibbs_matches_gate_lean() {
    // ψ(α) = −Q_hyd·α at reference — inventory C-03.
    let gibbs = reaction_gibbs_opc_hydration_joules();
    assert!((gibbs + OPC_REACTION_ENTHALPY_J_PER_KG).abs() < EPS);
}

#[test]
fn cluster_c_kinetics_spec_matches_chem_core() {
    let adapter = cement_reaction_extent_kinetics_spec();
    let material = material_transition_kinetics_spec();
    let chem = ChemKineticsSpec::cement_default();

    assert!(
        (f64::from(adapter.arrhenius_prefactor_s) - chem.arrhenius_prefactor_s).abs() < EPS_F32
    );
    assert!(
        (f64::from(adapter.activation_energy_j_per_mol) - chem.activation_energy_j_per_mol).abs()
            < EPS_F32
    );
    assert!(
        (f64::from(adapter.gas_constant_j_per_mol_k) - chem.gas_constant_j_per_mol_k).abs()
            < EPS_F32
    );
    assert!((f64::from(adapter.t_min_k) - chem.t_min_k).abs() < EPS_F32);
    assert!((f64::from(adapter.t_boost_ref_k) - chem.t_boost_ref_k).abs() < EPS_F32);
    assert!((f64::from(adapter.t_boost_per_k) - chem.t_boost_per_k).abs() < EPS_F32);
    assert!(
        (f64::from(adapter.exothermic_k_per_alpha_rate) - chem.exothermic_k_per_alpha_rate).abs()
            < EPS_F32
    );
    assert_eq!(adapter, material);
}

#[test]
fn cluster_c_gas_constant_f32_bridge() {
    assert!((gas_constant_j_per_mol_k() - GAS_CONSTANT_J_PER_MOL_K).abs() < 1e-6);
    assert!((f64::from(gas_constant_f32()) - GAS_CONSTANT_J_PER_MOL_K).abs() < 1e-5);
}

#[test]
fn cluster_c_thermo_cartridge_constants_held() {
    assert!((f64::from(CHEM_AFFINITY_EXPONENT) - 1.5).abs() < f64::EPSILON);
    assert!((f64::from(THERMO_REF_RATE) - 1e6).abs() < 1.0);
    assert!((f64::from(ADIABATIC_TEMP_RISE_PER_ALPHA) - 50.0).abs() < f64::EPSILON);
}

#[test]
fn cluster_c_dissipation_modulus_eta_matches_enthalpy_bridge() {
    for s_intrinsic in [40.0_f64, 80.0, 120.0] {
        let adapter = dissipation_modulus_eta_from_s_intrinsic_mpa(s_intrinsic);
        let consumer = dissipation_modulus_eta_from_s_intrinsic(s_intrinsic);
        let expected = OPC_REACTION_ENTHALPY_J_PER_KG * s_intrinsic * 1e6;
        assert!((adapter - expected).abs() < EPS);
        assert!(
            (adapter - consumer).abs() < EPS,
            "consumer compose must thin-delegate enthalpy SSOT"
        );
    }
}

#[test]
fn cluster_d_vinet_table_matches_chem_ssot() {
    for (tag, species) in [
        (ClinkerPhaseTag::AliteM3, SpeciesId::AliteM3),
        (ClinkerPhaseTag::BeliteBetaC2s, SpeciesId::BeliteBetaC2s),
        (ClinkerPhaseTag::Portlandite, SpeciesId::Portlandite),
        (ClinkerPhaseTag::Ettringite, SpeciesId::Ettringite),
        (
            ClinkerPhaseTag::Csh14nmTobermorite,
            SpeciesId::CshTobermorite14nm,
        ),
    ] {
        let adapter = clinker_vinet_params_f32(tag);
        let ssot = species.vinet_params();
        // V₀ uses VINET_F32_ABS_TOL (not EPS_F32): f64 SSOT → f32 cartridge boundary.
        assert!(
            (f64::from(adapter.v0_per_fu_ang3) - ssot.v0_per_fu_ang3).abs() < VINET_F32_ABS_TOL
        );
        assert!((f64::from(adapter.bulk_modulus_gpa) - ssot.bulk_modulus_gpa).abs() < EPS_F32);
        assert!((f64::from(adapter.k0_prime) - ssot.k0_prime).abs() < EPS_F32);
        assert!(
            (f64::from(clinker_bulk_modulus_ambient_gpa_f32(tag)) - ssot.bulk_modulus_gpa).abs()
                < EPS_F32
        );
    }
}

#[test]
fn cluster_d_vinet_pressure_matches_chem_closure() {
    let p = clinker_vinet_params_f32(ClinkerPhaseTag::AliteM3);
    let adapter = vinet_pressure_gpa_f32(
        p.v0_per_fu_ang3,
        p.bulk_modulus_gpa,
        p.k0_prime,
        p.v0_per_fu_ang3 * 0.97,
    );
    let chem = vinet_pressure_gpa(
        f64::from(p.v0_per_fu_ang3),
        f64::from(p.bulk_modulus_gpa),
        f64::from(p.k0_prime),
        f64::from(p.v0_per_fu_ang3 * 0.97),
    );
    assert!((f64::from(adapter) - chem).abs() < EPS_F32);
    assert!(chem > 0.0);
}

#[test]
fn cluster_d_voigt_bulk_modulus_matches_chem_closure() {
    let k_csh = clinker_bulk_modulus_ambient_gpa_f32(ClinkerPhaseTag::Csh14nmTobermorite);
    let k_ld = k_csh * csh_ld_scale_of_bulk_f32();
    let k_hd = k_csh * csh_hd_scale_of_bulk_f32();
    let fv = 0.45_f32;
    let adapter = voigt_bulk_modulus_gpa_f32(fv, k_ld, k_hd);
    let chem = voigt_bulk_modulus_gpa(f64::from(fv), f64::from(k_ld), f64::from(k_hd));
    assert!((f64::from(adapter) - chem).abs() < EPS_F32);
}

#[test]
fn cluster_e_csh_gel_modulus_scales_match_chem_ssot() {
    let scales = csh_gel_modulus_scales();
    assert!((f64::from(csh_ld_scale_of_bulk_f32()) - scales.ld_of_bulk).abs() < EPS_F32);
    assert!((f64::from(csh_hd_scale_of_bulk_f32()) - scales.hd_of_bulk).abs() < EPS_F32);
    assert!((scales.ld_of_bulk - CSH_LD_SCALE_OF_BULK).abs() < EPS);
    assert!((scales.hd_of_bulk - CSH_HD_SCALE_OF_BULK).abs() < EPS);
}

#[test]
fn cluster_e_ld_volume_fraction_matches_jennings_fit() {
    let wc = 0.45_f32;
    let expected = (CSH_LD_FRAC_SLOPE * f64::from(wc) + CSH_LD_FRAC_INTERCEPT).clamp(0.0, 1.0);
    assert!((f64::from(csh_ld_volume_fraction_f32(wc)) - expected).abs() < EPS_F32);
    assert!((csh_ld_volume_fraction(f64::from(wc)) - expected).abs() < EPS);
    assert!((f64::from(csh_ld_frac_slope_f32()) - CSH_LD_FRAC_SLOPE).abs() < EPS_F32);
    assert!(
        (f64::from(csh_ld_frac_intercept_subtrahend_f32()) + CSH_LD_FRAC_INTERCEPT).abs() < EPS_F32
    );
}

#[test]
fn cluster_e_stoichiometry_coefficients_match_chem_ssot() {
    assert!((f64::from(cement_volume_per_wc_f32()) - CEMENT_VOLUME_PER_WC).abs() < EPS_F32);
    assert!((f64::from(csh_volume_factor_f32()) - CSH_VOLUME_FACTOR).abs() < EPS_F32);
    assert!(
        (f64::from(powers_non_evap_water_coeff_f32()) - POWERS_NON_EVAP_WATER_COEFF).abs()
            < EPS_F32
    );
}

#[test]
fn cluster_e_youngs_moduli_match_ulm_constantinides_anchors() {
    let k0 = 70.0_f32;
    let (e_ld, e_hd) = csh_youngs_moduli_from_k0_f32(k0);
    let chem = csh_youngs_moduli_gpa(f64::from(k0));
    assert!((f64::from(e_ld) - chem.0).abs() < EPS_F32);
    assert!((f64::from(e_hd) - chem.1).abs() < EPS_F32);
    assert!((f64::from(e_ld) - 21.7).abs() < 0.5);
    assert!((f64::from(e_hd) - 29.4).abs() < 0.5);
}

#[test]
fn cluster_e_e_to_fc_stiffness_bridge_is_cartridge_witness_e09() {
    // Inventory E-09 — cartridge calibration; must not migrate into `umst-chem` SSOT.
    assert!((f64::from(e_to_fc_stiffness_bridge_f32()) - 0.05).abs() < EPS_F32);
}

#[test]
fn cluster_e_optical_paste_bulk_modulus_matches_chem_voigt_closure() {
    let wc = 0.4_f32;
    let adapter = paste_bulk_modulus_voigt_from_wc_gpa(wc);
    let optical = optical_paste_bulk_modulus_voigt_from_wc_gpa(wc);
    assert!(
        (f64::from(adapter) - f64::from(optical)).abs() < EPS_F32,
        "optical.rs must thin-delegate to chem_adapter closure"
    );

    let chem = csh_paste_bulk_modulus_voigt_gpa(f64::from(wc));
    assert!((f64::from(adapter) - chem).abs() < EPS_F32);
    assert!(f64::from(adapter) > 5.0);
    assert!(f64::from(adapter) < 40.0);
}

#[test]
fn cluster_f_dlvo_constants_match_chem_ssot() {
    let p = dlvo_params();
    assert!((dlvo_hamaker_j() - HAMAKER_CEMENT_WATER_J).abs() < 1e-25);
    assert!((dlvo_hamaker_j() - p.hamaker_j).abs() < 1e-25);
    assert!((f64::from(dlvo_hamaker_f32()) - HAMAKER_CEMENT_WATER_J).abs() < 1e-20);

    assert!((dlvo_dielectric_water() - DIELECTRIC_WATER).abs() < f64::EPSILON);
    assert!((f64::from(dlvo_dielectric_water_f32()) - DIELECTRIC_WATER).abs() < EPS_F32);

    assert!((dlvo_vacuum_permittivity() - VACUUM_PERMITTIVITY).abs() < 1e-20);
    assert!((f64::from(dlvo_vacuum_permittivity_f32()) - VACUUM_PERMITTIVITY).abs() < 1e-15);

    assert!((dlvo_boltzmann_j_per_k() - BOLTZMANN_J_PER_K).abs() < 1e-30);
    assert!((f64::from(dlvo_boltzmann_f32()) - BOLTZMANN_J_PER_K).abs() < 1e-20);

    assert!((dlvo_reference_temperature_k() - DLVO_REFERENCE_TEMPERATURE_K).abs() < f64::EPSILON);
    assert!(
        (f64::from(dlvo_reference_temperature_f32()) - DLVO_REFERENCE_TEMPERATURE_K).abs()
            < EPS_F32
    );

    assert!((dlvo_debye_prefactor_nm() - DEBYE_PREFACTOR_NM).abs() < f64::EPSILON);
    assert!((f64::from(dlvo_debye_prefactor_f32()) - DEBYE_PREFACTOR_NM).abs() < EPS_F32);

    assert!((dlvo_collapse_separation_nm() - DLVO_COLLAPSE_SEPARATION_NM).abs() < f64::EPSILON);
    assert!(
        (f64::from(dlvo_collapse_separation_f32()) - DLVO_COLLAPSE_SEPARATION_NM).abs() < EPS_F32
    );
}

#[test]
fn cluster_f_desiccation_constants_match_chem_ssot() {
    let p = desiccation_params();
    assert!((critical_wc() - CRITICAL_WC).abs() < f64::EPSILON);
    assert!((critical_wc() - p.critical_wc).abs() < f64::EPSILON);
    assert!((f64::from(critical_wc_f32()) - CRITICAL_WC).abs() < EPS_F32);

    assert!((desiccation_rh_drop_scale() - DESICCATION_RH_DROP_SCALE).abs() < f64::EPSILON);
    assert!((desiccation_rh_drop_scale() - p.rh_drop_scale).abs() < f64::EPSILON);
    assert!(
        (f64::from(desiccation_rh_drop_scale_f32()) - DESICCATION_RH_DROP_SCALE).abs() < EPS_F32
    );

    assert!((kelvin_capillary_scale_mpa() - KELVIN_CAPILLARY_SCALE_MPA).abs() < f64::EPSILON);
    assert!((kelvin_capillary_scale_mpa() - p.kelvin_scale_mpa).abs() < f64::EPSILON);
    assert!(
        (f64::from(kelvin_capillary_scale_mpa_f32()) - KELVIN_CAPILLARY_SCALE_MPA).abs() < EPS_F32
    );
}

#[test]
fn cluster_g_chemo_diffusion_weight_is_cartridge_witness() {
    // research_chem_ef §1.4 — porosity blend scale; cartridge policy, not umst-chem SSOT.
    assert!((f64::from(chemo_diffusion_weight_scale_f32()) - 2.0).abs() < EPS_F32);
}

#[test]
fn cluster_h_h01_ssa_ref_delegates_to_chem_ssot() {
    assert!((NANO_SSA_REF_M2_PER_G - 200.0).abs() < EPS);
    assert!((f64::from(nano_ssa_ref_m2_per_g_f32()) - NANO_SSA_REF_M2_PER_G).abs() < EPS_F32);
    assert_eq!(
        nano_inventory_disposition("H-01"),
        Some(NanoChemLiftDisposition::LiftedToChemSsot)
    );
}

#[test]
fn cluster_h_deferred_kinetics_pins_match_nano_rs_literals() {
    let pins = nano_deferred_kinetics_pins();
    assert!((f64::from(pins.ssa_ref_m2_per_g) - 200.0).abs() < EPS_F32);
    assert!((f64::from(pins.pozzolanic_alpha) - POZZOLANIC_ALPHA).abs() < EPS_F32);
    assert!(
        (f64::from(pins.nucleation_beta_min_per_decade) - NUCLEATION_BETA_MIN_PER_DECADE).abs()
            < EPS_F32
    );
    assert!((f64::from(nano_ssa_ref_m2_per_g_f32()) - 200.0).abs() < EPS_F32);
    assert!((f64::from(nano_pozzolanic_alpha_f32()) - POZZOLANIC_ALPHA).abs() < EPS_F32);
    assert!(
        (f64::from(nano_nucleation_beta_min_per_decade_f32()) - NUCLEATION_BETA_MIN_PER_DECADE)
            .abs()
            < EPS_F32
    );
}

#[test]
fn cluster_h_cartridge_retains_manifest_h04_h06() {
    let cal = nano_cartridge_calibration();
    assert!((f64::from(cal.optimal_dosage_pct) - 2.5).abs() < EPS_F32);
    assert!((f64::from(cal.strength_gamma) - 0.15).abs() < EPS_F32);
    assert!((f64::from(cal.pore_refinement_delta) - 5.0).abs() < EPS_F32);
    assert!((f64::from(nano_optimal_dosage_pct_f32()) - 2.5).abs() < EPS_F32);
    assert!((f64::from(nano_strength_gamma_f32()) - 0.15).abs() < EPS_F32);
    assert!((f64::from(nano_pore_refinement_delta_f32()) - 5.0).abs() < EPS_F32);

    for witness in CLUSTER_H_INVENTORY_MANIFEST {
        match witness.row_id {
            "H-01" | "H-02" | "H-03" | "H-07" => {
                assert_eq!(
                    witness.disposition,
                    NanoChemLiftDisposition::LiftedToChemSsot
                );
            }
            "H-04" | "H-05" | "H-06" => {
                assert_eq!(
                    witness.disposition,
                    NanoChemLiftDisposition::CartridgeRetains
                );
            }
            _ => panic!("unexpected cluster H row {}", witness.row_id),
        }
        assert_eq!(
            nano_inventory_disposition(witness.row_id),
            Some(witness.disposition)
        );
    }
}

#[test]
fn cluster_h_h02_pozzolanic_alpha_delegates_to_chem_ssot() {
    assert!((POZZOLANIC_ALPHA - 0.5).abs() < EPS);
    assert!((f64::from(nano_pozzolanic_alpha_f32()) - POZZOLANIC_ALPHA).abs() < EPS_F32);
    assert_eq!(
        nano_deferred_kinetics_pins().pozzolanic_alpha,
        nano_pozzolanic_alpha_f32()
    );
    assert_eq!(
        nano_inventory_disposition("H-02"),
        Some(NanoChemLiftDisposition::LiftedToChemSsot)
    );
}

#[test]
fn cluster_h_h03_nucleation_beta_delegates_to_chem_ssot() {
    assert!((NUCLEATION_BETA_MIN_PER_DECADE - 30.0).abs() < EPS);
    assert!(
        (f64::from(nano_nucleation_beta_min_per_decade_f32()) - NUCLEATION_BETA_MIN_PER_DECADE)
            .abs()
            < EPS_F32
    );
    assert_eq!(
        nano_deferred_kinetics_pins().nucleation_beta_min_per_decade,
        nano_nucleation_beta_min_per_decade_f32()
    );
    assert_eq!(
        nano_inventory_disposition("H-03"),
        Some(NanoChemLiftDisposition::LiftedToChemSsot)
    );
}

#[test]
fn cluster_h_self_heal_nano_boost_h07_lifted_to_chem_ssot() {
    assert!(
        (f64::from(nano_healing_boost_per_dosage_f32()) - NANO_HEALING_BOOST_PER_DOSAGE).abs()
            < EPS_F32
    );
    assert_eq!(
        nano_inventory_disposition("H-07"),
        Some(NanoChemLiftDisposition::LiftedToChemSsot)
    );
}

#[test]
fn cluster_h_deferred_boundary_does_not_block_e_f_clusters() {
    // Cluster H manifest is isolated — E/F chem_adapter sections remain independent.
    assert_eq!(CLUSTER_H_INVENTORY_MANIFEST.len(), 7);
    assert_eq!(
        nano_inventory_disposition("H-01"),
        Some(NanoChemLiftDisposition::LiftedToChemSsot)
    );
    assert_eq!(
        nano_inventory_disposition("H-04"),
        Some(NanoChemLiftDisposition::CartridgeRetains)
    );
    // Shrinkage chem seam (G-04) routes only `critical_wc` — nano literals are OUT-OF-SCOPE.
    assert!((f64::from(critical_wc_f32()) - CRITICAL_WC).abs() < EPS_F32);
}

// ── Cluster I — Jennings homogeneous gel-space (inventory B-19 / B-20) ────────

const G0_W_C: f32 = 0.45;
const G0_ALPHA: f32 = 0.55;
const G0_S_INTRINSIC: f32 = 80.0;

#[test]
fn cluster_i_jennings_exponent_default_matches_b19() {
    assert_eq!(
        jennings_strength_exponent_default(),
        JENNINGS_STRENGTH_EXPONENT_DEFAULT
    );
    assert_eq!(jennings_strength_exponent_default(), 3);
}

#[test]
fn cluster_i_jennings_phi_cap_matches_powers_capillary_porosity() {
    let adapter = jennings_capillary_porosity_clamped_f32(G0_W_C, G0_ALPHA);
    let chem = jennings_capillary_porosity_clamped(f64::from(G0_ALPHA), f64::from(G0_W_C));
    let powers = powers_capillary_porosity(f64::from(G0_ALPHA), f64::from(G0_W_C));
    assert!((f64::from(adapter) - chem).abs() < EPS_F32);
    assert!((f64::from(adapter) - powers).abs() < EPS_F32);
}

#[test]
fn cluster_i_jennings_strength_matches_chem_ssot_closure() {
    let p = jennings_strength_exponent_default();
    let adapter = jennings_compressive_strength_f32(G0_W_C, G0_ALPHA, G0_S_INTRINSIC, p);
    let chem = jennings_compressive_strength(
        f64::from(G0_W_C),
        f64::from(G0_ALPHA),
        f64::from(G0_S_INTRINSIC),
        p,
    );
    assert!((f64::from(adapter) - chem).abs() < EPS_F32);
    assert!(f64::from(adapter) > 0.0);
}

#[test]
fn cluster_i_jennings_strength_monotone_in_alpha() {
    let p = jennings_strength_exponent_default();
    let f_early = jennings_compressive_strength_f32(G0_W_C, 0.2, G0_S_INTRINSIC, p);
    let f_late = jennings_compressive_strength_f32(G0_W_C, 0.6, G0_S_INTRINSIC, p);
    assert!(
        f_late >= f_early,
        "monotone in α per jennings_strength_monotone"
    );
}

#[test]
fn cluster_i_jennings_strength_differs_from_powers_at_g0_pin() {
    let p = jennings_strength_exponent_default();
    let powers = powers_compressive_strength_f32(G0_W_C, G0_ALPHA, 0.02, G0_S_INTRINSIC);
    let jennings = jennings_compressive_strength_f32(G0_W_C, G0_ALPHA, G0_S_INTRINSIC, p);
    assert!(
        (powers - jennings).abs() > 1e-4,
        "parallel witnesses must not collapse"
    );
}

#[test]
fn cluster_i_bundled_jennings_gel_space_profile_loads() {
    let profile = Profile::load_bundled("jennings_gel_space").expect("bundled profile");
    assert_eq!(profile.model_section.kind, ModelKind::JenningsGelSpace);
    assert!((profile.powers.s_intrinsic - 80.0).abs() < f64::EPSILON);
    assert_eq!(profile.contract.verification_status, "Boundary");
    let formal = profile.provenance.formal.as_ref().expect("formal block");
    assert!(formal.anchor.contains("JenningsGelSpace"));
}
