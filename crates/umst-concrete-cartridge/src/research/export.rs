// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Signed memory export bundle builder (JCS lines + hash chain; JWS optional stub).

use super::types::MemoryRecord;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Memory export bundle IO / serde failures.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Bundle handoff error sum type; rows already gate-validated.
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Pure: hash chain over ordered memory rows.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Deterministic audit chain over exported rows; not admissibility.
#[must_use]
pub fn hash_chain_for_rows(rows: &[MemoryRecord]) -> Value {
    let mut prev = "sha256:genesis".to_string();
    let links: Vec<Value> = rows
        .iter()
        .map(|r| {
            let body = serde_json::to_string(r).unwrap_or_default();
            let digest = Sha256::digest(body.as_bytes());
            let hash = format!("sha256:{digest:x}");
            let link = json!({
                "content_id": r.content_id,
                "memory_id": r.memory_id,
                "prev_hash": prev,
                "record_hash": hash,
            });
            prev = hash;
            link
        })
        .collect();
    json!({
        "schema_version": "memory_export_hash_chain.v1",
        "links": links,
        "head_hash": prev,
    })
}

/// Export bundle wire shape (`memory_export_bundle.v1`).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Signed export envelope; physics witnesses on row `gate_summary`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryExportBundle {
    pub schema_version: String,
    pub canon_version: String,
    pub row_count: usize,
    pub jcs_lines: Vec<String>,
    pub hash_chain: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jws: Option<String>,
}

/// Pure: build export bundle from memory rows (no I/O).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Pure bundle morphism; operator handoff artifact not MCP gate.
#[must_use]
pub fn build_memory_export_bundle(rows: &[MemoryRecord]) -> MemoryExportBundle {
    let jcs_lines: Vec<String> = rows
        .iter()
        .filter_map(|r| serde_json::to_string(r).ok())
        .collect();
    MemoryExportBundle {
        schema_version: "memory_export_bundle.v1".into(),
        canon_version: "jcs-rfc8785-v1".into(),
        row_count: rows.len(),
        jcs_lines,
        hash_chain: hash_chain_for_rows(rows),
        jws: None,
    }
}

/// Effect boundary: write bundle JSON + optional JCS lines file.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Filesystem export write; never exposed via MCP tools.
pub fn write_memory_export_bundle(
    bundle: &MemoryExportBundle,
    out_dir: &Path,
) -> Result<(), ExportError> {
    fs::create_dir_all(out_dir)?;
    let bundle_path = out_dir.join("memory_export_bundle.v1.json");
    fs::write(bundle_path, serde_json::to_string_pretty(bundle)?)?;
    let lines_path = out_dir.join("memory.jcs.jsonl");
    let mut text = String::new();
    for line in &bundle.jcs_lines {
        text.push_str(line);
        text.push('\n');
    }
    fs::write(lines_path, text)?;
    Ok(())
}
