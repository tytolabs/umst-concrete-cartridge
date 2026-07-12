// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! S0 parity harness scaffold (MCP_BUILD_PLAN).
//!
//! **HELD** — ignored by default. Enable only after human authorizes the §5.1 build.
//! Does not change the `umst-mcp` binary or default feature set.

use serde_json::{json, Value};
use umst_concrete_cartridge::research::contribution::gate_check_mix_result;
use umst_concrete_cartridge::research::types::Profile;

/// Placeholder mix used by future golden fixtures (must stay stable once fixtures land).
fn fixture_mix() -> Value {
    json!({
        "w_c": "9/20",
        "temperature_k": "29315/100",
        "aggregate_volume_fraction": "7/10"
    })
}

/// Once authorized: compare canonical JSON of `gate_check_mix_result` to a checked-in fixture.
#[test]
#[ignore = "held: MCP build not authorized — see docs/MCP_BUILD_PLAN.md Stage S0"]
fn gate_check_mix_result_parity_fixture() {
    let profile = Profile::load_bundled("default").expect("bundled default profile");
    let result = gate_check_mix_result(&profile, &fixture_mix()).expect("gate_check_mix_result");
    // Scaffold only: assert the call succeeds; fixture byte-compare lands in a follow-up commit
    // after human review of ARCHITECTURE.md + MCP_BUILD_PLAN.md.
    let _ = serde_json::to_string(&result).expect("serialize");
    panic!("parity fixture not yet checked in — intentional until USER authorizes S0 completion");
}
