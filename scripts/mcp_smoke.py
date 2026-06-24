#!/usr/bin/env python3
"""Minimal stdio MCP smoke test for umst-mcp."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any


def rpc(proc: subprocess.Popen[str], payload: dict[str, Any]) -> dict[str, Any]:
    assert proc.stdin and proc.stdout
    proc.stdin.write(json.dumps(payload) + "\n")
    proc.stdin.flush()
    line = proc.stdout.readline()
    if not line:
        raise RuntimeError("MCP server closed stdout")
    return json.loads(line)


def run_smoke(*, agent_layer: bool, witness_mode: str | None) -> None:
    features: list[str] = []
    if agent_layer:
        features.extend(["agent-layer", "ucrs-provenance"])

    cargo_args = ["cargo", "run", "-q", "-p", "umst-mcp"]
    if features:
        cargo_args.extend(["--features", ",".join(features)])

    env = os.environ.copy()
    if witness_mode is not None:
        env["UMST_UCRS_WITNESS"] = witness_mode

    proc = subprocess.Popen(
        cargo_args,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        cwd=str(Path(__file__).resolve().parents[1]),
        env=env,
    )

    try:
        init = rpc(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "mcp_smoke", "version": "0.1"},
                },
            },
        )
        assert "result" in init, init

        tools = rpc(proc, {"jsonrpc": "2.0", "id": 2, "method": "tools/list"})
        names = {t["name"] for t in tools["result"]["tools"]}
        for required in ("umst_predict", "umst_profiles"):
            assert required in names, f"missing tool {required}"

        # W0: base facade tools return agent_error.v1 on recoverable failures (not JSON-RPC -32603).
        bad_predict = rpc(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": {
                    "name": "umst_predict",
                    "arguments": {
                        "profile": "nonexistent_profile_xyz",
                        "mix": {"w_c": "9/20", "temperature_k": "29315/100"},
                    },
                },
            },
        )
        assert "result" in bad_predict, bad_predict
        assert "error" not in bad_predict, bad_predict
        assert bad_predict["result"].get("isError") is True, bad_predict
        pred_err = json.loads(bad_predict["result"]["content"][0]["text"])
        assert pred_err.get("agent_error", {}).get("schema_version") == "agent_error.v1"
        assert pred_err["agent_error"].get("remediation")

        bad_audit = rpc(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 6,
                "method": "tools/call",
                "params": {
                    "name": "umst_audit",
                    "arguments": {"profile": "default"},
                },
            },
        )
        assert bad_audit["result"].get("isError") is True, bad_audit
        audit_err = json.loads(bad_audit["result"]["content"][0]["text"])
        assert audit_err.get("agent_error", {}).get("code") == "audit_missing_csv"

        bad_certify = rpc(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 7,
                "method": "tools/call",
                "params": {"name": "umst_certify", "arguments": {}},
            },
        )
        assert bad_certify["result"].get("isError") is True, bad_certify
        certify_err = json.loads(bad_certify["result"]["content"][0]["text"])
        assert certify_err.get("agent_error", {}).get("code") == "certify_missing_profile"

        if agent_layer:
            for required in (
                "umst_gate_check",
                "umst_contribute",
                "umst_contribute_status",
                "umst_memory_query",
                "umst_mi_estimate",
            ):
                assert required in names, f"missing agent tool {required}"
            resources = rpc(proc, {"jsonrpc": "2.0", "id": 3, "method": "resources/list"})
            assert "resources" in resources["result"]

            # Phase 8: gate_check REJECT → isError + gate_reject.v1
            reject_mix = {
                "w_c": "not-rational",
                "temperature_k": "29315/100",
            }
            gate_reject = rpc(
                proc,
                {
                    "jsonrpc": "2.0",
                    "id": 10,
                    "method": "tools/call",
                    "params": {
                        "name": "umst_gate_check",
                        "arguments": {
                            "mix": reject_mix,
                            "explain": True,
                        },
                    },
                },
            )
            assert "result" in gate_reject, gate_reject
            assert gate_reject["result"].get("isError") is True, gate_reject
            body = json.loads(gate_reject["result"]["content"][0]["text"])
            assert body.get("gate_reject") is not None
            assert body["gate_reject"]["schema_version"] == "gate_reject.v1"
            assert body.get("explain", {}).get("regime_violations")
            assert body.get("explain", {}).get("remediation")
            assert len(body["explain"]["remediation"]) >= 1
            assert body.get("explain", {}).get("fields")

            gate_pass = rpc(
                proc,
                {
                    "jsonrpc": "2.0",
                    "id": 11,
                    "method": "tools/call",
                    "params": {
                        "name": "umst_gate_check",
                        "arguments": {
                            "mix": {
                                "w_c": "9/20",
                                "temperature_k": "29315/100",
                                "aggregate_volume_fraction": "7/10",
                            },
                        },
                    },
                },
            )
            assert gate_pass["result"].get("isError") is False, gate_pass

            mem = rpc(
                proc,
                {
                    "jsonrpc": "2.0",
                    "id": 12,
                    "method": "tools/call",
                    "params": {
                        "name": "umst_memory_query",
                        "arguments": {"limit": 10},
                    },
                },
            )
            mem_body = json.loads(mem["result"]["content"][0]["text"])
            assert "rows" in mem_body
            assert "next_cursor" in mem_body or mem_body.get("next_cursor") is None

            prompts = rpc(proc, {"jsonrpc": "2.0", "id": 13, "method": "prompts/list"})
            prompt_names = {p["name"] for p in prompts.get("result", {}).get("prompts", [])}
            for required_prompt in (
                "interpret_gate_failure",
                "safe-exploration",
                "suggest_similar_mix",
                "audit_mix_csv",
            ):
                assert required_prompt in prompt_names, f"missing prompt {required_prompt}"

            transition = rpc(
                proc,
                {
                    "jsonrpc": "2.0",
                    "id": 14,
                    "method": "tools/call",
                    "params": {
                        "name": "umst_transition_propose",
                        "arguments": {
                            "mix": {
                                "w_c": "9/20",
                                "temperature_k": "29315/100",
                                "aggregate_volume_fraction": "7/10",
                            },
                        },
                    },
                },
            )
            assert "result" in transition, transition
            trans_body = json.loads(transition["result"]["content"][0]["text"])
            assert "job_id" in trans_body
            assert "prediction" in trans_body

        profiles = rpc(
            proc,
            {
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {"name": "umst_profiles", "arguments": {}},
            },
        )
        assert "result" in profiles, profiles
        label = witness_mode or "default"
        print(f"mcp_smoke: ok (witness={label})", file=sys.stderr)
    finally:
        proc.terminate()
        proc.wait(timeout=120)


def run_memory_export_cli(repo_root: Path) -> None:
    out_dir = repo_root / "target" / "mcp_smoke_export"
    if out_dir.exists():
        import shutil

        shutil.rmtree(out_dir)
    proc = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "umst-cli",
            "--bin",
            "umst",
            "--features",
            "agent-layer",
            "--",
            "memory",
            "export",
            "--out",
            str(out_dir),
        ],
        cwd=str(repo_root),
        capture_output=True,
        text=True,
        check=True,
    )
    bundle = out_dir / "memory_export_bundle.v1.json"
    assert bundle.is_file(), proc.stderr
    body = json.loads(bundle.read_text())
    assert body.get("schema_version") == "memory_export_bundle.v1"
    print("mcp_smoke: memory export CLI ok", file=sys.stderr)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--agent-layer",
        action="store_true",
        help="Build/run umst-mcp with agent-layer + ucrs-provenance features",
    )
    args = parser.parse_args()

    run_smoke(agent_layer=args.agent_layer, witness_mode=None)

    if args.agent_layer:
        run_smoke(agent_layer=True, witness_mode="synthetic")
        run_smoke(agent_layer=True, witness_mode="live")
        run_memory_export_cli(Path(__file__).resolve().parents[1])

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
