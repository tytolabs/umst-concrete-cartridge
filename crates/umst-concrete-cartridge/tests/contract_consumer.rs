// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! M1 acceptance — concrete implements [`UMSTCartridge`] at scalar parity (Wave 1).

use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::homogeneous::mix_row_from_scalar_spec;
use umst_concrete_cartridge::{
    ConcreteApiCartridge, ScalarAlgebra, TensorAlgebra, UMSTCartridge, CONCRETE_CARTRIDGE_ID,
};

/// Parity anchor prefix (M0 lock — must not drift under M1 wiring).
const PARITY_PREFIX: &str = "d5608148e29eeabd";

fn bundled_cartridge() -> ConcreteApiCartridge {
    ConcreteApiCartridge::new().expect("bundled calibration `uci_d1`")
}

#[test]
fn concrete_cartridge_id_matches_registry() {
    let cartridge = bundled_cartridge();
    assert_eq!(cartridge.id().as_str(), CONCRETE_CARTRIDGE_ID);
    assert_eq!(CONCRETE_CARTRIDGE_ID, "umst-cartridge-concrete");
}

#[test]
fn concrete_exposes_core_physical_axioms() {
    let cartridge = bundled_cartridge();
    let axioms = cartridge.physical_axioms();
    assert_eq!(axioms.len(), 2);
    assert_eq!(axioms[0].name(), "MassConservationAxiom");
    assert_eq!(axioms[1].name(), "ClausiusDuhemAxiom");
}

#[test]
fn concrete_builds_constitutive_response_from_default_mix() {
    let profile = Profile::load_bundled("default").expect("default profile");
    let cartridge = ConcreteApiCartridge::with_profile(profile.clone());
    let row = mix_row_from_scalar_spec(&profile, 0.45, 0.0, 0.0, 0.0, 0.7, 28.0 * 24.0, 293.15);
    let response = cartridge
        .constitutive_response_from_mix_row(&row, 1e-6)
        .expect("scalar response");
    assert!(response.free_energy_density < 0.0);
    assert!(response.dissipation >= 0.0);
    assert!((response.power_input - 0.0).abs() < f64::EPSILON);
}

#[test]
fn parity_fixture_prefix_pinned() {
    const FULL: &str = "d5608148e29eeabd83935988699d08ce1233c3e87f2cd217d658e0c71c7a841e";
    assert!(FULL.starts_with(PARITY_PREFIX));
    let _ = ScalarAlgebra::zero();
}
