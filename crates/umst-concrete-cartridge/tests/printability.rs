// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Roussel (2018) buildability constraints.
//!
//! For a cylindrical wet column of height H and bulk density ρ, the
//! Roussel buildability criterion requires the static yield stress to
//! satisfy
//!     τ_y ≥ ρ g H / √3.
//!
//! These tests verify that the buildability check correctly accepts and
//! rejects canonical mixes from Roussel (2018).

const G: f32 = 9.81;

fn roussel_min_yield(rho_kg_m3: f32, h_m: f32) -> f32 {
    rho_kg_m3 * G * h_m / 3.0_f32.sqrt()
}

fn buildable(tau_y_pa: f32, rho_kg_m3: f32, h_m: f32) -> bool {
    tau_y_pa >= roussel_min_yield(rho_kg_m3, h_m)
}

#[test]
fn passes_for_admissible_mix() {
    // Roussel 2018 example: ρ = 2300 kg/m³, H = 0.50 m, τ_y = 8000 Pa.
    let rho = 2300.0;
    let h = 0.50;
    let tau_y = 8_000.0;
    assert!(buildable(tau_y, rho, h));
}

#[test]
fn rejects_collapsing_mix() {
    // Same geometry, but τ_y = 200 Pa → collapses immediately.
    let rho = 2300.0;
    let h = 0.50;
    let tau_y = 200.0;
    assert!(!buildable(tau_y, rho, h));
}

#[test]
fn threshold_matches_published_value() {
    // For ρ = 2300 kg/m³, H = 0.30 m the Roussel threshold is
    // ≈ 3905 Pa; assert within 5 Pa of that value.
    let computed = roussel_min_yield(2300.0, 0.30);
    let expected = 3_905.0_f32;
    assert!(
        (computed - expected).abs() < 5.0,
        "expected ≈ {expected} Pa, got {computed:.1} Pa"
    );
}

#[test]
fn monotonic_in_height() {
    let rho = 2300.0;
    let mut last = 0.0_f32;
    for h in [0.05_f32, 0.10, 0.30, 0.50, 1.0, 2.0] {
        let t = roussel_min_yield(rho, h);
        assert!(t >= last);
        last = t;
    }
}
