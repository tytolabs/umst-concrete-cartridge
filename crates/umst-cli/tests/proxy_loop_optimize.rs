// SPDX-License-Identifier: MIT
// WS-TRACK-A: CLI integration — dual-gated printable_window optimise + sidecar JSON.

use assert_cmd::Command;
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[test]
fn optimize_printable_window_passes_dual_gate() -> Result<(), Box<dyn Error>> {
    let tmp = tempfile_named("proposed_next_mix.json");
    let mix = r#"{
        "w_c": 0.45,
        "temperature_k": 298.15,
        "superplasticiser_pct": 1.0,
        "silica_fume_pct": 10.0,
        "aggregate_volume_fraction": 0.35,
        "target_age_hours": 1.0
    }"#;

    let mut cmd = Command::cargo_bin("umst")?;
    cmd.arg("--profile")
        .arg("tyto_mortar")
        .arg("optimize")
        .arg("--target")
        .arg("printable_window=1")
        .arg("--steps")
        .arg("16")
        .arg("--output")
        .arg(&tmp)
        .write_stdin(mix)
        .assert()
        .success();

    let sidecar: Value = serde_json::from_str(&fs::read_to_string(&tmp)?)?;
    assert_eq!(sidecar["schema_version"], "proposed_next_mix.v1");
    assert_eq!(sidecar["calibration_profile"], "tyto_mortar");
    assert!(
        sidecar["dual_gate"]["passes"].as_bool().unwrap_or(false),
        "proposed mix should pass dual gate: {sidecar}"
    );
    assert!(sidecar["dual_gate"]["printability_ok"]
        .as_bool()
        .unwrap_or(false));
    assert!(sidecar["dual_gate"]["thermodynamic_ok"]
        .as_bool()
        .unwrap_or(false));

    fs::remove_file(&tmp).ok();
    Ok(())
}

fn tempfile_named(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("umst_proxy_loop_test");
    fs::create_dir_all(&dir).ok();
    dir.join(name)
}
