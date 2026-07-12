// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! S7 proposed-tools parity — features **off** keeps 13-tool golden; features **on** grow list.

#![allow(unexpected_cfgs)]

use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command as StdCmd, Stdio};

fn mcp_binary_path() -> PathBuf {
    let profile = option_env!("PROFILE").unwrap_or("debug");
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .join(profile)
        .join("umst-mcp")
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

#[cfg(all(
    feature = "agent-layer",
    not(any(
        feature = "gate-explain-v2",
        feature = "tool-dry-run",
        feature = "tool-promote",
        feature = "tool-arena-session-unified"
    ))
))]
#[test]
fn s7_features_off_thirteen_tools() {
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
    assert_eq!(list["id"], 2);
    assert_eq!(tool_names(&list).len(), 13);

    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(all(
    feature = "agent-layer",
    any(
        feature = "tool-dry-run",
        feature = "tool-promote",
        feature = "tool-arena-session-unified"
    )
))]
#[test]
fn s7_proposed_tools_extend_list() {
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
    let names = tool_names(&list);
    assert!(
        names.len() > 13,
        "expected proposed tools in list: {names:?}"
    );

    #[cfg(feature = "tool-dry-run")]
    assert!(names.contains("umst_dry_run"));
    #[cfg(feature = "tool-promote")]
    assert!(names.contains("umst_promote_contribution"));
    #[cfg(feature = "tool-arena-session-unified")]
    assert!(names.contains("umst_arena_session"));

    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(all(feature = "agent-layer", feature = "tool-dry-run"))]
#[test]
fn s7_dry_run_does_not_write_memory_flag() {
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
                "name":"umst_dry_run",
                "arguments":{
                    "mix":{"w_c":"9/20","temperature_k":"29315/100","aggregate_volume_fraction":"7/10"},
                    "profile":"default"
                }
            }
        }),
    );

    let mut reader = BufReader::new(child.stdout.take().expect("stdout"));
    let _ = read_json_line(&mut reader);
    let resp = read_json_line(&mut reader);
    let text = resp["result"]["content"][0]["text"].as_str().expect("text");
    let body: Value = serde_json::from_str(text).expect("json body");
    assert_eq!(body["dry_run"], json!(true));
    assert_eq!(body["writes_memory"], json!(false));

    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(feature = "tool-promote")]
#[test]
fn s7_promote_stub_returns_not_wired() {
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
                "name":"umst_promote_contribution",
                "arguments":{"proposal":{"schema_version":"promotion_proposal.v1"}}
            }
        }),
    );

    let mut reader = BufReader::new(child.stdout.take().expect("stdout"));
    let _ = read_json_line(&mut reader);
    let resp = read_json_line(&mut reader);
    assert_eq!(resp["result"]["isError"], json!(true));
    let text = resp["result"]["content"][0]["text"].as_str().expect("text");
    let body: Value = serde_json::from_str(text).expect("json");
    assert_eq!(
        body["agent_error"]["code"].as_str(),
        Some("promote_not_wired")
    );

    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(all(feature = "agent-layer", feature = "gate-explain-v2"))]
#[test]
fn s7_explain_v2_additive_field() {
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
                "name":"umst_gate_check",
                "arguments":{
                    "mix":{"w_c":"9/20","temperature_k":"29315/100","aggregate_volume_fraction":"7/10"},
                    "profile":"default",
                    "explain":true,
                    "explain_v2":true
                }
            }
        }),
    );

    let mut reader = BufReader::new(child.stdout.take().expect("stdout"));
    let _ = read_json_line(&mut reader);
    let resp = read_json_line(&mut reader);
    let text = resp["result"]["content"][0]["text"].as_str().expect("text");
    let body: Value = serde_json::from_str(text).expect("json");
    assert!(body.get("explain_v2").is_some());
    assert_eq!(
        body["explain_v2"]["schema_version"].as_str(),
        Some("gate_explain.v2")
    );
    assert_eq!(body["gate_summary"]["admissible"].as_bool(), Some(true));

    let _ = child.kill();
    let _ = child.wait();
}
