#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""Arena mmap load hot loop — parse once, read `UmstArenaView` in a tight loop.

Categorical:
  Path: HOT (load_arena / UmstArenaView)
  Morphisms: parse-once → state_bytes() hot reads
  Docs: docs/FAST_ARENA.md

Performance path: `load_arena(bytes)` validates ABI v1 header + commit_stamp witness,
returns a zero-copy `UmstArenaView`; hot loops call `state_bytes()` without re-parsing.
Typical gain vs stdio MCP: **5–10×+** (CI enforces ≥5× via `bench_arena_vs_mcp.py`).

This script proxies the Rust CI test `arena_mmap_hot_loop` (see `crates/umst-mcp/tests/`).
For the in-process gate batch without arena bytes, use `06_arena_batch.py`.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

from mcp_rpc import repo_root


def main() -> int:
    iters = int(os.environ.get("UMST_ARENA_HOT_ITERS", "100"))
    proc = subprocess.run(
        [
            "cargo",
            "test",
            "-p",
            "umst-mcp",
            "--features",
            "agent-layer",
            "--test",
            "arena_mmap_hot_loop",
            "bench_arena_mmap_hot_loop",
            "--",
            "--exact",
            "--nocapture",
        ],
        cwd=repo_root(),
        env={**os.environ, "UMST_ARENA_HOT_ITERS": str(iters)},
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        sys.stderr.write(proc.stdout)
        sys.stderr.write(proc.stderr)
        return proc.returncode
    print(f"07_arena_mmap_load: arena hot loop ok ({iters} iters)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
