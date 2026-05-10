// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Live-binary smoke tests for public CLI contracts (acceptance checks 7–10).

#![cfg(feature = "cli")]

use assert_cmd::Command;
use serde_json::Value;
use std::collections::BTreeSet;
use std::error::Error;

use umst_concrete_cartridge::calibration::BUNDLED_PROFILE_IDS;

#[test]
fn acceptance_7_profiles_list_matches_bundled_ids() -> Result<(), Box<dyn Error>> {
    let assert = Command::cargo_bin("umst")?
        .arg("profiles")
        .arg("list")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let mut ids = BTreeSet::new();
    for line in stdout.lines() {
        let id = line.split('\t').next().unwrap_or(line).trim();
        if id.is_empty() {
            continue;
        }
        ids.insert(id.to_string());
    }
    let expected: BTreeSet<String> = BUNDLED_PROFILE_IDS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    assert_eq!(
        ids, expected,
        "profiles list must enumerate exactly BUNDLED_PROFILE_IDS"
    );
    Ok(())
}

#[test]
fn acceptance_8_predict_uci_d1_populates_v2_contract_fields() -> Result<(), Box<dyn Error>> {
    let assert = Command::cargo_bin("umst")?
        .arg("--profile")
        .arg("uci_d1")
        .arg("predict")
        .write_stdin(r#"{"w_c": 0.40, "temperature_k": 293.15}"#)
        .assert()
        .success();
    let v: Value = serde_json::from_slice(assert.get_output().stdout.as_slice())?;
    assert_eq!(v["schema_version"].as_str(), Some("result.v2"));
    assert_eq!(v["calibration_profile"].as_str(), Some("uci_d1"));
    let model = v["calibration_model"]
        .as_str()
        .ok_or("calibration_model missing")?;
    assert!(!model.is_empty());
    let anchor = v["formal_anchor"].as_str().ok_or("formal_anchor missing")?;
    assert!(
        anchor.starts_with("lean://"),
        "formal_anchor should be a Lean URI, got {anchor:?}"
    );
    Ok(())
}

#[test]
fn acceptance_9_high_temperature_yields_regime_warnings() -> Result<(), Box<dyn Error>> {
    let assert = Command::cargo_bin("umst")?
        .arg("--profile")
        .arg("uci_d1")
        .arg("predict")
        .write_stdin(r#"{"w_c": 0.40, "temperature_k": 340}"#)
        .assert()
        .success();
    let v: Value = serde_json::from_slice(assert.get_output().stdout.as_slice())?;
    let warns = v["warnings"].as_array().ok_or("warnings not array")?;
    assert!(
        !warns.is_empty(),
        "uci_d1 regime caps temperature_k below 340 K — expect non-empty warnings"
    );
    let hits_temperature = warns.iter().any(|w| {
        w.as_str()
            .unwrap_or("")
            .to_ascii_lowercase()
            .contains("temperature")
    });
    assert!(
        hits_temperature,
        "warnings should name temperature dimension: {warns:?}"
    );
    Ok(())
}

#[test]
fn acceptance_10_certify_emits_chain_json() -> Result<(), Box<dyn Error>> {
    let assert = Command::cargo_bin("umst")?
        .arg("certify")
        .arg("uci_d1")
        .assert()
        .success();
    let v: Value = serde_json::from_slice(assert.get_output().stdout.as_slice())?;
    assert!(v["profile"].is_string());
    assert!(v["model_anchor"].is_string());
    assert!(v["acceptance_anchor"].is_string());
    assert!(v["axioms"].is_array());
    let formal_status = v["formal_status"]
        .as_str()
        .ok_or("formal_status must be a string")?;
    assert!(
        matches!(
            formal_status,
            "Mechanised" | "Structural" | "Empirical" | "Literature" | "NONE"
        ),
        "unexpected formal_status: {formal_status:?}"
    );
    Ok(())
}
