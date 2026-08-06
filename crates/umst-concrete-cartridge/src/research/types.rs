// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Wire types for `contribution.v1` / `memory_record.v1` (agent-layer).

use super::geometry::MixGeometryKey;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// contribution.v1 schema id wire constant.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Schema version string; validation in `validation` module.
pub const CONTRIBUTION_SCHEMA: &str = "contribution.v1";
/// memory_record.v1 schema id wire constant.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Schema version string for persisted rows.
pub const MEMORY_SCHEMA: &str = "memory_record.v1";
/// Canonical JSON serialization profile id.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: JCS profile label on wire; not a physics claim.
pub const CANON_VERSION: &str = "jcs-rfc8785-v1";

/// Gate verdict enum on contribution and memory wire.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Serde-shaped verdict tag; admissibility on `GateSummary.admissible`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GateVerdict {
    Pass,
    Reject,
    Warn,
}

/// `gate_summary` block — admissible rows require `admissible: true`.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Wire bundle of verdict + catalog_id witnesses from gate path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateSummary {
    pub admissible: bool,
    pub verdict: GateVerdict,
    pub catalog_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_margin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mi_bits_est: Option<String>,
}

/// UCRS / wall observation stamp on agent wire.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: observed_at.v1/v2 wire; monotonicity checked in provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedAt {
    pub stamp_tier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ucrs_seq: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_entropy_bits_q: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_entropy_bits_scale: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_head_bits_q: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_head_bits_scale: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wall_ms: Option<u64>,
}

/// Parsed contribution.v1 (mix/process/outcome as JSON rationals).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Agent ingest wire; gate fields validated before accept.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contribution {
    pub schema_version: String,
    pub canon_version: String,
    pub mix_spec: Value,
    pub process: Value,
    pub outcome: Value,
    pub gate_summary: GateSummary,
    pub catalog_hash: String,
    pub observed_at: ObservedAt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

/// Persisted memory_record.v1 row.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Gate-validated memory shape; query filters on payload + mix_geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub schema_version: String,
    pub canon_version: String,
    pub content_id: String,
    pub observed_at: ObservedAt,
    pub payload: MemoryPayload,
    pub catalog_hash: String,
    pub catalog_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mix_geometry: Option<MixGeometryKey>,
}

/// Memory row payload (mix, process, outcome, gate_summary).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Nested wire bundle inside memory_record.v1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryPayload {
    pub mix_spec: Value,
    pub process: Value,
    pub outcome: Value,
    pub gate_summary: GateSummary,
}

/// Query filters for research memory (`umst_memory_query`).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Pure filter record; semantics in `memory::filter_records`.
#[derive(Debug, Clone, Default)]
pub struct MemoryQuery {
    pub admissible_only: bool,
    pub curing_regime: Option<String>,
    pub limit: Option<usize>,
    /// Pagination cursor — `content_id` of the last row from the prior page.
    pub cursor: Option<String>,
    /// Filter rows whose `catalog_ids` contains this catalog witness id.
    pub catalog_id: Option<String>,
    /// Filter on `observed_at.stamp_tier`.
    pub stamp_tier: Option<String>,
    /// Filter on `payload.outcome.source` string field.
    pub outcome_source: Option<String>,
    /// Inclusive lower bound on `observed_at.wall_ms`.
    pub wall_ms_min: Option<u64>,
    /// Inclusive upper bound on `observed_at.wall_ms`.
    pub wall_ms_max: Option<u64>,
    /// Anchor mix for L1 distance in normalized mix space (w_c, T, φ_agg).
    pub near_mix_spec: Option<Value>,
    /// Keep rows with L1 distance ≤ this threshold (requires `near_mix_spec`).
    pub max_mix_l1: Option<f64>,
    /// Morton-curve bucket for locality query.
    pub hilbert_index: Option<u32>,
    /// Max Morton index distance from `hilbert_index` (default exact match).
    pub max_hilbert_distance: Option<u32>,
}

/// Paginated memory query result (`umst_memory_query` wire).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Stable-sort page envelope; `next_cursor` for MCP agents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryQueryPage {
    pub rows: Vec<MemoryRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Result of successful `contribution::accept`.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: MCP contribute response wire; ids assigned at accept boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcceptResult {
    pub memory_id: String,
    pub content_id: String,
    pub observed_at: ObservedAt,
    pub stamp_tier: String,
    /// Resolved catalog digest pinned on the memory row (SEC-CART-PROV).
    pub catalog_hash: String,
    /// Constitutive gate admit witness — always true on successful accept.
    pub constitutive_admit: bool,
    /// Optional `durable_accept.v0` when trust env + `ucrs-provenance` are configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "ucrs-provenance")]
    pub durable_accept: Option<serde_json::Value>,
}
