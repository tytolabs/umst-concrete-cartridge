// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! `propose-promotion` dry-run + hold-out metrics (human-only; never MCP).

use anyhow::{Context, Result};
use serde_json::json;
use std::fs;
use std::path::Path;
use umst_concrete_cartridge::research::{
    holdout_rmse_passes, parse_promotion_policy_yaml, PromotionPolicy,
};

/// Run propose-promotion dry-run with hold-out RMSE against bundled policy.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Human-only CLI proposal; calibration metrics not MCP gate.
pub fn run_propose_promotion(
    memory_id: &str,
    memory_record: Option<&Path>,
    observed_strengths: &[f64],
    predicted_strengths: &[f64],
    policy_path: Option<&Path>,
    dry_run: bool,
) -> Result<()> {
    let policy_text = match policy_path {
        Some(p) => fs::read_to_string(p).with_context(|| format!("read policy {}", p.display()))?,
        None => include_str!("../../../governance/promotion_policy.yaml").to_string(),
    };
    let policy: PromotionPolicy =
        parse_promotion_policy_yaml(&policy_text).map_err(|e| anyhow::anyhow!("{e}"))?;

    if observed_strengths.len() < policy.min_holdout_rows as usize {
        anyhow::bail!(
            "hold-out rows {} < policy min_holdout_rows {}",
            observed_strengths.len(),
            policy.min_holdout_rows
        );
    }

    let rmse_ok = holdout_rmse_passes(observed_strengths, predicted_strengths, policy.max_rmse_mpa);
    let record_path = memory_record
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| format!(".umst-memory/rows/{memory_id}.json"));

    let proposal = json!({
        "schema_version": "promotion_proposal.v1",
        "canon_version": "jcs-rfc8785-v1",
        "memory_id": memory_id,
        "memory_record_path": record_path,
        "policy_id": policy.policy_id,
        "holdout": {
            "row_count": observed_strengths.len(),
            "rmse_passes": rmse_ok,
            "max_rmse_mpa": policy.max_rmse_mpa,
        },
        "dry_run": dry_run,
        "decision_hint": if rmse_ok { "eligible_for_human_review" } else { "hold_out_failed" },
    });

    println!("{}", serde_json::to_string_pretty(&proposal)?);
    if dry_run {
        eprintln!("dry-run: no promotion approval minted");
    }
    Ok(())
}
