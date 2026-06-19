#!/usr/bin/env python3
"""Safe exploration: gate check (reject + pass) and memory query."""

from __future__ import annotations

import json
import sys

from mcp_rpc import call_tool, initialize, parse_tool_text, repo_root, spawn_mcp


def load_fixture_mix(name: str) -> dict:
    path = repo_root() / "fixtures" / "golden-adversarial" / name
    data = json.loads(path.read_text())
    return data["mix_spec"]


def main() -> int:
    proc = spawn_mcp()
    try:
        initialize(proc)

        reject = call_tool(
            proc,
            10,
            "umst_gate_check",
            {"mix": {"w_c": "not-rational", "temperature_k": "29315/100"}},
        )
        assert reject.get("isError") is True
        reject_body = parse_tool_text(reject)
        remediation = reject_body.get("explain", {}).get("remediation", [])
        print("REJECT remediation:", remediation[0] if remediation else "(none)")
        assert remediation, "expected remediation on parse fail"

        admix = load_fixture_mix("admissible_mix_01.json")
        gate_pass = call_tool(proc, 11, "umst_gate_check", {"mix": admix})
        assert gate_pass.get("isError") is False, gate_pass
        print("PASS gate_summary:", parse_tool_text(gate_pass)["gate_summary"]["verdict"])

        mem = call_tool(proc, 12, "umst_memory_query", {"limit": 10})
        mem_body = parse_tool_text(mem)
        print(f"memory_query rows: {len(mem_body.get('rows', []))}")
        assert "rows" in mem_body

        print("01_gate_explore: ok")
        return 0
    finally:
        proc.terminate()
        proc.wait(timeout=120)


if __name__ == "__main__":
    raise SystemExit(main())
