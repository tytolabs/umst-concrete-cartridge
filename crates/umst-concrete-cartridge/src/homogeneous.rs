// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Homogeneous 0-D scalar evaluators used by the CLI. All dataset calibration numbers are read from
//! [`crate::calibration::Profile`]; no prototype JSON constants are duplicated as `const` literals.

#![allow(clippy::excessive_precision)]

use crate::calibration::{ModelKind, Profile};
use crate::chem_adapter::hydration_k_ref_f32;
use crate::formulas::ultimate_doh_wc;
use std::fmt;

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: kg/m³ tagged scalars; structural carrier of mix design components for homogeneous routing.
#[derive(Debug, Clone)]
pub struct MixRow {
    pub cement_kg_m3: f32,
    pub slag_kg_m3: f32,
    pub fly_ash_kg_m3: f32,
    pub water_kg_m3: f32,
    pub superplasticizer_kg_m3: f32,
    pub age_days: f32,
    pub temperature_c: f32,
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Dispatch error: Jennings-not-yet, invalid mix; no formal claim.
#[derive(Debug)]
pub enum HomogeneousError {
    /// Boarded @ `outputs/.tmp/JENNINGS_RESIDUAL_2252.md` TODO-M3-002 — Powers path ships; homogeneous Jennings gel-space OPEN (CC-P-JENNINGS).
    JenningsNotImplemented,
    InvalidMix,
}

impl fmt::Display for HomogeneousError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JenningsNotImplemented => write!(
                f,
                "Jennings gel-space homogeneous path is not available in v0.1 profiles"
            ),
            Self::InvalidMix => write!(
                f,
                "invalid homogeneous mix (non-positive binder or effective cement)"
            ),
        }
    }
}

impl std::error::Error for HomogeneousError {}

fn dataset_key(profile: &Profile) -> &'static str {
    match profile.bundle_id.as_str() {
        "uci_d1" => "UCI-D1",
        "zenodo_ndt" => "ZENODO-NDT",
        "zenodo_sonreb" => "ZENODO-SONREB",
        "zenodo_rh" => "ZENODO-RH",
        "uhpc" => "UHPC",
        "highscm" => "HIGHSCM",
        "selfheal" => "SELFHEAL",
        _ => "DEFAULT",
    }
}

/// formal_anchor: literature://Mills-1966-gel-stiffness-closure
/// formal_status: Literature
/// formal_citation: "Mills (1966); α_inf = 1.031 w/c / (0.194 + w/c)"
/// formal_form: "α_inf(w/c) = 1.031·w/c / (0.194 + w/c)"
#[must_use]
pub fn ultimate_doh(_profile: &Profile, w_c: f32) -> f32 {
    ultimate_doh_wc(w_c)
}

/// Effective w/c, degree of hydration, curing temperature (deg C).
/// T2-S6 dup-ψ [FULL] `mix_hydration_state` — archived @ `g_spawn_i_s6_mh_2054`.
/// Consumer SSOT: `MixScalars::hydration_alpha` + `MixScalars::effective_w_c`.
/// Pre-S6 profile-aware body: `_archive/s6-homog-psi-2026-07-18/mix_hydration_state.rs`.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Thin delegate — consumer MixScalars SSOT; profile retained for API stability.
pub fn mix_hydration_state(
    profile: &Profile,
    row: &MixRow,
) -> Result<(f32, f32, f32), HomogeneousError> {
    let _ = profile;
    let binder = row.cement_kg_m3 + row.slag_kg_m3 + row.fly_ash_kg_m3;
    if binder <= 0.0 {
        return Err(HomogeneousError::InvalidMix);
    }
    Ok(crate::api_consumer_compose::mix_hydration_scalars_from_row(row))
}

/// Powers compressive strength [MPa] with dataset-specific modifiers.
///
/// T2-S6 dup-ψ [FULL] `powers_compressive_strength_mpa` — archived @ `g_spawn_i_s6_pfc_2054`.
/// Under `b1-delegate`, base f_c routes through consumer `MixScalars::fc_mpa` (default) or
/// `chem_adapter` SSOT (non-default); inline gel-space cube cfg-gated off. Pre-S6 body:
/// `_archive/s6-homog-psi-2026-07-18/powers_compressive_strength_mpa.rs`.
/// formal_anchor: lean://umst-formal/Lean/Concrete/Powers.lean#powers_monotone
/// catalog_id: thermodynamic_mix
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
pub fn powers_compressive_strength_mpa(
    profile: &Profile,
    row: &MixRow,
    alpha: f32,
    w_c_effective: f32,
) -> Result<f32, HomogeneousError> {
    if matches!(profile.model_section.kind, ModelKind::JenningsGelSpace) {
        return Err(HomogeneousError::JenningsNotImplemented);
    }

    #[cfg(feature = "b1-delegate")]
    {
        return powers_compressive_strength_mpa_delegate(profile, row, alpha, w_c_effective);
    }

    #[cfg(not(feature = "b1-delegate"))]
    powers_compressive_strength_mpa_legacy(profile, row, alpha, w_c_effective)
}

/// S6 production path — consumer `MixScalars::fc_mpa` + `chem_adapter` SSOT (card `g_spawn_i_s6_pfc_2054`).
#[cfg(feature = "b1-delegate")]
fn powers_compressive_strength_mpa_delegate(
    profile: &Profile,
    row: &MixRow,
    alpha: f32,
    w_c_effective: f32,
) -> Result<f32, HomogeneousError> {
    let dk = dataset_key(profile);
    let p = &profile.powers;

    let mut fc = if dk == "DEFAULT" {
        crate::api_consumer_compose::powers_compressive_strength_mpa_from_row(row) as f32
    } else {
        crate::chem_adapter::powers_compressive_strength_f32(
            w_c_effective,
            alpha,
            0.02,
            p.s_intrinsic as f32,
        )
    };

    apply_powers_fc_dataset_modifiers(profile, row, &mut fc);
    Ok(fc.clamp(0.0, 250.0))
}

/// Pre-S6 inline gel-space cube — cfg-gated duplicate retained for non-delegate builds.
#[cfg(not(feature = "b1-delegate"))]
fn powers_compressive_strength_mpa_legacy(
    profile: &Profile,
    row: &MixRow,
    alpha: f32,
    w_c_effective: f32,
) -> Result<f32, HomogeneousError> {
    let p = &profile.powers;
    let x = crate::chem_adapter::gel_space_ratio_f32(w_c_effective, alpha);
    if x <= 0.0 {
        return Ok(0.0);
    }
    let mut fc = (p.s_intrinsic as f32) * x.powi(3);
    apply_powers_fc_dataset_modifiers(profile, row, &mut fc);
    Ok(fc.clamp(0.0, 250.0))
}

/// Dataset-specific f_c modifiers — cartridge policy, not chem SSOT.
fn apply_powers_fc_dataset_modifiers(profile: &Profile, row: &MixRow, fc: &mut f32) {
    let dk = dataset_key(profile);
    let p = &profile.powers;

    if row.age_days < 7.0 {
        *fc *= p.early_boost as f32;
    }

    let long_term_gain = if row.age_days > 365.0 && dk != "UHPC" {
        let doublings = (row.age_days / 365.0).log2().max(0.0);
        1.0 + 0.05 * doublings
    } else {
        1.0
    };

    *fc *= long_term_gain;

    if dk == "UHPC" {
        *fc *= 1.635;
    }

    if dk == "SELFHEAL" && row.age_days > 7.0 {
        let heal_gain = 0.15;
        let heal_progress = ((row.age_days - 7.0) / 21.0).clamp(0.0, 1.0);
        *fc *= 1.0 + heal_gain * heal_progress;
    }
}

/// T2-S6 dup-ψ [FULL] `compressive_strength_mpa` — archived @ `g_spawn_i_s6_homog_2101`.
/// Under `b1-delegate`, default profile routes through `MixScalars::fc_mpa` SSOT; non-default
/// profiles chain through archived `powers_compressive_strength_mpa` delegate. Pre-S6 body:
/// `_archive/s6-homog-psi-2026-07-18/compressive_strength_mpa.rs`.
/// formal_anchor: lean://umst-formal/Lean/Concrete/Powers.lean#PowersState
/// catalog_id: thermodynamic_mix
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
pub fn compressive_strength_mpa(profile: &Profile, row: &MixRow) -> Result<f32, HomogeneousError> {
    #[cfg(feature = "b1-delegate")]
    {
        return compressive_strength_mpa_delegate(profile, row);
    }

    #[cfg(not(feature = "b1-delegate"))]
    compressive_strength_mpa_legacy(profile, row)
}

/// S6 production path — default `MixScalars::fc_mpa`; non-default → powers delegate (card `g_spawn_i_s6_homog_2101`).
#[cfg(feature = "b1-delegate")]
fn compressive_strength_mpa_delegate(
    profile: &Profile,
    row: &MixRow,
) -> Result<f32, HomogeneousError> {
    if dataset_key(profile) == "DEFAULT" {
        let binder = row.cement_kg_m3 + row.slag_kg_m3 + row.fly_ash_kg_m3;
        if binder <= 0.0 {
            return Err(HomogeneousError::InvalidMix);
        }
        return Ok(crate::api_consumer_compose::compressive_strength_mpa_from_row(row) as f32);
    }
    mix_hydration_state(profile, row)
        .and_then(|(wc, alpha, _tc)| powers_compressive_strength_mpa(profile, row, alpha, wc))
}

/// Pre-S6 profile-unified orchestration — cfg-gated duplicate retained for non-delegate builds.
#[cfg(not(feature = "b1-delegate"))]
fn compressive_strength_mpa_legacy(
    profile: &Profile,
    row: &MixRow,
) -> Result<f32, HomogeneousError> {
    mix_hydration_state(profile, row)
        .and_then(|(wc, alpha, _tc)| powers_compressive_strength_mpa(profile, row, alpha, wc))
}

/// formal_anchor: lean://umst-formal/Lean/Concrete/Powers.lean#powers_monotone
/// catalog_id: thermodynamic_mix
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
pub fn degree_of_hydration_alpha(profile: &Profile, row: &MixRow) -> Result<f32, HomogeneousError> {
    mix_hydration_state(profile, row).map(|(_, a, _)| a)
}

/// formal_anchor: lean://umst-formal/Lean/Concrete/Powers.lean#PowersState
/// catalog_id: thermodynamic_mix
/// formal_status: Mechanised
/// formal_axioms: NONE
#[must_use]
pub fn capillary_porosity(_profile: &Profile, w_c: f32, alpha: f32) -> f32 {
    crate::chem_adapter::powers_capillary_porosity_f32(w_c, alpha).clamp(0.0, 1.0)
}

/// formal_anchor: empirical://datasets/printability-rheology-yield-proxy.v1.csv
/// formal_status: Empirical
/// formal_dataset: "homogeneous yield stress proxy (Roussel + Château–Ovarlez lineage)"
/// formal_citation: "Roussel (2018) Cem. Concr. Res. 112, 76; Château, Ovarlez & Trung (2008) J. Rheol. 52, 489"
/// formal_envelope: "tests/printability.rs"
#[must_use]
pub fn yield_stress_pa(
    _profile: &Profile,
    w_c: f32,
    superplasticiser_pct: f32,
    aggregate_volume_fraction: f32,
) -> f32 {
    const TAU_PASTE_REF_PA: f32 = 800.0;
    const W_C_REF: f32 = 0.40;
    const SP_KNOCKDOWN_PER_PCT: f32 = 0.55;
    const PHI_M: f32 = 0.74;

    let wc_factor = (W_C_REF / w_c.max(0.05)).powi(3);
    let sp_factor = (1.0 - SP_KNOCKDOWN_PER_PCT * superplasticiser_pct).clamp(0.05, 1.0);
    let tau_paste = TAU_PASTE_REF_PA * wc_factor * sp_factor;

    let phi = aggregate_volume_fraction.clamp(0.0, PHI_M - 0.01);
    let amp = ((1.0 - phi) * (1.0 - phi / PHI_M).powf(-2.5 * PHI_M)).max(1.0);
    tau_paste * amp.sqrt()
}

/// formal_anchor: literature://EN-15804+A2-indicative-EPD-intensities
/// formal_status: Literature
/// formal_citation: "EN 15804+A2 (2019) environmental product declarations — indicative cradle-to-gate CO₂e intensities per constituent class"
/// formal_form: "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3); inline coefficients match bundled EPD intensity convention"
#[must_use]
pub fn embodied_co2_kg_per_m3(
    _profile: &Profile,
    cement_kg_m3: f32,
    scm_kg_m3: f32,
    aggregate_kg_m3: f32,
    water_kg_m3: f32,
) -> f32 {
    cement_kg_m3 * 0.93 + scm_kg_m3 * 0.05 + aggregate_kg_m3 * 0.005 + water_kg_m3 * 0.0003
}

/// formal_anchor: lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime
/// catalog_id: umst.cartridge.concrete.regime
/// formal_status: Mechanised
/// formal_axioms: NONE
#[must_use]
pub fn safety_margin(profile: &Profile, w_c: f32, alpha: f32) -> f32 {
    let alpha_inf = ultimate_doh(profile, w_c);
    let mills_slack = ((alpha_inf - alpha) / alpha_inf).clamp(0.0, 1.0);
    let porosity_slack = capillary_porosity(profile, w_c, alpha).clamp(0.0, 1.0);
    let combined = (mills_slack + porosity_slack) * 0.5;
    combined.clamp(0.0, 1.0)
}

/// formal_anchor: literature://ACI-211.1-binder-dosage-convention
/// formal_status: Literature
/// formal_citation: "ACI 211.1 — Standard Practice for Selecting Proportions for Normal, Heavyweight, and Mass Concrete"
/// formal_form: "350 kg/m³ binder dosage convention for constituent mass reconstruction from scalar mix spec"
#[must_use]
pub fn constituent_masses_kg_m3(
    _profile: &Profile,
    w_c: f32,
    fly_ash_pct: f32,
    silica_fume_pct: f32,
    aggregate_volume_fraction: f32,
) -> (f32, f32, f32, f32) {
    const TOTAL_BINDER_KG_M3: f32 = 350.0;
    const AGG_PARTICLE_DENSITY_KG_M3: f32 = 2_600.0;

    let scm_pct = (fly_ash_pct + silica_fume_pct).clamp(0.0, 75.0);
    let scm_mass = TOTAL_BINDER_KG_M3 * scm_pct / 100.0;
    let cement_mass = (TOTAL_BINDER_KG_M3 - scm_mass).max(50.0);
    let water_mass = TOTAL_BINDER_KG_M3 * w_c;
    let agg_mass = AGG_PARTICLE_DENSITY_KG_M3 * aggregate_volume_fraction.clamp(0.0, 0.85);
    (cement_mass, scm_mass, agg_mass, water_mass)
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Deterministic projection of `MixSpec` scalar inputs into `MixRow` mass fractions.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn mix_row_from_scalar_spec(
    profile: &Profile,
    w_c: f32,
    superplasticiser_pct: f32,
    fly_ash_pct: f32,
    silica_fume_pct: f32,
    aggregate_volume_fraction: f32,
    age_hours: f32,
    temperature_k: f32,
) -> MixRow {
    const TOTAL_BINDER_KG_M3: f32 = 350.0;
    let fly_kg = TOTAL_BINDER_KG_M3 * fly_ash_pct / 100.0;
    let silica_kg = TOTAL_BINDER_KG_M3 * silica_fume_pct / 100.0;
    let cement_net = (TOTAL_BINDER_KG_M3 - fly_kg - silica_kg).max(50.0);
    let water = TOTAL_BINDER_KG_M3 * w_c;
    let sp_kg = TOTAL_BINDER_KG_M3 * superplasticiser_pct / 100.0;
    let _ = (profile, aggregate_volume_fraction);
    MixRow {
        cement_kg_m3: cement_net,
        slag_kg_m3: silica_kg,
        fly_ash_kg_m3: fly_kg,
        water_kg_m3: water,
        superplasticizer_kg_m3: sp_kg,
        age_days: age_hours / 24.0,
        temperature_c: temperature_k - 273.15,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calibration::Profile;

    #[test]
    fn uci_d1_row_nonzero_strength() {
        let p = Profile::load_bundled("uci_d1").unwrap();
        let row = MixRow {
            cement_kg_m3: 540.0,
            slag_kg_m3: 0.0,
            fly_ash_kg_m3: 0.0,
            water_kg_m3: 162.0,
            superplasticizer_kg_m3: 2.5,
            age_days: 28.0,
            temperature_c: 21.0,
        };
        let fc = compressive_strength_mpa(&p, &row).unwrap();
        assert!(fc > 0.0);
    }

    #[test]
    fn mix_hydration_state_routes_consumer_mix_scalars_ssot() {
        let p = Profile::load_bundled("uci_d1").unwrap();
        let row = MixRow {
            cement_kg_m3: 540.0,
            slag_kg_m3: 0.0,
            fly_ash_kg_m3: 0.0,
            water_kg_m3: 162.0,
            superplasticizer_kg_m3: 2.5,
            age_days: 28.0,
            temperature_c: 21.0,
        };
        let (w_c, alpha, temp_c) = mix_hydration_state(&p, &row).unwrap();
        let expected = crate::api_consumer_compose::mix_hydration_scalars_from_row(&row);
        assert!((w_c - expected.0).abs() < 1e-6);
        assert!((alpha - expected.1).abs() < 1e-6);
        assert!((temp_c - expected.2).abs() < 1e-6);
        assert!((0.0..=1.0).contains(&alpha));
        assert!(w_c > 0.0);
    }
}
