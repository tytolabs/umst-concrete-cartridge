// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! IDEA-004 / HCOM-029 — `propose_communicative_act` MCP schema + mock LLM stdio integration.

#![allow(unexpected_cfgs)]

use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command as StdCmd, Stdio};
use umst_mcp::semantic_hcom_schema::{
    exec_propose_communicative_act_mock, hcom_mcp_schema_bundle,
    propose_communicative_act_tool_schema, PROPOSE_COMMUNICATIVE_ACT_TOOL,
};

fn mcp_binary_path() -> PathBuf {
    if let Ok(exe) = std::env::var("CARGO_BIN_EXE_umst-mcp") {
        return PathBuf::from(exe);
    }
    let profile = option_env!("PROFILE").unwrap_or("debug");
    let target_base = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    target_base.join(profile).join("umst-mcp")
}

fn read_json_line<R: BufRead>(reader: &mut R) -> Value {
    let mut buf = String::new();
    reader.read_line(&mut buf).expect("line");
    serde_json::from_str(buf.trim()).expect("json")
}

fn write_frame(stdin: &mut impl Write, frame: &Value) {
    writeln!(stdin, "{}", serde_json::to_string(frame).unwrap()).unwrap();
}

fn tool_names(resp: &Value) -> BTreeSet<String> {
    resp["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect()
}

#[test]
fn idea004_schema_bundle_honest() {
    let bundle = hcom_mcp_schema_bundle();
    assert_eq!(bundle["schema_version"], json!("hcom_mcp_v0"));
    assert_eq!(bundle["response_family"], json!("Semantic"));
    let tool = propose_communicative_act_tool_schema();
    assert_eq!(
        tool["name"].as_str(),
        Some(PROPOSE_COMMUNICATIVE_ACT_TOOL)
    );
}

#[test]
fn idea004_mock_llm_admit_and_reject_paths() {
    let admit = exec_propose_communicative_act_mock(&json!({
        "intent": "describe chair geometry",
        "context": { "lang": "en", "dialogue_turn": 0 },
        "mock_llm_fixture": "consistent_chair"
    }));
    assert_eq!(admit.0["gate"]["admissible"], json!(true));
    assert!(admit.0["audit"]["decision_id"]
        .as_str()
        .unwrap()
        .starts_with("hcom-act:en:"));

    let reject = exec_propose_communicative_act_mock(&json!({
        "intent": "chair missing back primitive",
        "context": { "lang": "ta", "dialogue_turn": 2 },
        "mock_llm_fixture": "inconsistent_no_back"
    }));
    assert_eq!(reject.0["gate"]["admissible"], json!(false));
    assert_eq!(reject.0["gate"]["verdict"], json!("REJECT"));
}

#[test]
fn idea004_stdio_tools_list_includes_propose_communicative_act() {
    let exe = mcp_binary_path();
    assert!(
        exe.exists(),
        "missing umst-mcp at {} — build with --features tool-semantic-hcom",
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
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    );

    let mut reader = BufReader::new(child.stdout.take().expect("stdout"));
    let _ = read_json_line(&mut reader);
    let list = read_json_line(&mut reader);
    let names = tool_names(&list);
    assert!(names.contains(PROPOSE_COMMUNICATIVE_ACT_TOOL));
    assert!(names.len() > 13, "expected additive tool beyond frozen 13: {names:?}");

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn idea004_stdio_mock_llm_integration_green() {
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
        &json!({
            "jsonrpc":"2.0",
            "id":2,
            "method":"tools/call",
            "params":{
                "name":"propose_communicative_act",
                "arguments":{
                    "intent":"operator requests chair description",
                    "context":{"lang":"en","dialogue_turn":0},
                    "mock_llm_fixture":"consistent_chair"
                }
            }
        }),
    );

    let mut reader = BufReader::new(child.stdout.take().expect("stdout"));
    let _ = read_json_line(&mut reader);
    let resp = read_json_line(&mut reader);
    assert_eq!(resp["result"]["isError"], json!(false));
    let text = resp["result"]["content"][0]["text"]
        .as_str()
        .expect("text");
    let body: Value = serde_json::from_str(text).expect("json body");
    assert_eq!(body["schema_version"], json!("gated_communicative_response.v0"));
    assert_eq!(body["gate"]["admissible"], json!(true));
    assert_eq!(body["mock_llm"], json!(true));
    assert_eq!(body["proposal"]["fixture"], json!("consistent_chair"));

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn idea004_stdio_mock_llm_reject_is_error_frame() {
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
        &json!({
            "jsonrpc":"2.0",
            "id":3,
            "method":"tools/call",
            "params":{
                "name":"propose_communicative_act",
                "arguments":{
                    "intent":"incomplete chair witness",
                    "context":{"lang":"en"},
                    "mock_llm_fixture":"inconsistent_no_back"
                }
            }
        }),
    );

    let mut reader = BufReader::new(child.stdout.take().expect("stdout"));
    let _ = read_json_line(&mut reader);
    let resp = read_json_line(&mut reader);
    assert_eq!(resp["result"]["isError"], json!(true));
    let text = resp["result"]["content"][0]["text"]
        .as_str()
        .expect("text");
    let body: Value = serde_json::from_str(text).expect("json");
    assert_eq!(body["gate"]["verdict"], json!("REJECT"));

    let _ = child.kill();
    let _ = child.wait();
}
