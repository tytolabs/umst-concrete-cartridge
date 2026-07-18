// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! CLI ops IO boundary — parity digest + Darwin scratch target.

use assert_cmd::Command;
use serde_json::Value;
use std::error::Error;

#[test]
fn ops_parity_digest_locked() -> Result<(), Box<dyn Error>> {
    let assert = Command::cargo_bin("umst")?
        .arg("ops")
        .arg("parity-digest")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("parity digest: 149081fa81a6525f… OK"),
        "expected locked digest line, got: {stdout}"
    );
    Ok(())
}

#[test]
fn ops_parity_digest_json_wire() -> Result<(), Box<dyn Error>> {
    let assert = Command::cargo_bin("umst")?
        .arg("ops")
        .arg("parity-digest")
        .arg("--json")
        .assert()
        .success();
    let v: Value = serde_json::from_slice(assert.get_output().stdout.as_slice())?;
    assert_eq!(
        v["sha256"].as_str(),
        Some("149081fa81a6525fb66ff01924c6656f30e2b67846d9945a25427c7be38d20f3")
    );
    assert_eq!(v["matches_locked"].as_bool(), Some(true));
    Ok(())
}

#[test]
#[cfg(target_os = "macos")]
fn ops_scratch_target_darwin() -> Result<(), Box<dyn Error>> {
    let assert = Command::cargo_bin("umst")?
        .arg("ops")
        .arg("scratch-target")
        .arg("i-cli-1122")
        .arg("--print-env")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("UMST_DYNAMIC_TARGET_DIR="),
        "expected picker env output, got: {stdout}"
    );
    assert!(
        stdout.contains("UMST_TARGET_PRESSURE="),
        "expected pressure field, got: {stdout}"
    );
    Ok(())
}
