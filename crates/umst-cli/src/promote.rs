// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Human-gated promotion CLI — pure record build + isolated filesystem writes.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use umst_concrete_cartridge::research::{
    apply_promotion_writes, build_promotion_record, MemoryRecord, PromotionApproval,
};

/// Default on-disk path for a promoted memory row JSON sidecar.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: CLI filesystem path helper; human-gated promotion only.
pub fn default_memory_record_path(memory_id: &str) -> PathBuf {
    PathBuf::from(".umst-memory/rows").join(format!("{memory_id}.json"))
}

/// Load a memory_record.v1 JSON file from disk (CLI IO boundary).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Filesystem read for promote-contribution; row already gate-validated.
pub fn load_memory_record(path: &Path) -> Result<MemoryRecord> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse memory record {}", path.display()))
}

/// Run human-gated promote-contribution CLI flow.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: CLI orchestration; never MCP-exposed; approval file required.
pub fn run_promote_contribution(
    memory_id: &str,
    approval_file: &Path,
    memory_record: Option<&Path>,
    dry_run: bool,
    pending_dir: Option<&Path>,
) -> Result<()> {
    let record_path = memory_record
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_memory_record_path(memory_id));
    let memory = load_memory_record(&record_path)?;

    let approval_text = fs::read_to_string(approval_file)
        .with_context(|| format!("read {}", approval_file.display()))?;
    let approval: PromotionApproval =
        serde_json::from_str(&approval_text).context("parse promotion_approval.v1")?;

    let record_id = uuid::Uuid::new_v4().to_string();
    let created_at = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );

    let out = build_promotion_record(
        &memory,
        memory_id,
        &approval,
        &approval_text,
        record_id,
        created_at,
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    if dry_run {
        println!("{}", serde_json::to_string_pretty(&out)?);
        eprintln!("dry-run: no promotion sidecar written");
        return Ok(());
    }

    apply_promotion_writes(&out, memory_id, pending_dir).map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
