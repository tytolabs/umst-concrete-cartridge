// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! S3 parity harness — hand-rolled `umst-mcp` vs `umst-mcp-rmcp` golden `tools/call` frames.
//!
//! Run: `cargo test -p umst-mcp --features rmcp-wire --test rmcp_parity`

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command as StdCmd, Stdio};
use umst_mcp::parity::{canonical_bytes, canonicalize_tools_call_result};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn load_fixture_root() -> Value {
    let path = fixtures_dir().join("gate_parity_v0.json");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn binary_path(name: &str) -> PathBuf {
    let profile = option_env!("PROFILE").unwrap_or("debug");
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .join(profile)
        .join(name)
}

fn read_json_line<R: BufRead>(reader: &mut R) -> Value {
    let mut buf = String::new();
    reader.read_line(&mut buf).expect("line");
    serde_json::from_str(buf.trim()).expect("json")
}

fn write_frame(stdin: &mut impl Write, frame: &Value) {
    writeln!(stdin, "{}", serde_json::to_string(frame).unwrap()).unwrap();
}

fn initialize_hand(stdin: &mut impl Write, reader: &mut impl BufRead) {
    write_frame(
        stdin,
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    );
    let r = read_json_line(reader);
    assert_eq!(r["id"], 1);
    assert!(r.get("result").is_some(), "{r}");
}

fn initialize_rmcp(stdin: &mut impl Write, reader: &mut impl BufRead) {
    write_frame(
        stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "rmcp-parity-test", "version": "1.0.0" }
            }
        }),
    );
    let r = read_json_line(reader);
    assert_eq!(r["id"], 1);
    assert!(r.get("result").is_some(), "{r}");
    write_frame(
        stdin,
        &json!({"jsonrpc":"2.0","method":"notifications/initialized","params":{}}),
    );
}

fn spawn_binary(
    name: &str,
) -> (
    std::process::Child,
    std::process::ChildStdin,
    BufReader<std::process::ChildStdout>,
) {
    let exe = binary_path(name);
    assert!(
        exe.exists(),
        "missing {name} at {} — build with `cargo test -p umst-mcp --features rmcp-wire`",
        exe.display()
    );
    let mut child = StdCmd::new(&exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn mcp");
    let stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    (child, stdin, BufReader::new(stdout))
}

fn collect_hand_frames(root: &Value) -> Vec<Value> {
    let pass_mix = &root["mix_table"]["pass_rational_default"];
    let frames_req = [
        json!({
            "jsonrpc": "2.0",
            "id": 100,
            "method": "tools/call",
            "params": { "name": "umst_profiles", "arguments": {} }
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 101,
            "method": "tools/call",
            "params": {
                "name": "umst_gate_check",
                "arguments": {
                    "mix": pass_mix,
                    "profile": "default",
                    "explain": true
                }
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 102,
            "method": "tools/call",
            "params": {
                "name": "umst_mi_estimate",
                "arguments": { "mix": pass_mix }
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 103,
            "method": "tools/call",
            "params": {
                "name": "umst_predict",
                "arguments": {
                    "mix": pass_mix,
                    "profile": "default",
                    "schema_version": "v2"
                }
            }
        }),
    ];

    let (mut child, mut stdin, mut reader) = spawn_binary("umst-mcp");
    initialize_hand(&mut stdin, &mut reader);

    let mut out = Vec::new();
    for req in &frames_req {
        write_frame(&mut stdin, req);
        let resp = read_json_line(&mut reader);
        let tool = req["params"]["name"].as_str().unwrap().to_string();
        out.push(json!({
            "tool": tool,
            "result": canonicalize_tools_call_result(&resp),
        }));
    }
    let _ = child.kill();
    let _ = child.wait();
    out
}

fn collect_rmcp_frames(root: &Value) -> Vec<Value> {
    let pass_mix = &root["mix_table"]["pass_rational_default"];
    let frames_req = [
        json!({
            "jsonrpc": "2.0",
            "id": 100,
            "method": "tools/call",
            "params": { "name": "umst_profiles", "arguments": {} }
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 101,
            "method": "tools/call",
            "params": {
                "name": "umst_gate_check",
                "arguments": {
                    "mix": pass_mix,
                    "profile": "default",
                    "explain": true
                }
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 102,
            "method": "tools/call",
            "params": {
                "name": "umst_mi_estimate",
                "arguments": { "mix": pass_mix }
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 103,
            "method": "tools/call",
            "params": {
                "name": "umst_predict",
                "arguments": {
                    "mix": pass_mix,
                    "profile": "default",
                    "schema_version": "v2"
                }
            }
        }),
    ];

    let (mut child, mut stdin, mut reader) = spawn_binary("umst-mcp-rmcp");
    initialize_rmcp(&mut stdin, &mut reader);

    let mut out = Vec::new();
    for req in &frames_req {
        write_frame(&mut stdin, req);
        let resp = read_json_line(&mut reader);
        let tool = req["params"]["name"].as_str().unwrap().to_string();
        out.push(json!({
            "tool": tool,
            "result": canonicalize_tools_call_result(&resp),
        }));
    }
    let _ = child.kill();
    let _ = child.wait();
    out
}

#[test]
fn hand_vs_rmcp_tools_call_result_parity() {
    let root = load_fixture_root();
    let golden_path = fixtures_dir().join("tools_call_result_frames_v0.json");
    let golden_text = std::fs::read_to_string(&golden_path).expect("read golden frames");
    let golden: Value = serde_json::from_str(&golden_text).expect("parse golden");
    let want = golden["frames"].as_array().expect("frames");

    let hand = collect_hand_frames(&root);
    let rmcp = collect_rmcp_frames(&root);

    assert_eq!(hand.len(), rmcp.len(), "frame count hand vs rmcp");
    assert_eq!(hand.len(), want.len(), "frame count vs golden");

    for (i, (hand_frame, rmcp_frame)) in hand.iter().zip(rmcp.iter()).enumerate() {
        assert_eq!(
            canonical_bytes(hand_frame),
            canonical_bytes(rmcp_frame),
            "hand vs rmcp drift at index {i}"
        );
    }

    for (i, (actual, expected)) in hand.iter().zip(want.iter()).enumerate() {
        assert_eq!(
            canonical_bytes(actual),
            canonical_bytes(expected),
            "hand vs S0 golden drift at index {i}"
        );
    }
}
