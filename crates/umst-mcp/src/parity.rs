// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! S0/S3 parity canonicalization — redact nondeterministic MCP fields for golden compare.
//!
//! ADDITIVE — shared by `gate_parity` and `rmcp_parity` integration tests.

use serde_json::{json, Map, Value};

/// Recursively sort object keys for byte-stable JSON compare.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Test harness canonicalization; not production wire.
#[must_use]
pub fn sort_keys(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().cloned().collect();
            keys.sort();
            let mut out = Map::new();
            for k in keys {
                let child = map.get(&k).expect("key present").clone();
                out.insert(k, sort_keys(child));
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(sort_keys).collect()),
        other => other,
    }
}

/// Canonical compact JSON bytes (sorted keys).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Test fixture byte compare helper.
#[must_use]
pub fn canonical_bytes(v: &Value) -> String {
    serde_json::to_string(&sort_keys(v.clone())).expect("serialize canonical")
}

/// Redact non-deterministic MCP envelope fields (ids / timestamps) for golden compare.
/// Profile `id` fields are preserved.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Parity harness redaction; not physics.
#[must_use]
pub fn redact_nondeterministic(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, child) in map {
                if matches!(
                    k.as_str(),
                    "wall_ms"
                        | "memory_id"
                        | "job_id"
                        | "arena_session_id"
                        | "content_id"
                        | "idempotency_key"
                ) {
                    out.insert(k, json!("<redacted>"));
                } else {
                    out.insert(k, redact_nondeterministic(child));
                }
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(redact_nondeterministic).collect()),
        other => other,
    }
}

/// Extract MCP `result` frame and redact; parse nested tool text JSON if present.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: S0 golden `tools/call` result canonicalizer.
#[must_use]
pub fn canonicalize_tools_call_result(resp: &Value) -> Value {
    let result = resp.get("result").cloned().unwrap_or(Value::Null);
    let mut redacted = redact_nondeterministic(result);
    if let Some(content) = redacted.get_mut("content").and_then(|c| c.as_array_mut()) {
        for item in content.iter_mut() {
            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                    let redacted_payload = redact_nondeterministic(parsed);
                    let canon = canonical_bytes(&redacted_payload);
                    item.as_object_mut()
                        .expect("content item object")
                        .insert("text".into(), Value::String(canon));
                }
            }
        }
    }
    sort_keys(redacted)
}
