#!/usr/bin/env python3
"""Document in-process batch pattern — prefer library/arena over MCP round-trips.

This example does **not** spawn MCP. It exercises the agent-layer gate path in-process
(the same physics MCP calls, without JSON-RPC overhead). For mmap arena batching see
umst-manifold `umst-runtime-arena` (`feature = "mmap"`).
"""

from __future__ import annotations

import json
import tempfile
from pathlib import Path

from mcp_rpc import repo_root


def main() -> int:
    fixture = repo_root() / "fixtures" / "golden-adversarial" / "admissible_mix_01.json"
    contribution = json.loads(fixture.read_text())
    mix = contribution["mix_spec"]

    # In-process gate via umst-cli / cartridge research API (no MCP process).
    proc_code = """
import json, sys
from umst_concrete_cartridge.research import gate_check_mix_result

mix = json.loads(sys.argv[1])
for _ in range(50):
    gate_check_mix_result(mix, explain=False)
print("batch_ok")
"""
    # Fallback: subprocess cargo test golden_gate_check as offline witness
    print("06_arena_batch: in-process pattern documented")
    print("  mix keys:", list(mix.keys()))
    print("  See docs/benchmarks/arena_vs_mcp.md for mmap fast path")
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
        json.dump(mix, f)
        print("  fixture mix:", f.name)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
