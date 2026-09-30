// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

use assert_cmd::Command;
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[test]
fn predict_pipe_validates_and_matches_ranges() -> Result<(), Box<dyn Error>> {
    let schema_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schema/result.v2.json");
    let schema_text = fs::read_to_string(&schema_path)?;
    let schema_val: Value = serde_json::from_str(&schema_text)?;
    let validator = jsonschema::validator_for(&schema_val)?;

    let mut cmd = Command::cargo_bin("umst")?;
    let assert = cmd
        .arg("predict")
        .write_stdin(r#"{"w_c": 0.40, "temperature_k": 293.15}"#)
        .assert()
        .success();

    let stdout = assert.get_output().stdout.as_slice();
    let parsed: Value = serde_json::from_slice(stdout)?;
    assert_eq!(
        parsed["schema_version"].as_str(),
        Some("result.v2"),
        "default predict must emit result.v2"
    );
    assert!(parsed["warnings"].is_array(), "warnings must be an array");
    assert_eq!(
        parsed["calibration_profile"].as_str(),
        Some("default"),
        "default global profile"
    );
    if !validator.is_valid(&parsed) {
        let msg = validator
            .iter_errors(&parsed)
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(msg.into());
    }

    assert!(
        parsed
            .get("physics_pipeline")
            .map(|x| x.get("schema_version").is_some())
            .unwrap_or(false),
        "predict must attach physics_pipeline.schema_version",
    );

    // Physical envelope checks — tensor pipeline + manifold summary tensors (ranges widened versus legacy homogeneous-only QA).
    let alpha = parsed["degree_of_hydration"]
        .as_f64()
        .ok_or("degree_of_hydration missing or not a number")?;
    assert!(
        (0.0..=1.0).contains(&alpha),
        "degree_of_hydration {alpha} outside [0, 1]",
    );

    let fc = parsed["compressive_strength_mpa"]
        .as_f64()
        .ok_or("compressive_strength_mpa missing or not a number")?;
    assert!(
        fc.is_finite(),
        "compressive_strength_mpa must be finite, got {fc}",
    );

    let tau = parsed["yield_stress_pa"]
        .as_f64()
        .ok_or("yield_stress_pa missing or not a number")?;
    assert!(tau.is_finite(), "yield_stress_pa must be finite, got {tau}",);

    let gwp = parsed["gwp_kg_co2_eq_per_m3"]
        .as_f64()
        .ok_or("gwp_kg_co2_eq_per_m3 missing or not a number")?;
    assert!(gwp.is_finite(), "GWP must be finite, got {gwp}",);

    let safety = parsed["safety_margin"]
        .as_f64()
        .ok_or("safety_margin missing or not a number")?;
    assert!(
        (0.0..=1.0).contains(&safety),
        "safety_margin {safety} outside [0, 1]"
    );

    Ok(())
}

#[test]
fn predict_v1_still_matches_schema_when_flagged() -> Result<(), Box<dyn Error>> {
    let schema_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schema/result.v1.json");
    let schema_text = fs::read_to_string(&schema_path)?;
    let schema_val: Value = serde_json::from_str(&schema_text)?;
    let validator = jsonschema::validator_for(&schema_val)?;

    let mut cmd = Command::cargo_bin("umst")?;
    let assert = cmd
        .arg("predict")
        .arg("--schema-version")
        .arg("v1")
        .write_stdin(r#"{"w_c": 0.40, "temperature_k": 293.15}"#)
        .assert()
        .success();

    let parsed: Value = serde_json::from_slice(assert.get_output().stdout.as_slice())?;
    assert_eq!(parsed["schema_version"].as_str(), Some("result.v1"));
    if !validator.is_valid(&parsed) {
        let msg = validator
            .iter_errors(&parsed)
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(msg.into());
    }
    Ok(())
}

/// Determinism: same input must yield byte-identical output.
#[test]
fn predict_is_deterministic() -> Result<(), Box<dyn Error>> {
    let invoke = || -> Result<Vec<u8>, Box<dyn Error>> {
        let mut cmd = Command::cargo_bin("umst")?;
        let assert = cmd
            .arg("predict")
            .write_stdin(r#"{"w_c": 0.40, "temperature_k": 293.15}"#)
            .assert()
            .success();
        Ok(assert.get_output().stdout.clone())
    };
    let a = invoke()?;
    let b = invoke()?;
    assert_eq!(a, b, "predict produced non-deterministic output");
    Ok(())
}
