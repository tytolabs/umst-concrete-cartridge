// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//
// AGAP-2033-SEC-MCP-WRAP — typed trust refuse at HCOM-029 semantic agent MCP surface.
//
// **Policy:** prefer trust-aware refuse — bootstrap `Ephemeral` fails closed for write-class
// semantic tools unless caller supplies an explicit `trust.scope` witness.

use serde_json::{json, Value};
use umst_trust::{check_admit_trust_gate, Trust, TrustError, TrustGate, TrustScope};

/// Wire schema id for trust gate outcomes on semantic agent tools.
pub const MCP_TRUST_GATE_SCHEMA: &str = "mcp_trust_gate.v1";

/// HCOM-029 owner fence — production deepen may overlap 1836 batch; this card is AGAP-2033 only.
pub const HCOM_029_OWNER: &str = "AGAP-2033-HCOM-029";

/// WEB-009 production closure remains 1836-spawn exclusive — not claimed here.
pub const WEB_009_PRODUCTION_OWNER: &str = "1836-spawn";

/// Minimum trust gate per HCOM-029 semantic agent MCP tool.
#[must_use]
pub fn trust_gate_for_semantic_agent_tool(tool_name: &str) -> Option<TrustGate> {
    match tool_name {
        "propose_communicative_act" | "refine_shape" => Some(TrustGate::DeviceWrite),
        "map_to_geometry" | "get_audit_digest" => Some(TrustGate::EphemeralRead),
        _ => None,
    }
}

/// Whether `tool_name` is subject to SEC-MCP-WRAP trust gate checks.
#[must_use]
pub fn is_trust_gated_semantic_agent_tool(tool_name: &str) -> bool {
    trust_gate_for_semantic_agent_tool(tool_name).is_some()
}

/// Parse optional `trust` witness from MCP tool arguments (`trust` or `context.trust`).
#[must_use]
pub fn trust_from_wire(args: &Value) -> Trust {
    let trust_block = args
        .get("trust")
        .or_else(|| args.get("context").and_then(|c| c.get("trust")));
    let scope_raw = trust_block
        .and_then(|t| t.get("scope"))
        .and_then(|s| s.as_str());
    let mut trust = Trust::bootstrap_unknown();
    trust.scope = match scope_raw {
        Some("device") => TrustScope::Device,
        Some("federated") => TrustScope::Federated,
        Some("high_assurance") | Some("high-assurance") => TrustScope::HighAssurance,
        Some("hardware_rooted") | Some("hardware-rooted") => TrustScope::HardwareRooted,
        Some("ephemeral") | None => TrustScope::Ephemeral,
        _ => TrustScope::Ephemeral,
    };
    if trust_block
        .and_then(|t| t.get("revoked"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        trust.revoke("mcp-trust-gate-test");
    }
    trust
}

/// Enforce typed trust gate for a semantic agent MCP tool — fails closed on insufficient scope.
pub fn check_semantic_agent_trust(tool_name: &str, args: &Value) -> Result<(), TrustError> {
    let Some(gate) = trust_gate_for_semantic_agent_tool(tool_name) else {
        return Ok(());
    };
    check_admit_trust_gate(&trust_from_wire(args), gate)
}

/// Pre-gate hook — returns refuse envelope when trust is insufficient.
#[must_use]
pub fn pre_gate_semantic_agent(tool_name: &str, args: &Value) -> Option<(Value, bool)> {
    match check_semantic_agent_trust(tool_name, args) {
        Ok(()) => None,
        Err(err) => Some((trust_refuse_wire(tool_name, &err), true)),
    }
}

/// Typed refuse wire for MCP `agent_error.v1` consumers.
#[must_use]
pub fn trust_refuse_wire(tool_name: &str, err: &TrustError) -> Value {
    let gate = trust_gate_for_semantic_agent_tool(tool_name)
        .map(|g| format!("{g:?}"))
        .unwrap_or_else(|| "none".into());
    json!({
        "agent_error": {
            "schema_version": "agent_error.v1",
            "code": "trust_refused",
            "tool": tool_name,
            "trust_gate_schema": MCP_TRUST_GATE_SCHEMA,
            "required_gate": gate,
            "message": format!("{err}"),
            "remediation": "Supply trust.scope >= device for propose_communicative_act/refine_shape; ephemeral suffices for map_to_geometry/get_audit_digest.",
            "hcom_029_owner": HCOM_029_OWNER,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_tools_require_device_scope() {
        let args = json!({ "intent": "chair" });
        let err = check_semantic_agent_trust("propose_communicative_act", &args).unwrap_err();
        assert!(matches!(err, TrustError::ScopeNotPermitted { .. }));
    }

    #[test]
    fn device_scope_admits_propose() {
        let args = json!({
            "intent": "chair",
            "trust": { "scope": "device" }
        });
        assert!(check_semantic_agent_trust("propose_communicative_act", &args).is_ok());
    }

    #[test]
    fn read_tools_admit_ephemeral() {
        let args = json!({ "surface": "chair" });
        assert!(check_semantic_agent_trust("map_to_geometry", &args).is_ok());
    }

    #[test]
    fn revoked_trust_refused() {
        let args = json!({
            "surface": "chair",
            "trust": { "scope": "device", "revoked": true }
        });
        assert!(check_semantic_agent_trust("map_to_geometry", &args).is_err());
    }
}
