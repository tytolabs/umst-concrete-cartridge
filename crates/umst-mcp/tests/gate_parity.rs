// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! S0 parity harness (`docs/MCP_BUILD_PLAN.md` Stage S0).
//!
//! Locks gate + `tools/list` / `tools/call` responses as golden fixtures.
//! GO-LIVE Step 3: package `default = ["agent-layer"]` → default `tools/list` is **13** tools.
//! Gate `gate_check_mix_result` bytes are unchanged (surface expansion only).
//!
//! Run:
//! - default (13-tool list + gate + call frames): `cargo test -p umst-mcp --test gate_parity`
//! - base-four surface (`--no-default-features`): still asserts the historical 4-tool list
//! - rewrite call-frame goldens: `UMST_GATE_PARITY_UPDATE=1 cargo test -p umst-mcp --test gate_parity tools_call_result_frames_parity -- --ignored`
//!
//! **Phase 0f lock:** fixture bytes SHA256 pinned below; must match census + manifold phase0f suite.

/// Live `gate_parity_v0.json` digest (Phase 0f / M0 acceptance receipt).
const GATE_PARITY_V0_SHA256: &str =
    "149081fa81a6525fb66ff01924c6656f30e2b67846d9945a25427c7be38d20f3";

use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as StdCmd, Stdio};
#[cfg(feature = "agent-layer")]
use umst_mcp::parity::{canonical_bytes, canonicalize_tools_call_result, sort_keys};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn load_fixture_root() -> Value {
    let path = fixtures_dir().join("gate_parity_v0.json");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn gate_parity_v0_fixture_sha256_locked() {
    let path = fixtures_dir().join("gate_parity_v0.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    assert_eq!(
        sha256_hex(&bytes),
        GATE_PARITY_V0_SHA256,
        "gate_parity_v0.json digest drift — update pin only after intentional fixture change"
    );
    let root = load_fixture_root();
    assert_eq!(
        root["schema_version"].as_str(),
        Some("gate_parity_v0"),
        "fixture schema_version must remain gate_parity_v0"
    );
}

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

fn tool_names_from_list_result(resp: &Value) -> BTreeSet<String> {
    resp["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect()
}

fn expected_names(root: &Value, key: &str) -> BTreeSet<String> {
    root["tools_list_names"][key]
        .as_array()
        .expect("name array")
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect()
}

fn spawn_mcp_in(
    cwd: Option<&Path>,
) -> (
    std::process::Child,
    std::process::ChildStdin,
    BufReader<std::process::ChildStdout>,
) {
    let exe = mcp_binary_path();
    assert!(
        exe.exists(),
        "missing umst-mcp binary at {} — `cargo test -p umst-mcp` should build deps first",
        exe.display()
    );
    let mut cmd = StdCmd::new(&exe);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let mut child = cmd.spawn().expect("spawn mcp");
    let stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    (child, stdin, BufReader::new(stdout))
}

fn spawn_mcp() -> (
    std::process::Child,
    std::process::ChildStdin,
    BufReader<std::process::ChildStdout>,
) {
    spawn_mcp_in(None)
}

#[cfg(feature = "agent-layer")]
fn temp_mcp_cwd() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("umst-mcp-gate-parity-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp cwd");
    dir
}

fn write_frame(stdin: &mut impl Write, frame: &Value) {
    writeln!(stdin, "{}", serde_json::to_string(frame).unwrap()).unwrap();
}

fn initialize(stdin: &mut impl Write, reader: &mut impl BufRead) {
    write_frame(
        stdin,
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    );
    let r = read_json_line(reader);
    assert_eq!(r["id"], 1);
    assert!(r.get("result").is_some(), "{r}");
}

/// `--no-default-features` binary: tools/list name set must be exactly the historical 4 base tools.
/// Compiled only when `agent-layer` is off (go-live default enables agent-layer).
#[cfg(not(feature = "agent-layer"))]
#[test]
fn tools_list_base_four_names_without_agent_layer() {
    let root = load_fixture_root();
    let expected = expected_names(&root, "base_four");
    assert_eq!(expected.len(), 4, "fixture base_four names");

    let (mut child, mut stdin, mut reader) = spawn_mcp();
    initialize(&mut stdin, &mut reader);

    write_frame(
        &mut stdin,
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    );
    let r2 = read_json_line(&mut reader);
    assert_eq!(r2["id"], 2);
    let names = tool_names_from_list_result(&r2);
    assert_eq!(
        names, expected,
        "base-four tools/list name set drifted from S0 golden"
    );

    let _ = child.kill();
    let _ = child.wait();
}

/// Default package features (`default = ["agent-layer"]`): tools/list must be the 13-tool set.
#[cfg(feature = "agent-layer")]
#[test]
fn tools_list_default_thirteen_names() {
    let root = load_fixture_root();
    let expected = expected_names(&root, "default");
    assert_eq!(expected.len(), 13, "fixture default names (agent-native)");

    let (mut child, mut stdin, mut reader) = spawn_mcp();
    initialize(&mut stdin, &mut reader);

    write_frame(
        &mut stdin,
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    );
    let r2 = read_json_line(&mut reader);
    assert_eq!(r2["id"], 2);
    let names = tool_names_from_list_result(&r2);
    assert_eq!(
        names, expected,
        "default tools/list name set drifted from S0 golden (expected 13)"
    );

    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(feature = "agent-layer")]
mod agent_layer_parity {
    use super::*;
    use umst_concrete_cartridge::calibration::Profile;
    use umst_concrete_cartridge::research::{gate_check_mix_result, ObservedAt};

    fn frozen_observed(root: &Value) -> ObservedAt {
        let o = &root["observed_at"];
        ObservedAt {
            stamp_tier: o["stamp_tier"].as_str().unwrap().into(),
            ucrs_seq: o["ucrs_seq"].as_u64(),
            phase_entropy_bits_q: None,
            phase_entropy_bits_scale: None,
            credit_head_bits_q: None,
            credit_head_bits_scale: None,
            wall_ms: o["wall_ms"].as_u64(),
        }
    }

    /// Cold library `gate_check_mix_result` JSON (sorted keys) byte-identical to fixture.
    #[test]
    fn gate_check_mix_result_parity_fixture() {
        let root = load_fixture_root();
        let profile = Profile::load_bundled("default").expect("bundled default profile");
        let observed = frozen_observed(&root);
        let mixes = root["mix_table"].as_object().expect("mix_table");
        let expected = root["gate_check_mix_result"].as_object().expect("results");

        for (id, mix) in mixes {
            let result = gate_check_mix_result(&profile, mix, true, observed.clone());
            let actual = sort_keys(serde_json::to_value(&result).expect("to_value"));
            let want = expected
                .get(id)
                .unwrap_or_else(|| panic!("missing golden for {id}"))
                .clone();
            let want = sort_keys(want);
            assert_eq!(
                canonical_bytes(&actual),
                canonical_bytes(&want),
                "gate_check_mix_result drift for mix_id={id}\nactual={actual}\nwant={want}"
            );
        }
    }

    /// Agent-layer binary: tools/list name set must be exactly the 13 tools.
    #[test]
    fn tools_list_agent_layer_thirteen_names() {
        let root = load_fixture_root();
        let expected = expected_names(&root, "agent_layer");
        assert_eq!(expected.len(), 13, "fixture agent_layer names");

        let (mut child, mut stdin, mut reader) = spawn_mcp();
        initialize(&mut stdin, &mut reader);

        write_frame(
            &mut stdin,
            &json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
        );
        let r2 = read_json_line(&mut reader);
        let names = tool_names_from_list_result(&r2);
        assert_eq!(
            names, expected,
            "agent-layer tools/list name set drifted from S0 golden"
        );

        let _ = child.kill();
        let _ = child.wait();
    }

    /// MCP `umst_gate_check` envelope matches cold library for `admissible` + `catalog_ids`.
    #[test]
    fn mcp_gate_check_matches_library_admissible_catalog() {
        let root = load_fixture_root();
        let profile = Profile::load_bundled("default").expect("profile");
        let observed = frozen_observed(&root);
        let mixes = root["mix_table"].as_object().expect("mix_table");

        let cwd = temp_mcp_cwd();
        let (mut child, mut stdin, mut reader) = spawn_mcp_in(Some(&cwd));
        initialize(&mut stdin, &mut reader);

        let mut next_id = 10u64;
        for (id, mix) in mixes {
            let cold = gate_check_mix_result(&profile, mix, true, observed.clone());

            write_frame(
                &mut stdin,
                &json!({
                    "jsonrpc": "2.0",
                    "id": next_id,
                    "method": "tools/call",
                    "params": {
                        "name": "umst_gate_check",
                        "arguments": {
                            "mix": mix,
                            "profile": "default",
                            "explain": true
                        }
                    }
                }),
            );
            let resp = read_json_line(&mut reader);
            assert_eq!(resp["id"], next_id);
            next_id += 1;

            let text = resp["result"]["content"][0]["text"].as_str().expect("text");
            let wire: Value = serde_json::from_str(text).expect("gate json");
            assert_eq!(
                wire["gate_summary"]["admissible"], cold.gate_summary.admissible,
                "admissible mismatch mix_id={id}"
            );
            assert_eq!(
                wire["gate_summary"]["catalog_ids"],
                serde_json::to_value(&cold.gate_summary.catalog_ids).unwrap(),
                "catalog_ids mismatch mix_id={id}"
            );
        }

        let _ = child.kill();
        let _ = child.wait();
        let _ = std::fs::remove_dir_all(&cwd);
    }

    fn call_frames_path() -> PathBuf {
        fixtures_dir().join("tools_call_result_frames_v0.json")
    }

    fn collect_call_frames(
        stdin: &mut impl Write,
        reader: &mut impl BufRead,
        root: &Value,
    ) -> Vec<Value> {
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

        let mut out = Vec::new();
        for req in &frames_req {
            write_frame(stdin, req);
            let resp = read_json_line(reader);
            let tool = req["params"]["name"].as_str().unwrap().to_string();
            out.push(json!({
                "tool": tool,
                "result": canonicalize_tools_call_result(&resp),
            }));
        }
        out
    }

    /// N golden `tools/call` `result` frames (ids/timestamps redacted).
    #[test]
    fn tools_call_result_frames_parity() {
        let root = load_fixture_root();
        let path = call_frames_path();

        let (mut child, mut stdin, mut reader) = spawn_mcp();
        initialize(&mut stdin, &mut reader);
        let frames = collect_call_frames(&mut stdin, &mut reader, &root);
        let _ = child.kill();
        let _ = child.wait();

        let update = std::env::var("UMST_GATE_PARITY_UPDATE").ok().as_deref() == Some("1");
        if update || !path.exists() {
            let doc = json!({
                "schema_version": "tools_call_result_frames_v0",
                "description": "S0 golden MCP tools/call result frames; ids/timestamps redacted; nested tool text canonicalized.",
                "frames": frames,
            });
            write_pretty_json(&path, &doc);
            if update {
                return;
            }
        }

        let text = std::fs::read_to_string(&path).expect("read call frames");
        let golden: Value = serde_json::from_str(&text).expect("parse call frames");
        let want = golden["frames"].as_array().expect("frames array");
        assert_eq!(frames.len(), want.len(), "frame count");
        for (i, (actual, expected)) in frames.iter().zip(want.iter()).enumerate() {
            assert_eq!(
                canonical_bytes(actual),
                canonical_bytes(expected),
                "tools/call result frame drift at index {i}"
            );
        }
    }

    fn write_pretty_json(path: &Path, v: &Value) {
        let s = serde_json::to_string_pretty(v).expect("pretty");
        std::fs::write(path, format!("{s}\n"))
            .unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    }
}
