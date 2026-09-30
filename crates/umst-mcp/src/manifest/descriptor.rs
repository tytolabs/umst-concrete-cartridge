// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Declarative MCP tool descriptors (Stage S1).
//!
//! ADDITIVE — compiled only with feature `tool-manifest` (default off).

use serde_json::Value;

/// Side-effect class for agent / orchestrator policy.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Algebraic partition of tool effects; invalid mixes unrepresentable.
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
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Typed MCP tool row; wire emission via `to_mcp_tool`.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub side_effect: SideEffectClass,
    /// Optional MCP annotations (`readOnlyHint`, …).
    pub annotations: Option<Value>,
    /// Optional 7-field B3 contract (from `tools_v1_contracts.json` when loaded).
    pub contract: Option<ToolContract>,
}

/// Seven-field tool contract (Pre/Post/Errors/Idempotent/SideEffectClass/Cost/Provenance).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: B3 contract row; docs generator SSOT companion to wire schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolContract {
    pub pre: String,
    pub post: String,
    pub errors: String,
    pub idempotent: String,
    /// Doc / policy spelling (may be `EpistemicMutating` while `side_effect` is `Mutating`).
    pub side_effect_class: String,
    pub cost: String,
    pub provenance: String,
}

impl ToolDescriptor {
    /// Emit one MCP `tools/list` tool object.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: JSON-RPC list_tools wire shape; not a physics morphism.
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
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Schema introspection helper for S1 parity tests.
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
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Name→effect table for descriptors; policy metadata only.
#[must_use]
pub fn side_effect_for(name: &str) -> SideEffectClass {
    match name {
        "umst_contribute" | "umst_transition_propose" => SideEffectClass::Mutating,
        "umst_arena_open" | "umst_arena_close" => SideEffectClass::Session,
        _ => SideEffectClass::ReadOnly,
    }
}
