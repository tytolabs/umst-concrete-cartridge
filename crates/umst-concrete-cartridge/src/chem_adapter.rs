// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! M3 T3 A4 chem adapter — **Clusters A–D, E–F, H boundary** (Powers stoichiometry,
//! hydration kinetics, reaction thermochemistry, Vinet phase EOS, C-S-H micromechanics,
//! colloidal / chemo-hydric, nano-silica partial lift).
//!
//! Thin delegate seam: cartridge tensor/transition closures call into `umst-chem` SSOT
//! where lifted. Cluster **H**: H-01, H-02, H-03, and H-07 delegate to `umst-chem` SSOT (A4b-5).
//! Physics closures (Powers strength, Voigt paste bulk) live in `umst-chem` — adapter
//! is f32/f64 cast + cartridge-policy witnesses only.

use umst_chem::{
    csh_gel_modulus_scales, csh_ld_volume_fraction, csh_paste_bulk_modulus_voigt_gpa,
    csh_youngs_moduli_gpa,
    hydration_degree_calibrated as chem_hydration_degree_calibrated,
    kinetics::ReactionExtentKineticsSpec as ChemKineticsSpec,
    powers::{
        gel_space_ratio as chem_gel_space_ratio,
        jennings_capillary_porosity_clamped as chem_jennings_capillary_porosity_clamped,
        jennings_compressive_strength as chem_jennings_compressive_strength,
        powers_capillary_water_volume as chem_powers_capillary_water_volume,
        powers_compressive_strength as chem_powers_compressive_strength,
    },
    powers_capillary_porosity as chem_powers_capillary_porosity,
    powers_gel_volume as chem_powers_gel_volume,
    set_time_activation_energy_j_per_mol,
    ultimate_degree_of_hydration as chem_ultimate_degree_of_hydration,
    vinet_pressure_gpa as chem_vinet_pressure_gpa,
    voigt_bulk_modulus_gpa as chem_voigt_bulk_modulus_gpa,
    BOLTZMANN_J_PER_K, CEMENT_VOLUME_PER_WC, CRITICAL_WC, CSH_LD_FRAC_INTERCEPT, CSH_LD_FRAC_SLOPE,
    CSH_VOLUME_FACTOR, DEBYE_PREFACTOR_NM, DESICCATION_RH_DROP_SCALE, DIELECTRIC_WATER, DLVO_COLLAPSE_SEPARATION_NM,
    HAMAKER_CEMENT_WATER_J, KELVIN_CAPILLARY_SCALE_MPA, POWERS_GEL_VOLUME_FACTOR,
    POWERS_NON_EVAP_WATER_COEFF, POWERS_PASTE_DENOMINATOR_OFFSET, SpeciesId, VACUUM_PERMITTIVITY,
    CementChemService, ChemistryService, GAS_CONSTANT_J_PER_MOL_K, HydrationKineticsBundle,
    OPC_REACTION_ENTHALPY_J_PER_KG, PowersIntrinsicStrength, Reaction, ThermoState,
    DLVO_REFERENCE_TEMPERATURE_K, JENNINGS_STRENGTH_EXPONENT_DEFAULT, NANO_HEALING_BOOST_PER_DOSAGE, NANO_SSA_REF_M2_PER_G,
    NUCLEATION_BETA_MIN_PER_DECADE, POZZOLANIC_ALPHA,
};
use umst_manifold::core::ReactionExtentKineticsSpec;

const HYDRATION_BUNDLE: HydrationKineticsBundle = HydrationKineticsBundle::opc_default();

// ── Cluster A — Powers / Jennings stoichiometry (inventory B-01 … B-08) ──────

/// Gel volume factor 0.68 — inventory B-01.
#[must_use]
pub const fn powers_gel_volume_factor_f32() -> f32 {
    POWERS_GEL_VOLUME_FACTOR as f32
}

/// Paste denominator offset 0.32 — inventory B-06.
#[must_use]
pub const fn powers_paste_denominator_offset_f32() -> f32 {
    POWERS_PASTE_DENOMINATOR_OFFSET as f32
}

/// Gel volume v_g = 0.68·α — inventory B-01.
#[must_use]
pub fn powers_gel_volume_f32(alpha: f32) -> f32 {
    chem_powers_gel_volume(f64::from(alpha)) as f32
}

/// Capillary water volume v_c = w/c − 0.36·α — inventory B-02.
#[must_use]
pub fn powers_capillary_water_volume_f32(water_cement: f32, alpha: f32) -> f32 {
    chem_powers_capillary_water_volume(f64::from(water_cement), f64::from(alpha)) as f32
}

/// Capillary porosity φ_c = (w/c − 0.36·α) / (w/c + 0.32) — inventory B-07.
#[must_use]
pub fn powers_capillary_porosity_f32(water_cement: f32, alpha: f32) -> f32 {
    chem_powers_capillary_porosity(f64::from(alpha), f64::from(water_cement)) as f32
}

/// Non-evaporable water coefficient 0.36 — inventory B-02 / B-05 / E-08.
#[must_use]
pub const fn powers_non_evap_water_coeff_f32() -> f32 {
    POWERS_NON_EVAP_WATER_COEFF as f32
}

/// Gel-space ratio X = v_g / (v_g + v_c + ε) — inventory B-09.
#[must_use]
pub fn gel_space_ratio_f32(water_cement: f32, alpha: f32) -> f32 {
    chem_gel_space_ratio(f64::from(alpha), f64::from(water_cement)) as f32
}

/// Asymptotic ultimate degree of hydration α∞(w/c) — Mills 1966 closure (inventory B-13).
#[must_use]
pub fn ultimate_degree_of_hydration_f32(water_cement: f32) -> f32 {
    chem_ultimate_degree_of_hydration(f64::from(water_cement)) as f32
}

/// f64 bridge — Mills α∞(w/c) for consumer oracles and FFI boundaries (inventory B-13).
#[must_use]
pub fn ultimate_degree_of_hydration_f64(water_cement: f64) -> f64 {
    chem_ultimate_degree_of_hydration(water_cement)
}

/// f64 bridge — gel volume v_g (inventory B-01).
#[must_use]
pub fn powers_gel_volume_f64(alpha: f64) -> f64 {
    chem_powers_gel_volume(alpha)
}

/// f64 bridge — capillary water volume v_c (inventory B-02).
#[must_use]
pub fn powers_capillary_water_volume_f64(water_cement: f64, alpha: f64) -> f64 {
    chem_powers_capillary_water_volume(water_cement, alpha)
}

/// f64 bridge — capillary porosity φ_c (inventory B-07).
#[must_use]
pub fn powers_capillary_porosity_f64(water_cement: f64, alpha: f64) -> f64 {
    chem_powers_capillary_porosity(alpha, water_cement)
}

/// f64 bridge — gel-space ratio X (inventory B-09).
#[must_use]
pub fn gel_space_ratio_f64(water_cement: f64, alpha: f64) -> f64 {
    chem_gel_space_ratio(alpha, water_cement)
}

/// Cartridge intrinsic strength scale (MPa) — inventory B-12; Lean formal = 234 MPa drift documented.
#[must_use]
pub const fn cartridge_default_intrinsic_strength_mpa() -> f64 {
    PowersIntrinsicStrength::tabulated().cartridge_default_mpa
}

/// f32 bridge for C-ABI / tensor strength closures.
#[must_use]
pub const fn cartridge_default_intrinsic_strength_mpa_f32() -> f32 {
    cartridge_default_intrinsic_strength_mpa() as f32
}

/// Powers compressive strength f_c = s_intrinsic · X³ with parameterized void volume (inventory B-09 tail).
///
/// Homogeneous gate path uses `voids_volume = 0.02` paste offset; C-ABI passes `air_content`.
#[must_use]
pub fn powers_compressive_strength_f32(
    water_cement: f32,
    alpha: f32,
    voids_volume: f32,
    intrinsic_strength: f32,
) -> f32 {
    chem_powers_compressive_strength(
        f64::from(water_cement),
        f64::from(alpha),
        f64::from(voids_volume),
        f64::from(intrinsic_strength),
    ) as f32
}

/// Jennings strength exponent `p` default — inventory B-19 (J-O1).
#[must_use]
pub const fn jennings_strength_exponent_default() -> u32 {
    JENNINGS_STRENGTH_EXPONENT_DEFAULT
}

/// Jennings φ_cap clamped — inventory B-10 / B-20 (mirrors Lean `φ_cap`).
#[must_use]
pub fn jennings_capillary_porosity_clamped_f32(water_cement: f32, alpha: f32) -> f32 {
    chem_jennings_capillary_porosity_clamped(f64::from(alpha), f64::from(water_cement)) as f32
}

/// Jennings compressive strength f_c = a · (1 − φ_cap)^p — inventory B-20.
#[must_use]
pub fn jennings_compressive_strength_f32(
    water_cement: f32,
    alpha: f32,
    intrinsic_strength: f32,
    exponent: u32,
) -> f32 {
    chem_jennings_compressive_strength(
        f64::from(water_cement),
        f64::from(alpha),
        f64::from(intrinsic_strength),
        exponent,
    ) as f32
}

// ── Cluster B — hydration kinetics (inventory B-14 … B-18) ───────────────────

/// OPC α_max intercept — inventory B-14.
#[must_use]
pub const fn hydration_alpha_max_opc_f32() -> f32 {
    HYDRATION_BUNDLE.alpha_max_opc as f32
}

/// SCM slope on α_max — inventory B-14.
#[must_use]
pub const fn hydration_alpha_max_scm_slope_f32() -> f32 {
    HYDRATION_BUNDLE.alpha_max_scm_slope as f32
}

/// Reference rate constant k_ref — inventory B-15.
#[must_use]
pub const fn hydration_k_ref_f32() -> f32 {
    HYDRATION_BUNDLE.k_ref as f32
}

/// Arrhenius activation E/R (K) — inventory B-16.
#[must_use]
pub const fn hydration_activation_over_r_f32() -> f32 {
    HYDRATION_BUNDLE.activation_over_r as f32
}

/// SCM slowdown slope on rate — inventory B-17.
#[must_use]
pub const fn hydration_scm_rate_slope_f32() -> f32 {
    HYDRATION_BUNDLE.scm_rate_slope as f32
}

/// Reference temperature (K) for Arrhenius factor — inventory C-21.
#[must_use]
pub const fn hydration_t_ref_k_f32() -> f32 {
    HYDRATION_BUNDLE.t_ref_k as f32
}

/// Calibrated hydration degree α(t) — f64 compute, single f32 cast (parity risk §4.2).
#[must_use]
pub fn hydration_degree_calibrated(
    age_days: f32,
    temp_c: f32,
    scm_ratio: f32,
    k_ref_multiplier: f32,
) -> f32 {
    chem_hydration_degree_calibrated(
        f64::from(age_days),
        f64::from(temp_c),
        f64::from(scm_ratio),
        f64::from(k_ref_multiplier),
        &HYDRATION_BUNDLE,
    ) as f32
}

/// Set-time Arrhenius E_a (J/mol) — inventory B-19 / I-02.
#[must_use]
pub fn set_time_activation_energy_f32() -> f32 {
    set_time_activation_energy_j_per_mol() as f32
}

// ── Cluster C — reaction thermochemistry (inventory C-01 … C-16) ─────────────

/// Cartridge mechanics witness — not chem SSOT (inventory C-11).
const TRANSITION_STIFFNESS_E_SCALE_PA: f64 = 30e9;

/// Cartridge mechanics witness — not chem SSOT (inventory C-12).
const TRANSITION_STIFFNESS_NU: f64 = 0.2;

/// Chem affinity exponent for dissipation closure (inventory C-14).
pub const CHEM_AFFINITY_EXPONENT: f32 = 1.5;

/// Reference rate scale for tensor heat engine (inventory C-15).
pub const THERMO_REF_RATE: f32 = 1e6;

/// Adiabatic temperature rise scale per unit α (inventory C-16).
pub const ADIABATIC_TEMP_RISE_PER_ALPHA: f32 = 50.0;

/// OPC hydration enthalpy (J/kg binder) — inventory C-01, C-02.
#[must_use]
pub const fn cement_reaction_enthalpy_j_per_kg() -> f64 {
    OPC_REACTION_ENTHALPY_J_PER_KG
}

/// ψ(α) reference Gibbs energy for OPC hydration at ambient — inventory C-03.
#[must_use]
pub fn reaction_gibbs_opc_hydration_joules() -> f64 {
    CementChemService::new()
        .reaction_gibbs(&Reaction::OpcHydration, &ThermoState::ambient())
        .as_joules()
}

/// Universal gas constant (J/mol·K) — inventory C-06, C-13.
#[must_use]
pub const fn gas_constant_j_per_mol_k() -> f64 {
    GAS_CONSTANT_J_PER_MOL_K
}

/// f32 bridge for tensor thermo engine — single cast site (parity risk C-13).
#[must_use]
pub const fn gas_constant_f32() -> f32 {
    GAS_CONSTANT_J_PER_MOL_K as f32
}

/// M1 dissipation modulus η [J·s/m³] — inventory C dissipation bridge (η = Q_hyd · s_intrinsic · 1e6).
#[must_use]
pub fn dissipation_modulus_eta_from_s_intrinsic_mpa(s_intrinsic_mpa: f64) -> f64 {
    (cement_reaction_enthalpy_j_per_kg() * s_intrinsic_mpa * 1e6).max(1.0)
}

/// THMC transition gate kinetics witness — chem core + cartridge mechanics tail.
#[must_use]
pub const fn cement_reaction_extent_kinetics_spec() -> ReactionExtentKineticsSpec {
    let chem = ChemKineticsSpec::cement_default();
    ReactionExtentKineticsSpec {
        arrhenius_prefactor_s: chem.arrhenius_prefactor_s as f32,
        activation_energy_j_per_mol: chem.activation_energy_j_per_mol as f32,
        gas_constant_j_per_mol_k: chem.gas_constant_j_per_mol_k as f32,
        t_min_k: chem.t_min_k as f32,
        t_boost_ref_k: chem.t_boost_ref_k as f32,
        t_boost_per_k: chem.t_boost_per_k as f32,
        exothermic_k_per_alpha_rate: chem.exothermic_k_per_alpha_rate as f32,
        stiffness_e_scale_pa: TRANSITION_STIFFNESS_E_SCALE_PA as f32,
        stiffness_nu: TRANSITION_STIFFNESS_NU as f32,
    }
}

// ── Cluster D — Phase EOS / Vinet (inventory A-01 … A-15) ────────────────────

/// Cartridge phase tag for DFT-backed Vinet table rows — inventory A-01…A-15.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClinkerPhaseTag {
    AliteM3,
    BeliteBetaC2s,
    Portlandite,
    Ettringite,
    Csh14nmTobermorite,
}

impl ClinkerPhaseTag {
    const fn species_id(self) -> SpeciesId {
        match self {
            Self::AliteM3 => SpeciesId::AliteM3,
            Self::BeliteBetaC2s => SpeciesId::BeliteBetaC2s,
            Self::Portlandite => SpeciesId::Portlandite,
            Self::Ettringite => SpeciesId::Ettringite,
            Self::Csh14nmTobermorite => SpeciesId::CshTobermorite14nm,
        }
    }
}

/// Reference Vinet parameter triple at the f32 cartridge boundary — inventory A-01…A-15.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VinetPhaseParamsF32 {
    pub v0_per_fu_ang3: f32,
    pub bulk_modulus_gpa: f32,
    pub k0_prime: f32,
}

/// Literature Vinet `(V₀, K₀, K₀′)` via `umst-chem` SSOT — inventory A-01…A-15.
#[must_use]
pub fn clinker_vinet_params_f32(phase: ClinkerPhaseTag) -> VinetPhaseParamsF32 {
    let p = phase.species_id().vinet_params();
    VinetPhaseParamsF32 {
        v0_per_fu_ang3: p.v0_per_fu_ang3 as f32,
        bulk_modulus_gpa: p.bulk_modulus_gpa as f32,
        k0_prime: p.k0_prime as f32,
    }
}

/// Ambient bulk modulus K₀ (GPa) from the Vinet fit — inventory A-02/A-05/…/A-14.
#[must_use]
pub fn clinker_bulk_modulus_ambient_gpa_f32(phase: ClinkerPhaseTag) -> f32 {
    clinker_vinet_params_f32(phase).bulk_modulus_gpa
}

/// Vinet isothermal pressure P (GPa) — inventory cluster D scalar closure.
#[must_use]
pub fn vinet_pressure_gpa_f32(v0: f32, k0_gpa: f32, k0_prime: f32, v_per_fu_ang3: f32) -> f32 {
    chem_vinet_pressure_gpa(
        f64::from(v0),
        f64::from(k0_gpa),
        f64::from(k0_prime),
        f64::from(v_per_fu_ang3),
    ) as f32
}

/// Voigt upper bound on bulk modulus (GPa) — inventory cluster D homogenisation.
#[must_use]
pub fn voigt_bulk_modulus_gpa_f32(fv_phase_a: f32, k_a: f32, k_b: f32) -> f32 {
    chem_voigt_bulk_modulus_gpa(
        f64::from(fv_phase_a),
        f64::from(k_a),
        f64::from(k_b),
    ) as f32
}

// ── Cluster E — C-S-H micromechanics (inventory E-01 … E-09) ─────────────────

/// LD gel bulk-modulus scale of Vinet C-S-H K₀ — inventory E-01.
#[must_use]
pub const fn csh_ld_scale_of_bulk_f32() -> f32 {
    csh_gel_modulus_scales().ld_of_bulk as f32
}

/// HD gel bulk-modulus scale of Vinet C-S-H K₀ — inventory E-02.
#[must_use]
pub const fn csh_hd_scale_of_bulk_f32() -> f32 {
    csh_gel_modulus_scales().hd_of_bulk as f32
}

/// Jennings LD slope `3.017` — inventory E-05 (tensor path).
#[must_use]
pub const fn csh_ld_frac_slope_f32() -> f32 {
    CSH_LD_FRAC_SLOPE as f32
}

/// Jennings LD intercept subtrahend `0.347` — inventory E-05 (tensor path).
#[must_use]
pub const fn csh_ld_frac_intercept_subtrahend_f32() -> f32 {
    (-CSH_LD_FRAC_INTERCEPT) as f32
}

/// Jennings LD volume fraction `3.017·w/c − 0.347`, clamped — inventory E-05 / E-10.
#[must_use]
pub fn csh_ld_volume_fraction_f32(water_cement: f32) -> f32 {
    csh_ld_volume_fraction(f64::from(water_cement)) as f32
}

/// Cement solid volume basis coefficient `0.317` — inventory E-07.
#[must_use]
pub const fn cement_volume_per_wc_f32() -> f32 {
    CEMENT_VOLUME_PER_WC as f32
}

/// C-S-H gel volume multiplier on cement volume `1.52` — inventory E-06.
#[must_use]
pub const fn csh_volume_factor_f32() -> f32 {
    CSH_VOLUME_FACTOR as f32
}

/// Vinet-anchored Young's moduli `(E_LD, E_HD)` in GPa from reference bulk K₀ — inventory E-01/E-02.
#[must_use]
pub fn csh_youngs_moduli_from_k0_f32(k0_csh_gpa: f32) -> (f32, f32) {
    let (e_ld, e_hd) = csh_youngs_moduli_gpa(f64::from(k0_csh_gpa));
    (e_ld as f32, e_hd as f32)
}

/// Cartridge calibration — E→fc stiffness bridge (inventory **E-09**).
///
/// **Not chem SSOT** — retained in cartridge per
/// `m3_concrete_chem_adapter_prep.md` §2.5 and `a4b_chem_extension_schedule.md` M3-P5-cartridge.
/// Scales effective paste modulus (GPa) into MPa compressive strength at the Jennings tail.
const E_TO_FC_STIFFNESS_BRIDGE: f32 = 0.05;

/// f32 witness for the E→fc stiffness bridge — inventory E-09.
#[must_use]
pub const fn e_to_fc_stiffness_bridge_f32() -> f32 {
    E_TO_FC_STIFFNESS_BRIDGE
}

/// Voigt bulk modulus (GPa) for Jennings-partitioned C-S-H paste — inventory E-03/E-04/E-10
/// with cluster D Vinet `K₀` (optical / strength bridge).
#[must_use]
pub fn paste_bulk_modulus_voigt_from_wc_gpa(wc_ratio: f32) -> f32 {
    csh_paste_bulk_modulus_voigt_gpa(f64::from(wc_ratio)) as f32
}

// ── Cluster F — colloidal / chemo-hydric (inventory F-01 … G-04) ─────────────

/// Hamaker constant for cement–water DLVO (J) — inventory F-01.
#[must_use]
pub const fn dlvo_hamaker_j() -> f64 {
    HAMAKER_CEMENT_WATER_J
}

/// f32 bridge for tensor colloidal engine — single cast site.
#[must_use]
pub const fn dlvo_hamaker_f32() -> f32 {
    HAMAKER_CEMENT_WATER_J as f32
}

/// Relative permittivity of pore water — inventory F-02.
#[must_use]
pub const fn dlvo_dielectric_water() -> f64 {
    DIELECTRIC_WATER
}

/// f32 bridge for tensor ε_r.
#[must_use]
pub const fn dlvo_dielectric_water_f32() -> f32 {
    DIELECTRIC_WATER as f32
}

/// Vacuum permittivity ε₀ (F/m) — inventory F-03.
#[must_use]
pub const fn dlvo_vacuum_permittivity() -> f64 {
    VACUUM_PERMITTIVITY
}

/// f32 bridge for tensor ε₀.
#[must_use]
pub const fn dlvo_vacuum_permittivity_f32() -> f32 {
    VACUUM_PERMITTIVITY as f32
}

/// Boltzmann constant (J/K) — inventory F-04.
#[must_use]
pub const fn dlvo_boltzmann_j_per_k() -> f64 {
    BOLTZMANN_J_PER_K
}

/// f32 bridge for tensor k_B.
#[must_use]
pub const fn dlvo_boltzmann_f32() -> f32 {
    BOLTZMANN_J_PER_K as f32
}

/// DLVO reference temperature (K) — inventory F-04.
#[must_use]
pub const fn dlvo_reference_temperature_k() -> f64 {
    DLVO_REFERENCE_TEMPERATURE_K
}

/// f32 bridge for tensor T_ref.
#[must_use]
pub const fn dlvo_reference_temperature_f32() -> f32 {
    DLVO_REFERENCE_TEMPERATURE_K as f32
}

/// Debye length prefactor κ⁻¹ ∝ prefactor/√I (nm) — inventory F-05.
#[must_use]
pub const fn dlvo_debye_prefactor_nm() -> f64 {
    DEBYE_PREFACTOR_NM
}

/// f32 bridge for tensor Debye prefactor.
#[must_use]
pub const fn dlvo_debye_prefactor_f32() -> f32 {
    DEBYE_PREFACTOR_NM as f32
}

/// Colloidal collapse separation threshold (nm) — inventory F-06.
#[must_use]
pub const fn dlvo_collapse_separation_nm() -> f64 {
    DLVO_COLLAPSE_SEPARATION_NM
}

/// f32 bridge for tensor collapse mask.
#[must_use]
pub const fn dlvo_collapse_separation_f32() -> f32 {
    DLVO_COLLAPSE_SEPARATION_NM as f32
}

// ── Cluster F — cartridge policy witnesses (not umst-chem SSOT) ──────────────

/// Cartridge tensor numerics witness — minimum separation clamp (nm).
const DLVO_TENSOR_SEP_FLOOR_NM: f32 = 0.1;

/// Cartridge tensor numerics witness — minimum ionic strength clamp (M).
const DLVO_TENSOR_IONIC_FLOOR_M: f32 = 0.001;

/// Cartridge tensor numerics witness — zeta potential mV→V divisor.
const DLVO_TENSOR_MV_TO_V: f32 = 1000.0;

/// Cartridge tensor numerics witness — collapse mask fill sentinel (kT).
const DLVO_TENSOR_COLLAPSE_SENTINEL_KT: f32 = -999.0;

/// Cartridge rheology witness — DLVO barrier below which flocculation ramps (kT).
const FLOCCULATION_BARRIER_KT: f32 = -5.0;

/// Cartridge rheology witness — yield-stress slope per kT below barrier.
const FLOCCULATION_YIELD_STRESS_SLOPE: f32 = -0.1;

/// Cartridge rheology witness — stable-suspension baseline multiplier.
const FLOCCULATION_MULTIPLIER_BASE: f32 = 1.0;

/// Cartridge rheology witness — maximum flocculation multiplier clamp.
const FLOCCULATION_MULTIPLIER_MAX: f32 = 5.0;

/// DLVO tensor separation floor (nm) — cartridge policy (research_chem_ef §1.4).
#[must_use]
pub const fn dlvo_tensor_sep_floor_nm_f32() -> f32 {
    DLVO_TENSOR_SEP_FLOOR_NM
}

/// DLVO tensor ionic strength floor (M) — cartridge policy.
#[must_use]
pub const fn dlvo_tensor_ionic_floor_m_f32() -> f32 {
    DLVO_TENSOR_IONIC_FLOOR_M
}

/// DLVO tensor zeta potential mV→V divisor — cartridge policy.
#[must_use]
pub const fn dlvo_tensor_mv_to_v_f32() -> f32 {
    DLVO_TENSOR_MV_TO_V
}

/// DLVO tensor collapse mask fill sentinel (kT) — cartridge policy.
#[must_use]
pub const fn dlvo_tensor_collapse_sentinel_kt_f32() -> f32 {
    DLVO_TENSOR_COLLAPSE_SENTINEL_KT
}

/// Flocculation DLVO barrier threshold (kT) — cartridge policy (research_chem_ef §1.4).
#[must_use]
pub const fn flocculation_barrier_kt_f32() -> f32 {
    FLOCCULATION_BARRIER_KT
}

/// Flocculation yield-stress slope per kT below barrier — cartridge policy.
#[must_use]
pub const fn flocculation_yield_stress_slope_f32() -> f32 {
    FLOCCULATION_YIELD_STRESS_SLOPE
}

/// Stable suspension baseline flocculation multiplier — cartridge policy.
#[must_use]
pub const fn flocculation_multiplier_base_f32() -> f32 {
    FLOCCULATION_MULTIPLIER_BASE
}

/// Maximum flocculation multiplier clamp — cartridge policy.
#[must_use]
pub const fn flocculation_multiplier_max_f32() -> f32 {
    FLOCCULATION_MULTIPLIER_MAX
}

/// Critical w/c for self-desiccation onset — inventory G-01 (DUP G-04 shrinkage).
#[must_use]
pub const fn critical_wc() -> f64 {
    CRITICAL_WC
}

/// f32 bridge for tensor desiccation / shrinkage closures.
#[must_use]
pub const fn critical_wc_f32() -> f32 {
    CRITICAL_WC as f32
}

/// RH drop scale in desiccation potential — inventory G-02.
#[must_use]
pub const fn desiccation_rh_drop_scale() -> f64 {
    DESICCATION_RH_DROP_SCALE
}

/// f32 bridge for tensor chemo-hydric engine.
#[must_use]
pub const fn desiccation_rh_drop_scale_f32() -> f32 {
    DESICCATION_RH_DROP_SCALE as f32
}

/// Kelvin capillary tension scale (MPa) — inventory G-03.
#[must_use]
pub const fn kelvin_capillary_scale_mpa() -> f64 {
    KELVIN_CAPILLARY_SCALE_MPA
}

/// f32 bridge for tensor Kelvin closure.
#[must_use]
pub const fn kelvin_capillary_scale_mpa_f32() -> f32 {
    KELVIN_CAPILLARY_SCALE_MPA as f32
}

/// Cartridge tensor witness — porosity weight scale for ambient-RH diffusion blend
/// (`chemo_water.rs` moisture transport compose).
const CHEMO_DIFFUSION_WEIGHT_SCALE: f32 = 2.0;

/// Porosity weight scale for ambient-RH diffusion blend — cartridge policy
/// (research_chem_ef §1.4 · G tail; not umst-chem SSOT).
#[must_use]
pub const fn chemo_diffusion_weight_scale_f32() -> f32 {
    CHEMO_DIFFUSION_WEIGHT_SCALE
}

// ── Cluster H — nano-silica / pozzolan (inventory H-01 … H-07) ───────────────
//
// **Lifted to `umst-chem` SSOT (A4b-5):** H-01 → `NANO_SSA_REF_M2_PER_G`; H-02 → `POZZOLANIC_ALPHA`;
// H-03 → `NUCLEATION_BETA_MIN_PER_DECADE`; H-07 → `NANO_HEALING_BOOST_PER_DOSAGE`.
// **TODO-M3-003 CLOSED** @ `archived/residuals/misc-outputs-tmp/RESEARCH_TODO_NIGHT_2334.md` — parity witnesses green.
// **TODO-M3-003b CLOSED** @ 2334 — `DeferredToChemSsot` has zero live manifest rows; `NanoDeferredKineticsPins` name is historical (H-01…H-03 all `LiftedToChemSsot`).
// H-04…H-06 are permanent cartridge calibration per a4b §2.4.

/// Inventory row lift disposition for cluster H boundary witnesses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NanoChemLiftDisposition {
    /// `umst-chem` SSOT wired through adapter delegate.
    LiftedToChemSsot,
    /// Reserved — `umst-chem` SSOT ready but cartridge delegate not yet wired.
    /// **Zero live manifest rows** @ `RESEARCH_TODO_NIGHT_2334` (TODO-M3-003b hygiene).
    DeferredToChemSsot,
    /// Empirical cartridge calibration — not scheduled for chem lift.
    CartridgeRetains,
}

/// Witness row for `chem_adapter_parity` cartridge_retains manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NanoInventoryRowWitness {
    /// Inventory row id (e.g. `H-01`).
    pub row_id: &'static str,
    /// Lift disposition — documents boundary without routing to `umst-chem`.
    pub disposition: NanoChemLiftDisposition,
}

/// Full cluster H inventory manifest — parity harness `cartridge_retains` / deferred pins.
pub const CLUSTER_H_INVENTORY_MANIFEST: &[NanoInventoryRowWitness] = &[
    NanoInventoryRowWitness {
        row_id: "H-01",
        disposition: NanoChemLiftDisposition::LiftedToChemSsot,
    },
    NanoInventoryRowWitness {
        row_id: "H-02",
        disposition: NanoChemLiftDisposition::LiftedToChemSsot,
    },
    NanoInventoryRowWitness {
        row_id: "H-03",
        disposition: NanoChemLiftDisposition::LiftedToChemSsot,
    },
    NanoInventoryRowWitness {
        row_id: "H-04",
        disposition: NanoChemLiftDisposition::CartridgeRetains,
    },
    NanoInventoryRowWitness {
        row_id: "H-05",
        disposition: NanoChemLiftDisposition::CartridgeRetains,
    },
    NanoInventoryRowWitness {
        row_id: "H-06",
        disposition: NanoChemLiftDisposition::CartridgeRetains,
    },
    NanoInventoryRowWitness {
        row_id: "H-07",
        disposition: NanoChemLiftDisposition::LiftedToChemSsot,
    },
];

/// Nano kinetics pins (H-01 … H-03) — delegate to `umst-chem` SSOT (`LiftedToChemSsot`).
///
/// Name retains "Deferred" for semver-stable API; all three pins are lifted @ TODO-M3-003.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NanoDeferredKineticsPins {
    /// Reference SSA for nano-silica (m²/g) — inventory H-01.
    pub ssa_ref_m2_per_g: f32,
    /// Pozzolanic activity exponent α — inventory H-02.
    pub pozzolanic_alpha: f32,
    /// Nucleation set-time shift per SSA decade (min) — inventory H-03.
    pub nucleation_beta_min_per_decade: f32,
}

/// Cartridge-retained nano calibration (H-04 … H-06) — empirical envelope pins.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NanoCartridgeCalibration {
    /// Optimal dosage (% cement) for strength efficiency curve — inventory H-04.
    pub optimal_dosage_pct: f32,
    /// Strength enhancement scale γ — inventory H-05.
    pub strength_gamma: f32,
    /// Pore refinement scale δ — inventory H-06.
    pub pore_refinement_delta: f32,
}

/// Kinetics pins for `NanoEngine` — H-01, H-02, and H-03 from `umst-chem` SSOT.
#[must_use]
pub const fn nano_deferred_kinetics_pins() -> NanoDeferredKineticsPins {
    NanoDeferredKineticsPins {
        ssa_ref_m2_per_g: NANO_SSA_REF_M2_PER_G as f32,
        pozzolanic_alpha: POZZOLANIC_ALPHA as f32,
        nucleation_beta_min_per_decade: NUCLEATION_BETA_MIN_PER_DECADE as f32,
    }
}

/// Cartridge calibration bundle for `NanoEngine` — inventory H-04 … H-06.
#[must_use]
pub const fn nano_cartridge_calibration() -> NanoCartridgeCalibration {
    NanoCartridgeCalibration {
        optimal_dosage_pct: 2.5,
        strength_gamma: 0.15,
        pore_refinement_delta: 5.0,
    }
}

/// Reference SSA (m²/g) — inventory H-01; disposition `LiftedToChemSsot`.
#[must_use]
pub const fn nano_ssa_ref_m2_per_g_f32() -> f32 {
    NANO_SSA_REF_M2_PER_G as f32
}

/// Pozzolanic exponent α — inventory H-02; disposition `LiftedToChemSsot`.
#[must_use]
pub const fn nano_pozzolanic_alpha_f32() -> f32 {
    nano_deferred_kinetics_pins().pozzolanic_alpha
}

/// Nucleation β (min/decade) — inventory H-03; disposition `LiftedToChemSsot`.
#[must_use]
pub const fn nano_nucleation_beta_min_per_decade_f32() -> f32 {
    NUCLEATION_BETA_MIN_PER_DECADE as f32
}

/// Optimal dosage (%) — inventory H-04; disposition `CartridgeRetains`.
#[must_use]
pub const fn nano_optimal_dosage_pct_f32() -> f32 {
    nano_cartridge_calibration().optimal_dosage_pct
}

/// Strength enhancement γ — inventory H-05; disposition `CartridgeRetains`.
#[must_use]
pub const fn nano_strength_gamma_f32() -> f32 {
    nano_cartridge_calibration().strength_gamma
}

/// Pore refinement δ — inventory H-06; disposition `CartridgeRetains`.
#[must_use]
pub const fn nano_pore_refinement_delta_f32() -> f32 {
    nano_cartridge_calibration().pore_refinement_delta
}

/// Self-healing nano boost slope per dosage (%) — inventory H-07; disposition `LiftedToChemSsot`.
#[must_use]
pub const fn nano_healing_boost_per_dosage_f32() -> f32 {
    NANO_HEALING_BOOST_PER_DOSAGE as f32
}

/// Lift disposition for a cluster H inventory row witness.
#[must_use]
pub fn nano_inventory_disposition(row_id: &str) -> Option<NanoChemLiftDisposition> {
    match row_id {
        "H-01" | "H-02" | "H-03" | "H-07" => Some(NanoChemLiftDisposition::LiftedToChemSsot),
        "H-04" | "H-05" | "H-06" => Some(NanoChemLiftDisposition::CartridgeRetains),
        _ => None,
    }
}
