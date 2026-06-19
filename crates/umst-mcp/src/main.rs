// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! MCP stdio JSON-RPC server — facade tools + optional Physical Reasoning Layer (`agent-layer`).

use std::io::{self, BufRead, Write};

use serde_json::{json, Value};
use umst_cli::canonical::canonical_json_bytes;
use umst_cli::{
    audit::audit_csv_buf,
    cli::{
        certify_profile_json, mix_spec_from_json_value, predict_with_options, serialize_prediction,
        MixSpec, PredictOptions, PredictionWireVersion,
    },
};
use umst_concrete_cartridge::calibration::{Profile, BUNDLED_PROFILE_IDS};

#[cfg(feature = "agent-layer")]
mod agent_layer;

#[cfg(feature = "agent-layer")]
use agent_layer::AgentSession;
#[cfg(feature = "agent-layer")]
use umst_concrete_cartridge::research::MemoryQuery;

fn err_frame(id: Value, msg: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": -32603, "message": msg.into() },
    })
}

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

fn base_tools() -> Vec<Value> {
    vec![
        json!({
            "name": "umst_predict",
            "description": "`result.v2` prediction JSON via `umst_concrete_cartridge::facade::predict_with_options`; optional `canonical` forces sorted-key deterministic bytes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mix": {
                        "type": "object",
                        "description": "MixSpec wire (`w_c`, `temperature_k`, optional fractions)."
                    },
                    "profile": { "type": "string", "default": "default" },
                    "compare_homogeneous": { "type": "boolean", "default": false },
                    "schema_version": { "type": "string", "enum": ["v1", "v2"], "default": "v2" },
                    "canonical": { "type": "boolean", "default": false, "description": "Emit canonical JSON bytes (UTF-8) as escaped string"}
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
    #[cfg(feature = "agent-layer")]
    let tools = {
        let mut tools = base_tools();
        tools.extend(agent_layer::agent_tools_schema());
        tools
    };
    #[cfg(not(feature = "agent-layer"))]
    let tools = base_tools();
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "tools": tools }
    })
}

fn tool_umst_predict(id: Value, args: &Value) -> Value {
    let profile_id = args
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let compare = args
        .get("compare_homogeneous")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    let canon = args
        .get("canonical")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    let schema_version = args.get("schema_version").and_then(|x| x.as_str());
    let wire = match schema_version.unwrap_or("v2") {
        "v1" | "V1" => PredictionWireVersion::V1,
        _ => PredictionWireVersion::V2,
    };

    let mix_v = args.get("mix").cloned().unwrap_or(json!({}));
    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => return err_frame(id, format!("profile load error: {e}")),
    };
    let mut spec: MixSpec = match mix_spec_from_json_value(mix_v.clone()) {
        Ok(s) => s,
        Err(e) => return err_frame(id, format!("mix parse error: {e}")),
    };
    spec.profile_name = profile_id.to_string();

    let bundle = match predict_with_options(
        &profile,
        &spec,
        PredictOptions {
            compare_homogeneous: compare,
        },
    ) {
        Ok(b) => b,
        Err(e) => return err_frame(id, format!("predict error: {e}")),
    };
    let out = match serialize_prediction(&bundle, wire) {
        Ok(v) => v,
        Err(e) => return err_frame(id, format!("serialize_prediction: {e}")),
    };

    if canon {
        let bytes = match canonical_json_bytes(&out) {
            Ok(b) => b,
            Err(e) => return err_frame(id, format!("canonical JSON: {e}")),
        };
        let escaped =
            serde_json::to_string(&String::from_utf8_lossy(&bytes).to_string()).unwrap_or_default();
        return text_result(id, escaped, false);
    }

    let pretty = serde_json::to_string_pretty(&out).unwrap_or_else(|_| "{}".to_string());
    text_result(id, pretty, false)
}

fn tool_umst_audit(id: Value, args: &Value) -> Value {
    let profile_id = args
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("uci_d1");
    let csv_text = match args.get("csv_text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return err_frame(id, String::from("missing csv_text")),
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
        Err(e) => return err_frame(id, format!("profile load error: {e}")),
    };

    let v = match audit_csv_buf(&profile, &csv_text, limit) {
        Ok(x) => x,
        Err(e) => return err_frame(id, format!("audit csv: {e}")),
    };

    if canon {
        let bytes = match canonical_json_bytes(&v) {
            Ok(b) => b,
            Err(e) => return err_frame(id, format!("canonical JSON: {e}")),
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
    let mut ids = BUNDLED_PROFILE_IDS.to_vec();
    ids.sort_unstable();

    let profs: Vec<Value> = ids
        .iter()
        .map(|pid| {
            let desc = umst_concrete_cartridge::calibration::profile_descriptions()
                .get(pid)
                .copied()
                .unwrap_or("no description");
            json!({ "id": pid, "description": desc })
        })
        .collect();

    text_result(
        id,
        serde_json::to_string_pretty(&profs).unwrap_or_default(),
        false,
    )
}

fn tool_umst_certify(id: Value, args: &Value) -> Value {
    let profile_id = match args.get("profile").and_then(|v| v.as_str()) {
        Some(x) => x,
        None => return err_frame(id, String::from("missing profile")),
    };
    let canon = args
        .get("canonical")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);

    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => return err_frame(id, format!("profile load error: {e}")),
    };
    let v = certify_profile_json(&profile);
    if canon {
        let bytes = match canonical_json_bytes(&v) {
            Ok(b) => b,
            Err(e) => return err_frame(id, format!("canonical JSON: {e}")),
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
        None => return err_frame(id, "missing mix"),
    };
    let explain = args.get("explain").and_then(|x| x.as_bool()).unwrap_or(false);
    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => return err_frame(id, format!("profile load error: {e}")),
    };
    let result = session.gate_check(&profile, &mix, explain);
    let is_error = !result.gate_summary.admissible;
    text_result(
        id,
        serde_json::to_string_pretty(&result).unwrap_or_default(),
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
            return (err_frame(id, "missing contribution"), session);
        }
    };
    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => {
            return (err_frame(id, format!("profile load error: {e}")), session);
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
        Err(e) => (err_frame(id, e), session),
    }
}

#[cfg(feature = "agent-layer")]
fn tool_umst_contribute_status(id: Value, args: &Value, session: &AgentSession) -> Value {
    let job_id = match args.get("job_id").and_then(|x| x.as_str()) {
        Some(j) => j,
        None => return err_frame(id, "missing job_id"),
    };
    match session.contribute_status(job_id) {
        Some(job) => text_result(
            id,
            serde_json::to_string_pretty(&job).unwrap_or_default(),
            false,
        ),
        None => err_frame(id, format!("unknown job_id: {job_id}")),
    }
}

#[cfg(feature = "agent-layer")]
fn tool_umst_mi_estimate(id: Value, args: &Value, session: &AgentSession) -> Value {
    let mix = match args.get("mix") {
        Some(m) => m.clone(),
        None => return err_frame(id, "missing mix"),
    };
    let out = session.mi_estimate(&mix);
    text_result(
        id,
        serde_json::to_string_pretty(&out).unwrap_or_default(),
        false,
    )
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
                Err(e) => (err_frame(id, e), session),
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
                Err(e) => (err_frame(id, e), session),
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
