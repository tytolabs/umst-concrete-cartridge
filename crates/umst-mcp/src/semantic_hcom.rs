// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
//
// HCOM-021 — Hybrid frontier proposal + local gate orchestration.
//
// MCP tool `propose_communicative_act` (feature `tool-propose-communicative-act`):
// 1. Frontier LLM proposes surface draft (mock injectable for tests)
// 2. Local cartridge maps surface → SemanticResponse
// 3. Local `gate<SemanticResponse>` hard-rejects inadmissible acts
// 4. Low-MI rows recommend external signal query (blueprint §3.5)

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use umst_semantics::{
    chair_compose_quotient_id, chair_cross_lang_semantic_response, chair_no_back_state,
    chair_ok_witness, chair_profile, chair_surface_form, gate_chair_no_back, gate_chair_surface,
    project_semantic_response, action_quotient_id_hex, get_audit_digest, AuditDigestError,
    LangCode, SemanticResponse, CHAIR_GATE_TOLERANCE,
};

const JSON_SCHEMA_2020: &str = "https://json-schema.org/draft/2020-12/schema";
const MAP_TO_GEOMETRY_JSON: &str = include_str!("schemas/map_to_geometry_v0.json");
const REFINE_SHAPE_JSON: &str = include_str!("schemas/refine_shape_v0.json");
const GET_AUDIT_DIGEST_JSON: &str = include_str!("schemas/get_audit_digest_v0.json");
const AUDIT_DIGEST_RESPONSE_JSON: &str = include_str!("schemas/audit_digest_response_v0.json");

/// HCOM-029 tools schema bundle version (map / refine / audit — excludes propose; SIM owns propose smoke).
pub const HCOM_TOOLS_SCHEMA_VERSION: &str = "hcom_tools_mcp_v0";

/// AGAP-2127 SIM slot owner — stdio smoke for propose_communicative_act.
pub const HCOM_SIM_SLOT_OWNER: &str = "AGAP-2127-HCOM-029-SIM";

/// AGAP-2127 TOOLS slot owner — map / refine / audit schema polish.
pub const HCOM_TOOLS_SLOT_OWNER: &str = "AGAP-2127-HCOM-029-TOOLS";
const WIRE_SCHEMA: &str = "gated_communicative_act.v1";
const MOCK_MODEL_ID: &str = "mock-frontier-v1";

/// Orchestration step labels (blueprint §3.5 hybrid loop).
pub const ORCHESTRATION_STEPS: &[&str] = &["frontier_propose", "cartridge_map", "local_gate"];

/// HCOM-029 additive semantic agent MCP tools (gateway routes semantic family).
pub const HCOM_SEMANTIC_AGENT_TOOLS: &[&str] = &[
    "propose_communicative_act",
    "map_to_geometry",
    "refine_shape",
    "get_audit_digest",
];

/// Whether `tool_name` is an HCOM-029 semantic agent MCP tool.
#[must_use]
pub fn is_hcom_semantic_agent_tool(tool_name: &str) -> bool {
    HCOM_SEMANTIC_AGENT_TOOLS.contains(&tool_name)
}

/// Injectable frontier LLM surface for tests and offline orchestration.
pub trait FrontierLlm {
    /// Propose a surface lemma from operator intent and language.
    fn propose_surface(&self, intent: &str, lang: LangCode) -> FrontierProposal;
}

/// Frontier model draft — step 1 of hybrid orchestration.
#[derive(Debug, Clone, PartialEq)]
pub struct FrontierProposal {
    pub surface: String,
    pub model_id: String,
    pub confidence: f64,
}

/// Mock frontier LLM — deterministic keyword routing for integration tests.
#[derive(Debug, Clone, Default)]
pub struct MockFrontierLlm {
    /// When set, bypasses keyword routing (stdio / unit tests).
    pub surface_override: Option<String>,
    /// When true, routes to W2 `no_back` injection (expect gate REJECT).
    pub no_back_injection: bool,
}

impl FrontierLlm for MockFrontierLlm {
    fn propose_surface(&self, intent: &str, lang: LangCode) -> FrontierProposal {
        if self.no_back_injection {
            return FrontierProposal {
                surface: "stool".to_string(),
                model_id: MOCK_MODEL_ID.into(),
                confidence: 0.55,
            };
        }
        if let Some(surface) = &self.surface_override {
            return FrontierProposal {
                surface: surface.clone(),
                model_id: MOCK_MODEL_ID.into(),
                confidence: 0.9,
            };
        }
        let lower = intent.to_ascii_lowercase();
        let surface = if lower.contains("no_back") || lower.contains("without back") {
            "stool".to_string()
        } else if lang == LangCode::Ta {
            "நாற்காலி".to_string()
        } else {
            "chair".to_string()
        };
        FrontierProposal {
            surface,
            model_id: MOCK_MODEL_ID.into(),
            confidence: 0.85,
        }
    }
}

/// Parse `context.lang` wire (`en`, `ta`, …).
#[must_use]
pub fn parse_lang_code(raw: &str) -> LangCode {
    match raw.to_ascii_lowercase().as_str() {
        "ta" | "tamil" => LangCode::Ta,
        _ => LangCode::En,
    }
}

/// Build mock LLM from optional `mock_llm` args block.
#[must_use]
pub fn mock_llm_from_args(args: &Value) -> MockFrontierLlm {
    let block = args.get("mock_llm").unwrap_or(&Value::Null);
    MockFrontierLlm {
        surface_override: block
            .get("surface")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        no_back_injection: block
            .get("no_back_injection")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
            || args
                .get("context")
                .and_then(|c| c.get("injection"))
                .and_then(|v| v.as_str())
                == Some("no_back"),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn gate_verdict_wire(outcome: umst_gate::CoreGateOutcome) -> Value {
    let admissible = outcome.is_accepted();
    json!({
        "admissible": admissible,
        "verdict": if admissible { "PASS" } else { "REJECT" },
        "net_dissipation": outcome.net_dissipation,
        "power_input": outcome.power_input,
        "dissipation": outcome.dissipation,
    })
}

fn semantic_response_enriched_wire(
    response: &SemanticResponse,
    lang: LangCode,
    surface: &str,
    session_id: &str,
) -> Value {
    let envelope = umst_semantics::envelope_from_chair_surface(lang, surface, session_id);
    let mut wire = umst_semantics::envelope_to_wire_json(&envelope);
    if let Some(obj) = wire.as_object_mut() {
        obj.insert(
            "consistency_defect".into(),
            json!(response.consistency_defect),
        );
        obj.insert("mi_deficit".into(), json!(response.mi_deficit));
        obj.insert(
            "understanding_cost".into(),
            json!(response.understanding_cost),
        );
    }
    wire
}

/// Hybrid orchestration core — frontier propose → cartridge map → local gate.
#[must_use]
pub fn orchestrate_communicative_act(
    intent: &str,
    lang: LangCode,
    mass_conserved: bool,
    mi_query_threshold: f64,
    llm: &impl FrontierLlm,
    no_back_injection: bool,
) -> Value {
    let frontier = llm.propose_surface(intent, lang);

    let (response, gate_outcome, mapping) = if no_back_injection {
        let state = chair_no_back_state(lang);
        let response =
            project_semantic_response(&state, &chair_ok_witness(), &chair_profile());
        let outcome = gate_chair_no_back(lang, mass_conserved);
        (
            response,
            outcome,
            json!({
                "path": "no_back_injection",
                "lang": lang_code_wire(lang),
                "cartridge_profile": "chair",
            }),
        )
    } else {
        let surface = chair_surface_form(lang, &frontier.surface);
        let response = chair_cross_lang_semantic_response(lang, &surface.lemma);
        let outcome = gate_chair_surface(&surface, mass_conserved);
        (
            response,
            outcome,
            json!({
                "path": "surface_map",
                "lang": lang_code_wire(lang),
                "surface": surface.lemma,
                "script": format!("{:?}", surface.script),
                "cartridge_profile": "chair",
            }),
        )
    };

    let admissible = gate_outcome.is_accepted();
    let mi_low = response.mi_deficit > mi_query_threshold;
    let external_signal_query = !admissible || mi_low;

    let audit_canon = serde_json::to_vec(&json!({
        "intent": intent,
        "frontier_surface": frontier.surface,
        "gate_admissible": admissible,
        "consistency_defect": response.consistency_defect,
    }))
    .unwrap_or_default();
    let audit_digest = format!("sha256:{}", sha256_hex(&audit_canon));

    json!({
        "schema_version": WIRE_SCHEMA,
        "proposed": true,
        "orchestration": {
            "schema_version": "hybrid_frontier_local_gate.v1",
            "steps": [
                { "step": "frontier_propose", "model_id": frontier.model_id, "confidence": frontier.confidence },
                { "step": "cartridge_map", "mapping": mapping },
                { "step": "local_gate", "tolerance": CHAIR_GATE_TOLERANCE },
            ],
            "external_signal_query_recommended": external_signal_query,
        },
        "frontier_proposal": {
            "surface": frontier.surface,
            "model_id": frontier.model_id,
            "confidence": frontier.confidence,
        },
        "semantic_response": semantic_response_enriched_wire(
            &response,
            lang,
            &frontier.surface,
            "mcp-orchestration",
        ),
        "gate_summary": gate_verdict_wire(gate_outcome),
        "audit_digest": audit_digest,
        "decision_id": audit_digest,
    })
}

fn lang_code_wire(lang: LangCode) -> &'static str {
    match lang {
        LangCode::En => "en",
        LangCode::Ta => "ta",
        LangCode::Zh => "zh",
        LangCode::Sa => "sa",
    }
}

fn parse_embedded_schema(raw: &str) -> Value {
    serde_json::from_str(raw).expect("embedded HCOM tool schema must be valid JSON")
}

fn with_schema_2020(mut tool: Value, read_only: bool) -> Value {
    tool["annotations"] = json!({
        "readOnlyHint": read_only,
        "destructiveHint": false,
    });
    if let Some(schema) = tool.get_mut("inputSchema").and_then(|s| s.as_object_mut()) {
        if schema.get("$schema").is_none() {
            schema.insert("$schema".into(), json!(JSON_SCHEMA_2020));
        }
    }
    tool
}

/// Versioned schema bundle for map / refine / audit tools (gateway dual-emit).
#[must_use]
pub fn hcom_semantic_tools_schema_bundle() -> Value {
    json!({
        "schema_version": HCOM_TOOLS_SCHEMA_VERSION,
        "tools": TOOLS_SLOT_TOOLS,
        "schemas": {
            "map_to_geometry_v0": parse_embedded_schema(MAP_TO_GEOMETRY_JSON),
            "refine_shape_v0": parse_embedded_schema(REFINE_SHAPE_JSON),
            "get_audit_digest_v0": parse_embedded_schema(GET_AUDIT_DIGEST_JSON),
            "audit_digest_response_v0": parse_embedded_schema(AUDIT_DIGEST_RESPONSE_JSON),
        },
        "response_family": "Semantic",
        "response_type": "SemanticResponse",
        "owner": HCOM_TOOLS_SLOT_OWNER,
        "sim_slot_owner": HCOM_SIM_SLOT_OWNER,
    })
}

/// SIM slot schema bundle — propose_communicative_act stdio smoke (deconflict TOOLS).
#[must_use]
pub fn hcom_sim_propose_schema_bundle() -> Value {
    json!({
        "schema_version": "hcom_sim_propose_mcp_v0",
        "tools": ["propose_communicative_act"],
        "schemas": {
            "propose_communicative_act_v0": parse_embedded_schema(include_str!("schemas/propose_communicative_act_v0.json")),
            "gated_communicative_response_v0": parse_embedded_schema(include_str!("schemas/gated_communicative_response_v0.json")),
        },
        "response_family": "Semantic",
        "response_type": "SemanticResponse",
        "owner": HCOM_SIM_SLOT_OWNER,
        "tools_slot_owner": HCOM_TOOLS_SLOT_OWNER,
    })
}

/// TOOLS slot tool names (map / refine / audit — SIM owns propose smoke).
pub const TOOLS_SLOT_TOOLS: &[&str] = &["map_to_geometry", "refine_shape", "get_audit_digest"];

/// MCP tool schemas for HCOM-029 semantic agent surface.
#[must_use]
pub fn hcom_semantic_agent_tool_schemas() -> Vec<Value> {
    vec![
        propose_communicative_act_tool_schema(),
        map_to_geometry_tool_schema(),
        refine_shape_tool_schema(),
        get_audit_digest_tool_schema(),
    ]
}

/// MCP tool schema for `map_to_geometry`.
#[must_use]
pub fn map_to_geometry_tool_schema() -> Value {
    with_schema_2020(
        json!({
            "name": "map_to_geometry",
            "description": "Map surface form to meaning geometry via semantic cartridge (HCOM-029). Chair EN/TA scope @ P1.",
            "inputSchema": parse_embedded_schema(MAP_TO_GEOMETRY_JSON),
        }),
        true,
    )
}

/// MCP tool schema for `refine_shape`.
#[must_use]
pub fn refine_shape_tool_schema() -> Value {
    with_schema_2020(
        json!({
            "name": "refine_shape",
            "description": "Refine a meaning shape with agent/human feedback (HCOM-029 stub — Kleisli dialogue deferred).",
            "inputSchema": parse_embedded_schema(REFINE_SHAPE_JSON),
        }),
        false,
    )
}

/// MCP tool schema for `get_audit_digest`.
#[must_use]
pub fn get_audit_digest_tool_schema() -> Value {
    with_schema_2020(
        json!({
            "name": "get_audit_digest",
            "description": "Content-addressed digest for a gated communicative decision (HCOM-029 · HCOM-022 fixture log).",
            "inputSchema": parse_embedded_schema(GET_AUDIT_DIGEST_JSON),
        }),
        true,
    )
}

/// MCP tool schema for `propose_communicative_act`.
#[must_use]
pub fn propose_communicative_act_tool_schema() -> Value {
    with_schema_2020(
        json!({
            "name": "propose_communicative_act",
            "description": "HCOM hybrid orchestration: frontier LLM proposes surface → local cartridge maps → local gate<SemanticResponse> refines. Returns gated_communicative_act.v1 with audit_digest. Chair EN/TA scope @ P1; use mock_llm for offline tests. Example: {\"intent\":\"describe chair to operator\",\"context\":{\"lang\":\"en\"}}.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "intent": {
                        "type": "string",
                        "description": "Communicative intent (natural language goal for the act)"
                    },
                    "context": {
                        "type": "object",
                        "description": "SemanticContext wire — lang, profile_id, optional injection",
                        "properties": {
                            "lang": { "type": "string", "default": "en", "description": "en | ta" },
                            "profile_id": { "type": "string", "default": "chair" },
                            "injection": {
                                "type": "string",
                                "description": "Test-only: no_back triggers W2 reject path"
                            }
                        }
                    },
                    "mass_conserved": { "type": "boolean", "default": true },
                    "mi_query_threshold": {
                        "type": "number",
                        "default": 1.0,
                        "description": "Recommend external signal query when mi_deficit exceeds this"
                    },
                    "mock_llm": {
                        "type": "object",
                        "description": "Offline mock frontier LLM (integration tests)",
                        "properties": {
                            "surface": { "type": "string", "description": "Override proposed surface lemma" },
                            "no_back_injection": { "type": "boolean", "description": "Force W2 no_back reject path" }
                        }
                    }
                },
                "required": ["intent"]
            }
        }),
        true,
    )
}

fn trust_pre_gate(tool_name: &str, args: &Value) -> Option<(Value, bool)> {
    crate::mcp_trust_gate::pre_gate_semantic_agent(tool_name, args)
}

/// Execute `propose_communicative_act` from MCP tool arguments.
#[must_use]
pub fn exec_propose_communicative_act(args: &Value) -> (Value, bool) {
    if let Some(refused) = trust_pre_gate("propose_communicative_act", args) {
        return refused;
    }
    let intent = match args.get("intent").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => {
            return (
                json!({
                    "agent_error": {
                        "schema_version": "agent_error.v1",
                        "code": "missing_intent",
                        "message": "propose_communicative_act requires non-empty intent",
                        "remediation": "Supply intent describing the communicative goal.",
                    }
                }),
                true,
            );
        }
    };

    let lang = args
        .get("context")
        .and_then(|c| c.get("lang"))
        .and_then(|v| v.as_str())
        .map(parse_lang_code)
        .unwrap_or(LangCode::En);

    let mass_conserved = args
        .get("mass_conserved")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let mi_query_threshold = args
        .get("mi_query_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    let mock = mock_llm_from_args(args);
    let no_back = mock.no_back_injection;
    let body = orchestrate_communicative_act(
        intent,
        lang,
        mass_conserved,
        mi_query_threshold,
        &mock,
        no_back,
    );
    let is_error = !body["gate_summary"]["admissible"]
        .as_bool()
        .unwrap_or(false);
    (body, is_error)
}

/// Execute `map_to_geometry` — cartridge map for chair EN/TA scope.
#[must_use]
pub fn exec_map_to_geometry(args: &Value) -> (Value, bool) {
    if let Some(refused) = trust_pre_gate("map_to_geometry", args) {
        return refused;
    }
    let surface = match args.get("surface").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => {
            return (
                json!({
                    "agent_error": {
                        "schema_version": "agent_error.v1",
                        "code": "missing_surface",
                        "message": "map_to_geometry requires non-empty surface",
                    }
                }),
                true,
            );
        }
    };
    let lang = args
        .get("lang")
        .and_then(|v| v.as_str())
        .map(parse_lang_code)
        .unwrap_or(LangCode::En);
    let form = chair_surface_form(lang, surface);
    let response = chair_cross_lang_semantic_response(lang, &form.lemma);
    let quotient_id = action_quotient_id_hex(&chair_compose_quotient_id(true, 1.0));
    (
        json!({
            "schema_version": "map_to_geometry.v1",
            "proposed": true,
            "surface": form.lemma,
            "lang": lang_code_wire(lang),
            "script": format!("{:?}", form.script),
            "semantic_response": semantic_response_enriched_wire(
                &response,
                lang,
                &form.lemma,
                "mcp-map-to-geometry",
            ),
            "quotient_id": quotient_id,
        }),
        false,
    )
}

/// Execute `refine_shape` — honest stub until HCOM-020 Kleisli wire.
#[must_use]
pub fn exec_refine_shape(args: &Value) -> (Value, bool) {
    if let Some(refused) = trust_pre_gate("refine_shape", args) {
        return refused;
    }
    let has_shape = args.get("shape").is_some();
    let feedback = args.get("feedback").and_then(|v| v.as_str()).unwrap_or("");
    if !has_shape || feedback.is_empty() {
        return (
            json!({
                "agent_error": {
                    "schema_version": "agent_error.v1",
                    "code": "missing_argument",
                    "message": "refine_shape requires shape and feedback",
                }
            }),
            true,
        );
    }
    (
        json!({
            "schema_version": "refine_shape.v1",
            "proposed": true,
            "status": "stub_honest",
            "feedback": feedback,
            "refined_shape": null,
            "message": "Kleisli dialogue refine not wired (HCOM-020 class)",
        }),
        false,
    )
}

/// Execute `get_audit_digest` — SHA-256 content address for decision_id.
#[must_use]
pub fn exec_get_audit_digest(args: &Value) -> (Value, bool) {
    if let Some(refused) = trust_pre_gate("get_audit_digest", args) {
        return refused;
    }
    let decision_id = match args.get("decision_id").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => {
            return (
                json!({
                    "agent_error": {
                        "schema_version": "agent_error.v1",
                        "code": "missing_decision_id",
                        "message": "get_audit_digest requires decision_id",
                    }
                }),
                true,
            );
        }
    };
    match get_audit_digest(decision_id) {
        Ok(digest) => (
            json!({
                "schema_version": "audit_digest.v1",
                "proposed": true,
                "decision_id": decision_id,
                "digest_hex": digest.as_str(),
                "digest_source": "fixture_log",
                "immutable_log_wired": true,
            }),
            false,
        ),
        Err(AuditDigestError::NotFound) => {
            let digest = format!("sha256:{}", sha256_hex(decision_id.as_bytes()));
            (
                json!({
                    "schema_version": "audit_digest.v1",
                    "proposed": true,
                    "decision_id": decision_id,
                    "digest_hex": digest,
                    "digest_source": "ephemeral_hash",
                    "immutable_log_wired": false,
                }),
                false,
            )
        }
        Err(AuditDigestError::Tampered {
            decision_id,
            stored_hex,
            computed_hex,
        }) => (
            json!({
                "agent_error": {
                    "schema_version": "agent_error.v1",
                    "code": "audit_tampered",
                    "message": format!("audit digest mismatch for {decision_id}"),
                    "stored_hex": stored_hex,
                    "computed_hex": computed_hex,
                }
            }),
            true,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_chair_en_passes_local_gate() {
        let llm = MockFrontierLlm::default();
        let body = orchestrate_communicative_act(
            "describe chair to human",
            LangCode::En,
            true,
            1.0,
            &llm,
            false,
        );
        assert_eq!(body["gate_summary"]["admissible"], json!(true));
        assert_eq!(body["schema_version"], json!(WIRE_SCHEMA));
        assert!(body["audit_digest"]
            .as_str()
            .unwrap_or("")
            .starts_with("sha256:"));
    }

    #[test]
    fn mock_ta_chair_passes_local_gate() {
        let llm = MockFrontierLlm::default();
        let body = orchestrate_communicative_act(
            "describe chair in Tamil",
            LangCode::Ta,
            true,
            1.0,
            &llm,
            false,
        );
        assert_eq!(body["gate_summary"]["admissible"], json!(true));
        assert_eq!(body["frontier_proposal"]["surface"], json!("நாற்காலி"));
    }

    #[test]
    fn no_back_injection_rejects_via_local_gate() {
        let llm = MockFrontierLlm {
            no_back_injection: true,
            ..Default::default()
        };
        let body = orchestrate_communicative_act(
            "describe stool",
            LangCode::En,
            true,
            1.0,
            &llm,
            true,
        );
        assert_eq!(body["gate_summary"]["admissible"], json!(false));
        assert_eq!(body["gate_summary"]["verdict"], json!("REJECT"));
        assert!(body["orchestration"]["external_signal_query_recommended"]
            .as_bool()
            .unwrap_or(false));
    }

    #[test]
    fn exec_rejects_missing_intent() {
        let (body, is_error) = exec_propose_communicative_act(&json!({
            "trust": { "scope": "device" }
        }));
        assert!(is_error);
        assert_eq!(
            body["agent_error"]["code"].as_str(),
            Some("missing_intent")
        );
    }

    #[test]
    fn hcom_agent_tool_schemas_emit_four() {
        assert_eq!(hcom_semantic_agent_tool_schemas().len(), 4);
        for tool in HCOM_SEMANTIC_AGENT_TOOLS {
            assert!(is_hcom_semantic_agent_tool(tool));
        }
    }

    #[test]
    fn map_to_geometry_chair_en() {
        let (body, err) = exec_map_to_geometry(&json!({"surface": "chair", "lang": "en"}));
        assert!(!err);
        assert_eq!(body["surface"], json!("chair"));
        assert!(body["quotient_id"].is_string());
    }

    #[test]
    fn get_audit_digest_deterministic() {
        let (a, _) = exec_get_audit_digest(&json!({"decision_id": "hcom-act:en:turn0"}));
        let (b, _) = exec_get_audit_digest(&json!({"decision_id": "hcom-act:en:turn0"}));
        assert_eq!(a["digest_hex"], b["digest_hex"]);
        assert_eq!(a["digest_source"], json!("ephemeral_hash"));
    }

    #[test]
    fn get_audit_digest_fixture_golden() {
        use umst_semantics::{FIXTURE_CHAIR_EN_PROPOSAL_ID, GOLDEN_CHAIR_EN_PROPOSAL_DIGEST_HEX};
        let (body, err) =
            exec_get_audit_digest(&json!({"decision_id": FIXTURE_CHAIR_EN_PROPOSAL_ID}));
        assert!(!err);
        assert_eq!(body["digest_hex"], json!(GOLDEN_CHAIR_EN_PROPOSAL_DIGEST_HEX));
        assert_eq!(body["digest_source"], json!("fixture_log"));
    }

    #[test]
    fn refine_shape_stub_honest_with_device_trust() {
        let (body, err) = exec_refine_shape(&json!({
            "shape": { "quotient_id": "chair" },
            "feedback": "add backrest",
            "trust": { "scope": "device" }
        }));
        assert!(!err);
        assert_eq!(body["status"], json!("stub_honest"));
    }

    #[test]
    fn hcom_tools_schema_bundle_covers_three() {
        let bundle = hcom_semantic_tools_schema_bundle();
        assert_eq!(bundle["schema_version"], json!(HCOM_TOOLS_SCHEMA_VERSION));
        assert_eq!(bundle["owner"], json!(HCOM_TOOLS_SLOT_OWNER));
        let tools = bundle["tools"].as_array().expect("tools");
        assert_eq!(tools.len(), 3);
        for raw in [
            MAP_TO_GEOMETRY_JSON,
            REFINE_SHAPE_JSON,
            GET_AUDIT_DIGEST_JSON,
            AUDIT_DIGEST_RESPONSE_JSON,
        ] {
            let _: Value = serde_json::from_str(raw).expect("valid embed");
        }
    }

    #[test]
    fn hcom_sim_propose_schema_bundle_owned_by_sim_slot() {
        let bundle = hcom_sim_propose_schema_bundle();
        assert_eq!(bundle["owner"], json!(HCOM_SIM_SLOT_OWNER));
        assert_eq!(bundle["tools_slot_owner"], json!(HCOM_TOOLS_SLOT_OWNER));
        let tools = bundle["tools"].as_array().expect("tools");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0], json!("propose_communicative_act"));
    }
}
