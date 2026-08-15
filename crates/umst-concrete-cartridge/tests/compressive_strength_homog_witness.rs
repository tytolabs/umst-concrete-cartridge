// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! T2-S6 batch B2 witness — `compressive_strength_mpa` archive closure ≡ `MixScalars` SSOT.
//!
//! Proves homogeneous FULL row archive preserves default-profile f_c on gate hot path.
//! Digest `d5608148…` held — fixture bytes unchanged.

use umst_concrete_cartridge::api_consumer_compose::{
    compressive_strength_mpa_from_row, mix_row_to_mix_scalars,
};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::homogeneous::{compressive_strength_mpa, mix_row_from_scalar_spec};

fn default_profile() -> Profile {
    Profile::load_bundled("default").expect("bundled default profile for S6 cs archive witness")
}

fn g0_pass_row(profile: &Profile) -> umst_concrete_cartridge::homogeneous::MixRow {
    mix_row_from_scalar_spec(profile, 9.0 / 20.0, 0.0, 0.0, 0.0, 0.7, 28.0 * 24.0, 293.15)
}

#[test]
fn s6_archived_compressive_strength_default_matches_mix_scalars() {
    let profile = default_profile();
    let row = g0_pass_row(&profile);
    let mix = mix_row_to_mix_scalars(&row);
    let fc_homog = compressive_strength_mpa(&profile, &row).expect("valid G0 pass row");
    let fc_bridge = compressive_strength_mpa_from_row(&row);
    assert!(
        (f64::from(fc_homog) - fc_bridge).abs() < 1e-6,
        "archived homogeneous default path must match bridge delegate"
    );
    assert!(
        (fc_bridge - mix.fc_mpa()).abs() < 1e-9,
        "bridge must read MixScalars::fc_mpa SSOT"
    );
}
