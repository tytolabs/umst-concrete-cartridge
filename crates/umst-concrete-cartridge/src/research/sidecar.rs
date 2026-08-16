// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! JCS JSONL sidecars for accepted memory rows (effect boundary).

use super::types::MemoryRecord;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Default path for accepted-memory JSONL sidecar.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Filesystem path constant; not admissible memory query surface.
pub const MEMORY_JSONL_DEFAULT: &str = ".umst-memory/memory.jcs.jsonl";

/// JSONL append failures (IO boundary).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Sidecar IO error sum type; rows validated before append.
#[derive(Debug, Error)]
pub enum SidecarError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Effect boundary: append one JCS JSON line to `memory.jcs.jsonl`.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Append-only audit sink; admissible rows from `accept` only.
pub fn append_memory_jsonl(record: &MemoryRecord, path: Option<&Path>) -> Result<(), SidecarError> {
    let path = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(MEMORY_JSONL_DEFAULT));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let line = serde_json::to_string(record)?;
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(file, "{line}")?;
    Ok(())
}
