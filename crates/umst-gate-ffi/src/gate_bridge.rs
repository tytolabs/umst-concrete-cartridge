// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Pure bridge: JSON C strings → `gate_check_mix_result` → C out-params / JSON buffer.
//!
//! formal_anchor: NONE
//! formal_status: NONE
//! formal_anchor_rationale: FFI boundary adapter; CD admissibility on cartridge gate path.

use serde_json::{Map, Value};
use std::ffi::CStr;
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::research::{gate_check_mix_result, GateVerdict, ObservedAt};

/// Sorted-key canonical JSON for parity with S0 fixtures.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Canonicalization helper for FFI↔Rust parity; not a gate morphism.
pub fn sort_keys(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().cloned().collect();
            keys.sort();
            let mut out = Map::new();
            for k in keys {
                let child = map.get(&k).expect("key").clone();
                out.insert(k, sort_keys(child));
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(sort_keys).collect()),
        other => other,
    }
}

pub fn observed_at(ucrs_seq: u64, wall_ms: u64) -> ObservedAt {
    ObservedAt {
        stamp_tier: "Synthetic".into(),
        ucrs_seq: Some(ucrs_seq),
        phase_entropy_bits_q: None,
        phase_entropy_bits_scale: None,
        credit_head_bits_q: None,
        credit_head_bits_scale: None,
        wall_ms: Some(wall_ms),
    }
}

/// Map GateVerdict → stable C int codes.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: FFI discriminant encoding; verdict semantics on GateVerdict.
pub fn verdict_code(v: GateVerdict) -> i32 {
    match v {
        GateVerdict::Pass => 0,
        GateVerdict::Reject => 1,
        GateVerdict::Warn => 2,
    }
}

/// Run gate; return canonical JSON string of `GateCheckResult`.
pub fn gate_check_canonical_json(
    profile_id: &str,
    mix: &Value,
    explain: bool,
    ucrs_seq: u64,
    wall_ms: u64,
) -> Result<(bool, i32, String), String> {
    let profile = Profile::load_bundled(profile_id).map_err(|e| format!("profile: {e}"))?;
    let result = gate_check_mix_result(&profile, mix, explain, observed_at(ucrs_seq, wall_ms));
    let admissible = result.gate_summary.admissible;
    let verdict = verdict_code(result.gate_summary.verdict);
    let json = serde_json::to_value(&result).map_err(|e| format!("serialize: {e}"))?;
    let canon = serde_json::to_string(&sort_keys(json)).map_err(|e| format!("canon: {e}"))?;
    Ok((admissible, verdict, canon))
}

pub unsafe fn cstr_to_str<'a>(ptr: *const std::os::raw::c_char) -> Result<&'a str, String> {
    if ptr.is_null() {
        return Err("null c_str".into());
    }
    CStr::from_ptr(ptr)
        .to_str()
        .map_err(|e| format!("utf8: {e}"))
}
