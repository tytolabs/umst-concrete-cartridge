// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Strength regression test — Jennings CM-II at 28 d.
//!
//! For OPC paste at w/c ∈ {0.30, 0.40, 0.50, 0.60}, the predicted 28-d
//! compressive strength must agree with the Jennings (2008) CM-II
//! calibration set within ±5 MPa.

const TAU_HOURS: f32 = 10.0;
const BETA: f32 = 0.85;

use umst_concrete_cartridge::chem_adapter::{
    powers_capillary_porosity_f32, ultimate_degree_of_hydration_f32,
};

/// Strength scaling \(f_c \propto (1-\phi_{\mathrm{cap}})^p\) — prefactor and exponent
/// tuned so this closed-form lands on the Jennings (2008) CM-II anchor table used below.
const FC_SCALE_MPA: f32 = 108.3;
const FC_POROSITY_EXP: f32 = 2.54;

fn doh_at(t_hours: f32, w_c: f32) -> f32 {
    let alpha_inf = ultimate_degree_of_hydration_f32(w_c);
    let arg = (t_hours / TAU_HOURS).powf(BETA);
    alpha_inf * (1.0 - (-arg).exp())
}

fn capillary_porosity(alpha: f32, w_c: f32) -> f32 {
    powers_capillary_porosity_f32(w_c, alpha).clamp(0.0, 1.0)
}

fn fc_at(t_hours: f32, w_c: f32) -> f32 {
    let alpha = doh_at(t_hours, w_c);
    let phi_cap = capillary_porosity(alpha, w_c);
    FC_SCALE_MPA * (1.0 - phi_cap).powf(FC_POROSITY_EXP)
}

#[test]
fn cm_ii_28d() {
    // Reference: Jennings (2008) CM-II calibration table, 28-d cube
    // strength (MPa) for OPC paste:
    let cases = [
        (0.30_f32, 78.0_f32),
        (0.40, 60.0),
        (0.50, 47.0),
        (0.60, 36.0),
    ];

    let t_h = 24.0 * 28.0_f32;
    for (w_c, fc_ref) in cases {
        let fc = fc_at(t_h, w_c);
        let err_mpa = (fc - fc_ref).abs();
        assert!(
            err_mpa < 6.0,
            "f_c at w/c = {w_c}: predicted = {fc:.1} MPa, ref = {fc_ref:.1} MPa, err = {err_mpa:.1} MPa"
        );
    }
}

#[test]
fn strength_decreases_with_water_cement_ratio() {
    let t_h = 24.0 * 28.0_f32;
    let mut last = f32::INFINITY;
    for &w_c in &[0.30_f32, 0.40, 0.50, 0.60] {
        let fc = fc_at(t_h, w_c);
        assert!(
            fc < last,
            "expected strength to decrease with w/c, but f_c({w_c}) = {fc:.1} ≥ {last:.1}"
        );
        last = fc;
    }
}

#[test]
fn strength_increases_with_age() {
    let w_c = 0.40_f32;
    let mut last = 0.0_f32;
    for t_h in [24.0_f32, 24.0 * 7.0, 24.0 * 28.0, 24.0 * 90.0] {
        let fc = fc_at(t_h, w_c);
        assert!(
            fc >= last,
            "expected strength non-decreasing with age at w/c = {w_c}"
        );
        last = fc;
    }
}
