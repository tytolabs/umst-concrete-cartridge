// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! T2-S5 path-dep bridge witness — monolith `concrete_bridge` ≡ `umst-cartridge-concrete` SSOT.
//!
//! Proves the monolith path-dep seam routes composed constitutive evaluation through the
//! consumer cartridge without duplicating gate bytes. MCP harness routing remains frozen.

use umst_concrete_cartridge::api_consumer_compose::{
    gate_admissible_via_compose, gate_route_via_compose, mix_row_to_mix_scalars,
};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::concrete_bridge::{
    g0_probe_atom_state, gate_route_composed, ContinuumAtomRates, MixScalars, D_CLOSURE_ABS_TOL,
    GATE_W_C_REGIME_HYPERBOX_MAX, PSI_CLOSURE_ABS_TOL,
};
use umst_concrete_cartridge::homogeneous::{mix_row_from_scalar_spec, MixRow};

fn default_profile() -> Profile {
    Profile::load_bundled("default").expect("bundled default profile for S5 bridge witness")
}

fn parity_mix_row(profile: &Profile) -> MixRow {
    mix_row_from_scalar_spec(profile, 0.45, 0.0, 0.0, 0.0, 0.7, 28.0 * 24.0, 293.15)
}

#[test]
fn s5_compose_delegate_routes_through_bridge_not_direct_consumer_import() {
    let profile = default_profile();
    let row = parity_mix_row(&profile);
    let mix = mix_row_to_mix_scalars(&row);
    let via_compose = gate_route_via_compose(&profile, &row, 0.0);
    let via_bridge = gate_route_composed(
        &mix,
        g0_probe_atom_state(),
        ContinuumAtomRates::PASSIVE,
        0.0,
        PSI_CLOSURE_ABS_TOL,
        D_CLOSURE_ABS_TOL,
    );
    assert_eq!(via_compose.route.admissible, via_bridge.route.admissible);
    assert!(
        (via_compose.constitutive.psi_total() - via_bridge.constitutive.psi_total()).abs() < 1e-9,
        "api_consumer_compose must route through concrete_bridge seam"
    );
}

#[test]
fn s5_mcp_gate_admissible_matches_bridge_oracle() {
    let profile = default_profile();
    let mix_json = serde_json::json!({
        "w_c": "9/20",
        "temperature_k": "29315/100",
        "aggregate_volume_fraction": "7/10"
    });
    let via_mcp = gate_admissible_via_compose(&profile, &mix_json);
    let wire = MixScalars {
        key: None,
        w_c: 0.45,
        temperature_k: 293.15,
        fly_ash_pct: 0.0,
        silica_fume_pct: 0.0,
        superplasticiser_pct: 0.0,
        target_age_hours: 28.0 * 24.0,
        s_intrinsic_mpa: None,
        w_c_max: None,
        total_binder_kg_m3: None,
        aggregate_volume_fraction: None,
        strength_model: None,
        jennings_exponent: None,
    };
    let via_bridge = gate_route_composed(
        &wire,
        g0_probe_atom_state(),
        ContinuumAtomRates::PASSIVE,
        0.0,
        PSI_CLOSURE_ABS_TOL,
        D_CLOSURE_ABS_TOL,
    )
    .route
    .admissible;
    assert_eq!(
        via_mcp, via_bridge,
        "MCP gate path must match bridge oracle"
    );
}

#[test]
fn s5_mcp_gate_admissible_pass_high_wc_in_regime_at_hyperbox() {
    let profile = default_profile();
    let mix_json = serde_json::json!({
        "w_c": "13/20",
        "temperature_k": "29315/100",
        "aggregate_volume_fraction": "7/10",
        "target_age_hours": "672/1"
    });
    assert!(
        gate_admissible_via_compose(&profile, &mix_json),
        "pass_high_wc_in_regime must PASS @ hyperbox max {}",
        GATE_W_C_REGIME_HYPERBOX_MAX
    );
    let wire = umst_concrete_cartridge::api_consumer_compose::mix_json_to_mix_scalars_for_profile(
        &profile,
        &mix_json,
        Some("pass_high_wc_in_regime"),
    )
    .expect("fixture mix wire");
    assert!(
        (wire.effective_w_c() - GATE_W_C_REGIME_HYPERBOX_MAX).abs() < 1e-9,
        "effective w/c must sit at hyperbox pin"
    );
    let outcome = gate_route_composed(
        &wire,
        g0_probe_atom_state(),
        ContinuumAtomRates::PASSIVE,
        0.0,
        PSI_CLOSURE_ABS_TOL,
        D_CLOSURE_ABS_TOL,
    );
    assert!(
        outcome.route.admissible,
        "composed gate must PASS for pass_high_wc_in_regime"
    );
    assert!(
        outcome.constitutive.psi_closure_holds(PSI_CLOSURE_ABS_TOL),
        "ψ closure must hold at hyperbox pin"
    );
}
