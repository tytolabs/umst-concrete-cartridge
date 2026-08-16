// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//
// FLEET-COMPOSER-H H08 — reproducible native stdio JSON-RPC smoke battery.
//
// Slots (4/4):
// 1. `initialize` jsonrpc 2.0 handshake + protocolVersion pin
// 2. `tools/list` count matches `tool_census` SSOT
// 3. `tools/call` `umst_predict` returns `result.v2`
// 4. unknown method returns `-32601` error frame (no stdout pollution)

use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command as StdCmd, Stdio};
use umst_mcp::stdio_smoke::{MCP_PROTOCOL_VERSION, NATIVE_STDIO_SMOKE_SLOT_COUNT};
use umst_mcp::tool_census;

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
    reader.read_line(&mut buf).expect("read stdout line");
    let trimmed = buf.trim();
    assert!(
        !trimmed.is_empty(),
        "empty stdout line — stderr may have leaked"
    );
    serde_json::from_str(trimmed).expect("valid json line")
}

fn write_frame(stdin: &mut impl Write, frame: &Value) {
    writeln!(stdin, "{}", serde_json::to_string(frame).unwrap()).unwrap();
}

fn spawn_mcp() -> (
    std::process::Child,
    std::process::ChildStdin,
    BufReader<std::process::ChildStdout>,
) {
    let exe = mcp_binary_path();
    assert!(
        exe.exists(),
        "missing umst-mcp at {} — run `cargo test -p umst-mcp` to build binary first",
        exe.display()
    );
    let mut child = StdCmd::new(&exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn umst-mcp");
    let stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    (child, stdin, BufReader::new(stdout))
}

fn assert_jsonrpc_ok(resp: &Value, id: i64) {
    assert_eq!(resp["jsonrpc"].as_str(), Some("2.0"), "{resp}");
    assert_eq!(resp["id"], json!(id), "{resp}");
    assert!(
        resp.get("error").is_none(),
        "unexpected error frame: {resp}"
    );
    assert!(resp.get("result").is_some(), "missing result: {resp}");
}

fn tool_names(resp: &Value) -> BTreeSet<String> {
    resp["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect()
}

/// Slot 1 — initialize handshake.
#[test]
fn h08_stdio_initialize_jsonrpc_handshake_green() {
    let (mut child, mut stdin, mut reader) = spawn_mcp();
    write_frame(
        &mut stdin,
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    );
    let resp = read_json_line(&mut reader);
    assert_jsonrpc_ok(&resp, 1);
    assert_eq!(
        resp["result"]["protocolVersion"].as_str(),
        Some(MCP_PROTOCOL_VERSION)
    );
    assert!(resp["result"]["serverInfo"]["name"].is_string());
    let _ = child.kill();
    let _ = child.wait();
}

/// Slot 2 — tools/list census SSOT.
#[test]
fn h08_stdio_tools_list_census_matches_ssot() {
    let expected = tool_census::expected_tools_list_count_for_build();
    let (mut child, mut stdin, mut reader) = spawn_mcp();
    write_frame(
        &mut stdin,
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    );
    let _ = read_json_line(&mut reader);
    write_frame(
        &mut stdin,
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    );
    let list = read_json_line(&mut reader);
    assert_jsonrpc_ok(&list, 2);
    let names = tool_names(&list);
    assert_eq!(
        names.len(),
        expected,
        "tools/list count drift vs tool_census SSOT: got {:?}",
        names
    );
    let _ = child.kill();
    let _ = child.wait();
}

/// Slot 3 — umst_predict tools/call smoke.
#[test]
fn h08_stdio_umst_predict_call_green() {
    let (mut child, mut stdin, mut reader) = spawn_mcp();
    write_frame(
        &mut stdin,
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    );
    let _ = read_json_line(&mut reader);
    write_frame(
        &mut stdin,
        &json!({
            "jsonrpc":"2.0",
            "id":3,
            "method":"tools/call",
            "params":{
                "name":"umst_predict",
                "arguments":{
                    "profile":"default",
                    "mix":{"w_c":0.4,"temperature_k":293.15},
                    "schema_version":"v2"
                }
            }
        }),
    );
    let resp = read_json_line(&mut reader);
    assert_jsonrpc_ok(&resp, 3);
    let text = resp["result"]["content"][0]["text"]
        .as_str()
        .expect("predict text payload");
    let pred: Value = serde_json::from_str(text).expect("predict json");
    assert_eq!(pred["schema_version"].as_str(), Some("result.v2"));
    let _ = child.kill();
    let _ = child.wait();
}

/// Slot 4 — unknown method error discipline.
#[test]
fn h08_stdio_unknown_method_error_frame_green() {
    let (mut child, mut stdin, mut reader) = spawn_mcp();
    write_frame(
        &mut stdin,
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    );
    let _ = read_json_line(&mut reader);
    write_frame(
        &mut stdin,
        &json!({"jsonrpc":"2.0","id":99,"method":"fleet/h08/nonexistent","params":{}}),
    );
    let resp = read_json_line(&mut reader);
    assert_eq!(resp["jsonrpc"].as_str(), Some("2.0"));
    assert_eq!(resp["id"], json!(99));
    assert!(resp.get("result").is_none());
    assert_eq!(resp["error"]["code"].as_i64(), Some(-32601));
    let _ = child.kill();
    let _ = child.wait();
}

/// Meta — battery slot count pin (reproducible GREEN ledger).
#[test]
fn h08_stdio_smoke_battery_slot_count_pin() {
    assert_eq!(NATIVE_STDIO_SMOKE_SLOT_COUNT, 4);
}
