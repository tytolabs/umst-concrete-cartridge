// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! M1 parity harness — concrete implements `umst-cartridge-api` without changing gate bytes.

use umst_cartridge_api::UMSTCartridge;
use umst_concrete_cartridge::api_consumer::{ConcreteApiCartridge, CONCRETE_CARTRIDGE_ID};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::homogeneous::{mix_row_from_scalar_spec, MixRow};
use umst_concrete_cartridge::research::{gate_check_mix, gate_check_mix_result, ObservedAt};
use umst_manifold::gate::{GATE_PARITY_V0_SHA256, GATE_PARITY_V0_SHA256_PREFIX};

fn default_profile() -> Profile {
    Profile::load_bundled("default").expect(
        "Profile::load_bundled(\"default\") for M1 api_consumer parity harness (FP §6 Track M1 api_consumer parity)",
    )
}

fn parity_mix_row(profile: &Profile) -> MixRow {
    mix_row_from_scalar_spec(profile, 0.45, 0.0, 0.0, 0.0, 0.7, 28.0 * 24.0, 293.15)
}

#[test]
fn delegate_wire_path_uses_gate_route_composed() {
    let profile = default_profile();
    let cartridge = ConcreteApiCartridge::with_profile(profile.clone());
    let row = parity_mix_row(&profile);
    let outcome = cartridge.gate_route_via_compose(&row, 0.0);
    assert!(
        outcome.route.admissible,
        "G1 parity mix must PASS via composed delegate"
    );
    assert!(outcome.constitutive.psi_closure_holds(1e-3));
}

#[test]
fn concrete_implements_umst_cartridge_contract() {
    let cartridge = ConcreteApiCartridge::with_profile(default_profile());
    assert_eq!(cartridge.id().as_str(), CONCRETE_CARTRIDGE_ID);
    assert_eq!(cartridge.physical_axioms().len(), 2);
    assert_eq!(cartridge.state_schema().vars.len(), 3);
}

#[test]
fn constitutive_response_is_passive_and_non_negative_dissipation() {
    let profile = default_profile();
    let cartridge = ConcreteApiCartridge::with_profile(profile.clone());
    let row = parity_mix_row(&profile);
    let response = cartridge
        .constitutive_response_from_mix_row(&row, 0.01)
        .expect(
            "ConcreteApiCartridge::constitutive_response_from_mix_row on parity MixRow (FP §6 Track M1 api_consumer parity)",
        );
    assert!(response.dissipation >= 0.0);
    assert!((response.power_input - 0.0).abs() < f64::EPSILON);
    assert!(response.free_energy_density < 0.0);
}

#[test]
fn gate_admissibility_unchanged_for_parity_mixes() {
    let profile = default_profile();
    let cartridge = ConcreteApiCartridge::with_profile(profile.clone());
    let mixes = [
        (
            serde_json::json!({
                "w_c": "9/20",
                "temperature_k": "29315/100",
                "aggregate_volume_fraction": "7/10"
            }),
            true,
        ),
        (
            serde_json::json!({
                "w_c": "3/4",
                "temperature_k": "29315/100",
                "aggregate_volume_fraction": "7/10",
                "superplasticiser_pct": "0/1",
                "silica_fume_pct": "0/1",
                "fly_ash_pct": "0/1",
                "target_age_hours": "672/1"
            }),
            false,
        ),
    ];

    for (mix, expect_admissible) in mixes {
        let gate = gate_check_mix(&profile, &mix);
        assert_eq!(
            gate.admissible, expect_admissible,
            "gate_check_mix admissibility drift for mix={mix}"
        );

        let row = parity_mix_row(&profile);
        let _ = cartridge
            .constitutive_response_from_mix_row(&row, 0.0)
            .expect(
                "constitutive_response_from_mix_row admissibility drift guard lift at zero rate (FP §6 Track M1 api_consumer parity)",
            );

        let observed = ObservedAt {
            stamp_tier: "Synthetic".into(),
            ucrs_seq: Some(0),
            phase_entropy_bits_q: None,
            phase_entropy_bits_scale: None,
            credit_head_bits_q: None,
            credit_head_bits_scale: None,
            wall_ms: Some(1000),
        };
        let cold = gate_check_mix_result(&profile, &mix, true, observed);
        assert_eq!(
            cold.gate_summary.admissible, expect_admissible,
            "gate_check_mix_result admissibility drift for mix={mix}"
        );
    }
}

#[test]
fn parity_fixture_sha256_prefix_is_pinned() {
    assert!(GATE_PARITY_V0_SHA256.starts_with(GATE_PARITY_V0_SHA256_PREFIX));
}

#[test]
fn scalar_algebra_idempotency_on_zero_rates() {
    let profile = default_profile();
    let cartridge = ConcreteApiCartridge::with_profile(profile.clone());
    let row = parity_mix_row(&profile);
    let r0 = cartridge
        .constitutive_response_from_mix_row(&row, 0.0)
        .expect(
            "constitutive_response_from_mix_row idempotency first arm at zero rate (FP §6 Track M1 api_consumer parity)",
        );
    let r1 = cartridge
        .constitutive_response_from_mix_row(&row, 0.0)
        .expect(
            "constitutive_response_from_mix_row idempotency second arm at zero rate (FP §6 Track M1 api_consumer parity)",
        );
    assert_eq!(r0, r1);
}

#[test]
fn core_axiom_witnesses_accept_hydration_transition() {
    use umst_cartridge_api::{
        ClausiusDuhemWitness, MassConservationWitness, PhysicalAxiom, StateSnapshot,
    };

    let profile = default_profile();
    let cartridge = ConcreteApiCartridge::with_profile(profile.clone());
    let row = parity_mix_row(&profile);
    let (schema, before_vals) = cartridge
        .scalar_state_from_mix_row(&row)
        .expect(
            "scalar_state_from_mix_row before hydration transition StateSnapshot (FP §6 Track M1 api_consumer parity)",
        );
    let before = StateSnapshot {
        density: before_vals[1],
        free_energy: before_vals[0],
        internals: vec![before_vals[0]],
    };

    let mut aged = row.clone();
    aged.age_days = (aged.age_days + 7.0).min(365.0);
    let (_, after_vals) = cartridge
        .scalar_state_from_mix_row(&aged)
        .expect(
            "scalar_state_from_mix_row after hydration transition StateSnapshot (FP §6 Track M1 api_consumer parity)",
        );
    let after = StateSnapshot {
        density: after_vals[1],
        free_energy: after_vals[0],
        internals: vec![after_vals[0]],
    };
    let _ = schema;

    let mass = MassConservationWitness::parity_default();
    let cd = ClausiusDuhemWitness::default();
    let w_mass = mass.verify_transition(&before, &after);
    let w_cd = cd.verify_transition(&before, &after);
    assert!(w_mass.satisfied);
    assert!(w_cd.satisfied);
}

/// T2-DMG slice-2 — production delegate threads history; verdict matches bare path.
#[test]
fn delegate_history_threading_matches_bare_verdict_at_g0() {
    let profile = default_profile();
    let cartridge = ConcreteApiCartridge::with_profile(profile.clone());
    let row = parity_mix_row(&profile);
    let bare = cartridge.gate_route_via_compose(&row, 0.0);
    let strict = cartridge
        .try_gate_route_via_compose_with_history(&row, 0.0, 1.0)
        .expect("strict production delegate history");
    let saturating = cartridge.gate_route_via_compose_with_history(&row, 0.0, 1.0);
    assert_eq!(strict, saturating);
    assert_eq!(
        bare, strict.0,
        "history threading must not change gate verdict"
    );
    assert!(bare.route.admissible);
}
