// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Pure validation for `contribution.v1` JSON (rationals, required fields, admissible gate).

use super::types::{Contribution, CANON_VERSION, CONTRIBUTION_SCHEMA};
use serde_json::Value;
use thiserror::Error;

/// contribution.v1 validation failures (pure, no I/O).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Schema routing for contribution.v1 / memory_record.v1 wire shapes.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("invalid JSON: {0}")]
    Json(String),
    #[error("schema_version must be {CONTRIBUTION_SCHEMA}")]
    SchemaVersion,
    #[error("canon_version must be {CANON_VERSION}")]
    CanonVersion,
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("invalid rational at {path}: {value}")]
    InvalidRational { path: String, value: String },
    #[error("gate_summary.admissible must be true for accept")]
    NotAdmissible,
    #[error("catalog_hash must match sha256:… pattern")]
    CatalogHash,
    #[error("observed_at.stamp_tier required")]
    StampTier,
}

/// Pure: `n/d` rational wire syntax check.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Rational grammar for mix_spec rationals; no physics claim.
#[must_use]
pub fn is_valid_rational(s: &str) -> bool {
    let Some((num, den)) = s.split_once('/') else {
        return false;
    };
    let Ok(d) = den.parse::<i64>() else {
        return false;
    };
    if d == 0 {
        return false;
    }
    num.parse::<i64>().is_ok()
}

fn rational_errors_in_object(obj: &Value, path: &str) -> Vec<ValidationError> {
    let Some(map) = obj.as_object() else {
        return Vec::new();
    };
    map.iter()
        .flat_map(|(k, v)| {
            let child = if path.is_empty() {
                k.clone()
            } else {
                format!("{path}.{k}")
            };
            match v {
                Value::String(s) if s.contains('/') && !is_valid_rational(s) => {
                    vec![ValidationError::InvalidRational {
                        path: child,
                        value: s.clone(),
                    }]
                }
                Value::Object(_) => rational_errors_in_object(v, &child),
                _ => Vec::new(),
            }
        })
        .collect()
}

#[must_use]
fn sha256_pattern_ok(s: &str) -> bool {
    let Some(hex) = s.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit())
}

fn gate_summary_rational_errors(c: &Contribution) -> Vec<ValidationError> {
    let mut out = Vec::new();
    if let Some(m) = &c.gate_summary.safety_margin {
        if !is_valid_rational(m) {
            out.push(ValidationError::InvalidRational {
                path: "gate_summary.safety_margin".into(),
                value: m.clone(),
            });
        }
    }
    if let Some(m) = &c.gate_summary.mi_bits_est {
        if !is_valid_rational(m) {
            out.push(ValidationError::InvalidRational {
                path: "gate_summary.mi_bits_est".into(),
                value: m.clone(),
            });
        }
    }
    out
}

/// Parse and validate a contribution JSON value (pure — no I/O).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: contribution.v1 schema routing; gate fields checked before accept.
pub fn validate_contribution_value(v: &Value) -> Result<Contribution, ValidationError> {
    let c: Contribution =
        serde_json::from_value(v.clone()).map_err(|e| ValidationError::Json(e.to_string()))?;

    if c.schema_version != CONTRIBUTION_SCHEMA {
        return Err(ValidationError::SchemaVersion);
    }
    if c.canon_version != CANON_VERSION {
        return Err(ValidationError::CanonVersion);
    }
    if !sha256_pattern_ok(&c.catalog_hash) {
        return Err(ValidationError::CatalogHash);
    }
    if c.observed_at.stamp_tier.is_empty() {
        return Err(ValidationError::StampTier);
    }
    if c.gate_summary.catalog_ids.is_empty() {
        return Err(ValidationError::MissingField("gate_summary.catalog_ids"));
    }

    let rational_errors = rational_errors_in_object(&c.mix_spec, "mix_spec")
        .into_iter()
        .chain(rational_errors_in_object(&c.process, "process"))
        .chain(rational_errors_in_object(&c.outcome, "outcome"))
        .chain(gate_summary_rational_errors(&c))
        .collect::<Vec<_>>();

    rational_errors
        .into_iter()
        .next()
        .map_or(Ok(c), Err)
}

/// Validate for ingest accept — requires `gate_summary.admissible == true`.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Accept-path guard; thermodynamic re-check on `gate_recheck` after parse.
pub fn validate_for_accept(v: &Value) -> Result<Contribution, ValidationError> {
    let c = validate_contribution_value(v)?;
    if !c.gate_summary.admissible {
        return Err(ValidationError::NotAdmissible);
    }
    Ok(c)
}

/// Parse contribution from JSON string (pure).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: JSON parse + `validate_contribution_value`; no store I/O.
pub fn parse_contribution_json(text: &str) -> Result<Contribution, ValidationError> {
    let v: Value =
        serde_json::from_str(text).map_err(|e| ValidationError::Json(e.to_string()))?;
    validate_contribution_value(&v)
}
