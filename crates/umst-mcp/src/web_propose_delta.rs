// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! WEB-009 — MCP `web.propose_delta` schema + honest mock gate wire.
//!
//! **FENCE:** additive schema in `umst-mcp`; frozen 13-tool manifest unchanged unless
//! `tool-web-propose-delta` feature is enabled.
//!
//! Blueprint method `web.propose_delta` maps to MCP tool [`WEB_PROPOSE_DELTA_TOOL`].

use serde_json::{json, Value};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Schema bundle version — bump on breaking field changes.
pub const WEB_PROPOSE_DELTA_SCHEMA_VERSION: &str = "web_propose_delta_mcp_v0";

/// MCP tool name (blueprint §5 `web.propose_delta`).
pub const WEB_PROPOSE_DELTA_TOOL: &str = "web_propose_delta";

/// JSON-RPC method alias documented for agent discovery.
pub const WEB_PROPOSE_DELTA_METHOD: &str = "web.propose_delta";

/// Input schema id.
pub const WEB_PROPOSE_DELTA_V0: &str = "web_propose_delta_v0";

/// Output schema id.
pub const WEB_PROPOSE_DELTA_RESPONSE_V0: &str = "web_propose_delta_response_v0";

/// WEB-009 L5 stdio delegate schema id — typed JSON-RPC frame contract (X04 deepen).
pub const WEB_009_L5_STDIO_SCHEMA_ID: &str = "web_009_l5_stdio_delegate_v0";

/// WEB-009 L5 delegate wire hop count (5 core + 1 UCRS prep).
pub const WEB_009_L5_DELEGATE_WIRE_HOP_COUNT: u8 = 6;

/// Required input fields — gateway parity contract.
pub const WEB_PROPOSE_DELTA_INPUT_REQUIRED: &[&str] =
    &["current_state", "proposed_delta", "intent_witness"];

const WEB_PROPOSE_DELTA_JSON: &str = include_str!("schemas/web_propose_delta_v0.json");
const WEB_PROPOSE_DELTA_RESPONSE_JSON: &str =
    include_str!("schemas/web_propose_delta_response_v0.json");

const JSON_SCHEMA_2020: &str = "https://json-schema.org/draft/2020-12/schema";

/// Mock gate fixtures — deterministic informational outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockWebFixture {
    BalancedAccept,
    UnderBudgetReject,
    MissingWitnessReject,
}

impl MockWebFixture {
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "balanced_accept" => Some(Self::BalancedAccept),
            "under_budget_reject" => Some(Self::UnderBudgetReject),
            "missing_witness_reject" => Some(Self::MissingWitnessReject),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BalancedAccept => "balanced_accept",
            Self::UnderBudgetReject => "under_budget_reject",
            Self::MissingWitnessReject => "missing_witness_reject",
        }
    }

    #[must_use]
    pub fn admissible(self) -> bool {
        matches!(self, Self::BalancedAccept)
    }
}

/// Parse embedded schema JSON — panics only on corrupt embed (build-time invariant).
#[must_use]
pub fn parse_schema(raw: &str) -> Value {
    serde_json::from_str(raw).expect("embedded web propose_delta schema must be valid JSON")
}

/// MCP tool descriptor for `web_propose_delta`.
#[must_use]
pub fn web_propose_delta_tool_schema() -> Value {
    let mut tool = json!({
        "name": WEB_PROPOSE_DELTA_TOOL,
        "description": "Propose informational web state delta; gate returns Accept { new_state, ucrs_stamp, proof } or Reject { gradient, reason } (WEB-009). Honest mock when tool-web-propose-delta enabled.",
        "inputSchema": parse_schema(WEB_PROPOSE_DELTA_JSON),
    });
    tool["annotations"] = json!({
        "readOnlyHint": true,
        "destructiveHint": false,
    });
    if let Some(schema) = tool
        .get_mut("inputSchema")
        .and_then(|s| s.as_object_mut())
    {
        schema.insert("$schema".into(), json!(JSON_SCHEMA_2020));
    }
    tool
}

/// Versioned schema bundle for gateway dual-emit / documentation.
#[must_use]
pub fn web_propose_delta_schema_bundle() -> Value {
    json!({
        "schema_version": WEB_PROPOSE_DELTA_SCHEMA_VERSION,
        "method": WEB_PROPOSE_DELTA_METHOD,
        "tool": WEB_PROPOSE_DELTA_TOOL,
        "schemas": {
            WEB_PROPOSE_DELTA_V0: parse_schema(WEB_PROPOSE_DELTA_JSON),
            WEB_PROPOSE_DELTA_RESPONSE_V0: parse_schema(WEB_PROPOSE_DELTA_RESPONSE_JSON),
        },
        "input_required": WEB_PROPOSE_DELTA_INPUT_REQUIRED,
        "response_family": "Informational",
        "response_type": "InformationalResponse",
        "discipline": "additive_umst_mcp_schema_frozen_13_unchanged_without_feature",
    })
}

fn content_hash_prefix(seed: &str) -> String {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn witness_present(intent_witness: &Value) -> bool {
    match intent_witness {
        Value::String(s) => !s.is_empty(),
        Value::Array(v) => !v.is_empty(),
        _ => false,
    }
}

fn infer_fixture(
    current_state: &str,
    proposed_delta: &str,
    intent_witness: &Value,
    explicit: Option<&str>,
) -> MockWebFixture {
    if let Some(raw) = explicit {
        if let Some(fixture) = MockWebFixture::parse(raw) {
            return fixture;
        }
    }
    if !witness_present(intent_witness) {
        return MockWebFixture::MissingWitnessReject;
    }
    let lower = proposed_delta.to_ascii_lowercase();
    if lower.contains("reject") || lower.contains("under_budget") {
        MockWebFixture::UnderBudgetReject
    } else if current_state.is_empty() {
        MockWebFixture::MissingWitnessReject
    } else {
        MockWebFixture::BalancedAccept
    }
}

/// Whether Accept-arm `ucrs_stamp` on mock response is honest (observed_at.v2 Synthetic).
#[must_use]
pub fn accept_response_ucrs_stamp_honest() -> bool {
    let out = exec_web_propose_delta_mock(&json!({
        "current_state": "dGVuc29yLXYx",
        "proposed_delta": "ZGVsdGEtdjE=",
        "intent_witness": [0.1, 0.2],
        "mock_fixture": "balanced_accept"
    }));
    out.get("ucrs_stamp")
        .and_then(|s| s.get("schema_version"))
        .and_then(|v| v.as_str())
        == Some("observed_at.v2")
        && out
            .get("ucrs_stamp")
            .and_then(|s| s.get("stamp_tier"))
            .and_then(|t| t.as_str())
            == Some("Synthetic")
}

/// WEB-009 L5 stdio delegate schema pins honest — schema id + UCRS Accept arm.
#[must_use]
pub fn web_009_l5_stdio_delegate_schema_honest() -> bool {
    WEB_009_L5_STDIO_SCHEMA_ID == "web_009_l5_stdio_delegate_v0"
        && WEB_009_L5_DELEGATE_WIRE_HOP_COUNT == 6
        && accept_response_ucrs_stamp_honest()
}

/// Mock informational gate for `web.propose_delta` — honest stub until live wasm fold.
///
/// formal_anchor: NONE
/// formal_anchor_rationale: Stub wire; production path delegates to umst-web + gateway.
#[must_use]
pub fn exec_web_propose_delta_mock(args: &Value) -> Value {
    let current_state = args
        .get("current_state")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let proposed_delta = args
        .get("proposed_delta")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let intent_witness = args
        .get("intent_witness")
        .cloned()
        .unwrap_or(Value::Null);
    let mock_fixture = args.get("mock_fixture").and_then(|f| f.as_str());
    let fixture = infer_fixture(current_state, proposed_delta, &intent_witness, mock_fixture);
    let seed = format!(
        "{current_state}|{proposed_delta}|{}",
        fixture.as_str()
    );
    let proof = format!("web-proof:{}", content_hash_prefix(&seed));

    if fixture.admissible() {
        let new_state = format!("{current_state}+{proposed_delta}");
        json!({
            "schema_version": "web_propose_delta_response.v0",
            "outcome": "Accept",
            "new_state": new_state,
            "ucrs_stamp": {
                "schema_version": "observed_at.v2",
                "stamp_tier": "Synthetic",
                "ucrs_seq": 1
            },
            "proof": proof,
            "response_family": "Informational",
            "mock_fixture": fixture.as_str(),
        })
    } else {
        let (informational_net, reason) = match fixture {
            MockWebFixture::UnderBudgetReject => (
                -0.5,
                "informational_net under budget — complexity spike in proposed_delta",
            ),
            MockWebFixture::MissingWitnessReject => (
                -1.0,
                "intent_witness missing or empty — WEB-007 enforcement",
            ),
            MockWebFixture::BalancedAccept => (-0.0, "unreachable"),
        };
        json!({
            "schema_version": "web_propose_delta_response.v0",
            "outcome": "Reject",
            "gradient": {
                "informational_net": informational_net,
                "repair_direction": "reduce complexity_cost or supply valid intent_witness"
            },
            "reason": reason,
            "response_family": "Informational",
            "mock_fixture": fixture.as_str(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_schemas_parse() {
        for raw in [WEB_PROPOSE_DELTA_JSON, WEB_PROPOSE_DELTA_RESPONSE_JSON] {
            let v: Value = serde_json::from_str(raw).expect("valid JSON");
            assert!(v.get("$schema").is_some());
        }
    }

    #[test]
    fn bundle_exports_propose_delta_tool() {
        let bundle = web_propose_delta_schema_bundle();
        assert_eq!(bundle["tool"].as_str(), Some(WEB_PROPOSE_DELTA_TOOL));
        assert_eq!(bundle["method"].as_str(), Some(WEB_PROPOSE_DELTA_METHOD));
        let schemas = bundle["schemas"].as_object().expect("schemas");
        assert!(schemas.contains_key(WEB_PROPOSE_DELTA_V0));
        assert!(schemas.contains_key(WEB_PROPOSE_DELTA_RESPONSE_V0));
    }

    #[test]
    fn web_009_l5_stdio_delegate_schema_honest() {
        assert_eq!(WEB_009_L5_STDIO_SCHEMA_ID, "web_009_l5_stdio_delegate_v0");
        assert_eq!(WEB_009_L5_DELEGATE_WIRE_HOP_COUNT, 6);
        assert!(accept_response_ucrs_stamp_honest());
        assert!(web_009_l5_stdio_delegate_schema_honest());
    }

    #[test]
    fn mock_accepts_balanced_fixture() {
        let out = exec_web_propose_delta_mock(&json!({
            "current_state": "dGVuc29yLXYx",
            "proposed_delta": "ZGVsdGEtdjE=",
            "intent_witness": [0.1, 0.2],
            "mock_fixture": "balanced_accept"
        }));
        assert_eq!(out["outcome"], json!("Accept"));
        assert!(out.get("ucrs_stamp").is_some());
    }

    #[test]
    fn mock_rejects_under_budget_fixture() {
        let out = exec_web_propose_delta_mock(&json!({
            "current_state": "dGVuc29yLXYx",
            "proposed_delta": "under_budget",
            "intent_witness": "lean://Web/intent",
            "mock_fixture": "under_budget_reject"
        }));
        assert_eq!(out["outcome"], json!("Reject"));
        assert!(out["gradient"]["informational_net"].as_f64().unwrap() < 0.0);
    }

    #[test]
    fn missing_witness_rejects_without_fixture() {
        let out = exec_web_propose_delta_mock(&json!({
            "current_state": "dGVuc29yLXYx",
            "proposed_delta": "ZGVsdGE=",
            "intent_witness": ""
        }));
        assert_eq!(out["outcome"], json!("Reject"));
    }
}
