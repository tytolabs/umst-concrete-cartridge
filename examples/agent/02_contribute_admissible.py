#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""Gate pass → contribute admissible fixture → memory query.

Categorical:
  Objects: Contribution, MemoryStore row
  Morphisms: umst_gate_check → umst_contribute → umst_memory_query (COLD)
  Guarantees: G1 contribute-after-PASS only
  Provenance: observed_at / UCRS stamp fields on contribute response
"""

from __future__ import annotations

import json
import sys
import tempfile
from pathlib import Path

from mcp_rpc import call_tool, initialize, parse_tool_text, repo_root, spawn_mcp


def main() -> int:
    fixture_path = repo_root() / "fixtures" / "golden-adversarial" / "admissible_mix_01.json"
    contribution = json.loads(fixture_path.read_text())

    with tempfile.TemporaryDirectory() as tmp:
        db = str(Path(tmp) / "memory.db")
        proc = spawn_mcp(memory_db=db)
        try:
            initialize(proc)

            mix = contribution["mix_spec"]
            gate = call_tool(proc, 20, "umst_gate_check", {"mix": mix})
            assert gate.get("isError") is False, gate

            contrib = call_tool(
                proc,
                21,
                "umst_contribute",
                {"contribution": contribution},
            )
            assert contrib.get("isError") is not True, contrib
            accept = parse_tool_text(contrib)
            print("contributed memory_id:", accept.get("memory_id"))
            assert accept.get("memory_id")

            query = call_tool(
                proc,
                22,
                "umst_memory_query",
                {"limit": 10, "admissible_only": True},
            )
            rows = parse_tool_text(query).get("rows", [])
            print(f"memory_query rows after contribute: {len(rows)}")
            assert len(rows) >= 1

            print("02_contribute_admissible: ok")
            return 0
        finally:
            proc.terminate()
            proc.wait(timeout=120)


if __name__ == "__main__":
    raise SystemExit(main())
