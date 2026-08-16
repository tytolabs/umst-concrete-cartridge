// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Deterministic JSON: lexicographically sorted object keys recursively, finite numbers only.
//! Floating-point literals use **Ryū** shortest round-trip decimals; `serde_json` then materialises each
//! number as [`Number`] so **`serde_json::to_vec`** emits the same literals (canonical wire contract).

use ryu::Buffer;
use serde_json::{Map, Number, Value};

/// Sort object keys recursively. Replaces non-finite floats with an error surface for callers that
/// need strict JSON (`allow_nan`-style refusal).
///
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Transport-only deterministic JSON contract; physical claims live upstream of this layer.
pub fn canonical_json_value(v: &Value) -> Result<Value, CanonicalJsonError> {
    canonicalize_inner(v)
}

fn canonicalize_inner(v: &Value) -> Result<Value, CanonicalJsonError> {
    Ok(match v {
        Value::Null | Value::Bool(_) => v.clone(),
        Value::Number(n) => Value::Number(stable_float_number(n)?),
        Value::String(s) => Value::String(s.clone()),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(canonicalize_inner)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut out = Map::new();
            for k in keys {
                let next = canonicalize_inner(&map[k])?;
                out.insert(k.clone(), next);
            }
            Value::Object(out)
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Structural transport error for malformed or non-finite JSON numbers.
pub enum CanonicalJsonError {
    NonFiniteFloat,
    InvalidNumberRepr,
}

impl std::fmt::Display for CanonicalJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonFiniteFloat => write!(
                f,
                "non-finite JSON number rejected for canonical serialization"
            ),
            Self::InvalidNumberRepr => {
                write!(f, "number could not be normalised deterministically")
            }
        }
    }
}

impl std::error::Error for CanonicalJsonError {}

fn stable_float_number(n: &Number) -> Result<Number, CanonicalJsonError> {
    if let Some(i) = n.as_i64() {
        return Ok(Number::from(i));
    }
    if let Some(u) = n.as_u64() {
        return Ok(Number::from(u));
    }
    let f = n.as_f64().ok_or(CanonicalJsonError::NonFiniteFloat)?;
    if !f.is_finite() {
        return Err(CanonicalJsonError::NonFiniteFloat);
    }

    let mut buf = Buffer::new();
    let lit = buf.format_finite(f);
    let v: Value = serde_json::from_str(lit).map_err(|_| CanonicalJsonError::InvalidNumberRepr)?;
    v.as_number()
        .cloned()
        .ok_or(CanonicalJsonError::InvalidNumberRepr)
}

/// Serialise **`Value`** compactly without extra whitespace after **[`canonical_json_value`]** preprocessing.
///
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Byte-stable wire encoding for MCP / CLI / acceptance scripts.
pub fn canonical_json_bytes(value: &Value) -> Result<Vec<u8>, CanonicalJsonError> {
    let canon = canonical_json_value(value)?;
    serde_json::to_vec(&canon).map_err(|_| CanonicalJsonError::InvalidNumberRepr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sorts_keys_deeply() {
        let raw = json!({"z":1,"a":{"m":3,"b":2}});
        let _ = canonical_json_value(&raw).unwrap();
        let s = String::from_utf8(canonical_json_bytes(&raw).unwrap()).unwrap();
        assert!(
            s.starts_with("{\"a\":"),
            "lexicographic key order broken: {s}",
        );
        assert!(s.contains("\"b\":2,\"m\":3"), "{s}");
    }

    #[test]
    fn matching_consecutive_calls() {
        let raw = json!({"w": 0.1 + 0.2});
        let a = canonical_json_bytes(&raw).unwrap();
        let b = canonical_json_bytes(&raw).unwrap();
        assert_eq!(a, b);
    }
}
