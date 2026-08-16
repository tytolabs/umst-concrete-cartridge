#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""Shared stdio JSON-RPC helper for umst-mcp agent examples."""

from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path
from typing import Any


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def rpc(proc: subprocess.Popen[str], payload: dict[str, Any]) -> dict[str, Any]:
    assert proc.stdin and proc.stdout
    proc.stdin.write(json.dumps(payload) + "\n")
    proc.stdin.flush()
    line = proc.stdout.readline()
    if not line:
        raise RuntimeError("MCP server closed stdout")
    return json.loads(line)


def spawn_mcp(*, memory_db: str | None = None) -> subprocess.Popen[str]:
    env = os.environ.copy()
    if memory_db:
        env["UMST_MEMORY_DB"] = memory_db
    return subprocess.Popen(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "umst-mcp",
            "--features",
            "agent-layer",
        ],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        cwd=str(repo_root()),
        env=env,
    )


def initialize(proc: subprocess.Popen[str]) -> None:
    init = rpc(
        proc,
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "agent-example", "version": "0.1"},
            },
        },
    )
    if "result" not in init:
        raise RuntimeError(f"initialize failed: {init}")


def call_tool(proc: subprocess.Popen[str], tool_id: int, name: str, arguments: dict[str, Any]) -> dict[str, Any]:
    frame = rpc(
        proc,
        {
            "jsonrpc": "2.0",
            "id": tool_id,
            "method": "tools/call",
            "params": {"name": name, "arguments": arguments},
        },
    )
    if "result" not in frame:
        raise RuntimeError(f"{name} failed: {frame}")
    return frame["result"]


def parse_tool_text(result: dict[str, Any]) -> dict[str, Any]:
    return json.loads(result["content"][0]["text"])
