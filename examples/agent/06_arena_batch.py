#!/usr/bin/env python3
"""In-process batch gate checks — prefer library path over MCP round-trips.

Runs `gate_check_mix` in a tight loop (same physics as `umst_gate_check`, no JSON-RPC).
Typical gain vs stdio MCP: **5–10×+** (CI enforces ≥5×). For parse-once arena bytes
and `UmstArenaView` hot reads, see `07_arena_mmap_load.py` and
`umst-manifold/docs/benchmarks/arena_vs_mcp.md`.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from mcp_rpc import repo_root


def main() -> int:
    fixture = repo_root() / "fixtures" / "golden-adversarial" / "admissible_mix_01.json"
    contribution = json.loads(fixture.read_text())
    mix = contribution["mix_spec"]
    iters = int(__import__("os").environ.get("UMST_BATCH_GATE_ITERS", "50"))

    proc = subprocess.run(
        [
            "cargo",
            "test",
            "-p",
            "umst-concrete-cartridge",
            "--features",
            "agent-layer",
            "--test",
            "inprocess_gate_batch",
            "inprocess_gate_batch_hot_loop",
            "--",
            "--exact",
            "--nocapture",
        ],
        cwd=repo_root(),
        env={**__import__("os").environ, "UMST_INPROCESS_GATE_ITERS": str(iters)},
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        sys.stderr.write(proc.stdout)
        sys.stderr.write(proc.stderr)
        return proc.returncode

    print(f"06_arena_batch: in-process gate batch ok ({iters} iters)")
    print("  mix keys:", list(mix.keys()))
    print("  fixture:", fixture)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
