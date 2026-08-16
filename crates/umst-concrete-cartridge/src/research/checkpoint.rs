// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Hourly Merkle batch roots — pure helper + append hook for `checkpoints.jsonl`.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Default checkpoints JSONL path under `.umst-memory/`.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Filesystem path constant; Merkle batch is audit metadata only.
pub const CHECKPOINTS_JSONL_DEFAULT: &str = ".umst-memory/checkpoints.jsonl";

/// Checkpoint append / serde failures (IO boundary).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: JSONL sidecar error sum type; not admissibility gate.
#[derive(Debug, Error)]
pub enum CheckpointError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Merkle batch checkpoint wire row (`memory_checkpoint.v1`).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Audit batch metadata; memory admissibility on `accept` path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckpointRecord {
    pub schema_version: String,
    pub batch_id: String,
    pub row_count: usize,
    pub merkle_root: String,
    pub wall_ms: u64,
}

/// Pure: binary Merkle root over leaf digests (hex sha256 strings without prefix).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Hash combiner for batch audit; no thermodynamic claim.
#[must_use]
pub fn merkle_root_from_leaves(leaves: &[String]) -> String {
    if leaves.is_empty() {
        return "sha256:0000000000000000000000000000000000000000000000000000000000000000".into();
    }
    let mut level: Vec<String> = leaves
        .iter()
        .map(|leaf| {
            let hex = leaf.strip_prefix("sha256:").unwrap_or(leaf);
            format!("sha256:{hex}")
        })
        .collect();
    while level.len() > 1 {
        let mut next = Vec::new();
        let mut i = 0;
        while i < level.len() {
            let left = &level[i];
            let right = if i + 1 < level.len() {
                &level[i + 1]
            } else {
                left
            };
            let combined = format!("{left}{right}");
            let digest = Sha256::digest(combined.as_bytes());
            next.push(format!("sha256:{digest:x}"));
            i += 2;
        }
        level = next;
    }
    level[0].clone()
}

/// Pure: build checkpoint record for a batch of content_id digests.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Pure record morphism over content_id leaves; persistence is IO.
#[must_use]
pub fn build_checkpoint(
    batch_id: impl Into<String>,
    content_ids: &[String],
    wall_ms: u64,
) -> CheckpointRecord {
    CheckpointRecord {
        schema_version: "memory_checkpoint.v1".into(),
        batch_id: batch_id.into(),
        row_count: content_ids.len(),
        merkle_root: merkle_root_from_leaves(content_ids),
        wall_ms,
    }
}

/// Effect boundary: append one checkpoint line (schedule hook: hourly cron / agent post-contribute).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Append-only checkpoint JSONL; not queryable admissible memory.
pub fn append_checkpoint_jsonl(
    record: &CheckpointRecord,
    path: Option<&Path>,
) -> Result<(), CheckpointError> {
    let path = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(CHECKPOINTS_JSONL_DEFAULT));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let line = serde_json::to_string(record)?;
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(file, "{line}")?;
    Ok(())
}
