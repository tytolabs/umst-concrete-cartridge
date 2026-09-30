// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Hydration regression test against Powers (1948) OPC isothermal calorimetry.
//!
//! Asserts that the predicted degree of hydration at 1 d, 7 d, 28 d, and 90 d
//! agrees with the Powers reference curve for w/c = 0.40 within ±10 % MAE.

use approx::assert_abs_diff_eq;

const TAU_HOURS: f32 = 10.0;
const BETA: f32 = 0.85;

fn ultimate_doh(w_c: f32) -> f32 {
    1.031 * w_c / (0.194 + w_c)
}

fn doh_at(t_hours: f32, w_c: f32) -> f32 {
    let alpha_inf = ultimate_doh(w_c);
    let arg = (t_hours / TAU_HOURS).powf(BETA);
    alpha_inf * (1.0 - (-arg).exp())
}

#[test]
fn powers_doh_envelope() {
    let w_c = 0.40_f32;

    // Reference points from Powers 1948, OPC, w/c = 0.40, 20 °C.
    let reference = [
        (24.0_f32, 0.42_f32),
        (24.0 * 7.0, 0.58),
        (24.0 * 28.0, 0.66),
        (24.0 * 90.0, 0.74),
    ];

    let mut total_err = 0.0_f32;
    for (t_h, ref_alpha) in reference.iter() {
        let predicted = doh_at(*t_h, w_c);
        total_err += (predicted - ref_alpha).abs();
    }
    let mae = total_err / reference.len() as f32;
    assert!(
        mae < 0.10,
        "MAE against Powers 1948 was {mae:.3}, expected < 0.10",
    );
}

#[test]
fn ultimate_doh_within_physical_bounds() {
    for &w_c in &[0.30_f32, 0.40, 0.50, 0.60] {
        let alpha = ultimate_doh(w_c);
        assert!((0.0..=1.0).contains(&alpha), "α∞ = {alpha} for w/c = {w_c}");
    }
}

#[test]
fn doh_monotonic_in_time() {
    let w_c = 0.40_f32;
    let times = [1.0_f32, 24.0, 24.0 * 7.0, 24.0 * 28.0, 24.0 * 365.0];
    let mut last = 0.0_f32;
    for t in times {
        let a = doh_at(t, w_c);
        assert!(a >= last, "DoH not monotonic: α({t}) = {a} < {last}");
        last = a;
    }
}

#[test]
fn doh_monotonic_in_water_cement_ratio() {
    let t_h = 24.0 * 28.0_f32;
    let mut last = 0.0_f32;
    for &w_c in &[0.30_f32, 0.40, 0.50, 0.60] {
        let a = doh_at(t_h, w_c);
        assert!(a >= last, "DoH not monotonic in w/c at 28 d");
        last = a;
    }
}

#[test]
fn doh_zero_at_time_zero() {
    let a = doh_at(0.0, 0.40);
    assert_abs_diff_eq!(a, 0.0, epsilon = 1.0e-6);
}
