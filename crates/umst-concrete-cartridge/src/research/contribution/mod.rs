// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Pure contribution → memory morphisms and functional accept pipeline.
//!
//! ## Phase 0c three-way split (blueprint §7 0c)
//! - [`adapter`] — mix JSON wire → [`crate::facade::MixSpec`] (parse only)
//! - [`gate`] — pure admissibility predicate (`gate_check_mix`, `gate_recheck`)
//! - [`infra`] — explain codes, remediation, MCP `gate_check_mix_result` wire
//!
//! Accept / memory morphisms remain in this root module until Phase 0d routes all callers.

pub mod adapter;
pub mod gate;
pub mod infra;

use super::geometry::mix_geometry_key;
use super::governance::validate_scope_token;
use super::memory::{ResearchStore, StoreError};
use super::provenance::ProvenanceClock;
use super::provenance::{ensure_observed_at, is_monotonic_after, observed_at_for_tick, WallClock};
use super::reject::{build_gate_reject_from_contribution, GateRejectRow};
use super::types::{
    AcceptResult, Contribution, MemoryPayload, MemoryQuery, MemoryRecord, ObservedAt,
    CANON_VERSION, MEMORY_SCHEMA,
};
use super::validation::{validate_for_accept, ValidationError};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports Phase 0c adapter symbols from [`adapter`].
pub use adapter::{mix_spec_from_json, mix_wire_from_spec_value, rational_to_f64};
pub(crate) use adapter::field_as_rational;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports Phase 0c gate predicate symbols from [`gate`].
pub use gate::{
    gate_check_mix, gate_recheck, gate_recheck_with_spec, gate_reject_row_for_mix, GateContext,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports Phase 0c infra / MCP wire symbols from [`infra`].
pub use infra::{
    gate_check_mix_result, GateCheckExplain, GateCheckResult, GateFieldIssue,
};

/// Accept pipeline errors (validation, gate reject, scope, monotonic stamp, store).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Error sum type; thermodynamic verdict on `GateReject` leg only.
#[derive(Debug, Error)]
pub enum AcceptError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("gate re-check failed: mix not thermodynamically admissible")]
    GateReject(Box<GateRejectRow>),
    #[error(transparent)]
    Scope(#[from] super::governance::ScopeError),
    #[error("observed_at stamp is not monotonic after session clock")]
    NonMonotonicStamp,
    #[error(transparent)]
    Store(#[from] StoreError),
}

/// MCP `umst_contribute` error alias (same variants as [`AcceptError`]).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Type alias for wire ergonomics; gate reject leg documented on `AcceptError`.
pub type ContributeError = AcceptError;

/// Agent CI catalog digest placeholder when manifest grounding is not loaded.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Build-time pin override in manifest_bridge tests; not a physics claim.
pub const DEFAULT_CATALOG_HASH: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000001";

/// Pure memory query delegate over [`ResearchStore`].
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Functor to store `query`; filter semantics on `filter_records`.
#[must_use]
pub fn query(store: &ResearchStore, q: &MemoryQuery) -> Vec<MemoryRecord> {
    store.query(q)
}

/// JCS-shaped preimage for content_id hashing.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Deterministic field bundle for SHA-256; no admissibility claim.
#[must_use]
pub fn content_hash_preimage(contribution: &Contribution) -> Value {
    serde_json::json!({
        "schema_version": contribution.schema_version,
        "canon_version": contribution.canon_version,
        "mix_spec": contribution.mix_spec,
        "process": contribution.process,
        "outcome": contribution.outcome,
        "gate_summary": contribution.gate_summary,
        "catalog_hash": contribution.catalog_hash,
        "observed_at": contribution.observed_at,
    })
}

/// SHA-256 content id over canonical preimage fields.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Content-addressed memory key; hash of wire fields not physics.
#[must_use]
pub fn content_id(contribution: &Contribution) -> String {
    let bytes = serde_json::to_vec(&content_hash_preimage(contribution)).unwrap_or_default();
    let digest = Sha256::digest(bytes);
    format!("sha256:{}", hex::encode(digest))
}

/// Pure contribution → memory row (no store I/O).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: memory_record.v1 projection; mix_geometry from Morton on mix_spec.
#[must_use]
pub fn memory_record_from_contribution(
    contribution: &Contribution,
    memory_id: String,
    observed_at: ObservedAt,
) -> MemoryRecord {
    let regime = contribution
        .process
        .get("curing_regime")
        .and_then(|v| v.as_str());
    let mix_geometry = mix_geometry_key(&contribution.mix_spec, regime);
    MemoryRecord {
        schema_version: MEMORY_SCHEMA.to_string(),
        canon_version: CANON_VERSION.to_string(),
        content_id: content_id(contribution),
        observed_at,
        payload: MemoryPayload {
            mix_spec: contribution.mix_spec.clone(),
            process: contribution.process.clone(),
            outcome: contribution.outcome.clone(),
            gate_summary: contribution.gate_summary.clone(),
        },
        catalog_hash: contribution.catalog_hash.clone(),
        catalog_ids: contribution.gate_summary.catalog_ids.clone(),
        memory_id: Some(memory_id),
        mix_geometry,
    }
}

/// Functional accept: validate → gate → stamp → append; returns new store + clock.
/// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
pub fn accept(
    store: ResearchStore,
    clock: ProvenanceClock,
    wall: WallClock,
    ctx: &GateContext<'_>,
    contribution_json: &Value,
) -> Result<(ResearchStore, ProvenanceClock, AcceptResult), AcceptError> {
    let scope = contribution_json
        .get("scope_token")
        .and_then(|v| v.as_str());
    validate_scope_token(scope)?;

    let idempotency_key = contribution_json
        .get("idempotency_key")
        .and_then(|v| v.as_str());

    let contribution = validate_for_accept(contribution_json)?;

    if contribution.observed_at.ucrs_seq.is_some() {
        let baseline = observed_at_for_tick(clock.sequence(), WallClock.epoch_ms());
        if !is_monotonic_after(&baseline, &contribution.observed_at) {
            return Err(AcceptError::NonMonotonicStamp);
        }
    }

    if !gate_recheck(ctx, &contribution) {
        let row = build_gate_reject_from_contribution(
            &contribution,
            Some(vec!["thermodynamic_fail".into()]),
        );
        return Err(AcceptError::GateReject(Box::new(row)));
    }

    let (clock, observed_at) =
        ensure_observed_at(Some(contribution.observed_at.clone()), clock, wall);
    let memory_id = Uuid::new_v4().to_string();
    let record =
        memory_record_from_contribution(&contribution, memory_id.clone(), observed_at.clone());
    let content = record.content_id.clone();
    let stamp_tier = observed_at.stamp_tier.clone();

    let (store, ()) = store.append(record, idempotency_key)?;

    Ok((
        store,
        clock,
        AcceptResult {
            memory_id,
            content_id: content,
            observed_at,
            stamp_tier,
        },
    ))
}

mod hex {
    /// Hex encode digest bytes for `content_id` (internal helper).
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Formatting helper; content addressing on `content_id`.
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
    }
}
