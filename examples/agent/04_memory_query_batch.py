#!/usr/bin/env python3
"""Seed memory then run a batch of umst_memory_query filters (pagination pattern)."""

from __future__ import annotations

import json
import tempfile
from pathlib import Path

from mcp_rpc import call_tool, initialize, parse_tool_text, repo_root, spawn_mcp


def main() -> int:
    fixture_path = repo_root() / "fixtures" / "golden-adversarial" / "admissible_mix_01.json"
    contribution = json.loads(fixture_path.read_text())
    mix = contribution["mix_spec"]

    with tempfile.TemporaryDirectory() as tmp:
        db = str(Path(tmp) / "memory.db")
        proc = spawn_mcp(memory_db=db)
        try:
            initialize(proc)

            gate = call_tool(proc, 30, "umst_gate_check", {"mix": mix})
            assert gate.get("isError") is False, gate

            contrib = call_tool(
                proc,
                31,
                "umst_contribute",
                {"contribution": contribution},
            )
            assert contrib.get("isError") is not True, contrib

            queries = [
                ("broad", {"limit": 10}),
                ("near_mix", {"near_mix_spec": mix, "max_mix_l1": 0.1, "limit": 10}),
                (
                    "regime_miss",
                    {"curing_regime": "impossible_regime", "limit": 10},
                ),
                ("admissible_only", {"admissible_only": True, "limit": 10}),
            ]

            for idx, (name, args) in enumerate(queries):
                result = call_tool(proc, 40 + idx, "umst_memory_query", args)
                body = parse_tool_text(result)
                rows = body.get("rows", [])
                print(f"{name}: rows={len(rows)} cursor={body.get('next_cursor')!r}")
                if name == "regime_miss":
                    assert len(rows) == 0, body
                elif name in ("broad", "near_mix", "admissible_only"):
                    assert len(rows) >= 1, body

            print("04_memory_query_batch: ok")
            return 0
        finally:
            proc.terminate()
            proc.wait(timeout=120)


if __name__ == "__main__":
    raise SystemExit(main())
