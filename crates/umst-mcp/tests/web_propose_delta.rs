// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! WEB-009 — umst-mcp `web.propose_delta` schema + mock wire tests.

use serde_json::json;
use umst_mcp::web_propose_delta::{
    exec_web_propose_delta_mock, web_propose_delta_schema_bundle, web_propose_delta_tool_schema,
    WEB_PROPOSE_DELTA_INPUT_REQUIRED, WEB_PROPOSE_DELTA_METHOD, WEB_PROPOSE_DELTA_TOOL,
};

#[test]
fn tool_schema_emits_web_propose_delta() {
    let tool = web_propose_delta_tool_schema();
    assert_eq!(tool["name"].as_str(), Some(WEB_PROPOSE_DELTA_TOOL));
    let input = tool.get("inputSchema").expect("inputSchema");
    let required: Vec<&str> = input
        .get("required")
        .and_then(|r| r.as_array())
        .expect("required array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    for field in WEB_PROPOSE_DELTA_INPUT_REQUIRED {
        assert!(required.contains(field), "missing required field {field}");
    }
}

#[test]
fn bundle_method_alias_matches_blueprint() {
    let bundle = web_propose_delta_schema_bundle();
    assert_eq!(bundle["method"].as_str(), Some(WEB_PROPOSE_DELTA_METHOD));
    assert_eq!(bundle["response_family"].as_str(), Some("Informational"));
}

#[test]
fn mock_accept_reject_wire_shapes() {
    let accept = exec_web_propose_delta_mock(&json!({
        "current_state": "dGVuc29yLXYx",
        "proposed_delta": "ZGVsdGE=",
        "intent_witness": [0.1, 0.2],
        "mock_fixture": "balanced_accept"
    }));
    assert_eq!(accept["outcome"], json!("Accept"));
    assert!(accept.get("ucrs_stamp").is_some());
    assert!(accept.get("proof").is_some());

    let reject = exec_web_propose_delta_mock(&json!({
        "current_state": "dGVuc29yLXYx",
        "proposed_delta": "reject-spike",
        "intent_witness": "lean://Web/intent",
        "mock_fixture": "under_budget_reject"
    }));
    assert_eq!(reject["outcome"], json!("Reject"));
    assert!(reject.get("gradient").is_some());
    assert!(reject.get("reason").is_some());
}
