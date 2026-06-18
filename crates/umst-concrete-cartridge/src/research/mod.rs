// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Per-cartridge research memory — **pure core** + effects at store/IO boundaries.
//!
//! ## FP discipline
//! - **Pure:** `validation::*`, `filter_records`, `content_id`, `gate_check_mix`, `build_promotion_record`, `ProvenanceClock::advance`.
//! - **Functional store:** `InMemoryStore::append` returns a new store (`MemoryStore` trait).
//! - **Effects only at boundary:** `WallClock::epoch_ms`, `SqliteStore`, `apply_promotion_writes`, MCP/CLI session loop.
//! - **No** global `Mutex`/`RefCell` in this module.

pub mod checkpoint;
pub mod contribution;
pub mod export;
pub mod geometry;
pub mod governance;
pub mod layer;
pub mod memory;
pub mod mi;
pub mod policy;
pub mod promotion;
pub mod provenance;
pub mod reject;
pub mod sidecar;
pub mod types;
pub mod validation;
pub mod wire_v2;

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports Merkle checkpoint helpers; audit IO on defining module.
pub use checkpoint::{
    append_checkpoint_jsonl, build_checkpoint, merkle_root_from_leaves, CheckpointError,
    CheckpointRecord, CHECKPOINTS_JSONL_DEFAULT,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports gate/accept morphisms; Mechanised on `gate_check_mix` / `accept` in `contribution`.
pub use contribution::{
    accept, content_hash_preimage, content_id, gate_check_mix, gate_recheck,
    gate_reject_row_for_mix, memory_record_from_contribution, mix_wire_from_spec_value, query,
    rational_to_f64, AcceptError, ContributeError, GateContext, DEFAULT_CATALOG_HASH,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports memory export bundle builders; human handoff only.
pub use export::{
    build_memory_export_bundle, hash_chain_for_rows, write_memory_export_bundle, ExportError,
    MemoryExportBundle,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports Morton mix geometry; locality index not admissibility gate.
pub use geometry::{
    mix_geometry_key, mix_l1_distance, morton_index, morton_index_distance, MixGeometryKey,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports scope token validation; operator governance only.
pub use governance::{validate_scope_token, ScopeError};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports Physical Reasoning Layer port trait and concrete impl.
pub use layer::{ConcretePhysicalReasoningLayer, PhysicalReasoningLayer};
#[cfg(feature = "agent-layer")]
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-export of SQLite IO store when `agent-layer` enabled.
pub use memory::SqliteStore;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports functional memory store port and filter morphisms.
pub use memory::{
    filter_records, find_by_memory_id, InMemoryStore, MemoryError, MemoryStore, ResearchStore,
    StoreError,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports Landauer advisory MI surrogates; not admissibility gate.
pub use mi::{estimate_mi_bits_from_mix, estimate_mi_bits_rational};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports promotion policy YAML validation; human-gated CLI only.
pub use policy::{
    holdout_rmse_passes, parse_promotion_policy_yaml, validate_promotion_policy, PolicyError,
    PromotionPolicy,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports human-gated promotion record morphisms; never MCP.
pub use promotion::{
    apply_promotion_writes, build_promotion_record, promote_contribution, PromotionApproval,
    PromotionError, PromotionRecordOut,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports UCRS provenance clock and monotonic stamp helpers.
pub use provenance::{
    ensure_observed_at, is_monotonic_after, observed_at_for_tick, synthetic_observed_at,
    ProvenanceClock, WallClock,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports gate reject audit stream; excluded from admissible memory.
pub use reject::{
    append_gate_reject_jsonl, build_gate_reject, build_gate_reject_from_contribution,
    GateRejectRow, RejectError, GATE_REJECT_SCHEMA,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports accepted-memory JSONL sidecar append hook.
pub use sidecar::{append_memory_jsonl, SidecarError, MEMORY_JSONL_DEFAULT};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports contribution.v1 / memory_record.v1 wire types.
pub use types::{
    AcceptResult, Contribution, GateSummary, GateVerdict, MemoryPayload, MemoryQuery, MemoryRecord,
    ObservedAt, CANON_VERSION, CONTRIBUTION_SCHEMA, MEMORY_SCHEMA,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports contribution.v1 schema validators.
pub use validation::{
    parse_contribution_json, validate_contribution_value, validate_for_accept, ValidationError,
};
#[cfg(feature = "ucrs-provenance")]
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-export UCRS → v2 wire when `ucrs-provenance` feature enabled.
pub use wire_v2::ucrs_observed_at_to_v2;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports observed_at.v2 integer wire mapping.
pub use wire_v2::{observed_at_to_v2, ObservedAtV2, OBSERVED_AT_V2_SCHEMA};

/// Alias used by promotion/MCP callers.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-export alias for `memory_record_from_contribution`; see `contribution`.
pub use contribution::memory_record_from_contribution as build_memory_record;
