// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

#![cfg(feature = "agent-layer")]

//! In-process gate batch hot loop — Phase 2 witness companion to MCP stdio.

use serde_json::json;
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::research::{gate_check_mix, GateVerdict};

#[test]
fn inprocess_gate_batch_hot_loop() {
    let iters: usize = std::env::var("UMST_INPROCESS_GATE_ITERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);
    let profile = Profile::load_bundled("default").expect("default profile");
    let mix = json!({
        "w_c": "9/20",
        "temperature_k": "29315/100",
        "aggregate_volume_fraction": "7/10",
    });
    let mut passes = 0usize;
    for _ in 0..iters {
        let summary = gate_check_mix(&profile, &mix);
        if summary.verdict == GateVerdict::Pass {
            passes += 1;
        }
    }
    assert!(passes > 0, "expected at least one PASS in batch");
    eprintln!("inprocess_gate_batch_ok iters={iters} passes={passes}");
}
