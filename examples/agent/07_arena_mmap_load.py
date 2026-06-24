#!/usr/bin/env python3
"""Arena mmap load hot loop — CI proxy for `umst-runtime-arena` bench_load_arena_hot_loop."""

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
