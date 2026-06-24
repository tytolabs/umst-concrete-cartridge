// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Gate explain vocabulary — mirrors [`umst_manifold::runtime::gate::explain_codes`] until pin bump.

/// Rational mix field failed to parse as `a/b`.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: gate_reject.v1 regime code; operator diagnostics not admissibility proof.
pub const MIX_SPEC_RATIONAL_PARSE_FAIL: &str = "mix_spec_rational_parse_fail";
/// `MixSpec` wire validation failed.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: gate_reject.v1 regime code; wire shape only, gate on `MixSpec::try_from`.
pub const MIX_SPEC_WIRE_INVALID: &str = "mix_spec_wire_invalid";
/// Clausius–Duhem thermodynamic margin negative.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: gate_reject.v1 regime code; CD admissibility checked on `gate_recheck`.
pub const THERMODYNAMIC_CD_FAIL: &str = "thermodynamic_cd_fail";
/// Generic thermodynamic admissibility failure.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: gate_reject.v1 regime code; fallback when no finer CD code applies.
pub const THERMODYNAMIC_FAIL: &str = "thermodynamic_fail";
/// MCP built without manifest-bridge / thermodynamic gate.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Build-feature regime code; not a mix physics claim.
pub const MANIFEST_BRIDGE_DISABLED: &str = "manifest_bridge_disabled";

/// One-line remediation for a reject code.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Operator remediation strings; mirrors manifold explain_codes until pin bump.
#[must_use]
pub fn remediation_for_code(code: &str) -> &'static str {
    match code {
        MIX_SPEC_RATIONAL_PARSE_FAIL => {
            "Use rational strings like \"3/4\" for all mix fields (not floats or bare numbers); ensure w_c and temperature_k are present."
        }
        MIX_SPEC_WIRE_INVALID => {
            "mix_spec failed MixSpec validation; compare field names and rational formats against umst://schemas/contribution.v1.json."
        }
        THERMODYNAMIC_CD_FAIL => {
            "Mix violates Clausius–Duhem margin; reduce w_c, adjust temperature_k, or change curing regime before re-checking."
        }
        MANIFEST_BRIDGE_DISABLED => {
            "Build umst-mcp with agent-layer and manifest-bridge features so the thermodynamic gate runs."
        }
        THERMODYNAMIC_FAIL => {
            "Thermodynamic admissibility failed; run umst_gate_check with explain:true and adjust mix_spec until verdict is PASS."
        }
        _ => "See regime_violations codes and umst://schemas/gate_reject.v1.json; fix mix_spec and re-run gate check.",
    }
}

/// Field paths implicated by a reject code.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Operator field hints for explain block; not admissibility proof.
#[must_use]
pub fn fields_for_code(code: &str, mix_has_temperature_k: bool) -> Vec<(String, String)> {
    match code {
        MIX_SPEC_RATIONAL_PARSE_FAIL => vec![("mix".into(), "rational_parse_fail".into())],
        MIX_SPEC_WIRE_INVALID => vec![("mix".into(), "wire_invalid".into())],
        THERMODYNAMIC_CD_FAIL | THERMODYNAMIC_FAIL => {
            let mut fields = vec![("mix.w_c".into(), "cd_margin_negative".into())];
            if mix_has_temperature_k {
                fields.push(("mix.temperature_k".into(), "regime_out_of_envelope".into()));
            }
            fields
        }
        MANIFEST_BRIDGE_DISABLED => vec![("build.features".into(), "manifest_bridge_disabled".into())],
        _ => Vec::new(),
    }
}
