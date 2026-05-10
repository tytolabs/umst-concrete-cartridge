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
    let schema_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schema/result.v1.json");
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
    if !validator.is_valid(&parsed) {
        let msg = validator
            .iter_errors(&parsed)
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(msg.into());
    }

    let alpha = parsed["degree_of_hydration"]
        .as_f64()
        .ok_or("degree_of_hydration missing or not a number")?;
    assert!((0.0..=1.0).contains(&alpha));

    let fc = parsed["compressive_strength_mpa"]
        .as_f64()
        .ok_or("compressive_strength_mpa missing or not a number")?;
    assert!(fc > 0.0);

    Ok(())
}
