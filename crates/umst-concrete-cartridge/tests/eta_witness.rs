// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! T2-S6 PARTIAL η lift witness — `dissipation_modulus_from_profile` → consumer SSOT.

use umst_cartridge_concrete::{
    dissipation_modulus_eta, dissipation_modulus_eta_from_profile as consumer_eta_from_profile,
    MixScalars,
};
use umst_concrete_cartridge::api_consumer_compose::{
    dissipation_modulus_eta_from_profile, scalar_fields_from_composed,
};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::homogeneous::MixRow;

fn default_profile() -> Profile {
    Profile::load_bundled("default").expect("bundled default profile for S6 η witness")
}

fn g0_mix_row() -> MixRow {
    MixRow {
        cement_kg_m3: 315.0,
        slag_kg_m3: 0.0,
        fly_ash_kg_m3: 0.0,
        water_kg_m3: 141.75,
        superplasticizer_kg_m3: 0.0,
        age_days: 28.0,
        temperature_c: 20.0,
    }
}

#[test]
fn s6_eta_default_profile_matches_g0_pin() {
    let profile = default_profile();
    let monolith_eta = dissipation_modulus_eta_from_profile(&profile);
    let consumer_pin = dissipation_modulus_eta();
    assert!(
        (monolith_eta - consumer_pin).abs() < 1e-6,
        "default profile s_intrinsic=80 must match G0 η pin"
    );
}

#[test]
fn s6_eta_profile_s_intrinsic_scales_eta() {
    let s_high = 120.0_f64;
    let s_low = 40.0_f64;
    let eta_high = consumer_eta_from_profile(s_high);
    let eta_low = consumer_eta_from_profile(s_low);
    assert!(eta_high > eta_low);
    assert_eq!(
        eta_high / eta_low,
        3.0,
        "η ∝ s_intrinsic at fixed enthalpy pin"
    );
}

#[test]
fn s6_scalar_fields_eta_threads_profile_s_intrinsic() {
    let profile = default_profile();
    let row = g0_mix_row();
    let (_, _, eta) = scalar_fields_from_composed(&profile, &row, 0.0);
    let mix = MixScalars::g0_pass_rational_default()
        .with_profile_s_intrinsic_mpa(f64::from(profile.powers.s_intrinsic));
    assert!(
        (eta - mix.dissipation_modulus_eta()).abs() < 1e-6,
        "scalar_fields η must match profile-threaded MixScalars"
    );
}
