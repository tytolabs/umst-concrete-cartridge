// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! HCOM-021 — hybrid frontier proposal + local gate orchestration (mock LLM integration).

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command as StdCmd, Stdio};
use umst_mcp::semantic_hcom::{
    exec_propose_communicative_act, orchestrate_communicative_act, propose_communicative_act_tool_schema,
    MockFrontierLlm, ORCHESTRATION_STEPS,
};
use umst_semantics::LangCode;

const GATED_WIRE_SCHEMA: &str = "gated_communicative_act.v1";
const HYBRID_SCHEMA: &str = "hybrid_frontier_local_gate.v1";

fn mcp_binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_umst-mcp"))
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
fn hcom_021_tool_schema_documents_hybrid_orchestration() {
    let tool = propose_communicative_act_tool_schema();
    assert_eq!(tool["name"], json!("propose_communicative_act"));
    assert!(tool["description"]
        .as_str()
        .unwrap_or("")
        .contains("hybrid"));
    assert!(tool["inputSchema"]["properties"]["mock_llm"].is_object());
}

fn with_device_trust(mut args: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = args.as_object_mut() {
        obj.insert("trust".into(), json!({ "scope": "device" }));
    }
    args
}

#[test]
fn hcom_021_mock_llm_three_step_orchestration_admits() {
    let (body, is_error) = exec_propose_communicative_act(&with_device_trust(json!({
        "intent": "describe chair geometry to operator",
        "context": { "lang": "en" }
    })));
    assert!(!is_error);
    assert_eq!(body["schema_version"], json!(GATED_WIRE_SCHEMA));
    assert_eq!(body["gate_summary"]["admissible"], json!(true));
    assert_eq!(body["orchestration"]["schema_version"], json!(HYBRID_SCHEMA));
    let steps = body["orchestration"]["steps"].as_array().expect("steps");
    assert_eq!(steps.len(), ORCHESTRATION_STEPS.len());
    for (step, expected) in steps.iter().zip(ORCHESTRATION_STEPS.iter()) {
        assert_eq!(step["step"], json!(expected));
    }
    assert!(body["audit_digest"]
        .as_str()
        .unwrap_or("")
        .starts_with("sha256:"));
}

#[test]
fn hcom_021_no_back_injection_rejects_via_local_gate() {
    let (body, is_error) = exec_propose_communicative_act(&with_device_trust(json!({
        "intent": "describe stool without back",
        "context": { "lang": "en", "injection": "no_back" },
        "mock_llm": { "no_back_injection": true }
    })));
    assert!(is_error);
    assert_eq!(body["gate_summary"]["admissible"], json!(false));
    assert_eq!(body["gate_summary"]["verdict"], json!("REJECT"));
    assert!(body["orchestration"]["external_signal_query_recommended"]
        .as_bool()
        .unwrap_or(false));
}

#[test]
fn hcom_021_ta_chair_surface_maps_and_admits() {
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
fn hcom_021_stdio_mock_llm_integration_green() {
    let exe = mcp_binary_path();
    assert!(
        exe.exists(),
        "missing umst-mcp at {} — build with --features tool-propose-communicative-act",
        exe.display()
    );

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
            "id":21,
            "method":"tools/call",
            "params":{
                "name":"propose_communicative_act",
                "arguments":{
                    "intent":"HCOM-021 stdio mock LLM chair admit",
                    "context":{"lang":"en"},
                    "trust":{"scope":"device"}
                }
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
    let text = resp["result"]["content"][0]["text"]
        .as_str()
        .expect("text");
    let body: Value = serde_json::from_str(text).expect("json body");
    assert_eq!(body["schema_version"], json!(GATED_WIRE_SCHEMA));
    assert_eq!(body["gate_summary"]["admissible"], json!(true));
    assert_eq!(body["orchestration"]["schema_version"], json!(HYBRID_SCHEMA));
}
