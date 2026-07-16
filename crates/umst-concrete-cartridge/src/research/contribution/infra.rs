// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Phase 0c — **Infra** boundary: explain codes, remediation, field-issue formatting.
//!
//! Operator diagnostics and MCP wire shaping — **not** admissibility math (blueprint §7 0c).

use super::adapter::{mix_wire_from_spec_value, rational_to_f64};
use super::gate::{gate_check_mix, gate_reject_row_for_mix};
use crate::calibration::Profile;
use serde::Serialize;
use serde_json::Value;

#[cfg(not(feature = "manifest-bridge"))]
use super::super::gate_explain_ssot::MANIFEST_BRIDGE_DISABLED;
use super::super::gate_explain_ssot::{
    fields_for_code as ssot_fields_for_code, remediation_for_code, MIX_SPEC_RATIONAL_PARSE_FAIL,
    MIX_SPEC_WIRE_INVALID, THERMODYNAMIC_CD_FAIL, THERMODYNAMIC_FAIL,
};
use super::super::types::{GateSummary, ObservedAt};
#[cfg(feature = "manifest-bridge")]
use crate::pipeline::canonical_gate::thermodynamic_admissible;

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
    pub gate_reject: Option<super::super::reject::GateRejectRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explain: Option<GateCheckExplain>,
}

/// MCP `umst_gate_check` response builder — optional explain + embedded `gate_reject.v1`.
/// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
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
            .map(|f| GateFieldIssue {
                path: f.path,
                issue: f.issue,
            })
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
    let Ok(mut spec) = crate::facade::MixSpec::try_from(wire) else {
        codes.push(explain_code_wire_invalid());
        return codes;
    };
    spec.profile_name = profile.bundle_id.clone();
    if admissible {
        return codes;
    }
    #[cfg(feature = "manifest-bridge")]
    {
        if !thermodynamic_admissible(profile, &spec) {
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

#[cfg(not(feature = "manifest-bridge"))]
fn explain_code_manifest_bridge_disabled() -> String {
    MANIFEST_BRIDGE_DISABLED.into()
}

fn explain_code_thermodynamic_fail() -> String {
    THERMODYNAMIC_FAIL.into()
}
