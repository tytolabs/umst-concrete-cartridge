// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! S1 parity: `tool-manifest` dual-emit vs hand-rolled `tools/list`.
//!
//! With feature off (default): this module is empty — existing integration/gate_parity cover hand path.
//! With feature on: same binary, `UMST_MCP_MANIFEST=0|1` — names + required inputSchema keys deep-equal.

#![cfg(feature = "tool-manifest")]

use serde_json::{json, Value};
use std::collections::BTreeMap;
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

fn tools_list_with_env(manifest: &str) -> Vec<Value> {
    let exe = mcp_binary_path();
    assert!(exe.exists(), "missing {}", exe.display());
    let mut child = StdCmd::new(&exe)
        .env("UMST_MCP_MANIFEST", manifest)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn");
    let stdin = child.stdin.as_mut().expect("stdin");
    for frame in [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    ] {
        writeln!(stdin, "{}", serde_json::to_string(&frame).unwrap()).unwrap();
    }
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);
    let _ = read_json_line(&mut reader);
    let list = read_json_line(&mut reader);
    let _ = child.kill();
    let _ = child.wait();
    list["result"]["tools"].as_array().expect("tools").clone()
}

fn name_and_required(tools: &[Value]) -> BTreeMap<String, Vec<String>> {
    let mut map = BTreeMap::new();
    for t in tools {
        let name = t["name"].as_str().expect("name").to_string();
        let mut req: Vec<String> = t["inputSchema"]
            .get("required")
            .and_then(|r| r.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        req.sort();
        map.insert(name, req);
    }
    map
}

#[test]
fn tool_manifest_list_names_and_required_keys_parity() {
    let hand = tools_list_with_env("0");
    let manifest = tools_list_with_env("1");
    assert_eq!(
        name_and_required(&hand),
        name_and_required(&manifest),
        "tools/list names + required inputSchema keys must deep-equal (hand vs manifest)"
    );
    // Library path agrees with env=1 emission.
    let lib = umst_mcp::manifest::tools_v1::mcp_tools_schema();
    assert_eq!(
        name_and_required(&manifest),
        name_and_required(&lib),
        "UMST_MCP_MANIFEST=1 must match umst_mcp::manifest::tools_v1::mcp_tools_schema()"
    );
}
