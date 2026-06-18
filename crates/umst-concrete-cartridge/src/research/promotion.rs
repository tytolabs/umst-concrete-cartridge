// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Human-gated promotion — pure record morphism + isolated filesystem writes.

use super::memory::{find_by_memory_id, ResearchStore, MemoryError};
use super::types::MemoryRecord;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

/// Human-gated promotion pipeline failures.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: CLI-only promotion path; never MCP-exposed.
#[derive(Debug, Error)]
pub enum PromotionError {
    #[error("memory: {0}")]
    Memory(#[from] MemoryError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("approval: {0}")]
    Approval(String),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Signed human approval wire (`promotion_approval.v1`).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: JWS-backed human decision; not agent auto-promotion.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PromotionApproval {
    pub schema_version: String,
    pub proposal_hash: String,
    pub ucrs_observation_tier: String,
    pub decision: String,
    pub approver: String,
    pub approved_at: String,
    #[serde(default)]
    pub jws: Option<String>,
}

/// Persisted promotion record (`promotion_record.v1`).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Calibration handoff bundle; admissibility already on memory row.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromotionRecordOut {
    pub schema_version: String,
    pub canon_version: String,
    pub record_id: String,
    pub created_at: String,
    pub contribution: Value,
    pub proposal: Value,
    pub approval: Value,
    pub promotion: Value,
    pub hash_chain: Value,
}

/// Pure: validate approval wire + build promotion record (no I/O).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Record morphism from gate-validated memory + human approval.
pub fn build_promotion_record(
    memory: &MemoryRecord,
    memory_id: &str,
    approval: &PromotionApproval,
    approval_text: &str,
    record_id: String,
    created_at: String,
) -> Result<PromotionRecordOut, PromotionError> {
    validate_approval(approval)?;

    let contribution_hash = memory.content_id.clone();
    let approval_hash = sha256_hex(approval_text);
    let proposal_hash = approval.proposal_hash.clone();

    let mut record = PromotionRecordOut {
        schema_version: "promotion_record.v1".into(),
        canon_version: "jcs-rfc8785-v1".into(),
        record_id,
        created_at,
        contribution: json!({
            "content_hash": contribution_hash,
            "schema_version": "contribution.v1",
        }),
        proposal: json!({
            "content_hash": proposal_hash,
            "schema_version": "promotion_proposal.v1",
        }),
        approval: json!({
            "content_hash": format!("sha256:{approval_hash}"),
            "schema_version": "promotion_approval.v1",
            "approver": approval.approver,
            "ucrs_observation_tier": approval.ucrs_observation_tier,
            "jws": approval.jws,
        }),
        promotion: json!({
            "target_ref": format!("memory:{memory_id}"),
            "policy_id": "governance/promotion_policy.yaml",
        }),
        hash_chain: json!({
            "contribution_hash": contribution_hash,
            "proposal_hash": proposal_hash,
            "approval_hash": format!("sha256:{approval_hash}"),
            "record_hash": "sha256:pending",
        }),
    };

    let record_json = serde_json::to_string(&record)?;
    let record_hash = sha256_hex(&record_json);
    if let Some(chain) = record.hash_chain.as_object_mut() {
        chain.insert(
            "record_hash".into(),
            Value::String(format!("sha256:{record_hash}")),
        );
    }

    Ok(record)
}

fn validate_approval(approval: &PromotionApproval) -> Result<(), PromotionError> {
    if approval.schema_version != "promotion_approval.v1" {
        return Err(PromotionError::Approval(
            "schema_version must be promotion_approval.v1".into(),
        ));
    }
    if approval.decision != "approve" {
        return Err(PromotionError::Approval(
            "decision must be approve for apply path".into(),
        ));
    }
    if let Some(jws) = &approval.jws {
        if jws.is_empty() {
            return Err(PromotionError::Approval("jws must be non-empty when present".into()));
        }
        if !jws.contains('.') {
            return Err(PromotionError::Approval(
                "jws must be compact JWS (header.payload.signature)".into(),
            ));
        }
    }
    Ok(())
}

/// Effect boundary: persist promotion sidecar(s).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Filesystem writes for human-gated calibration promotion.
pub fn apply_promotion_writes(
    record: &PromotionRecordOut,
    memory_id: &str,
    pending_calibration_dir: Option<&Path>,
) -> Result<(), PromotionError> {
    let final_json = serde_json::to_string_pretty(record)?;
    let sidecar_path = promotion_sidecar_path(memory_id);
    if let Some(parent) = sidecar_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&sidecar_path, &final_json)?;

    if let Some(dir) = pending_calibration_dir {
        fs::create_dir_all(dir)?;
        let pending = dir.join(format!("{memory_id}.promotion_record.v1.json"));
        fs::write(pending, &final_json)?;
    }

    Ok(())
}

/// Human-gated promotion: load memory → pure record → optional writes.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: CLI orchestration; physics gate already on accepted memory row.
pub fn promote_contribution(
    store: &ResearchStore,
    memory_id: &str,
    approval_path: &Path,
    dry_run: bool,
    pending_calibration_dir: Option<&Path>,
) -> Result<PromotionRecordOut, PromotionError> {
    if approval_path.as_os_str().is_empty() {
        return Err(PromotionError::Approval(
            "human-gated promotion requires --approval-file".into(),
        ));
    }

    let approval_text = fs::read_to_string(approval_path)?;
    let approval: PromotionApproval = serde_json::from_str(&approval_text)?;
    let memory = find_by_memory_id(store, memory_id)?;
    let record_id = Uuid::new_v4().to_string();
    let created_at = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );

    let record = build_promotion_record(
        &memory,
        memory_id,
        &approval,
        &approval_text,
        record_id,
        created_at,
    )?;

    if dry_run {
        return Ok(record);
    }

    apply_promotion_writes(&record, memory_id, pending_calibration_dir)?;
    Ok(record)
}

fn promotion_sidecar_path(memory_id: &str) -> PathBuf {
    PathBuf::from(".umst-memory/promotions").join(format!("{memory_id}.promotion_record.v1.json"))
}

fn sha256_hex(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    format!("{digest:x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::research::types::{GateSummary, GateVerdict, MemoryPayload, ObservedAt, CANON_VERSION, MEMORY_SCHEMA};

    fn sample_memory() -> MemoryRecord {
        MemoryRecord {
            schema_version: MEMORY_SCHEMA.to_string(),
            canon_version: CANON_VERSION.to_string(),
            content_id: "sha256:abc".into(),
            observed_at: ObservedAt {
                stamp_tier: "Synthetic".into(),
                ucrs_seq: Some(1),
                phase_entropy_bits_q: None,
                phase_entropy_bits_scale: None,
                credit_head_bits_q: None,
                credit_head_bits_scale: None,
                wall_ms: None,
            },
            payload: MemoryPayload {
                mix_spec: json!({}),
                process: json!({}),
                outcome: json!({}),
                gate_summary: GateSummary {
                    admissible: true,
                    verdict: GateVerdict::Pass,
                    catalog_ids: vec!["umst.gate.cd_transition".into()],
                    safety_margin: None,
                    mi_bits_est: None,
                },
            },
            catalog_hash: "sha256:abc".into(),
            catalog_ids: vec!["umst.gate.cd_transition".into()],
            memory_id: Some("mem-1".into()),
            mix_geometry: None,
        }
    }

    #[test]
    fn build_record_is_pure() {
        let approval = PromotionApproval {
            schema_version: "promotion_approval.v1".into(),
            proposal_hash: "sha256:def".into(),
            ucrs_observation_tier: "Synthetic".into(),
            decision: "approve".into(),
            approver: "human".into(),
            approved_at: "2026-01-01T00:00:00Z".into(),
            jws: None,
        };
        let text = serde_json::to_string(&approval).unwrap();
        let out = build_promotion_record(
            &sample_memory(),
            "mem-1",
            &approval,
            &text,
            "rid".into(),
            "ts".into(),
        )
        .unwrap();
        assert_eq!(out.schema_version, "promotion_record.v1");
        assert!(out
            .hash_chain
            .get("record_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .starts_with("sha256:"));
    }
}
