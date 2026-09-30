// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! CLI memory export — signed bundle builder (human handoff; not MCP).

use anyhow::{Context, Result};
use std::path::Path;
use umst_concrete_cartridge::research::{
    build_memory_export_bundle, write_memory_export_bundle, ResearchStore,
};

/// Export gate-validated memory rows to a signed JCS bundle directory.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Operator filesystem IO; rows already gate-validated at ingest.
pub fn run_memory_export(db: Option<&Path>, out_dir: &Path) -> Result<()> {
    let store = match db {
        Some(path) => ResearchStore::open_sqlite(path)
            .with_context(|| format!("open memory db {}", path.display()))?,
        None => ResearchStore::from_env().context("UMST_MEMORY_DB or --db required")?,
    };
    let rows = store.rows();
    let bundle = build_memory_export_bundle(&rows);
    write_memory_export_bundle(&bundle, out_dir)
        .with_context(|| format!("write export bundle to {}", out_dir.display()))?;
    eprintln!(
        "info: exported {} memory rows to {}",
        bundle.row_count,
        out_dir.display()
    );
    Ok(())
}
