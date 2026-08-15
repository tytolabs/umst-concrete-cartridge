// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
//
// AGAP-2046-HCOM-029-TOOLS — map_to_geometry / refine_shape / get_audit_digest MCP surface.
// Deconflict: propose_communicative_act smoke owned by AGAP-2046-HCOM-029-SIM.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command as StdCmd, Stdio};
use umst_mcp::semantic_hcom::{
    exec_get_audit_digest, exec_map_to_geometry, exec_refine_shape, get_audit_digest_tool_schema,
    hcom_semantic_tools_schema_bundle, map_to_geometry_tool_schema, refine_shape_tool_schema,
    HCOM_TOOLS_SCHEMA_VERSION, HCOM_TOOLS_SLOT_OWNER,
};
use umst_semantics::{FIXTURE_CHAIR_EN_PROPOSAL_ID, GOLDEN_CHAIR_EN_PROPOSAL_DIGEST_HEX};

const TOOLS: &[&str] = &["map_to_geometry", "refine_shape", "get_audit_digest"];

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

fn stdio_tool_call(tool_name: &str, arguments: &Value) -> Value {
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
        &json!({
            "jsonrpc":"2.0",
            "id":2046,
            "method":"tools/call",
            "params":{ "name": tool_name, "arguments": arguments }
        }),
    );

    let mut reader = BufReader::new(child.stdout.take().expect("stdout"));
    let _ = read_json_line(&mut reader);
    let resp = read_json_line(&mut reader);
    let _ = child.kill();
    let _ = child.wait();
    resp
}

#[test]
fn hcom029_tools_embedded_schemas_registered() {
    for schema in [
        map_to_geometry_tool_schema(),
        refine_shape_tool_schema(),
        get_audit_digest_tool_schema(),
    ] {
        assert!(schema.get("name").and_then(|n| n.as_str()).is_some());
        let input = &schema["inputSchema"];
        assert!(input.get("$schema").is_some());
        assert!(input.get("properties").is_some());
    }
    let bundle = hcom_semantic_tools_schema_bundle();
    assert_eq!(bundle["schema_version"], json!(HCOM_TOOLS_SCHEMA_VERSION));
    assert_eq!(bundle["owner"], json!(HCOM_TOOLS_SLOT_OWNER));
}

#[test]
fn hcom029_tools_in_process_map_to_geometry() {
    let (body, is_error) = exec_map_to_geometry(&json!({
        "surface": "chair",
        "lang": "en"
    }));
    assert!(!is_error);
    assert_eq!(body["schema_version"], json!("map_to_geometry.v1"));
    assert!(body["quotient_id"].is_string());
}

#[test]
fn hcom029_tools_in_process_refine_shape_stub() {
    let (refused, err) = exec_refine_shape(&json!({
        "shape": { "quotient_id": "chair" },
        "feedback": "add armrests"
    }));
    assert!(err);
    assert_eq!(refused["agent_error"]["code"], json!("trust_refused"));

    let (body, is_error) = exec_refine_shape(&with_device_trust(json!({
        "shape": { "quotient_id": "chair" },
        "feedback": "add armrests"
    })));
    assert!(!is_error);
    assert_eq!(body["status"], json!("stub_honest"));
}

#[test]
fn hcom029_tools_in_process_audit_fixture_golden() {
    let (body, is_error) =
        exec_get_audit_digest(&json!({ "decision_id": FIXTURE_CHAIR_EN_PROPOSAL_ID }));
    assert!(!is_error);
    assert_eq!(
        body["digest_hex"],
        json!(GOLDEN_CHAIR_EN_PROPOSAL_DIGEST_HEX)
    );
    assert_eq!(body["digest_source"], json!("fixture_log"));
}

#[test]
fn hcom029_tools_stdio_map_to_geometry_green() {
    let resp = stdio_tool_call(
        "map_to_geometry",
        &json!({ "surface": "chair", "lang": "en" }),
    );
    assert!(resp.get("error").is_none(), "stdio error: {resp}");
    assert_eq!(resp["result"]["isError"], json!(false));
    let text = resp["result"]["content"][0]["text"].as_str().expect("text");
    let body: Value = serde_json::from_str(text).expect("json body");
    assert_eq!(body["schema_version"], json!("map_to_geometry.v1"));
}

#[test]
fn hcom029_tools_stdio_refine_shape_stub_honest() {
    let resp = stdio_tool_call(
        "refine_shape",
        &with_device_trust(json!({
            "shape": { "quotient_id": "chair" },
            "feedback": "add backrest"
        })),
    );
    assert!(resp.get("error").is_none(), "stdio error: {resp}");
    assert_eq!(resp["result"]["isError"], json!(false));
}

#[test]
fn hcom029_tools_stdio_get_audit_digest_green() {
    let resp = stdio_tool_call(
        "get_audit_digest",
        &json!({ "decision_id": FIXTURE_CHAIR_EN_PROPOSAL_ID }),
    );
    assert!(resp.get("error").is_none(), "stdio error: {resp}");
    assert_eq!(resp["result"]["isError"], json!(false));
    let text = resp["result"]["content"][0]["text"].as_str().expect("text");
    let body: Value = serde_json::from_str(text).expect("json body");
    assert_eq!(
        body["digest_hex"],
        json!(GOLDEN_CHAIR_EN_PROPOSAL_DIGEST_HEX)
    );
}

#[test]
fn hcom029_tools_stdio_list_excludes_propose_owner_sim() {
    let exe = mcp_binary_path();
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

    let names: Vec<_> = list["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();
    for tool in TOOLS {
        assert!(names.contains(tool), "tools/list missing {tool}");
    }
}
