// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
use assert_cmd::Command;
use jsonschema::{Draft, Validator};
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use umst_cli::audit::audit_csv_buf;
use umst_concrete_cartridge::calibration::Profile;

fn datasets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../datasets")
}

fn schema_audit_v1_root() -> Value {
    let p = datasets_dir()
        .parent()
        .unwrap()
        .join("schema/audit.v1.json");
    let s = fs::read_to_string(p).unwrap();
    serde_json::from_str(&s).unwrap()
}

#[test]
fn audit_cli_uci_d1_json_validates_schema() -> Result<(), Box<dyn Error>> {
    let profile = Profile::load_bundled("uci_d1")?;
    let csv_path = datasets_dir().join("dataset_d1.csv");
    let text = fs::read_to_string(&csv_path)?;
    let v = audit_csv_buf(&profile, &text, Some(10))?;

    let schema = schema_audit_v1_root();
    let validator = Validator::options()
        .with_draft(Draft::Draft202012)
        .build(&schema)?;
    if let Err(e) = validator.validate(&v) {
        return Err(format!("schema validation: {e:?}").into());
    }
    Ok(())
}

#[test]
fn audit_cli_bin_streams_valid_json_first_ten_rows() -> Result<(), Box<dyn Error>> {
    let csv_path = datasets_dir().join("dataset_d1.csv");

    let mut cmd = Command::cargo_bin("umst")?;
    let assert = cmd
        .args(["--profile", "uci_d1", "audit", "--input"])
        .arg(&csv_path)
        .args(["--limit", "10"])
        .assert()
        .success();

    let got: Value = serde_json::from_slice(assert.get_output().stdout.as_slice())?;
    assert_eq!(got["schema_version"], "audit.v1");
    assert_eq!(got["summary"]["row_count"], 10);
    Ok(())
}

#[test]
fn audit_first_ten_predictions_finite() -> Result<(), Box<dyn Error>> {
    let profile = Profile::load_bundled("uci_d1")?;
    let csv_path = datasets_dir().join("dataset_d1.csv");
    let text = fs::read_to_string(&csv_path)?;
    let v = audit_csv_buf(&profile, &text, Some(10))?;
    let rows = v["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 10);
    for row in rows {
        let p = row["predicted_strength_mpa"]
            .as_f64()
            .expect("predicted strength");
        assert!(p.is_finite() && p >= 0.0, "non-finite strength {p}");
    }
    let mae = v["summary"]["mean_absolute_error_mpa"]
        .as_f64()
        .expect("MAE with observations");
    assert!(mae.is_finite());
    Ok(())
}
