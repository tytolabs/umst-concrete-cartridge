// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

use assert_cmd::Command;
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[test]
fn schema_mix_matches_bundled_file() -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schema/mix.v1.json");
    let disk_text = fs::read_to_string(&path)?;
    let expected: Value = serde_json::from_str(&disk_text)?;

    let mut cmd = Command::cargo_bin("umst")?;
    let assert = cmd.arg("schema").arg("mix").assert().success();
    let got: Value = serde_json::from_slice(assert.get_output().stdout.as_slice())?;

    assert_eq!(got, expected);
    Ok(())
}

#[test]
fn schema_result_matches_bundled_file() -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schema/result.v1.json");
    let disk_text = fs::read_to_string(&path)?;
    let expected: Value = serde_json::from_str(&disk_text)?;

    let mut cmd = Command::cargo_bin("umst")?;
    let assert = cmd.arg("schema").arg("result").assert().success();
    let got: Value = serde_json::from_slice(assert.get_output().stdout.as_slice())?;

    assert_eq!(got, expected);
    Ok(())
}

#[test]
fn schema_result_v2_matches_bundled_file() -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schema/result.v2.json");
    let disk_text = fs::read_to_string(&path)?;
    let expected: Value = serde_json::from_str(&disk_text)?;

    let mut cmd = Command::cargo_bin("umst")?;
    let assert = cmd.arg("schema").arg("result-v2").assert().success();
    let got: Value = serde_json::from_slice(assert.get_output().stdout.as_slice())?;

    assert_eq!(got, expected);
    Ok(())
}

#[test]
fn schema_audit_matches_bundled_file() -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schema/audit.v1.json");
    let disk_text = fs::read_to_string(&path)?;
    let expected: Value = serde_json::from_str(&disk_text)?;

    let mut cmd = Command::cargo_bin("umst")?;
    let assert = cmd.arg("schema").arg("audit").assert().success();
    let got: Value = serde_json::from_slice(assert.get_output().stdout.as_slice())?;

    assert_eq!(got, expected);
    Ok(())
}
