// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Gate reject audit stream — pure row build + JSONL append at IO boundary (never in memory queries).

use super::contribution::rational_to_f64;
use super::types::{Contribution, GateVerdict, ObservedAt, CANON_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use thiserror::Error;

/// gate_reject.v1 schema id wire constant.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Reject stream schema label; rows excluded from admissible memory.
pub const GATE_REJECT_SCHEMA: &str = "gate_reject.v1";

/// Gate reject JSONL append / serde failures.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Audit stream IO errors; not admissible memory store.
#[derive(Debug, Error)]
pub enum RejectError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Durable gate-failure row (`gate_reject.v1`).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Audit wire for failed gate; never in `admissible_only` query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateRejectRow {
    pub schema_version: String,
    pub canon_version: String,
    pub mix_content_hash: String,
    pub verdict: String,
    pub catalog_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_hash: Option<String>,
    pub stamp_tier: String,
    pub rejected_at: ObservedAt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_codes: Option<Vec<String>>,
}

/// Pure: SHA-256 over `mix_spec` rational preimage.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Content hash for reject audit; gate verdict on manifold path.
#[must_use]
pub fn mix_content_hash(mix_spec: &Value) -> String {
    let bytes = serde_json::to_vec(mix_spec).unwrap_or_default();
    let digest = Sha256::digest(bytes);
    format!("sha256:{}", hex_encode(&digest))
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Pure: build reject row from gate-check or failed accept.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Reject row morphism; thermodynamic fail codes from gate path.
#[must_use]
pub fn build_gate_reject(
    mix_spec: &Value,
    verdict: GateVerdict,
    catalog_ids: Vec<String>,
    catalog_hash: Option<String>,
    observed_at: ObservedAt,
    reason_codes: Option<Vec<String>>,
) -> GateRejectRow {
    GateRejectRow {
        schema_version: GATE_REJECT_SCHEMA.to_string(),
        canon_version: CANON_VERSION.to_string(),
        mix_content_hash: mix_content_hash(mix_spec),
        verdict: match verdict {
            GateVerdict::Pass => "PASS".into(),
            GateVerdict::Reject => "REJECT".into(),
            GateVerdict::Warn => "WARN".into(),
        },
        catalog_ids,
        catalog_hash,
        stamp_tier: observed_at.stamp_tier.clone(),
        rejected_at: observed_at,
        reason_codes,
    }
}

/// Pure: reject row from full contribution that failed gate re-check.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Maps failed accept contribution to gate_reject.v1 audit row.
#[must_use]
pub fn build_gate_reject_from_contribution(
    contribution: &Contribution,
    reason_codes: Option<Vec<String>>,
) -> GateRejectRow {
    build_gate_reject(
        &contribution.mix_spec,
        contribution.gate_summary.verdict,
        contribution.gate_summary.catalog_ids.clone(),
        Some(contribution.catalog_hash.clone()),
        contribution.observed_at.clone(),
        reason_codes,
    )
}

/// Effect boundary: append one JCS JSON line to `.umst-memory/gate_reject.jcs.jsonl`.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Append-only reject audit sink; not admissible memory.
pub fn append_gate_reject_jsonl(
    row: &GateRejectRow,
    path: Option<&Path>,
) -> Result<(), RejectError> {
    let path = path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from(".umst-memory/gate_reject.jcs.jsonl"));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let line = serde_json::to_string(row)?;
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

/// Pure helper: mix_spec has parseable rationals for hashing.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Pre-hash shape check; admissibility on `gate_recheck`.
#[must_use]
pub fn mix_spec_hashable(mix_spec: &Value) -> bool {
    mix_spec
        .get("w_c")
        .and_then(|v| v.as_str())
        .and_then(rational_to_f64)
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn build_reject_row_shape() {
        let mix = json!({ "w_c": "9/20", "temperature_k": "29315/100" });
        let row = build_gate_reject(
            &mix,
            GateVerdict::Reject,
            vec!["umst.gate.cd_transition".into()],
            None,
            ObservedAt {
                stamp_tier: "Synthetic".into(),
                ucrs_seq: Some(1),
                phase_entropy_bits_q: None,
                phase_entropy_bits_scale: None,
                credit_head_bits_q: None,
                credit_head_bits_scale: None,
                wall_ms: Some(0),
            },
            Some(vec!["thermodynamic_fail".into()]),
        );
        assert_eq!(row.schema_version, "gate_reject.v1");
        assert!(row.mix_content_hash.starts_with("sha256:"));
    }
}
