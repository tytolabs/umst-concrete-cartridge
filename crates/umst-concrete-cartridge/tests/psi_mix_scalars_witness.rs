// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! T2-S6 batch B2 witness — `psi_m1_oracle` / `compressive_strength_mpa` ≡ `MixScalars` SSOT.
//!
//! Proves monolith thin delegates route through consumer compose oracles without duplicating
//! Powers closure math on the default-profile hot path. Digest `d5608148…` held — fixture bytes unchanged.

use umst_concrete_cartridge::api_consumer_compose::{
    compressive_strength_mpa_from_row, mix_row_to_mix_scalars, powers_compressive_strength_mpa_from_row,
    psi_m1_oracle_from_row,
};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::homogeneous::{
    compressive_strength_mpa, mix_hydration_state, mix_row_from_scalar_spec, powers_compressive_strength_mpa,
};

fn default_profile() -> Profile {
    Profile::load_bundled("default").expect("bundled default profile for S6 ψ witness")
}

fn g0_pass_row(profile: &Profile) -> umst_concrete_cartridge::homogeneous::MixRow {
    mix_row_from_scalar_spec(
        profile,
        9.0 / 20.0,
        0.0,
        0.0,
        0.0,
        0.7,
        28.0 * 24.0,
        293.15,
    )
}

#[test]
fn s6_powers_fc_matches_mix_scalars_on_default_profile() {
    let profile = default_profile();
    let row = g0_pass_row(&profile);
    let mix = mix_row_to_mix_scalars(&row);
    let (w_c, alpha, _) = mix_hydration_state(&profile, &row).expect("valid G0 pass row");
    let fc_powers = powers_compressive_strength_mpa(&profile, &row, alpha, w_c)
        .expect("powers closure on default profile");
    let fc_bridge = powers_compressive_strength_mpa_from_row(&row);
    assert!(
        (f64::from(fc_powers) - fc_bridge).abs() < 1e-6,
        "powers_compressive_strength_mpa default path must match bridge delegate"
    );
    assert!(
        (fc_bridge - mix.fc_mpa()).abs() < 1e-9,
        "bridge must read MixScalars::fc_mpa SSOT"
    );
}

#[test]
fn s6_default_fc_matches_mix_scalars_fc_mpa() {
    let profile = default_profile();
    let row = g0_pass_row(&profile);
    let mix = mix_row_to_mix_scalars(&row);
    let fc_homog = compressive_strength_mpa(&profile, &row).expect("valid G0 pass row");
    let fc_bridge = compressive_strength_mpa_from_row(&row);
    assert!(
        (f64::from(fc_homog) - fc_bridge).abs() < 1e-6,
        "homogeneous default path must match bridge delegate"
    );
    assert!(
        (fc_bridge - mix.fc_mpa()).abs() < 1e-9,
        "bridge must read MixScalars::fc_mpa SSOT"
    );
}

#[test]
fn s6_psi_m1_oracle_matches_mix_scalars() {
    let profile = default_profile();
    let row = g0_pass_row(&profile);
    let mix = mix_row_to_mix_scalars(&row);
    let psi_bridge = psi_m1_oracle_from_row(&row);
    assert!(
        (psi_bridge - mix.psi_m1()).abs() < 1e-3,
        "psi_m1_oracle must match MixScalars::psi_m1"
    );
    assert!(
        (psi_bridge + mix.fc_mpa() * 1e6).abs() < 1e-3,
        "ψ_M1 = −f_c·10⁶ closure"
    );
    assert!(psi_bridge < 0.0, "M1 ψ oracle must be negative");
}
