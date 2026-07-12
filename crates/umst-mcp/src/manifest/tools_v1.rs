// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! `tools_v1` — encoded MCP tool table (frozen schema snapshot + side-effect class).
//!
//! Snapshot captured from today's hand-rolled `tools/list` (S0 lock era). Hand tables in
//! `main.rs` / `agent_layer.rs` remain the live default until cutover.

use super::descriptor::{side_effect_for, ToolDescriptor};
use serde_json::Value;

const SCHEMA_JSON: &str = include_str!("tools_v1_schema.json");

#[cfg(not(feature = "agent-layer"))]
const BASE_TOOL_NAMES: &[&str] = &[
    "umst_audit",
    "umst_certify",
    "umst_predict",
    "umst_profiles",
];

fn parse_snapshot() -> Vec<ToolDescriptor> {
    let root: Value = serde_json::from_str(SCHEMA_JSON).expect("tools_v1_schema.json");
    let tools = root["tools"].as_array().expect("tools array");
    tools
        .iter()
        .map(|t| {
            let name = t["name"].as_str().expect("name").to_string();
            ToolDescriptor {
                name: name.clone(),
                description: t["description"].as_str().unwrap_or("").to_string(),
                input_schema: t["inputSchema"].clone(),
                side_effect: side_effect_for(&name),
                annotations: t.get("annotations").cloned(),
            }
        })
        .collect()
}

/// All descriptors encoded for this build (4 base, or 13 with `agent-layer`).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Loads frozen tools_v1 schema snapshot; not a physics morphism.
#[must_use]
pub fn tools_v1() -> Vec<ToolDescriptor> {
    let all = parse_snapshot();
    #[cfg(feature = "agent-layer")]
    {
        let mut out = all;
        out.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(out.len(), 13, "tools_v1 snapshot must encode 13 tools");
        out
    }
    #[cfg(not(feature = "agent-layer"))]
    {
        let mut out: Vec<_> = all
            .into_iter()
            .filter(|d| BASE_TOOL_NAMES.contains(&d.name.as_str()))
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(out.len(), 4, "base tools_v1 must encode 4 tools");
        out
    }
}

/// MCP `tools/list` payload emitted from the manifest (dual-emit path).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Dual-emit list_tools from descriptors; hand tables remain default.
#[must_use]
pub fn mcp_tools_schema() -> Vec<Value> {
    // Preserve hand-table order: base tools first, then agent tools.
    let descriptors = tools_v1();
    let mut by_name: std::collections::BTreeMap<_, _> = descriptors
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect();

    let mut ordered_names: Vec<&str> = Vec::new();
    ordered_names.extend_from_slice(&[
        "umst_predict",
        "umst_audit",
        "umst_profiles",
        "umst_certify",
    ]);
    #[cfg(feature = "agent-layer")]
    {
        ordered_names.extend_from_slice(&[
            "umst_gate_check",
            "umst_contribute",
            "umst_contribute_status",
            "umst_memory_query",
            "umst_mi_estimate",
            "umst_transition_propose",
            "umst_arena_open",
            "umst_gate_check_arena",
            "umst_arena_close",
        ]);
    }

    ordered_names
        .into_iter()
        .filter_map(|n| by_name.remove(n).map(|d| d.to_mcp_tool()))
        .collect()
}
