#!/usr/bin/env python3
"""MCP arena session workflow — open → gate_check_arena → close.

Categorical:
  Path: HOT (MCP arena trio)
  Morphisms: umst_arena_open → umst_gate_check_arena → umst_arena_close
  Docs: docs/FAST_ARENA.md · docs/TOOL_CONTRACTS.md

Use when an external agent owns the MCP process but needs repeated gate checks:
`umst_arena_open` parses arena bytes once (Warm boundary); subsequent
`umst_gate_check_arena` calls skip per-proposal JSON parse overhead.
For highest throughput in your own process, prefer `06_arena_batch.py` or
`load_arena` / mmap directly (see `07_arena_mmap_load.py`).
"""

from __future__ import annotations

import json
import struct
import sys
import tempfile
from pathlib import Path

from mcp_rpc import call_tool, initialize, parse_tool_text, repo_root, spawn_mcp


def write_minimal_arena(path: Path) -> None:
    buf = bytearray(72)
    struct.pack_into("<I", buf, 0, 0x5453_4D55)
    struct.pack_into("<I", buf, 4, 1)
    struct.pack_into("<I", buf, 8, 64)
    struct.pack_into("<Q", buf, 48, 64)
    struct.pack_into("<Q", buf, 56, 8)
    buf[64:] = bytes([0x42] * 8)
    path.write_bytes(buf)


def load_fixture_mix(name: str) -> dict:
    path = repo_root() / "fixtures" / "golden-adversarial" / name
    return json.loads(path.read_text())["mix_spec"]


def main() -> int:
    fixture_dir = repo_root() / "fixtures" / "arena"
    fixture_dir.mkdir(parents=True, exist_ok=True)
    arena_path = fixture_dir / "minimal.v1.bin"
    write_minimal_arena(arena_path)

    mix = load_fixture_mix("admissible_mix_01.json")
    proc = spawn_mcp()
    try:
        initialize(proc)

        opened = call_tool(
            proc,
            20,
            "umst_arena_open",
            {"arena_path": str(arena_path.relative_to(repo_root()))},
        )
        open_body = parse_tool_text(opened)
        session_id = open_body["arena_session_id"]
        print("arena_session_id:", session_id)

        gate = call_tool(
            proc,
            21,
            "umst_gate_check_arena",
            {"arena_session_id": session_id, "mix": mix},
        )
        assert gate.get("isError") is False, gate
        gate_body = parse_tool_text(gate)
        print("gate verdict:", gate_body["gate_summary"]["verdict"])

        closed = call_tool(
            proc,
            22,
            "umst_arena_close",
            {"arena_session_id": session_id},
        )
        close_body = parse_tool_text(closed)
        assert close_body.get("closed") == session_id

        print("08_arena_mcp_session: ok")
        return 0
    finally:
        proc.terminate()
        proc.wait(timeout=120)


if __name__ == "__main__":
    raise SystemExit(main())
