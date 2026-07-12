// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Declarative MCP tool descriptors (Stage S1).
//!
//! ADDITIVE — compiled only with feature `tool-manifest` (default off).

use serde_json::Value;

/// Side-effect class for agent / orchestrator policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideEffectClass {
    /// No durable writes (gate, predict, profiles, …).
    ReadOnly,
    /// May mutate research memory / jobs (contribute, transition_propose).
    Mutating,
    /// Session-local arena lifecycle (open/close); not durable memory.
    Session,
}

/// Public tool descriptor — schema SSOT candidate (cutover still USER-gated).
#[derive(Debug, Clone, PartialEq)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub side_effect: SideEffectClass,
    /// Optional MCP annotations (`readOnlyHint`, …).
    pub annotations: Option<Value>,
}

impl ToolDescriptor {
    /// Emit one MCP `tools/list` tool object.
    #[must_use]
    pub fn to_mcp_tool(&self) -> Value {
        let mut tool = serde_json::json!({
            "name": self.name,
            "description": self.description,
            "inputSchema": self.input_schema,
        });
        if let Some(ann) = &self.annotations {
            tool["annotations"] = ann.clone();
        }
        tool
    }

    /// Required inputSchema property names (sorted).
    #[must_use]
    pub fn required_keys(&self) -> Vec<String> {
        self.input_schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| {
                let mut keys: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                keys.sort();
                keys
            })
            .unwrap_or_default()
    }
}

/// Map tool name → side-effect class (hand annotations + known mutators).
#[must_use]
pub fn side_effect_for(name: &str) -> SideEffectClass {
    match name {
        "umst_contribute" | "umst_transition_propose" => SideEffectClass::Mutating,
        "umst_arena_open" | "umst_arena_close" => SideEffectClass::Session,
        _ => SideEffectClass::ReadOnly,
    }
}
