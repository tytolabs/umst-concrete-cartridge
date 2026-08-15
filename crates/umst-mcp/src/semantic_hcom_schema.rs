// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! HCOM semantic MCP schema — `propose_communicative_act` (HCOM-029 · IDEA-004).
//!
//! **FENCE:** additive schema + honest mock stub in `umst-mcp` only; frozen 13-tool manifest
//! unchanged unless `tool-semantic-hcom` feature is enabled.

use serde_json::{json, Value};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Schema bundle version — bump on breaking field changes.
pub const HCOM_MCP_SCHEMA_VERSION: &str = "hcom_mcp_v0";

/// MCP tool name (blueprint §5).
pub const PROPOSE_COMMUNICATIVE_ACT_TOOL: &str = "propose_communicative_act";

/// Input schema id.
pub const PROPOSE_COMMUNICATIVE_ACT_V0: &str = "propose_communicative_act_v0";

/// Output schema id.
pub const GATED_COMMUNICATIVE_RESPONSE_V0: &str = "gated_communicative_response_v0";

const PROPOSE_COMMUNICATIVE_ACT_JSON: &str =
    include_str!("schemas/propose_communicative_act_v0.json");
const GATED_COMMUNICATIVE_RESPONSE_JSON: &str =
    include_str!("schemas/gated_communicative_response_v0.json");

const JSON_SCHEMA_2020: &str = "https://json-schema.org/draft/2020-12/schema";

/// Mock frontier fixtures — deterministic witness legs (mirrors `SemanticResponse` presets).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockLlmFixture {
    ConsistentChair,
    InconsistentNoBack,
    MiShortfall,
}

impl MockLlmFixture {
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "consistent_chair" => Some(Self::ConsistentChair),
            "inconsistent_no_back" => Some(Self::InconsistentNoBack),
            "mi_shortfall" => Some(Self::MiShortfall),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ConsistentChair => "consistent_chair",
            Self::InconsistentNoBack => "inconsistent_no_back",
            Self::MiShortfall => "mi_shortfall",
        }
    }

    #[must_use]
    pub fn witness_legs(self) -> (f64, f64, f64) {
        match self {
            Self::ConsistentChair => (0.0, 0.0, 0.0),
            Self::InconsistentNoBack => (1.0, 0.0, 0.0),
            Self::MiShortfall => (0.0, 1.5, 0.0),
        }
    }

    #[must_use]
    pub fn admissible(self) -> bool {
        matches!(self, Self::ConsistentChair)
    }

    #[must_use]
    pub fn geometry_ref(self) -> &'static str {
        match self {
            Self::ConsistentChair => "chair:quotient:v0",
            Self::InconsistentNoBack => "chair:quotient:incomplete",
            Self::MiShortfall => "chair:quotient:mi_gap",
        }
    }
}

/// Parse embedded schema JSON — panics only on corrupt embed (build-time invariant).
#[must_use]
pub fn parse_schema(raw: &str) -> Value {
    serde_json::from_str(raw).expect("embedded HCOM MCP schema must be valid JSON")
}

/// MCP tool descriptor for `propose_communicative_act`.
#[must_use]
pub fn propose_communicative_act_tool_schema() -> Value {
    let mut tool = json!({
        "name": PROPOSE_COMMUNICATIVE_ACT_TOOL,
        "description": "Hybrid frontier proposal + local semantic gate (HCOM-029). Frontier LLM proposes; local cartridge maps; local gate refines. Honest mock when tool-semantic-hcom feature enabled.",
        "inputSchema": parse_schema(PROPOSE_COMMUNICATIVE_ACT_JSON),
    });
    tool["annotations"] = json!({
        "readOnlyHint": true,
        "destructiveHint": false,
    });
    if let Some(schema) = tool.get_mut("inputSchema").and_then(|s| s.as_object_mut()) {
        schema.insert("$schema".into(), json!(JSON_SCHEMA_2020));
    }
    tool
}

/// Versioned schema bundle for gateway dual-emit / documentation.
#[must_use]
pub fn hcom_mcp_schema_bundle() -> Value {
    json!({
        "schema_version": HCOM_MCP_SCHEMA_VERSION,
        "tool": PROPOSE_COMMUNICATIVE_ACT_TOOL,
        "schemas": {
            PROPOSE_COMMUNICATIVE_ACT_V0: parse_schema(PROPOSE_COMMUNICATIVE_ACT_JSON),
            GATED_COMMUNICATIVE_RESPONSE_V0: parse_schema(GATED_COMMUNICATIVE_RESPONSE_JSON),
        },
        "response_family": "Semantic",
        "response_type": "SemanticResponse",
        "discipline": "additive_umst_mcp_schema_frozen_13_unchanged_without_feature",
    })
}

fn content_hash_prefix(seed: &str) -> String {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn infer_fixture(intent: &str, explicit: Option<&str>) -> MockLlmFixture {
    if let Some(raw) = explicit {
        if let Some(fixture) = MockLlmFixture::parse(raw) {
            return fixture;
        }
    }
    let lower = intent.to_ascii_lowercase();
    if lower.contains("no back") || lower.contains("incomplete") {
        MockLlmFixture::InconsistentNoBack
    } else if lower.contains("mi") || lower.contains("shortfall") {
        MockLlmFixture::MiShortfall
    } else {
        MockLlmFixture::ConsistentChair
    }
}

/// Mock hybrid loop: frontier proposal → geometry ref → local gate witness.
#[must_use]
pub fn exec_propose_communicative_act_mock(args: &Value) -> (Value, bool) {
    let intent = args
        .get("intent")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let lang = args
        .get("context")
        .and_then(|c| c.get("lang"))
        .and_then(|l| l.as_str())
        .unwrap_or("en");
    let dialogue_turn = args
        .get("context")
        .and_then(|c| c.get("dialogue_turn"))
        .and_then(|t| t.as_u64())
        .unwrap_or(0);
    let mock_fixture = args.get("mock_llm_fixture").and_then(|f| f.as_str());
    let used_mock = mock_fixture.is_some();
    let fixture = infer_fixture(intent, mock_fixture);
    let (consistency_defect, mi_deficit, understanding_cost) = fixture.witness_legs();
    let admissible = fixture.admissible();
    let verdict = if admissible { "ADMIT" } else { "REJECT" };

    let decision_seed = format!("{intent}|{lang}|{dialogue_turn}|{}", fixture.as_str());
    let decision_id = format!("hcom-act:{lang}:turn{dialogue_turn}:{}", fixture.as_str());
    let hash_prefix = content_hash_prefix(&decision_seed);

    let body = json!({
        "schema_version": "gated_communicative_response.v0",
        "proposed": true,
        "mock_llm": used_mock,
        "proposal": {
            "surface_form": intent,
            "geometry_ref": fixture.geometry_ref(),
            "fixture": fixture.as_str(),
            "lang": lang,
        },
        "gate": {
            "admissible": admissible,
            "verdict": verdict,
            "response_family": "Semantic",
            "response_type": "SemanticResponse",
        },
        "semantic_witness": {
            "consistency_defect": consistency_defect,
            "mi_deficit": mi_deficit,
            "understanding_cost": understanding_cost,
        },
        "audit": {
            "decision_id": decision_id,
            "content_hash_prefix": hash_prefix,
        },
    });
    (body, !admissible)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_schemas_parse() {
        for raw in [
            PROPOSE_COMMUNICATIVE_ACT_JSON,
            GATED_COMMUNICATIVE_RESPONSE_JSON,
        ] {
            let v: Value = serde_json::from_str(raw).expect("valid JSON");
            assert!(v.get("$schema").is_some());
            assert!(v.get("properties").is_some());
        }
    }

    #[test]
    fn mock_admits_consistent_fixture() {
        let (out, is_error) = exec_propose_communicative_act_mock(&json!({
            "intent": "describe chair with back support",
            "context": { "lang": "en", "dialogue_turn": 0 },
            "mock_llm_fixture": "consistent_chair"
        }));
        assert!(!is_error);
        assert_eq!(out["gate"]["admissible"], json!(true));
    }
}
