// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! S0 parity harness scaffold (`docs/MCP_BUILD_PLAN.md` Stage S0).
//!
//! **HELD** — ignored by default; requires `--features agent-layer` to compile the body.
//! Does not change the `umst-mcp` binary or default feature set.

#![cfg(feature = "agent-layer")]

use serde_json::{json, Value};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::research::{gate_check_mix_result, ObservedAt};

/// Placeholder mix for future golden fixtures (freeze once fixtures land).
fn fixture_mix() -> Value {
    json!({
        "w_c": "9/20",
        "temperature_k": "29315/100",
        "aggregate_volume_fraction": "7/10"
    })
}

fn synthetic_observed(seq: u64) -> ObservedAt {
    ObservedAt {
        stamp_tier: "Synthetic".into(),
        ucrs_seq: Some(seq),
        phase_entropy_bits_q: None,
        phase_entropy_bits_scale: None,
        credit_head_bits_q: None,
        credit_head_bits_scale: None,
        wall_ms: Some(1_000),
    }
}

/// Once authorized: compare canonical JSON of `gate_check_mix_result` to a checked-in fixture.
#[test]
#[ignore = "held: MCP build not authorized — see MaOS docs/MCP_BUILD_PLAN.md Stage S0"]
fn gate_check_mix_result_parity_fixture() {
    let profile = Profile::load_bundled("default").expect("bundled default profile");
    let result = gate_check_mix_result(&profile, &fixture_mix(), true, synthetic_observed(0));
    let _ = serde_json::to_string(&result.gate_summary).expect("serialize gate_summary");
    // Scaffold only: fixture byte-compare lands after human review of ARCHITECTURE + MCP_BUILD_PLAN.
    panic!("parity fixture not yet checked in — intentional until USER authorizes S0 completion");
}
