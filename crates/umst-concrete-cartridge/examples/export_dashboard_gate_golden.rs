// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Export frozen `gate_check_mix` verdicts for the UMST dashboard parity bundle.
//! Run: `cargo run -p umst-concrete-cartridge --features agent-layer --example export_dashboard_gate_golden`

use serde_json::{json, Value};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::research::{gate_check_mix, GateVerdict};

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
        "temperature_k": f64_to_rational(mix["temperature_k"].as_f64().unwrap_or(293.15)),
        "aggregate_volume_fraction": f64_to_rational(mix["aggregate_volume_fraction"].as_f64().unwrap_or(0.7)),
        "superplasticiser_pct": f64_to_rational(mix["superplasticiser_pct"].as_f64().unwrap_or(0.0)),
        "silica_fume_pct": f64_to_rational(mix["silica_fume_pct"].as_f64().unwrap_or(0.0)),
        "fly_ash_pct": f64_to_rational(mix["fly_ash_pct"].as_f64().unwrap_or(0.0)),
        "target_age_hours": f64_to_rational(mix["target_age_hours"].as_f64().unwrap_or(672.0)),
    })
}

fn verdict_tag(v: GateVerdict) -> &'static str {
    match v {
        GateVerdict::Pass => "PASS",
        GateVerdict::Reject => "REJECT",
        GateVerdict::Warn => "WARN",
    }
}

fn main() {
    let profile = Profile::load_bundled("default").expect("default profile");
    let wc_grid: &[f64] = &[
        0.30, 0.32, 0.34, 0.36, 0.38, 0.40, 0.42, 0.45, 0.48, 0.50, 0.52, 0.55, 0.58, 0.62, 0.75,
    ];

    let mut anchors = Vec::new();
    for wc in wc_grid {
        let mix_input = json!({
            "w_c": wc,
            "temperature_k": 293.15,
            "aggregate_volume_fraction": 0.7,
            "superplasticiser_pct": 0.0,
            "silica_fume_pct": 0.0,
            "fly_ash_pct": 0.0,
            "target_age_hours": 672.0,
        });
        let spec = mix_input_to_spec(&mix_input);
        let summary = gate_check_mix(&profile, &spec);
        anchors.push(json!({
            "w_c": wc,
            "mix_spec": spec,
            "expected": {
                "admissible": summary.admissible,
                "verdict": verdict_tag(summary.verdict),
            },
        }));
    }

    let out = json!({
        "schema_version": "cartridge_gate_anchor.v1",
        "description": "Frozen umst_gate_check (gate_check_mix) verdicts — default profile, standard mix wire",
        "profile_id": profile.bundle_id,
        "source": "umst-concrete-cartridge/examples/export_dashboard_gate_golden.rs",
        "anchors": anchors,
    });

    println!("{}", serde_json::to_string_pretty(&out).expect("serialize"));
}
