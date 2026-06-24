// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Pure contribution → memory morphisms and functional accept pipeline.

use super::geometry::mix_geometry_key;
use super::governance::validate_scope_token;
use super::memory::{ResearchStore, StoreError};
use super::mi::estimate_mi_bits_rational;
use super::provenance::ProvenanceClock;
use super::provenance::{ensure_observed_at, is_monotonic_after, observed_at_for_tick, WallClock};
use super::reject::{build_gate_reject, build_gate_reject_from_contribution, GateRejectRow};
use super::types::{
    AcceptResult, Contribution, GateSummary, GateVerdict, MemoryPayload, MemoryQuery, MemoryRecord,
    ObservedAt, CANON_VERSION, CONTRIBUTION_SCHEMA, MEMORY_SCHEMA,
};
use super::validation::{validate_for_accept, ValidationError};
use crate::calibration::Profile;
use crate::facade::{MixSpec, MixSpecWire};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::convert::TryFrom;
use thiserror::Error;
use uuid::Uuid;

use super::gate_explain_ssot::{
    fields_for_code as ssot_fields_for_code, remediation_for_code, MANIFEST_BRIDGE_DISABLED,
    MIX_SPEC_RATIONAL_PARSE_FAIL, MIX_SPEC_WIRE_INVALID, THERMODYNAMIC_CD_FAIL, THERMODYNAMIC_FAIL,
};
#[cfg(feature = "manifest-bridge")]
use crate::pipeline::dual_gate::thermodynamic_ok;

/// Accept pipeline errors (validation, gate reject, scope, monotonic stamp, store).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Error sum type; thermodynamic verdict on `GateReject` leg only.
#[derive(Debug, Error)]
pub enum AcceptError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("gate re-check failed: mix not thermodynamically admissible")]
    GateReject(GateRejectRow),
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

/// Gate evaluation context (bundled calibration profile).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Profile carrier for `gate_recheck`; CD math on manifold when manifest-bridge on.
pub struct GateContext<'a> {
    pub profile: &'a Profile,
}

/// MCP `umst_gate_check` core — thermodynamic admissibility for a mix_spec.
/// formal_anchor: lean://umst-formal/Lean/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
#[must_use]
pub fn gate_check_mix(profile: &Profile, mix_json: &Value) -> GateSummary {
    let admissible = mix_wire_from_spec_value(mix_json)
        .and_then(|wire| MixSpec::try_from(wire).ok())
        .map(|mut spec| {
            spec.profile_name = profile.bundle_id.clone();
            gate_recheck(&GateContext { profile }, &stub_contribution(mix_json))
        })
        .unwrap_or(false);

    let mi_bits_est = estimate_mi_bits_rational(mix_json, profile);

    GateSummary {
        admissible,
        verdict: if admissible {
            GateVerdict::Pass
        } else {
            GateVerdict::Reject
        },
        catalog_ids: vec!["umst.gate.cd_transition".into()],
        safety_margin: None,
        mi_bits_est,
    }
}

/// Field-level hint for gate REJECT diagnostics.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Operator diagnostics; not admissibility proof.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GateFieldIssue {
    pub path: String,
    pub issue: String,
}

/// Optional explain block for MCP `umst_gate_check` when `explain: true`.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Operator diagnostics; not admissibility proof.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GateCheckExplain {
    pub regime_violations: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub remediation: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<GateFieldIssue>,
    pub catalog_witnesses: Vec<String>,
}

/// Full MCP gate-check wire (`gate_summary` + optional `gate_reject` + explain).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Structured tool result; reject row matches `gate_reject.v1`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GateCheckResult {
    pub gate_summary: GateSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate_reject: Option<GateRejectRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explain: Option<GateCheckExplain>,
}

/// MCP `umst_gate_check` response builder — optional explain + embedded `gate_reject.v1`.
/// formal_anchor: lean://umst-formal/Lean/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
#[must_use]
pub fn gate_check_mix_result(
    profile: &Profile,
    mix_json: &Value,
    explain: bool,
    observed_at: ObservedAt,
) -> GateCheckResult {
    let gate_summary = gate_check_mix(profile, mix_json);
    let gate_reject = gate_reject_row_for_mix(mix_json, &gate_summary, observed_at);
    let explain_block = if explain {
        Some(build_gate_explain(
            profile,
            mix_json,
            gate_summary.admissible,
            &gate_summary.catalog_ids,
        ))
    } else {
        None
    };
    GateCheckResult {
        gate_summary,
        gate_reject,
        explain: explain_block,
    }
}

fn build_gate_explain(
    profile: &Profile,
    mix_json: &Value,
    admissible: bool,
    catalog_witnesses: &[String],
) -> GateCheckExplain {
    let regime_violations = collect_gate_explain_codes(profile, mix_json, admissible);
    let remediation: Vec<String> = regime_violations
        .iter()
        .map(|c| gate_remediation_for_code(c).to_string())
        .collect();
    let mut fields = Vec::new();
    for code in &regime_violations {
        fields.extend(gate_fields_for_code(code, mix_json));
    }
    GateCheckExplain {
        regime_violations,
        remediation,
        fields,
        catalog_witnesses: catalog_witnesses.to_vec(),
    }
}

fn gate_remediation_for_code(code: &str) -> &'static str {
    remediation_for_code(code)
}

fn gate_fields_for_code(code: &str, mix_json: &Value) -> Vec<GateFieldIssue> {
    if code == MIX_SPEC_RATIONAL_PARSE_FAIL {
        rational_parse_field_issues(mix_json)
    } else {
        ssot_fields_for_code(code, mix_json.get("temperature_k").is_some())
            .into_iter()
            .map(|(path, issue)| GateFieldIssue { path, issue })
            .collect()
    }
}

fn rational_parse_field_issues(mix_json: &Value) -> Vec<GateFieldIssue> {
    let mut fields = Vec::new();
    for key in [
        "w_c",
        "temperature_k",
        "superplasticiser_pct",
        "silica_fume_pct",
        "fly_ash_pct",
        "aggregate_volume_fraction",
        "target_age_hours",
    ] {
        let issue = match mix_json.get(key) {
            None if matches!(key, "w_c" | "temperature_k") => Some("missing_required"),
            None => None,
            Some(v) if v.as_str().is_some_and(|s| rational_to_f64(s).is_none()) => {
                Some("rational_parse_fail")
            }
            Some(v) if !v.is_string() => Some("expected_rational_string"),
            _ => None,
        };
        if let Some(issue) = issue {
            fields.push(GateFieldIssue {
                path: format!("mix.{key}"),
                issue: issue.into(),
            });
        }
    }
    if fields.is_empty() {
        fields.push(GateFieldIssue {
            path: "mix".into(),
            issue: "rational_parse_fail".into(),
        });
    }
    fields
}

fn collect_gate_explain_codes(
    profile: &Profile,
    mix_json: &Value,
    admissible: bool,
) -> Vec<String> {
    let mut codes = Vec::new();
    let Some(wire) = mix_wire_from_spec_value(mix_json) else {
        codes.push(explain_code_rational_parse_fail());
        return codes;
    };
    let Ok(mut spec) = MixSpec::try_from(wire) else {
        codes.push(explain_code_wire_invalid());
        return codes;
    };
    spec.profile_name = profile.bundle_id.clone();
    if admissible {
        return codes;
    }
    #[cfg(feature = "manifest-bridge")]
    {
        if !thermodynamic_ok(profile, &spec) {
            codes.push(explain_code_cd_fail());
        }
    }
    #[cfg(not(feature = "manifest-bridge"))]
    {
        let _ = spec;
        codes.push(explain_code_manifest_bridge_disabled());
    }
    if codes.is_empty() {
        codes.push(explain_code_thermodynamic_fail());
    }
    codes
}

fn explain_code_rational_parse_fail() -> String {
    MIX_SPEC_RATIONAL_PARSE_FAIL.into()
}

fn explain_code_wire_invalid() -> String {
    MIX_SPEC_WIRE_INVALID.into()
}

fn explain_code_cd_fail() -> String {
    THERMODYNAMIC_CD_FAIL.into()
}

fn explain_code_manifest_bridge_disabled() -> String {
    MANIFEST_BRIDGE_DISABLED.into()
}

fn explain_code_thermodynamic_fail() -> String {
    THERMODYNAMIC_FAIL.into()
}

/// Build `gate_reject.v1` row when mix fails gate (never enters admissible memory).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Audit stream morphism; reject rows excluded from `admissible_only` query.
#[must_use]
pub fn gate_reject_row_for_mix(
    mix_json: &Value,
    summary: &GateSummary,
    observed_at: super::types::ObservedAt,
) -> Option<GateRejectRow> {
    if summary.admissible {
        return None;
    }
    Some(build_gate_reject(
        mix_json,
        summary.verdict,
        summary.catalog_ids.clone(),
        None,
        observed_at,
        Some(vec!["thermodynamic_fail".into()]),
    ))
}

fn stub_contribution(mix_json: &Value) -> Contribution {
    Contribution {
        schema_version: CONTRIBUTION_SCHEMA.to_string(),
        canon_version: CANON_VERSION.to_string(),
        mix_spec: mix_json.clone(),
        process: Value::Object(Default::default()),
        outcome: Value::Object(Default::default()),
        gate_summary: GateSummary {
            admissible: true,
            verdict: GateVerdict::Pass,
            catalog_ids: vec!["umst.gate.cd_transition".into()],
            safety_margin: None,
            mi_bits_est: None,
        },
        catalog_hash: DEFAULT_CATALOG_HASH.to_string(),
        observed_at: super::types::ObservedAt {
            stamp_tier: "Synthetic".into(),
            ucrs_seq: Some(0),
            phase_entropy_bits_q: None,
            phase_entropy_bits_scale: None,
            credit_head_bits_q: None,
            credit_head_bits_scale: None,
            wall_ms: None,
        },
        content_hash: None,
        scope_token: None,
        idempotency_key: None,
    }
}

/// Rational wire `n/d` → `f64` for mix_spec parsing.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Wire decode only; physical units validated downstream on `MixSpec`.
#[must_use]
pub fn rational_to_f64(s: &str) -> Option<f64> {
    let (n, d) = s.split_once('/')?;
    let nf = n.parse::<f64>().ok()?;
    let df = d.parse::<f64>().ok()?;
    if df == 0.0 {
        return None;
    }
    Some(nf / df)
}

/// Pure: rational wire field `n/d` → `f64`.
#[must_use]
pub(crate) fn field_as_rational(obj: &Value, key: &str) -> Option<f64> {
    obj.get(key)
        .and_then(|v| v.as_str())
        .and_then(rational_to_f64)
}

/// Parse mix_spec JSON rationals into [`MixSpecWire`].
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Serde routing to facade wire; gate on `MixSpec::try_from`.
#[must_use]
pub fn mix_wire_from_spec_value(v: &Value) -> Option<MixSpecWire> {
    Some(MixSpecWire {
        w_c: field_as_rational(v, "w_c")?,
        temperature_k: field_as_rational(v, "temperature_k")?,
        superplasticiser_pct: v
            .get("superplasticiser_pct")
            .and_then(|x| x.as_str())
            .and_then(rational_to_f64),
        silica_fume_pct: v
            .get("silica_fume_pct")
            .and_then(|x| x.as_str())
            .and_then(rational_to_f64),
        fly_ash_pct: v
            .get("fly_ash_pct")
            .and_then(|x| x.as_str())
            .and_then(rational_to_f64),
        aggregate_volume_fraction: v
            .get("aggregate_volume_fraction")
            .and_then(|x| x.as_str())
            .and_then(rational_to_f64),
        target_age_hours: v
            .get("target_age_hours")
            .and_then(|x| x.as_str())
            .and_then(rational_to_f64),
    })
}

/// Re-check thermodynamic admissibility before memory append.
/// formal_anchor: lean://umst-formal/Lean/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
#[must_use]
pub fn gate_recheck(ctx: &GateContext<'_>, contribution: &Contribution) -> bool {
    #[cfg(feature = "manifest-bridge")]
    {
        let Some(wire) = mix_wire_from_spec_value(&contribution.mix_spec) else {
            return false;
        };
        let Ok(mut spec) = MixSpec::try_from(wire) else {
            return false;
        };
        spec.profile_name = ctx.profile.bundle_id.clone();
        return thermodynamic_ok(ctx.profile, &spec);
    }

    #[cfg(not(feature = "manifest-bridge"))]
    {
        let _ = ctx;
        contribution.gate_summary.admissible
    }
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
    observed_at: super::types::ObservedAt,
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
/// formal_anchor: lean://umst-formal/Lean/Gate.lean#Admissible
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
        return Err(AcceptError::GateReject(row));
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
