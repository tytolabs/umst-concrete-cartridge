// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
//
//! MCP-082 deepen — stdio `tools/list` count must match `tool_census` SSOT.

use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command as StdCmd, Stdio};
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
fn stdio_tools_list_count_matches_tool_census_ssot() {
    let expected = tool_census::expected_tools_list_count_for_build();
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
    let init = read_json_line(&mut reader);
    assert_eq!(init["jsonrpc"].as_str(), Some("2.0"));
    assert_eq!(init["id"], 1);
    assert!(init.get("result").is_some(), "initialize result: {init}");

    let list = read_json_line(&mut reader);
    assert_eq!(list["jsonrpc"].as_str(), Some("2.0"));
    assert_eq!(list["id"], 2);
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
