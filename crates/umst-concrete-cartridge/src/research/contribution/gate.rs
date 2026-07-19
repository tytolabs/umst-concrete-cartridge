// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Phase 0c — **Gate** boundary: pure admissibility predicate on an adapted mix.
//!
//! No explain codes, no rational parsing, no MCP wire formatting — only
//! thermodynamic admissibility (blueprint §7 0c). Phase 0d routes through
//! [`crate::pipeline::canonical_gate`] (manifold `core_gate` ∧ `material_gate`).
//!
//! T2-S6 dup-ψ inventory — card `g_spawn_i_s6_gate_2054`. Under `b1-delegate`,
//! admissibility routes through consumer `gate_route_composed`; legacy
//! `thermodynamic_admissible` formula cfg-gated off on the delegate path.

use super::adapter::mix_spec_from_json;
use crate::calibration::Profile;
use crate::facade::MixSpec;
use serde_json::Value;

#[cfg(feature = "b1-delegate")]
use crate::api_consumer_compose::gate_admissible_via_compose;

use super::super::mi::estimate_mi_bits_rational;
use super::super::reject::{build_gate_reject, GateRejectRow};
use super::super::types::{Contribution, GateSummary, GateVerdict, ObservedAt};

#[cfg(all(not(feature = "b1-delegate"), feature = "manifest-bridge"))]
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
///
/// T2-S6 dup-ψ `gate_check_mix` — cfg-gated @ `g_spawn_i_s6_gate_2054`.
/// `b1-delegate`: consumer `gate_route_composed` SSOT; legacy `thermodynamic_admissible` cfg-gated off.
#[must_use]
pub fn gate_check_mix(profile: &Profile, mix_json: &Value) -> GateSummary {
    let admissible = gate_admissible_for_mix(profile, mix_json);
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

/// S6 production path — composed gate delegate (card `g_spawn_i_s6_gate_2054`).
#[cfg(feature = "b1-delegate")]
fn gate_admissible_for_mix(profile: &Profile, mix_json: &Value) -> bool {
    gate_admissible_via_compose(profile, mix_json)
}

/// Pre-S6 manifest-bridge thermodynamic formula — cfg-gated duplicate retained for non-delegate builds.
#[cfg(not(feature = "b1-delegate"))]
fn gate_admissible_for_mix(profile: &Profile, mix_json: &Value) -> bool {
    #[cfg(feature = "manifest-bridge")]
    {
        return mix_spec_from_json(profile, mix_json)
            .map(|spec| thermodynamic_admissible(profile, &spec))
            .unwrap_or(false);
    }
    #[cfg(not(feature = "manifest-bridge"))]
    {
        let _ = (profile, mix_json);
        false
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
/// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
///
/// T2-S6 dup-ψ `gate_recheck_with_spec` — cfg-gated @ `g_spawn_i_s6_gate_2054`.
/// `b1-delegate`: routes via `gate_admissible_via_compose`; legacy `thermodynamic_admissible` cfg-gated off.
#[must_use]
pub fn gate_recheck_with_spec(
    ctx: &GateContext<'_>,
    contribution: &Contribution,
    spec: &MixSpec,
) -> bool {
    #[cfg(feature = "b1-delegate")]
    {
        let _ = spec;
        return gate_admissible_via_compose(ctx.profile, &contribution.mix_spec);
    }

    #[cfg(all(not(feature = "b1-delegate"), feature = "manifest-bridge"))]
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
