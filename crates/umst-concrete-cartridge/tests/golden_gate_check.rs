// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
#![cfg(feature = "agent-layer")]

use serde_json::Value;
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::research::{gate_check_mix, GateVerdict};

fn repo_root() -> String {
    env!("CARGO_MANIFEST_DIR").replace("/crates/umst-concrete-cartridge", "")
}

fn load_json(path: &str) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

fn f64_to_rational(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{}/1", v.round() as i64)
    } else {
        let scaled = (v * 100.0).round() as i64;
        format!("{scaled}/100")
    }
}

fn mix_input_to_spec(mix: &Value) -> Value {
    serde_json::json!({
        "w_c": f64_to_rational(mix["w_c"].as_f64().unwrap()),
        "temperature_k": f64_to_rational(mix["temperature_k"].as_f64().unwrap()),
        "aggregate_volume_fraction": f64_to_rational(mix["aggregate_volume_fraction"].as_f64().unwrap_or(0.7)),
        "superplasticiser_pct": f64_to_rational(mix["superplasticiser_pct"].as_f64().unwrap_or(0.0)),
        "silica_fume_pct": f64_to_rational(mix["silica_fume_pct"].as_f64().unwrap_or(0.0)),
        "fly_ash_pct": f64_to_rational(mix["fly_ash_pct"].as_f64().unwrap_or(0.0)),
        "target_age_hours": f64_to_rational(mix["target_age_hours"].as_f64().unwrap_or(672.0)),
    })
}

#[test]
fn golden_adversarial_gate_check_verdicts() {
    let root = repo_root();
    let expected_path = format!("{root}/fixtures/golden-adversarial/expected_verdicts.json");
    let expected = load_json(&expected_path);
    let profile = Profile::load_bundled("default").expect("default profile");

    for fixture in expected["fixtures"].as_array().expect("fixtures array") {
        let mix = mix_input_to_spec(&fixture["mix_input"]);
        let summary = gate_check_mix(&profile, &mix);
        let exp = &fixture["expected"];
        assert_eq!(
            summary.admissible,
            exp["admissible"].as_bool().unwrap(),
            "admissible mismatch for {}",
            fixture["file"]
        );
        let verdict = match summary.verdict {
            GateVerdict::Pass => "PASS",
            GateVerdict::Reject => "REJECT",
            GateVerdict::Warn => "WARN",
        };
        assert_eq!(
            verdict,
            exp["verdict"].as_str().unwrap(),
            "verdict mismatch for {}",
            fixture["file"]
        );
    }
}
