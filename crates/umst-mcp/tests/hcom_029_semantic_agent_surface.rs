// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//
// AGAP-2033-HCOM-029 — 4-tool semantic agent MCP surface + trust-aware refuse.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command as StdCmd, Stdio};
use umst_mcp::mcp_trust_gate::{check_semantic_agent_trust, MCP_TRUST_GATE_SCHEMA};
use umst_mcp::semantic_hcom::{
    exec_get_audit_digest, exec_map_to_geometry, exec_propose_communicative_act, exec_refine_shape,
    hcom_semantic_agent_tool_schemas, HCOM_SEMANTIC_AGENT_TOOLS,
};

fn mcp_binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_umst-mcp"))
}

fn with_device_trust(mut args: Value) -> Value {
    if let Some(obj) = args.as_object_mut() {
        obj.insert("trust".into(), json!({ "scope": "device" }));
    }
    args
}

fn read_json_line<R: BufRead>(reader: &mut R) -> Value {
    let mut buf = String::new();
    reader.read_line(&mut buf).expect("line");
    serde_json::from_str(buf.trim()).expect("json")
}

fn write_frame(stdin: &mut impl Write, frame: &Value) {
    writeln!(stdin, "{}", serde_json::to_string(frame).unwrap()).unwrap();
}

#[test]
fn hcom_029_four_tool_schemas_registered() {
    let schemas = hcom_semantic_agent_tool_schemas();
    assert_eq!(schemas.len(), 4);
    let names: Vec<_> = schemas
        .iter()
        .filter_map(|s| s.get("name").and_then(|n| n.as_str()))
        .collect();
    for tool in HCOM_SEMANTIC_AGENT_TOOLS {
        assert!(names.contains(&tool), "missing schema for {tool}");
    }
}

#[test]
fn hcom_029_trust_refuse_propose_without_device_scope() {
    let (body, is_error) = exec_propose_communicative_act(&json!({
        "intent": "describe chair"
    }));
    assert!(is_error);
    assert_eq!(body["agent_error"]["code"], json!("trust_refused"));
    assert_eq!(
        body["agent_error"]["trust_gate_schema"],
        json!(MCP_TRUST_GATE_SCHEMA)
    );
}

#[test]
fn hcom_029_propose_admits_with_device_trust() {
    let (body, is_error) = exec_propose_communicative_act(&with_device_trust(json!({
        "intent": "describe chair geometry to operator",
        "context": { "lang": "en" }
    })));
    assert!(!is_error);
    assert_eq!(body["gate_summary"]["admissible"], json!(true));
}

#[test]
fn hcom_029_map_to_geometry_ephemeral_ok() {
    let (body, is_error) = exec_map_to_geometry(&json!({
        "surface": "chair",
        "lang": "en"
    }));
    assert!(!is_error);
    assert_eq!(body["schema_version"], json!("map_to_geometry.v1"));
    assert!(body["quotient_id"].is_string());
}

#[test]
fn hcom_029_refine_shape_requires_device_trust() {
    let (refused, err) = exec_refine_shape(&json!({
        "shape": { "quotient_id": "chair:v0" },
        "feedback": "add armrests"
    }));
    assert!(err);
    assert_eq!(refused["agent_error"]["code"], json!("trust_refused"));

    let (body, is_error) = exec_refine_shape(&with_device_trust(json!({
        "shape": { "quotient_id": "chair:v0" },
        "feedback": "add armrests"
    })));
    assert!(!is_error);
    assert_eq!(body["status"], json!("stub_honest"));
}

#[test]
fn hcom_029_get_audit_digest_fixture_log() {
    let (body, _) = exec_get_audit_digest(&json!({ "decision_id": "hcom-chair-en-propose-0001" }));
    assert_eq!(body["digest_source"], json!("fixture_log"));
    assert!(body["immutable_log_wired"].as_bool().unwrap_or(false));
}

#[test]
fn hcom_029_trust_gate_unit_matches_tool_classes() {
    assert!(check_semantic_agent_trust("map_to_geometry", &json!({ "surface": "x" })).is_ok());
    assert!(
        check_semantic_agent_trust("propose_communicative_act", &json!({ "intent": "x" })).is_err()
    );
}

#[test]
fn hcom_029_stdio_tools_list_includes_quartet() {
    let exe = mcp_binary_path();
    assert!(exe.exists(), "missing umst-mcp at {}", exe.display());

    let mut child = StdCmd::new(&exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn");

    let stdin = child.stdin.as_mut().expect("stdin");
    write_frame(
        stdin,
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    );
    write_frame(
        stdin,
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    );

    let mut reader = BufReader::new(child.stdout.take().expect("stdout"));
    let _ = read_json_line(&mut reader);
    let list = read_json_line(&mut reader);
    let _ = child.kill();
    let _ = child.wait();

    let tools = list["result"]["tools"].as_array().expect("tools");
    let names: Vec<_> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();
    for tool in HCOM_SEMANTIC_AGENT_TOOLS {
        assert!(names.contains(&tool), "tools/list missing {tool}");
    }
}

#[test]
fn hcom_029_stdio_map_to_geometry_green() {
    let exe = mcp_binary_path();
    assert!(exe.exists());

    let mut child = StdCmd::new(&exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn");

    let stdin = child.stdin.as_mut().expect("stdin");
    write_frame(
        stdin,
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    );
    write_frame(
        stdin,
        &json!({
            "jsonrpc":"2.0",
            "id":29,
            "method":"tools/call",
            "params":{
                "name":"map_to_geometry",
                "arguments":{ "surface": "chair", "lang": "en" }
            }
        }),
    );

    let mut reader = BufReader::new(child.stdout.take().expect("stdout"));
    let _ = read_json_line(&mut reader);
    let resp = read_json_line(&mut reader);
    let _ = child.kill();
    let _ = child.wait();

    assert!(resp.get("error").is_none(), "stdio error: {resp}");
    assert_eq!(resp["result"]["isError"], json!(false));
    let text = resp["result"]["content"][0]["text"].as_str().expect("text");
    let body: Value = serde_json::from_str(text).expect("json body");
    assert_eq!(body["schema_version"], json!("map_to_geometry.v1"));
}
