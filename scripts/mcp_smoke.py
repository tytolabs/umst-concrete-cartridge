#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Smoke-test MCP stdio handshake + umst_predict (requires `cargo build -p umst-mcp`).

import json
import os
import subprocess
import sys
from pathlib import Path


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def mcp_bin() -> Path:
    profile = os.environ.get("PROFILE", "debug")
    return repo_root() / "target" / profile / "umst-mcp"


def main() -> int:
    exe = mcp_bin()
    if not exe.is_file():
        subprocess.run(["cargo", "build", "-p", "umst-mcp"], cwd=repo_root(), check=True)

    exe = mcp_bin()
    if not exe.is_file():
        print("FAILED: umst-mcp binary missing after build", file=sys.stderr)
        return 1

    proc = subprocess.Popen(
        [str(exe)],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        cwd=repo_root(),
        text=True,
    )

    def send(line: dict) -> None:
        assert proc.stdin
        proc.stdin.write(json.dumps(line) + "\n")
        proc.stdin.flush()

    frames = (
        {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
        {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "umst_predict",
                "arguments": {
                    "profile": "default",
                    "mix": {"w_c": 0.42, "temperature_k": 293.15},
                },
            },
        },
    )
    for frame in frames:
        send(frame)
    assert proc.stdin
    proc.stdin.close()

    decs = []
    for _ in range(3):
        line = proc.stdout.readline()
        if not line:
            print("unexpected EOF", file=sys.stderr)
            return 2
        decs.append(json.loads(line))

    for d in decs:
        if d.get("error"):
            print("RPC error:", d["error"])
            return 3

    txt = decs[-1]["result"]["content"][0]["text"]
    payload = json.loads(txt)
    if payload.get("schema_version") != "result.v2":
        print("BAD schema_version", payload)
        return 4

    proc.terminate()
    print("OK: MCP umst_predict result.v2")
    return 0


if __name__ == "__main__":
    sys.exit(main())
