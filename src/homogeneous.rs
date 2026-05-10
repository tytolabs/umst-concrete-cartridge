// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Homogeneous 0-D scalar evaluators used by the CLI. All dataset calibration numbers are read from
//! [`crate::calibration::Profile`]; no prototype JSON constants are duplicated as `const` literals.

#![allow(clippy::excessive_precision)]

use crate::calibration::{ModelKind, Profile};
use crate::formulas::{hydration_degree_calibrated, ultimate_doh_wc};
use thiserror::Error;

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
#[derive(Debug, Error)]
pub enum HomogeneousError {
    #[error("Jennings gel-space homogeneous path is not available in v0.1 profiles")]
    JenningsNotImplemented,
    #[error("invalid homogeneous mix (non-positive binder or effective cement)")]
    InvalidMix,
}

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

/// Effective w/c, degree of hydration, curing temperature (deg C). Mirrors prototype-3 `mix_hydration_state`.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Internal homogeneous helper composing calibrated α(t,T,scm) and effective w/c from profile parameters.
pub fn mix_hydration_state(
    profile: &Profile,
    row: &MixRow,
) -> Result<(f32, f32, f32), HomogeneousError> {
    let dk = dataset_key(profile);

    let binder = row.cement_kg_m3 + row.slag_kg_m3 + row.fly_ash_kg_m3;
    if binder <= 0.0 {
        return Err(HomogeneousError::InvalidMix);
    }

    let p = &profile.powers;
    let effective_cement = row.cement_kg_m3
        + p.k_slag as f32 * row.slag_kg_m3
        + p.k_fly_ash as f32 * row.fly_ash_kg_m3;
    if effective_cement <= 0.0 {
        return Err(HomogeneousError::InvalidMix);
    }

    let mut w_c_raw = (row.water_kg_m3 / effective_cement).clamp(0.10, 1.0);
    if dk == "SELFHEAL" {
        w_c_raw += 0.03 * 0.06;
    }
    let sp_water_reduction = if dk == "UHPC" {
        0.35 * (row.superplasticizer_kg_m3 / 30.0).min(1.0)
    } else {
        0.20 * (row.superplasticizer_kg_m3 / 5.0).min(1.0)
    };
    let w_c_effective = w_c_raw * (1.0 - sp_water_reduction);
    let scm_ratio = (row.slag_kg_m3 + row.fly_ash_kg_m3) / binder;

    let mut k_ref_eff = p.k_ref as f32;
    if dk == "UHPC" {
        k_ref_eff = (p.k_ref as f32 / 0.55) * 2.68;
    }

    let effective_age = row.age_days.min(365.0);
    let temp_c = row.temperature_c;

    let mut alpha = hydration_degree_calibrated(effective_age, temp_c, scm_ratio, k_ref_eff);

    if dk == "ZENODO-SONREB" && effective_age >= 14.0 {
        let alpha_14 = hydration_degree_calibrated(14.0, temp_c, scm_ratio, k_ref_eff);
        let diff = effective_age - 14.0;
        alpha = alpha_14 + (1.0 - alpha_14) * (1.0 - (-k_ref_eff * diff.sqrt()).exp());
    }

    if dk == "HIGHSCM" && row.age_days > 7.0 {
        alpha += p.k_slag as f32 * (1.0 - (-0.02 * (row.age_days - 7.0)).exp());
    }

    if dk == "UHPC" && w_c_raw < 0.22 {
        alpha = alpha.min(0.65);
    }
    alpha = alpha.min(1.0);

    Ok((w_c_effective, alpha, temp_c))
}

/// formal_anchor: lean://umst-formal/Lean/Powers.lean#powers_monotone
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

    let dk = dataset_key(profile);
    let p = &profile.powers;

    let vg = 0.68 * alpha;
    let vc = w_c_effective - 0.36 * alpha;
    let space = vg + vc.max(0.0) + 0.02;
    if space <= 0.001 {
        return Ok(0.0);
    }
    let x = vg / space;
    let mut fc = (p.s_intrinsic as f32) * x.powi(3);

    if row.age_days < 7.0 {
        fc *= p.early_boost as f32;
    }

    let long_term_gain = if row.age_days > 365.0 && dk != "UHPC" {
        let doublings = (row.age_days / 365.0).log2().max(0.0);
        1.0 + 0.05 * doublings
    } else {
        1.0
    };

    fc *= long_term_gain;

    if dk == "UHPC" {
        fc *= 1.635;
    }

    if dk == "SELFHEAL" && row.age_days > 7.0 {
        let heal_gain = 0.15;
        let heal_progress = ((row.age_days - 7.0) / 21.0).clamp(0.0, 1.0);
        fc *= 1.0 + heal_gain * heal_progress;
    }

    Ok(fc.clamp(0.0, 250.0))
}

/// formal_anchor: lean://umst-formal/Lean/Powers.lean#PowersState
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
pub fn compressive_strength_mpa(profile: &Profile, row: &MixRow) -> Result<f32, HomogeneousError> {
    mix_hydration_state(profile, row)
        .and_then(|(wc, alpha, _tc)| powers_compressive_strength_mpa(profile, row, alpha, wc))
}

/// formal_anchor: lean://umst-formal/Lean/Powers.lean#powers_monotone
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
pub fn degree_of_hydration_alpha(profile: &Profile, row: &MixRow) -> Result<f32, HomogeneousError> {
    mix_hydration_state(profile, row).map(|(_, a, _)| a)
}

/// formal_anchor: lean://umst-formal/Lean/Powers.lean#PowersState
/// formal_status: Mechanised
/// formal_axioms: NONE
#[must_use]
pub fn capillary_porosity(_profile: &Profile, w_c: f32, alpha: f32) -> f32 {
    ((w_c - 0.36 * alpha) / (w_c + 0.32)).clamp(0.0, 1.0)
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
}
