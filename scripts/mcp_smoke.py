#!/usr/bin/env python3
"""Minimal stdio MCP smoke test for umst-mcp."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from typing import Any


def rpc(proc: subprocess.Popen[str], payload: dict[str, Any]) -> dict[str, Any]:
    assert proc.stdin and proc.stdout
    proc.stdin.write(json.dumps(payload) + "\n")
    proc.stdin.flush()
    line = proc.stdout.readline()
    if not line:
        raise RuntimeError("MCP server closed stdout")
    return json.loads(line)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--agent-layer",
        action="store_true",
        help="Build/run umst-mcp with agent-layer feature",
    )
    args = parser.parse_args()

    features = ["agent-layer"] if args.agent_layer else []
    cargo_args = ["cargo", "run", "-q", "-p", "umst-mcp"]
    if features:
        cargo_args.extend(["--features", ",".join(features)])

    proc = subprocess.Popen(
        cargo_args,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        cwd=str(__import__("pathlib").Path(__file__).resolve().parents[1]),
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

        if args.agent_layer:
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
        print("mcp_smoke: ok", file=sys.stderr)
        return 0
    finally:
        proc.terminate()
        proc.wait(timeout=10)


if __name__ == "__main__":
    raise SystemExit(main())
