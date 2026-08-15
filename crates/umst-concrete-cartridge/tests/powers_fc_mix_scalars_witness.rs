// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! T2-S6 batch B2 witness — `powers_compressive_strength_mpa` default path ≡ `MixScalars` SSOT.
//!
//! Proves archived inline gel-space cube removal preserves default-profile f_c on the gate hot path.
//! Digest `d5608148…` held — fixture bytes unchanged.
//!
//! Requires `b1-delegate` — production cfg-gate delegate path (`g_spawn_i_s6_hom_2054`).

#![cfg(feature = "b1-delegate")]

use umst_concrete_cartridge::api_consumer_compose::{
    compressive_strength_mpa_from_row, mix_hydration_scalars_from_row, mix_row_to_mix_scalars,
};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::homogeneous::{
    mix_hydration_state, mix_row_from_scalar_spec, powers_compressive_strength_mpa,
};

fn default_profile() -> Profile {
    Profile::load_bundled("default").expect("bundled default profile for S6 powers witness")
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
fn s6_default_powers_fc_matches_mix_scalars_fc_mpa() {
    let profile = default_profile();
    let row = g0_pass_row(&profile);
    let mix = mix_row_to_mix_scalars(&row);
    let (w_c, alpha, _) = mix_hydration_state(&profile, &row).expect("valid G0 pass row");
    let fc_powers =
        powers_compressive_strength_mpa(&profile, &row, alpha, w_c).expect("valid powers fc");
    let fc_bridge = compressive_strength_mpa_from_row(&row);
    assert!(
        (f64::from(fc_powers) - fc_bridge).abs() < 1e-6,
        "powers default path must match bridge delegate"
    );
    assert!(
        (fc_bridge - mix.fc_mpa()).abs() < 1e-9,
        "bridge must read MixScalars::fc_mpa SSOT"
    );
}

#[test]
fn s6_mix_hydration_scalars_match_bridge() {
    let profile = default_profile();
    let row = g0_pass_row(&profile);
    let (w_c, alpha, temp_c) = mix_hydration_state(&profile, &row).expect("valid row");
    let (w_c_b, alpha_b, temp_c_b) = mix_hydration_scalars_from_row(&row);
    assert!((w_c - w_c_b).abs() < 1e-6);
    assert!((alpha - alpha_b).abs() < 1e-6);
    assert!((temp_c - temp_c_b).abs() < 1e-6);
}
