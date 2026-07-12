// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Stage S7 proposed tools (P1–P4) — **default-off** features behind `gate-explain-v2`,
//! `tool-dry-run`, `tool-promote`, `tool-arena-session-unified`.
//!
//! ADDITIVE: with all S7 features disabled, the shipped 13-tool suite is unchanged.

use serde_json::{json, Value};
use std::convert::TryFrom;
#[cfg(all(feature = "agent-layer", feature = "tool-arena-session-unified"))]
use std::path::Path;
use umst_cli::cli::{predict_with_options, serialize_prediction, PredictOptions};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::facade::{MixSpec, PredictionWireVersion};
use umst_concrete_cartridge::research::{
    gate_check_mix_result, mix_wire_from_spec_value, synthetic_observed_at, GateCheckResult,
};

#[cfg(feature = "agent-layer")]
use crate::agent_layer::AgentSession;

const JSON_SCHEMA_2020: &str = "https://json-schema.org/draft/2020-12/schema";

fn with_schema_2020(mut tool: Value, read_only: bool) -> Value {
    tool["annotations"] = json!({
        "readOnlyHint": read_only,
        "destructiveHint": false,
    });
    if let Some(schema) = tool.get_mut("inputSchema").and_then(|s| s.as_object_mut()) {
        schema.insert("$schema".into(), json!(JSON_SCHEMA_2020));
    }
    tool
}

/// Patch `umst_gate_check` input schema when `gate-explain-v2` is enabled.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Optional wire extension; old clients ignore unknown fields.
#[cfg(feature = "gate-explain-v2")]
pub fn patch_gate_check_schema(tools: &mut [Value]) {
    for tool in tools.iter_mut() {
        if tool.get("name").and_then(|n| n.as_str()) != Some("umst_gate_check") {
            continue;
        }
        if let Some(props) = tool
            .get_mut("inputSchema")
            .and_then(|s| s.get_mut("properties"))
            .and_then(|p| p.as_object_mut())
        {
            props.insert(
                "explain_v2".into(),
                json!({
                    "type": "boolean",
                    "default": false,
                    "description": "When true (requires gate-explain-v2 build), attach explain_v2 enrichment block with violation graph and remediation pack ids"
                }),
            );
        }
        if let Some(desc) = tool.get_mut("description").and_then(|d| d.as_str()) {
            let extended = format!(
                "{desc} Optional explain_v2:true (gate-explain-v2 feature) adds structured violation_graph and remediation_pack_id on REJECT."
            );
            tool["description"] = json!(extended);
        }
        break;
    }
}

/// Additive `explain_v2` block for gate-check responses (P1).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Operator enrichment; admissibility unchanged on gate_summary.
#[cfg(feature = "gate-explain-v2")]
#[must_use]
pub fn build_explain_v2_enrichment(result: &GateCheckResult) -> Value {
    let codes = result
        .explain
        .as_ref()
        .map(|e| e.regime_violations.clone())
        .unwrap_or_default();
    let field_paths: Vec<String> = result
        .explain
        .as_ref()
        .map(|e| {
            let mut paths: Vec<String> = e.fields.iter().map(|f| f.path.clone()).collect();
            paths.sort();
            paths.dedup();
            paths
        })
        .unwrap_or_default();
    let violation_graph: Vec<Value> = codes
        .iter()
        .enumerate()
        .map(|(idx, code)| {
            json!({
                "code": code,
                "ordinal": idx,
                "remediation_pack_id": format!("remediation.{code}.v1"),
            })
        })
        .collect();
    json!({
        "schema_version": "gate_explain.v2",
        "proposed": true,
        "violation_graph": violation_graph,
        "remediation_pack_ids": codes.iter().map(|c| format!("remediation.{c}.v1")).collect::<Vec<_>>(),
        "field_paths_sorted": field_paths,
        "catalog_witnesses": result
            .explain
            .as_ref()
            .map(|e| e.catalog_witnesses.clone())
            .unwrap_or_default(),
    })
}

/// Merge gate-check library result with optional explain_v2 enrichment into MCP JSON.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Wire composition; physics on gate_summary only.
#[cfg(feature = "gate-explain-v2")]
#[must_use]
pub fn gate_check_wire_json(result: &GateCheckResult, explain_v2: bool) -> Value {
    let mut wire = serde_json::to_value(result).unwrap_or_else(|_| json!({}));
    if explain_v2 {
        if let Value::Object(ref mut map) = wire {
            map.insert(
                "explain_v2".into(),
                build_explain_v2_enrichment(result),
            );
        }
    }
    wire
}

/// MCP tool schemas for enabled S7 features (P2–P4).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Proposed tool discovery; not in S0 13-tool golden set.
#[cfg(feature = "agent-layer")]
#[must_use]
pub fn proposed_tool_schemas() -> Vec<Value> {
    let mut out = Vec::new();

    #[cfg(feature = "tool-dry-run")]
    {
        out.push(with_schema_2020(
            json!({
                "name": "umst_dry_run",
                "description": "Proposed read-only predict+gate evaluation without memory writes (tool-dry-run feature). Never persists gate rejects or contributions.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "mix": { "type": "object", "description": "mix_spec.v1 rational fields" },
                        "profile": { "type": "string", "default": "default" },
                        "explain": { "type": "boolean", "default": true }
                    },
                    "required": ["mix"]
                }
            }),
            true,
        ));
    }

    #[cfg(feature = "tool-promote")]
    {
        out.push(with_schema_2020(
            json!({
                "name": "umst_promote_contribution",
                "description": "Proposed human-gated promotion into shared corpus (tool-promote feature). Stub only — does not mutate memory or git inbox.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "proposal": {
                            "type": "object",
                            "description": "promotion_proposal.v1 — see umst://schemas/promotion_proposal.v1.json"
                        },
                        "human_capability_token": {
                            "type": "string",
                            "description": "Required when wired; maintainer-signed approval envelope"
                        }
                    },
                    "required": ["proposal"]
                }
            }),
            false,
        ));
    }

    #[cfg(feature = "tool-arena-session-unified")]
    {
        out.push(with_schema_2020(
            json!({
                "name": "umst_arena_session",
                "description": "Proposed unified arena session tool (tool-arena-session-unified feature). Wraps open/gate_check/close; umst_arena_open / umst_gate_check_arena / umst_arena_close remain canonical.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["open", "gate_check", "close"],
                            "description": "Session lifecycle action"
                        },
                        "arena_path": { "type": "string", "description": "Required for action=open" },
                        "arena_session_id": { "type": "string", "description": "Required for action=gate_check or close" },
                        "mix": { "type": "object", "description": "Required for action=gate_check" },
                        "profile": { "type": "string", "default": "default" },
                        "explain": { "type": "boolean", "default": true }
                    },
                    "required": ["action"]
                }
            }),
            false,
        ));
    }

    out
}

/// Side-effect-free predict+gate dry run (P2) — no memory or reject ledger writes.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Read-only proposal eval; not admissibility gate mutation.
#[cfg(all(feature = "agent-layer", feature = "tool-dry-run"))]
#[must_use]
pub fn exec_dry_run(session: &AgentSession, profile: &Profile, mix: &Value, explain: bool) -> Value {
    let observed = synthetic_observed_at(session.clock.sequence());
    let gate = gate_check_mix_result(profile, mix, explain, observed);

    let prediction = mix_wire_from_spec_value(mix)
        .and_then(|wire| MixSpec::try_from(wire).ok())
        .map(|mut spec| {
            spec.profile_name = profile.bundle_id.clone();
            predict_with_options(
                profile,
                &spec,
                PredictOptions {
                    compare_homogeneous: false,
                },
            )
            .ok()
            .and_then(|bundle| serialize_prediction(&bundle, PredictionWireVersion::V2).ok())
        })
        .flatten();

    json!({
        "schema_version": "dry_run.v1",
        "proposed": true,
        "dry_run": true,
        "writes_memory": false,
        "gate_summary": gate.gate_summary,
        "explain": gate.explain,
        "gate_reject": gate.gate_reject,
        "prediction": prediction,
    })
}

/// Promotion stub (P3) — structured not-wired response; never writes memory.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Human gate placeholder; federated inbox remains SSOT.
#[cfg(feature = "tool-promote")]
#[must_use]
pub fn exec_promote_contribution_stub(args: &Value) -> Value {
    let has_proposal = args.get("proposal").is_some();
    let has_token = args
        .get("human_capability_token")
        .and_then(|t| t.as_str())
        .is_some_and(|s| !s.is_empty());
    json!({
        "agent_error": {
            "schema_version": "agent_error.v1",
            "code": "promote_not_wired",
            "message": "umst_promote_contribution is proposed-only (tool-promote feature); human capability gate not wired",
            "remediation": "Export admissible rows via scripts/export_contributions_jsonl.py and open a federated inbox PR; see prompt export_for_git_inbox.",
        },
        "promotion_stub": {
            "schema_version": "promotion_stub.v1",
            "status": "proposed_not_wired",
            "proposal_received": has_proposal,
            "human_capability_token_present": has_token,
        }
    })
}

/// Unified arena session dispatch (P4) — wraps trio without removing them.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Session lifecycle composition; gate physics unchanged.
#[cfg(all(feature = "agent-layer", feature = "tool-arena-session-unified"))]
pub fn exec_arena_session(
    session: AgentSession,
    action: &str,
    args: &Value,
    profile: &Profile,
) -> Result<(AgentSession, Value), String> {
    match action {
        "open" => {
            let arena_path = args
                .get("arena_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "missing arena_path for action=open".to_string())?;
            let (next, id) = session.arena_open(Path::new(arena_path))?;
            Ok((
                next,
                json!({
                    "schema_version": "arena_session.v1",
                    "proposed": true,
                    "action": "open",
                    "arena_session_id": id.to_string(),
                    "arena_path": arena_path,
                }),
            ))
        }
        "gate_check" => {
            let arena_session_id = args
                .get("arena_session_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "missing arena_session_id for action=gate_check".to_string())?;
            let mix = args
                .get("mix")
                .cloned()
                .ok_or_else(|| "missing mix for action=gate_check".to_string())?;
            let explain = args
                .get("explain")
                .and_then(|x| x.as_bool())
                .unwrap_or(true);
            let session_uuid = uuid::Uuid::parse_str(arena_session_id)
                .map_err(|e| format!("invalid arena_session_id: {e}"))?;
            let result = session.gate_check_arena(profile, session_uuid, &mix, explain)?;
            Ok((
                session,
                json!({
                    "schema_version": "arena_session.v1",
                    "proposed": true,
                    "action": "gate_check",
                    "gate_result": result,
                }),
            ))
        }
        "close" => {
            let arena_session_id = args
                .get("arena_session_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "missing arena_session_id for action=close".to_string())?;
            let session_uuid = uuid::Uuid::parse_str(arena_session_id)
                .map_err(|e| format!("invalid arena_session_id: {e}"))?;
            let next = session.arena_close(session_uuid)?;
            Ok((
                next,
                json!({
                    "schema_version": "arena_session.v1",
                    "proposed": true,
                    "action": "close",
                    "closed": arena_session_id,
                }),
            ))
        }
        other => Err(format!("unknown arena_session action: {other}")),
    }
}
