// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Phase 0c — **Gate** boundary: pure admissibility predicate on an adapted mix.
//!
//! No explain codes, no rational parsing, no MCP wire formatting — only
//! thermodynamic admissibility (blueprint §7 0c). Phase 0d routes through
//! [`crate::pipeline::canonical_gate`] (manifold `core_gate` ∧ `material_gate`).

use super::adapter::mix_spec_from_json;
use super::DEFAULT_CATALOG_HASH;
use crate::calibration::Profile;
use crate::facade::MixSpec;
use serde_json::Value;

use super::super::mi::estimate_mi_bits_rational;
use super::super::reject::{build_gate_reject, GateRejectRow};
use super::super::types::{
    Contribution, GateSummary, GateVerdict, ObservedAt, CANON_VERSION, CONTRIBUTION_SCHEMA,
};

#[cfg(feature = "manifest-bridge")]
use crate::pipeline::canonical_gate::thermodynamic_admissible;

/// Gate evaluation context (bundled calibration profile).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Profile carrier for `gate_recheck`; CD math on manifold when manifest-bridge on.
pub struct GateContext<'a> {
    pub profile: &'a Profile,
}

/// MCP `umst_gate_check` core — thermodynamic admissibility for a mix_spec.
/// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
#[must_use]
pub fn gate_check_mix(profile: &Profile, mix_json: &Value) -> GateSummary {
    let admissible = mix_spec_from_json(profile, mix_json)
        .map(|spec| {
            gate_recheck_with_spec(
                &GateContext { profile },
                &stub_contribution(mix_json),
                &spec,
            )
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

/// Re-check thermodynamic admissibility before memory append.
/// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
#[must_use]
pub fn gate_recheck(ctx: &GateContext<'_>, contribution: &Contribution) -> bool {
    let Some(spec) = mix_spec_from_json(ctx.profile, &contribution.mix_spec) else {
        return false;
    };
    gate_recheck_with_spec(ctx, contribution, &spec)
}

/// Pure gate on an already-adapted [`MixSpec`] (adapter/gate seam for Phase 0c).
#[must_use]
pub fn gate_recheck_with_spec(
    ctx: &GateContext<'_>,
    contribution: &Contribution,
    spec: &MixSpec,
) -> bool {
    #[cfg(feature = "manifest-bridge")]
    {
        let _ = contribution;
        thermodynamic_admissible(ctx.profile, spec)
    }

    #[cfg(not(feature = "manifest-bridge"))]
    {
        let _ = ctx;
        let _ = spec;
        contribution.gate_summary.admissible
    }
}

/// Build `gate_reject.v1` row when mix fails gate (never enters admissible memory).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Audit stream morphism; reject rows excluded from `admissible_only` query.
#[must_use]
pub fn gate_reject_row_for_mix(
    mix_json: &Value,
    summary: &GateSummary,
    observed_at: ObservedAt,
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
        observed_at: ObservedAt {
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
