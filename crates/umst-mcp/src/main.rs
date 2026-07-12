// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! MCP stdio JSON-RPC server — facade tools + optional Physical Reasoning Layer (`agent-layer`).
//!
//! **Performance:** stdio MCP is the stable default for external agents. For batch gate loops
//! or optimization sweeps, prefer the in-process arena path (`umst_arena_open` →
//! `umst_gate_check_arena`) or cartridge library (`gate_check_mix`) — see
//! `docs/AGENT_MCP.md` and `umst-manifold/docs/benchmarks/arena_vs_mcp.md` (CI ≥5× MCP).

use std::io::{self, BufRead, Write};

use serde_json::{json, Value};
use umst_cli::canonical::canonical_json_bytes;
use umst_cli::{audit::audit_csv_buf, cli::certify_profile_json};
use umst_concrete_cartridge::calibration::Profile;

#[cfg(feature = "agent-layer")]
use umst_concrete_cartridge::research::ContributeError;
#[cfg(feature = "agent-layer")]
use umst_concrete_cartridge::research::MemoryQuery;
#[cfg(feature = "agent-layer")]
use umst_mcp::agent_layer::{self, AgentSession};

fn text_result(id: Value, text: String, is_error: bool) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [{ "type": "text", "text": text }],
            "isError": is_error,
        },
    })
}

/// Recoverable tool failures: structured `agent_error.v1` + `isError: true` (not JSON-RPC `-32603`).
fn agent_tool_error(id: Value, code: &str, message: impl Into<String>, remediation: &str) -> Value {
    let body = json!({
        "agent_error": {
            "schema_version": "agent_error.v1",
            "code": code,
            "message": message.into(),
            "remediation": remediation,
        }
    });
    text_result(
        id,
        serde_json::to_string_pretty(&body).unwrap_or_default(),
        true,
    )
}

#[cfg(feature = "agent-layer")]
fn contribute_tool_error(id: Value, err: ContributeError) -> Value {
    let (code, remediation) = match &err {
        ContributeError::Validation(_) => (
            "contribute_validation_fail",
            "Fix contribution wire against umst://schemas/contribution.v1.json; use rational strings.",
        ),
        ContributeError::GateReject(_) => (
            "contribute_gate_reject",
            "Run umst_gate_check first; gate_summary.admissible must be true before contribute.",
        ),
        ContributeError::Scope(_) => (
            "contribute_scope_fail",
            "Supply a valid scope_token when UMST_AGENT_SCOPE_TOKENS is set.",
        ),
        ContributeError::NonMonotonicStamp => (
            "contribute_non_monotonic_stamp",
            "Use server-assigned observed_at stamps; do not regress session clock.",
        ),
        ContributeError::Store(_) => (
            "contribute_store_fail",
            "Check UMST_MEMORY_DB path and permissions; duplicate content_id may be idempotent success.",
        ),
    };
    agent_tool_error(id, code, err.to_string(), remediation)
}

#[cfg(feature = "agent-layer")]
fn transition_tool_error(id: Value, msg: String) -> Value {
    let (code, remediation) = if msg.contains("gate reject") {
        (
            "transition_gate_reject",
            "Run umst_gate_check first; gate_summary.admissible must be true before transition propose.",
        )
    } else if msg.contains("parse") {
        (
            "mix_parse_fail",
            "Use rational strings like \"9/20\" for w_c and temperature_k; see contribution.v1 schema.",
        )
    } else {
        (
            "transition_propose_fail",
            "Verify mix fields and profile calibration; see umst_transition_propose schema.",
        )
    };
    agent_tool_error(id, code, msg, remediation)
}

fn base_tools() -> Vec<Value> {
    vec![
        json!({
            "name": "umst_predict",
            "description": "Constitutive prediction envelope result.v2 (read-only). Optional step after umst_gate_check in safe exploration; does not write memory. Example input: {\"mix\":{\"w_c\":\"9/20\",\"temperature_k\":\"29315/100\",\"aggregate_volume_fraction\":\"7/10\"},\"profile\":\"default\"}. Example output: {\"schema_version\":\"result.v2\",\"compressive_strength_mpa\":53.8,\"degree_of_hydration\":0.91,\"calibration_profile\":\"default\",\"formal_anchor\":\"lean://umst-formal/Lean/Concrete/Powers.lean#powers_monotone\",\"physics_pipeline\":{...}}. Prefer rational strings for mix fields; schema_version v2 default.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mix": {
                        "type": "object",
                        "description": "mix_spec wire — e.g. {\"w_c\":\"9/20\",\"temperature_k\":\"29315/100\",\"aggregate_volume_fraction\":\"7/10\"}; rational strings preferred"
                    },
                    "profile": { "type": "string", "default": "default", "description": "Bundled calibration profile id (call umst_profiles when unsure)" },
                    "compare_homogeneous": { "type": "boolean", "default": false, "description": "When true, include homogeneous baseline comparison in the bundle" },
                    "schema_version": { "type": "string", "enum": ["v1", "v2"], "default": "v2", "description": "Wire format tag — prefer v2 (result.v2); v1 is deprecated" },
                    "canonical": { "type": "boolean", "default": false, "description": "When true, emit sorted-key canonical JSON bytes as an escaped UTF-8 string"}
                },
                "required": ["mix"]
            }
        }),
        json!({
            "name": "umst_audit",
            "description": "Batch CSV audit envelope `audit.v1` (dataset_d1-compatible headers).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "profile": { "type": "string", "default": "uci_d1" },
                    "csv_text": {
                        "type": "string",
                        "description": "Full CSV text including header row"
                    },
                    "limit": { "type": "integer", "minimum": 0 },
                    "canonical": { "type": "boolean", "default": false }
                },
                "required": ["csv_text"]
            }
        }),
        json!({
            "name": "umst_profiles",
            "description": "List bundled calibration profile ids sorted lexicographically with descriptions.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "umst_certify",
            "description": "Emit certify chain JSON (`CertifyChain`) for a bundled profile.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "profile": { "type": "string" },
                    "canonical": { "type": "boolean", "default": false }
                },
                "required": ["profile"]
            }
        }),
    ]
}

fn dispatch_tools_list(id: Value) -> Value {
    #[cfg(feature = "tool-manifest")]
    let tools = {
        if umst_mcp::manifest::manifest_env_enabled() {
            umst_mcp::manifest::tools_v1::mcp_tools_schema()
        } else {
            hand_tools_list()
        }
    };
    #[cfg(not(feature = "tool-manifest"))]
    let tools = hand_tools_list();
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "tools": tools }
    })
}

fn hand_tools_list() -> Vec<Value> {
    #[cfg(feature = "agent-layer")]
    {
        let mut tools = base_tools();
        tools.extend(agent_layer::agent_tools_schema());
        tools
    }
    #[cfg(not(feature = "agent-layer"))]
    {
        base_tools()
    }
}

fn tool_umst_predict(id: Value, args: &Value) -> Value {
    let p = umst_mcp::handlers::exec_umst_predict(args);
    text_result(id, p.text, p.is_error)
}

fn tool_umst_audit(id: Value, args: &Value) -> Value {
    let profile_id = args
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("uci_d1");
    let csv_text = match args.get("csv_text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => {
            return agent_tool_error(
                id,
                "audit_missing_csv",
                "missing csv_text",
                "Supply csv_text with header row and mix columns; see umst_audit schema.",
            )
        }
    };
    let limit = args.get("limit").and_then(|v| {
        let n = v.as_u64()? as usize;
        Some(n)
    });
    let canon = args
        .get("canonical")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);

    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => {
            return agent_tool_error(
                id,
                "profile_load_fail",
                format!("profile load error: {e}"),
                "Call umst_profiles for bundled ids (e.g. default, uci_d1) or fix the profile argument.",
            )
        }
    };

    let v = match audit_csv_buf(&profile, &csv_text, limit) {
        Ok(x) => x,
        Err(e) => {
            return agent_tool_error(
                id,
                "audit_fail",
                format!("audit csv: {e}"),
                "Verify CSV headers and rational mix fields; see umst_audit schema.",
            )
        }
    };

    if canon {
        let bytes = match canonical_json_bytes(&v) {
            Ok(b) => b,
            Err(e) => {
                return agent_tool_error(
                    id,
                    "canonical_json_fail",
                    format!("canonical JSON: {e}"),
                    "Retry without canonical=true or fix audit output shape.",
                )
            }
        };
        let escaped =
            serde_json::to_string(&String::from_utf8_lossy(&bytes).to_string()).unwrap_or_default();
        return text_result(id, escaped, false);
    }
    text_result(
        id,
        serde_json::to_string_pretty(&v).unwrap_or_default(),
        false,
    )
}

fn tool_umst_profiles(id: Value) -> Value {
    let p = umst_mcp::handlers::exec_umst_profiles();
    text_result(id, p.text, p.is_error)
}

fn tool_umst_certify(id: Value, args: &Value) -> Value {
    let profile_id = match args.get("profile").and_then(|v| v.as_str()) {
        Some(x) => x,
        None => {
            return agent_tool_error(
                id,
                "certify_missing_profile",
                "missing profile",
                "Supply profile id from umst_profiles (e.g. default, uci_d1).",
            )
        }
    };
    let canon = args
        .get("canonical")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);

    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => {
            return agent_tool_error(
                id,
                "profile_load_fail",
                format!("profile load error: {e}"),
                "Call umst_profiles for bundled ids (e.g. default, uci_d1) or fix the profile argument.",
            )
        }
    };
    let v = certify_profile_json(&profile);
    if canon {
        let bytes = match canonical_json_bytes(&v) {
            Ok(b) => b,
            Err(e) => {
                return agent_tool_error(
                    id,
                    "canonical_json_fail",
                    format!("canonical JSON: {e}"),
                    "Retry without canonical=true or fix certify output shape.",
                )
            }
        };
        let escaped =
            serde_json::to_string(&String::from_utf8_lossy(&bytes).to_string()).unwrap_or_default();
        return text_result(id, escaped, false);
    }
    text_result(
        id,
        serde_json::to_string_pretty(&v).unwrap_or_default(),
        false,
    )
}

#[cfg(feature = "agent-layer")]
fn parse_memory_query(args: &Value) -> MemoryQuery {
    MemoryQuery {
        admissible_only: args
            .get("admissible_only")
            .and_then(|x| x.as_bool())
            .unwrap_or(true),
        curing_regime: args
            .get("curing_regime")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        limit: args
            .get("limit")
            .and_then(|x| x.as_u64())
            .map(|n| n as usize),
        cursor: args
            .get("cursor")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        catalog_id: args
            .get("catalog_id")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        stamp_tier: args
            .get("stamp_tier")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        outcome_source: args
            .get("outcome_source")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        wall_ms_min: args.get("wall_ms_min").and_then(|x| x.as_u64()),
        wall_ms_max: args.get("wall_ms_max").and_then(|x| x.as_u64()),
        near_mix_spec: args.get("near_mix_spec").cloned(),
        max_mix_l1: args.get("max_mix_l1").and_then(|x| x.as_f64()),
        hilbert_index: args
            .get("hilbert_index")
            .and_then(|x| x.as_u64())
            .map(|n| n as u32),
        max_hilbert_distance: args
            .get("max_hilbert_distance")
            .and_then(|x| x.as_u64())
            .map(|n| n as u32),
    }
}

#[cfg(feature = "agent-layer")]
fn tool_umst_gate_check(id: Value, args: &Value, session: &AgentSession) -> Value {
    let profile_id = args
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let mix = match args.get("mix") {
        Some(m) => m.clone(),
        None => {
            return agent_tool_error(
                id,
                "missing_argument",
                "missing mix",
                "Supply mix with rational fields (w_c, temperature_k, …) per contribution.v1.",
            )
        }
    };
    let explain = args
        .get("explain")
        .and_then(|x| x.as_bool())
        .unwrap_or(true);
    #[cfg(feature = "gate-explain-v2")]
    let explain_v2 = args
        .get("explain_v2")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => {
            return agent_tool_error(
                id,
                "profile_load_fail",
                format!("profile load error: {e}"),
                "Call umst_profiles for bundled ids or use profile: \"default\".",
            )
        }
    };
    let result = session.gate_check(&profile, &mix, explain);
    let is_error = !result.gate_summary.admissible;
    #[cfg(feature = "gate-explain-v2")]
    let body = umst_mcp::proposed_tools::gate_check_wire_json(&result, explain_v2);
    #[cfg(not(feature = "gate-explain-v2"))]
    let body = serde_json::to_value(&result).unwrap_or_else(|_| json!({}));
    text_result(
        id,
        serde_json::to_string_pretty(&body).unwrap_or_default(),
        is_error,
    )
}

#[cfg(feature = "agent-layer")]
fn tool_umst_contribute(id: Value, args: &Value, session: AgentSession) -> (Value, AgentSession) {
    let profile_id = args
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let contribution = match args.get("contribution") {
        Some(c) => c.clone(),
        None => {
            return (
                agent_tool_error(
                    id,
                    "missing_argument",
                    "missing contribution",
                    "Supply contribution object matching contribution.v1 schema.",
                ),
                session,
            );
        }
    };
    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => {
            return (
                agent_tool_error(
                    id,
                    "profile_load_fail",
                    format!("profile load error: {e}"),
                    "Call umst_profiles for bundled ids or use profile: \"default\".",
                ),
                session,
            );
        }
    };
    let async_mode = args.get("async").and_then(|x| x.as_bool()).unwrap_or(false);
    if async_mode {
        let (next, job_id) = session.contribute_async(&profile, &contribution);
        return (
            text_result(
                id,
                serde_json::to_string_pretty(&json!({ "job_id": job_id })).unwrap_or_default(),
                false,
            ),
            next,
        );
    }
    match session.clone().contribute(&profile, &contribution) {
        Ok((next, result)) => (
            text_result(
                id,
                serde_json::to_string_pretty(&result).unwrap_or_default(),
                false,
            ),
            next,
        ),
        Err(e) => (contribute_tool_error(id, e), session),
    }
}

#[cfg(feature = "agent-layer")]
fn tool_umst_contribute_status(id: Value, args: &Value, session: &AgentSession) -> Value {
    let job_id = match args.get("job_id").and_then(|x| x.as_str()) {
        Some(j) => j,
        None => {
            return agent_tool_error(
                id,
                "missing_argument",
                "missing job_id",
                "Poll umst_contribute_status with job_id returned from umst_contribute async:true.",
            );
        }
    };
    match session.contribute_status(job_id) {
        Some(job) => text_result(
            id,
            serde_json::to_string_pretty(&job).unwrap_or_default(),
            false,
        ),
        None => agent_tool_error(
            id,
            "unknown_job_id",
            format!("unknown job_id: {job_id}"),
            "Re-submit contribute or check contribute_jobs.json beside UMST_MEMORY_DB; job may have expired after MCP restart.",
        ),
    }
}

#[cfg(feature = "agent-layer")]
fn tool_umst_mi_estimate(id: Value, args: &Value, session: &AgentSession) -> Value {
    let p = umst_mcp::handlers::exec_umst_mi_estimate(args, session);
    text_result(id, p.text, p.is_error)
}

#[cfg(feature = "agent-layer")]
fn tool_umst_memory_query(id: Value, args: &Value, session: &AgentSession) -> Value {
    let q = parse_memory_query(args);
    let page = session.memory_query(&q);
    text_result(
        id,
        serde_json::to_string_pretty(&page).unwrap_or_default(),
        false,
    )
}

#[cfg(feature = "agent-layer")]
fn tool_umst_transition_propose(
    id: Value,
    args: &Value,
    session: AgentSession,
) -> (Value, AgentSession) {
    let profile_id = args
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let mix = match args.get("mix") {
        Some(m) => m.clone(),
        None => {
            return (
                agent_tool_error(
                    id,
                    "missing_argument",
                    "missing mix",
                    "Supply mix with rational fields (w_c, temperature_k, …).",
                ),
                session,
            );
        }
    };
    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => {
            return (
                agent_tool_error(
                    id,
                    "profile_load_fail",
                    format!("profile load error: {e}"),
                    "Call umst_profiles for bundled ids or use profile: \"default\".",
                ),
                session,
            );
        }
    };
    let outcome = args.get("outcome");
    let process = args.get("process");
    match session
        .clone()
        .transition_propose(&profile, &mix, outcome, process)
    {
        Ok((next, body)) => (
            text_result(
                id,
                serde_json::to_string_pretty(&body).unwrap_or_default(),
                false,
            ),
            next,
        ),
        Err(e) => (transition_tool_error(id, e), session),
    }
}

#[cfg(feature = "agent-layer")]
fn tool_umst_arena_open(id: Value, args: &Value, session: AgentSession) -> (Value, AgentSession) {
    let arena_path = match args.get("arena_path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => {
            return (
                agent_tool_error(
                    id,
                    "missing_argument",
                    "missing arena_path",
                    "Supply arena_path to a versioned ABI v1 arena file.",
                ),
                session,
            );
        }
    };
    match session.clone().arena_open(std::path::Path::new(arena_path)) {
        Ok((next, arena_session_id)) => (
            text_result(
                id,
                serde_json::to_string_pretty(&json!({
                    "arena_session_id": arena_session_id.to_string(),
                    "arena_path": arena_path,
                }))
                .unwrap_or_default(),
                false,
            ),
            next,
        ),
        Err(e) => (
            agent_tool_error(
                id,
                "arena_open_fail",
                e,
                "Verify arena file exists and matches ABI v1 header layout.",
            ),
            session,
        ),
    }
}

#[cfg(feature = "agent-layer")]
fn tool_umst_gate_check_arena(id: Value, args: &Value, session: &AgentSession) -> Value {
    let profile_id = args
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let arena_session_id = match args.get("arena_session_id").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            return agent_tool_error(
                id,
                "missing_argument",
                "missing arena_session_id",
                "Call umst_arena_open first; pass returned arena_session_id.",
            );
        }
    };
    let session_uuid = match uuid::Uuid::parse_str(arena_session_id) {
        Ok(u) => u,
        Err(e) => {
            return agent_tool_error(
                id,
                "invalid_arena_session_id",
                format!("invalid arena_session_id: {e}"),
                "Use the UUID string returned from umst_arena_open.",
            );
        }
    };
    let mix = match args.get("mix") {
        Some(m) => m.clone(),
        None => {
            return agent_tool_error(
                id,
                "missing_argument",
                "missing mix",
                "Supply mix with rational fields per contribution.v1.",
            );
        }
    };
    let explain = args
        .get("explain")
        .and_then(|x| x.as_bool())
        .unwrap_or(true);
    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => {
            return agent_tool_error(
                id,
                "profile_load_fail",
                format!("profile load error: {e}"),
                "Call umst_profiles for bundled ids or use profile: \"default\".",
            );
        }
    };
    match session.gate_check_arena(&profile, session_uuid, &mix, explain) {
        Ok(result) => {
            let is_error = !result.gate_summary.admissible;
            text_result(
                id,
                serde_json::to_string_pretty(&result).unwrap_or_default(),
                is_error,
            )
        }
        Err(e) => agent_tool_error(
            id,
            "arena_gate_check_fail",
            e,
            "Ensure arena_session_id is open; re-open with umst_arena_open if MCP restarted.",
        ),
    }
}

#[cfg(feature = "agent-layer")]
fn tool_umst_arena_close(id: Value, args: &Value, session: AgentSession) -> (Value, AgentSession) {
    let arena_session_id = match args.get("arena_session_id").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            return (
                agent_tool_error(
                    id,
                    "missing_argument",
                    "missing arena_session_id",
                    "Pass arena_session_id from umst_arena_open.",
                ),
                session,
            );
        }
    };
    let session_uuid = match uuid::Uuid::parse_str(arena_session_id) {
        Ok(u) => u,
        Err(e) => {
            return (
                agent_tool_error(
                    id,
                    "invalid_arena_session_id",
                    format!("invalid arena_session_id: {e}"),
                    "Use the UUID string returned from umst_arena_open.",
                ),
                session,
            );
        }
    };
    match session.clone().arena_close(session_uuid) {
        Ok(next) => (
            text_result(
                id,
                serde_json::to_string_pretty(&json!({
                    "closed": arena_session_id,
                }))
                .unwrap_or_default(),
                false,
            ),
            next,
        ),
        Err(e) => (
            agent_tool_error(
                id,
                "arena_close_fail",
                e,
                "Session may have expired after MCP restart; call umst_arena_open again.",
            ),
            session,
        ),
    }
}

#[cfg(all(feature = "agent-layer", feature = "tool-dry-run"))]
fn tool_umst_dry_run(id: Value, args: &Value, session: &AgentSession) -> Value {
    let profile_id = args
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let mix = match args.get("mix") {
        Some(m) => m.clone(),
        None => {
            return agent_tool_error(
                id,
                "missing_argument",
                "missing mix",
                "Supply mix with rational fields for dry-run predict+gate.",
            )
        }
    };
    let explain = args
        .get("explain")
        .and_then(|x| x.as_bool())
        .unwrap_or(true);
    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => {
            return agent_tool_error(
                id,
                "profile_load_fail",
                format!("profile load error: {e}"),
                "Call umst_profiles for bundled ids or use profile: \"default\".",
            )
        }
    };
    let out = umst_mcp::proposed_tools::exec_dry_run(session, &profile, &mix, explain);
    let is_error = out
        .get("gate_summary")
        .and_then(|g| g.get("admissible"))
        .and_then(|a| a.as_bool())
        == Some(false);
    text_result(
        id,
        serde_json::to_string_pretty(&out).unwrap_or_default(),
        is_error,
    )
}

#[cfg(feature = "tool-promote")]
fn tool_umst_promote_contribution(id: Value, args: &Value) -> Value {
    let out = umst_mcp::proposed_tools::exec_promote_contribution_stub(args);
    text_result(
        id,
        serde_json::to_string_pretty(&out).unwrap_or_default(),
        true,
    )
}

#[cfg(all(feature = "agent-layer", feature = "tool-arena-session-unified"))]
fn tool_umst_arena_session(id: Value, args: &Value, session: AgentSession) -> (Value, AgentSession) {
    let action = match args.get("action").and_then(|v| v.as_str()) {
        Some(a) => a,
        None => {
            return (
                agent_tool_error(
                    id,
                    "missing_argument",
                    "missing action",
                    "Supply action: open | gate_check | close.",
                ),
                session,
            );
        }
    };
    let profile_id = args
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => {
            return (
                agent_tool_error(
                    id,
                    "profile_load_fail",
                    format!("profile load error: {e}"),
                    "Call umst_profiles for bundled ids or use profile: \"default\".",
                ),
                session,
            );
        }
    };
    match umst_mcp::proposed_tools::exec_arena_session(session, action, args, &profile) {
        Ok((next, body)) => {
            let is_error = action == "gate_check"
                && body
                    .get("gate_result")
                    .and_then(|g| g.get("gate_summary"))
                    .and_then(|s| s.get("admissible"))
                    .and_then(|a| a.as_bool())
                    == Some(false);
            (
                text_result(
                    id,
                    serde_json::to_string_pretty(&body).unwrap_or_default(),
                    is_error,
                ),
                next,
            )
        }
        Err(e) => (
            agent_tool_error(
                id,
                "arena_session_fail",
                e,
                "Verify action and required fields; canonical trio tools remain umst_arena_open / umst_gate_check_arena / umst_arena_close.",
            ),
            session,
        ),
    }
}

#[cfg(feature = "agent-layer")]
fn handle_tools_call(
    id: Value,
    params: Option<&Value>,
    session: AgentSession,
) -> (Value, AgentSession) {
    let name = params
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");

    let args = params
        .and_then(|p| p.get("arguments"))
        .cloned()
        .unwrap_or_else(|| json!({}));

    match name {
        "umst_predict" => (tool_umst_predict(id, &args), session),
        "umst_audit" => (tool_umst_audit(id, &args), session),
        "umst_profiles" => (tool_umst_profiles(id), session),
        "umst_certify" => (tool_umst_certify(id, &args), session),
        "umst_gate_check" => (tool_umst_gate_check(id, &args, &session), session),
        "umst_contribute" => tool_umst_contribute(id, &args, session),
        "umst_contribute_status" => (tool_umst_contribute_status(id, &args, &session), session),
        "umst_memory_query" => (tool_umst_memory_query(id, &args, &session), session),
        "umst_mi_estimate" => (tool_umst_mi_estimate(id, &args, &session), session),
        "umst_transition_propose" => tool_umst_transition_propose(id, &args, session),
        "umst_arena_open" => tool_umst_arena_open(id, &args, session),
        "umst_gate_check_arena" => (tool_umst_gate_check_arena(id, &args, &session), session),
        "umst_arena_close" => tool_umst_arena_close(id, &args, session),
        #[cfg(feature = "tool-dry-run")]
        "umst_dry_run" => (tool_umst_dry_run(id, &args, &session), session),
        #[cfg(feature = "tool-promote")]
        "umst_promote_contribution" => (tool_umst_promote_contribution(id, &args), session),
        #[cfg(feature = "tool-arena-session-unified")]
        "umst_arena_session" => tool_umst_arena_session(id, &args, session),
        other => (
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("Unknown tool: {other}") },
            }),
            session,
        ),
    }
}

#[cfg(not(feature = "agent-layer"))]
fn handle_tools_call(id: Value, params: Option<&Value>) -> Value {
    let name = params
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");

    let args = params
        .and_then(|p| p.get("arguments"))
        .cloned()
        .unwrap_or_else(|| json!({}));

    match name {
        "umst_predict" => tool_umst_predict(id, &args),
        "umst_audit" => tool_umst_audit(id, &args),
        "umst_profiles" => tool_umst_profiles(id),
        "umst_certify" => tool_umst_certify(id, &args),
        other => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("Unknown tool: {other}") },
        }),
    }
}

#[cfg(feature = "agent-layer")]
fn dispatch(req: &Value, session: AgentSession) -> (Value, AgentSession) {
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

    match method {
        "initialize" => (
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {}, "resources": {}, "prompts": {} },
                    "serverInfo": {
                        "name": "umst-concrete-cartridge",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                },
            }),
            session,
        ),
        "tools/list" => (dispatch_tools_list(id), session),
        "tools/call" => handle_tools_call(id, req.get("params"), session),
        "resources/list" => (
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": agent_layer::resources_list_result(),
            }),
            session,
        ),
        "resources/read" => {
            let uri = req
                .get("params")
                .and_then(|p| p.get("uri"))
                .and_then(|u| u.as_str())
                .unwrap_or("");
            match agent_layer::resources_read_result(uri) {
                Ok(result) => (
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": result,
                    }),
                    session,
                ),
                Err(e) => (
                    agent_tool_error(
                        id,
                        "resource_read_fail",
                        e,
                        "Call resources/list for valid umst:// URIs; read schema resources before contribute.",
                    ),
                    session,
                ),
            }
        }
        "prompts/list" => (
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": agent_layer::prompts_list_result(),
            }),
            session,
        ),
        "prompts/get" => {
            let name = req
                .get("params")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");
            match agent_layer::prompts_get_result(name) {
                Ok(result) => (
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": result,
                    }),
                    session,
                ),
                Err(e) => (
                    agent_tool_error(
                        id,
                        "prompt_not_found",
                        e,
                        "Call prompts/list for shipped prompt names (e.g. safe-exploration, interpret_gate_failure).",
                    ),
                    session,
                ),
            }
        }
        _ => (
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("Method not found: {method}") },
            }),
            session,
        ),
    }
}

#[cfg(not(feature = "agent-layer"))]
fn dispatch(req: &Value) -> Value {
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

    match method {
        "initialize" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "umst-concrete-cartridge",
                    "version": env!("CARGO_PKG_VERSION"),
                },
            },
        }),
        "tools/list" => dispatch_tools_list(id),
        "tools/call" => handle_tools_call(id, req.get("params")),
        _ => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("Method not found: {method}") },
        }),
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("UMST MCP server (stdio).");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    #[cfg(feature = "agent-layer")]
    let mut session = AgentSession::default();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("stdin read failed: {e}");
                continue;
            }
        };
        if line.trim().is_empty() {
            continue;
        }

        #[cfg(feature = "agent-layer")]
        let response = match serde_json::from_str::<Value>(&line) {
            Ok(req) => {
                let (frame, next) = dispatch(&req, session);
                session = next;
                frame
            }
            Err(e) => {
                tracing::error!("Bad JSON-RPC frame: {e}");
                continue;
            }
        };

        #[cfg(not(feature = "agent-layer"))]
        let response = match serde_json::from_str::<Value>(&line) {
            Ok(req) => dispatch(&req),
            Err(e) => {
                tracing::error!("Bad JSON-RPC frame: {e}");
                continue;
            }
        };

        let out = match serde_json::to_string(&response) {
            Ok(s) => s + "\n",
            Err(_) => "{}".to_string(),
        };

        let _ = stdout.write_all(out.as_bytes());
        let _ = stdout.flush();
    }
}
