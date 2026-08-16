// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Cold-path export for six-mix fixture land — operator GO only.
//! Run: `cargo test -p umst-concrete-cartridge six_mix_cold_export -- --ignored --nocapture`

use serde_json::{json, Value};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::research::{gate_check_mix_result, ObservedAt};

fn frozen_observed() -> ObservedAt {
    ObservedAt {
        stamp_tier: "Synthetic".into(),
        ucrs_seq: Some(0),
        phase_entropy_bits_q: None,
        phase_entropy_bits_scale: None,
        credit_head_bits_q: None,
        credit_head_bits_scale: None,
        wall_ms: Some(1000),
    }
}

fn sort_keys(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().cloned().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
            for k in keys {
                out.insert(k.clone(), sort_keys(map[&k].clone()));
            }
            Value::Object(out)
        }
        Value::Array(a) => Value::Array(a.into_iter().map(sort_keys).collect()),
        other => other,
    }
}

/// DRAFT §5.1 mix_table — verdicts from library output only (not invented).
fn draft_mix_table() -> serde_json::Map<String, Value> {
    serde_json::from_value(json!({
        "pass_rational_default": {
            "w_c": "9/20",
            "temperature_k": "29315/100",
            "aggregate_volume_fraction": "7/10"
        },
        "reject_high_wc": {
            "w_c": "3/4",
            "temperature_k": "29315/100",
            "aggregate_volume_fraction": "7/10",
            "superplasticiser_pct": "0/1",
            "silica_fume_pct": "0/1",
            "fly_ash_pct": "0/1",
            "target_age_hours": "672/1"
        },
        "pass_low_wc_mature": {
            "w_c": "3/10",
            "temperature_k": "29315/100",
            "aggregate_volume_fraction": "7/10",
            "target_age_hours": "672/1"
        },
        "pass_early_age": {
            "w_c": "9/20",
            "temperature_k": "29315/100",
            "aggregate_volume_fraction": "7/10",
            "target_age_hours": "1/1"
        },
        "pass_high_wc_in_regime": {
            "w_c": "13/20",
            "temperature_k": "29315/100",
            "aggregate_volume_fraction": "7/10",
            "target_age_hours": "672/1"
        },
        "reject_cold_regime": {
            "w_c": "9/20",
            "temperature_k": "277/1",
            "aggregate_volume_fraction": "7/10",
            "target_age_hours": "672/1"
        }
    }))
    .expect("draft mix_table")
}

#[test]
#[ignore = "operator cold capture — rustc 1.88+ only; run via scripts/six_mix_cold_capture.sh"]
fn six_mix_cold_export() {
    let profile = Profile::load_bundled("default").expect("default profile");
    let observed = frozen_observed();
    let mixes = draft_mix_table();
    let mut gate_results = serde_json::Map::new();

    for (id, mix) in &mixes {
        let result = gate_check_mix_result(&profile, mix, true, observed.clone());
        let actual = sort_keys(serde_json::to_value(&result).expect("to_value"));
        gate_results.insert(id.clone(), actual);
    }

    let export = json!({
        "mix_table": mixes,
        "gate_check_mix_result": gate_results,
        "observed_at": {
            "stamp_tier": "Synthetic",
            "ucrs_seq": 0,
            "wall_ms": 1000
        }
    });

    println!("{}", serde_json::to_string_pretty(&export).expect("pretty"));
}
