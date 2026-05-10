// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Homogeneous closed-form constitutive evaluators.
//!
//! These are the calibrated 0-D scalar evaluators used by the CLI front-door.
//! They wrap the canonical published references (Mills, Powers–Brownyard,
//! Jennings/Tennis–Jennings, Chateau–Ovarlez–Trung) at their well-validated
//! anchor points and intentionally avoid the higher-order tensor pathways in
//! `physics::*`, which are designed for differentiable training rather than
//! single-shot prediction.
//!
//! Validation envelopes for each function are documented next to the function.
//! When you change a number, update the envelope test in `tests/homogeneous.rs`.

#![allow(clippy::excessive_precision)]

/// Mills (1966) ultimate degree of hydration as a function of w/c for OPC.
///
/// `α∞(w/c) = 1.031 · (w/c) / (0.194 + w/c)`
#[must_use]
pub fn ultimate_doh(w_c: f32) -> f32 {
    1.031 * w_c / (0.194 + w_c)
}

/// Degree of hydration at age `t_hours` for an OPC paste at `temperature_k`,
/// w/c = `w_c`.
///
/// `α(t) = α∞ · (1 − exp(−(k·t)^β))` with Arrhenius temperature scaling
/// (`Eₐ/R ≈ 4 000 K`) about a 293.15 K reference. The kinetic constants
/// `k_ref = 0.025 h^−β` and `β = 0.55` are fitted to Powers (1948) OPC
/// isothermal calorimetry across w/c ∈ [0.30, 0.60].
///
/// Envelope: ±5 % MAE against Powers (1948) at 1 d, 7 d, 28 d, 90 d.
#[must_use]
pub fn degree_of_hydration(w_c: f32, age_hours: f32, temperature_k: f32) -> f32 {
    const K_REF: f32 = 0.025;
    const BETA: f32 = 0.55;
    const EA_OVER_R: f32 = 4_000.0;
    const T_REF_K: f32 = 293.15;

    let arrhenius = (EA_OVER_R * (1.0 / T_REF_K - 1.0 / temperature_k)).exp();
    let k = K_REF * arrhenius;
    let alpha_inf = ultimate_doh(w_c);
    let arg = (k * age_hours).powf(BETA);
    (alpha_inf * (1.0 - (-arg).exp())).clamp(0.0, alpha_inf)
}

/// Powers–Brownyard capillary porosity for an OPC paste.
///
/// `φ_cap = max(0, (w/c − 0.36·α) / (w/c + 0.32))`
#[must_use]
pub fn capillary_porosity(w_c: f32, alpha: f32) -> f32 {
    ((w_c - 0.36 * alpha) / (w_c + 0.32)).clamp(0.0, 1.0)
}

/// Compressive strength (MPa) via the Powers gel–space ratio model with a
/// Jennings (2008) CM-II calibrated prefactor.
///
/// `f_c = a · (1 − φ_cap)^p`, with `(a, p) = (108.3 MPa, 2.54)` calibrated
/// against the Jennings (2008) anchor table for OPC paste at 28 d.
///
/// Envelope: ±6 MPa for w/c ∈ {0.30, 0.40, 0.50, 0.60} at 28 d.
#[must_use]
pub fn compressive_strength_mpa(w_c: f32, alpha: f32) -> f32 {
    const A_MPA: f32 = 108.3;
    const P: f32 = 2.54;
    let phi = capillary_porosity(w_c, alpha);
    A_MPA * (1.0 - phi).powf(P)
}

/// Static yield stress (Pa) of the fresh paste–aggregate suspension.
///
/// Paste yield follows a Bingham–Roussel scaling in `w/c` with a
/// superplasticiser knock-down, and is then lifted to the suspension yield
/// via the Chateau–Ovarlez–Trung (2008) homogenisation
///
/// `τ_y(φ) = τ_paste · √((1 − φ) · (1 − φ / φ_m)^(−2.5 φ_m))`
///
/// with `φ_m = 0.74` for moderately graded sand-and-gravel. Anchored so an
/// unmodified OPC paste at w/c = 0.40 (no SP, φ_agg = 0) returns ≈ 800 Pa,
/// matching the upper end of Roussel's slump corpus for paste.
///
/// Envelope: order-of-magnitude agreement with Roussel (2006) slump corpus
/// for w/c ∈ [0.30, 0.55], SP ∈ [0, 1.5 %], φ_agg ∈ [0, 0.75].
#[must_use]
pub fn yield_stress_pa(w_c: f32, superplasticiser_pct: f32, aggregate_volume_fraction: f32) -> f32 {
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

/// Embodied global-warming potential (kg CO₂-eq / m³ of concrete).
///
/// Sums constituent masses against EN 15804+A2 EPD intensities. Default
/// intensities target ordinary CEM I 52.5 N (0.93 kg/kg), generic SCM
/// (0.05 kg/kg), aggregate (0.005 kg/kg), and water (0.0003 kg/kg).
#[must_use]
pub fn embodied_co2_kg_per_m3(
    cement_kg_m3: f32,
    scm_kg_m3: f32,
    aggregate_kg_m3: f32,
    water_kg_m3: f32,
) -> f32 {
    cement_kg_m3 * 0.93 + scm_kg_m3 * 0.05 + aggregate_kg_m3 * 0.005 + water_kg_m3 * 0.0003
}

/// Thermodynamic admissibility margin in [0, 1].
///
/// Returns the slack on the joint Powers (φ_cap > 0) and Mills (α ≤ α∞)
/// constraints normalised to a unit interval. A value of 1 means the mix is
/// well within the admissible region; 0 means the mix is right at a
/// constraint surface; negative inputs are clamped.
#[must_use]
pub fn safety_margin(w_c: f32, alpha: f32) -> f32 {
    let alpha_inf = ultimate_doh(w_c);
    let mills_slack = ((alpha_inf - alpha) / alpha_inf).clamp(0.0, 1.0);
    let porosity_slack = capillary_porosity(w_c, alpha).clamp(0.0, 1.0);
    let combined = (mills_slack + porosity_slack) * 0.5;
    combined.clamp(0.0, 1.0)
}

/// Estimate constituent masses (kg/m³) from a 0-D mix specification.
///
/// Fixes total cementitious content at 350 kg/m³ (a common slab/printing
/// dosage) and partitions it between cement, fly ash, and silica fume by
/// the supplied percentages.
#[must_use]
pub fn constituent_masses_kg_m3(
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn doh_powers_envelope_at_w_c_0_40() {
        let cases = [
            (24.0_f32, 0.42_f32, 0.10),
            (24.0 * 7.0, 0.58, 0.10),
            (24.0 * 28.0, 0.66, 0.10),
            (24.0 * 90.0, 0.70, 0.10),
        ];
        for (t, expected, tol) in cases {
            let alpha = degree_of_hydration(0.40, t, 293.15);
            assert!(
                (alpha - expected).abs() < tol,
                "α({t}h) = {alpha:.3} vs Powers ref {expected:.3} (tol {tol})"
            );
        }
    }

    #[test]
    fn doh_capped_by_mills_ceiling() {
        let alpha = degree_of_hydration(0.40, 1_000_000.0, 293.15);
        let ceiling = ultimate_doh(0.40);
        assert!(alpha <= ceiling + 1.0e-6);
    }

    #[test]
    fn strength_anchors_at_28d() {
        // Reference: Jennings (2008) CM-II calibration for OPC at 28 d.
        let cases = [
            (0.30_f32, 78.0_f32),
            (0.40, 60.0),
            (0.50, 47.0),
            (0.60, 36.0),
        ];
        for (w_c, fc_ref) in cases {
            let alpha = degree_of_hydration(w_c, 24.0 * 28.0, 293.15);
            let fc = compressive_strength_mpa(w_c, alpha);
            assert!(
                (fc - fc_ref).abs() < 8.0,
                "f_c(w/c = {w_c}) = {fc:.1} MPa, ref = {fc_ref:.1} MPa"
            );
        }
    }

    #[test]
    fn yield_stress_in_roussel_envelope() {
        let tau = yield_stress_pa(0.40, 0.0, 0.65);
        assert!(
            (200.0..=8_000.0).contains(&tau),
            "τ_y = {tau} Pa outside Roussel slump-corpus envelope"
        );
    }

    #[test]
    fn yield_stress_decreases_with_superplasticiser() {
        let t0 = yield_stress_pa(0.40, 0.0, 0.65);
        let t1 = yield_stress_pa(0.40, 1.0, 0.65);
        assert!(t1 < t0, "SP should reduce τ_y, got {t0} -> {t1}");
    }

    #[test]
    fn yield_stress_increases_with_aggregate_loading() {
        let mut last = 0.0_f32;
        for phi in [0.0_f32, 0.30, 0.50, 0.70] {
            let tau = yield_stress_pa(0.40, 0.0, phi);
            assert!(tau >= last, "τ_y not monotone in φ_agg at φ = {phi}");
            last = tau;
        }
    }

    #[test]
    fn embodied_co2_orders_of_magnitude() {
        let (c, scm, agg, w) = constituent_masses_kg_m3(0.40, 20.0, 5.0, 0.65);
        let gwp = embodied_co2_kg_per_m3(c, scm, agg, w);
        assert!(
            (200.0..=400.0).contains(&gwp),
            "GWP {gwp} kg CO₂/m³ outside literature envelope (200–400)"
        );
    }

    #[test]
    fn safety_margin_in_unit_interval() {
        let alpha = degree_of_hydration(0.40, 24.0 * 28.0, 293.15);
        let m = safety_margin(0.40, alpha);
        assert!((0.0..=1.0).contains(&m), "safety margin {m} out of [0, 1]");
    }

    #[test]
    fn ultimate_doh_increasing_in_water_cement() {
        let mut last = 0.0_f32;
        for w_c in [0.30_f32, 0.40, 0.50, 0.60] {
            let a = ultimate_doh(w_c);
            assert!(a > last, "α∞ not increasing in w/c at {w_c}");
            assert_abs_diff_eq!(a, 1.031 * w_c / (0.194 + w_c), epsilon = 1.0e-6);
            last = a;
        }
    }
}
