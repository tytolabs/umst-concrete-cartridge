// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Subprocess MCP smoke: **`umst-mcp`** binary under **`target/{{PROFILE}}`** (cargo test builds it first).

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command as StdCmd, Stdio};

fn read_json_line<R: BufRead>(reader: &mut R) -> Value {
    let mut buf = String::new();
    reader.read_line(&mut buf).expect("line");
    serde_json::from_str(buf.trim()).expect("json")
}

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

#[test]
fn mcp_tools_list_and_predict_smoke() {
    let exe = mcp_binary_path();
    assert!(
        exe.exists(),
        "missing umst-mcp binary at {} — `cargo test -p umst-mcp` should build deps first",
        exe.display()
    );

    let mut child = StdCmd::new(&exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn mcp");

    let stdin = child.stdin.as_mut().expect("stdin piped");

    let frames = [
        json!({
          "jsonrpc": "2.0",
          "id": 1,
          "method": "initialize",
          "params": {}
        }),
        json!({
          "jsonrpc": "2.0",
          "id": 2,
          "method": "tools/list",
          "params": {}
        }),
        json!({
          "jsonrpc": "2.0",
          "id": 3,
          "method": "tools/call",
          "params": {
            "name": "umst_predict",
            "arguments": {
              "profile": "default",
              "mix": {"w_c": 0.4, "temperature_k": 293.15},
              "schema_version": "v2"
            }
          }
        }),
    ];
    for f in frames {
        writeln!(stdin, "{}", serde_json::to_string(&f).unwrap()).unwrap();
    }

    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let r1 = read_json_line(&mut reader);
    assert_eq!(r1["id"], 1);
    assert!(r1.get("result").is_some(), "{r1}");

    let r2 = read_json_line(&mut reader);
    assert_eq!(r2["id"], 2);
    let tools = &r2["result"]["tools"];
    let names: Vec<String> = tools
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect();
    assert!(names.contains(&"umst_predict".to_string()), "{names:?}");

    let r3 = read_json_line(&mut reader);
    assert_eq!(r3["id"], 3);
    let text = r3["result"]["content"][0]["text"]
        .as_str()
        .expect("text field");
    let pred: Value = serde_json::from_str(text).expect("predict json");
    assert_eq!(pred["schema_version"].as_str(), Some("result.v2"));

    let _ = child.kill();
    let _ = child.wait();
}
