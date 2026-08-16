// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! S4 parity: FFI out ↔ `gate_check_mix_result` (canonical JSON + admissible/verdict).

use serde_json::{json, Value};
use std::ffi::CString;
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::research::{gate_check_mix_result, ObservedAt};
use umst_gate_ffi::{
    sort_keys, umst_gate_check, umst_gate_ffi_abi_version, verdict_code, CGateSummary,
};

fn frozen_observed() -> ObservedAt {
    ObservedAt {
        stamp_tier: "Synthetic".into(),
        ucrs_seq: Some(0),
        phase_entropy_bits_q: None,
        phase_entropy_bits_scale: None,
        credit_head_bits_q: None,
        credit_head_bits_scale: None,
        wall_ms: Some(1_000),
    }
}

fn mix_table() -> Vec<(&'static str, Value)> {
    vec![
        (
            "pass_rational_default",
            json!({
                "w_c": "9/20",
                "temperature_k": "29315/100",
                "aggregate_volume_fraction": "7/10"
            }),
        ),
        (
            "reject_high_wc",
            json!({
                "w_c": "3/4",
                "temperature_k": "29315/100",
                "aggregate_volume_fraction": "7/10",
                "superplasticiser_pct": "0/1",
                "silica_fume_pct": "0/1",
                "fly_ash_pct": "0/1",
                "target_age_hours": "672/1"
            }),
        ),
    ]
}

#[test]
fn abi_version_is_one() {
    assert_eq!(umst_gate_ffi_abi_version(), 1);
}

#[test]
fn ffi_matches_rust_gate_check_mix_result() {
    let profile = Profile::load_bundled("default").expect("profile");
    let observed = frozen_observed();

    for (id, mix) in mix_table() {
        let cold = gate_check_mix_result(&profile, &mix, true, observed.clone());
        let cold_json = serde_json::to_value(&cold).unwrap();
        let cold_canon = serde_json::to_string(&sort_keys(cold_json)).unwrap();

        let profile_c = CString::new("default").unwrap();
        let mix_c = CString::new(serde_json::to_string(&mix).unwrap()).unwrap();
        let mut summary = CGateSummary {
            admissible: 0,
            verdict: -1,
        };
        let mut buf = vec![0u8; 64 * 1024];
        let rc = unsafe {
            umst_gate_check(
                profile_c.as_ptr(),
                mix_c.as_ptr(),
                1,
                0,
                1_000,
                &mut summary,
                buf.as_mut_ptr() as *mut i8,
                buf.len(),
            )
        };
        assert_eq!(rc, 0, "ffi rc for {id}");
        assert_eq!(
            summary.admissible != 0,
            cold.gate_summary.admissible,
            "admissible {id}"
        );
        assert_eq!(
            summary.verdict,
            verdict_code(cold.gate_summary.verdict),
            "verdict {id}"
        );

        let nul = buf.iter().position(|&b| b == 0).expect("nul");
        let ffi_json = std::str::from_utf8(&buf[..nul]).expect("utf8");
        assert_eq!(ffi_json, cold_canon, "canonical JSON drift mix_id={id}");
    }
}
