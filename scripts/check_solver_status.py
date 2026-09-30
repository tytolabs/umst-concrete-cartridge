#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
"""Run umst-manifold's `check_solver_status.py` against this repo's `docs/Solver-Status.md`.

From `umst-concrete-cartridge/`::

    python3 scripts/check_solver_status.py
    python3 scripts/check_solver_status.py --check-paths --check-memo-links

If `../umst-manifold/scripts/check_solver_status.py` is missing (no sibling checkout), exits **0**
with a short stderr note so local-only workflows do not hard-fail.
"""
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def main() -> int:
    here = Path(__file__).resolve().parent
    root = here.parent
    manifold_script = root.parent / "umst-manifold" / "scripts" / "check_solver_status.py"
    status_md = root / "docs" / "Solver-Status.md"
    manifold_root = root.parent / "umst-manifold"

    if not manifold_script.is_file():
        print(
            "check_solver_status: skip (no ../umst-manifold/scripts/check_solver_status.py)",
            file=sys.stderr,
        )
        return 0

    cmd = [
        sys.executable,
        str(manifold_script),
        "--status-md",
        str(status_md),
        "--root",
        str(manifold_root),
    ]
    # Default: same invariant checks as workspace CI (see docs/Solver-Status.md).
    if len(sys.argv) <= 1:
        cmd.extend(
            [
                "--check-paths",
                "--check-memo-links",
                "--check-statmech-verification-set",
            ]
        )
    else:
        cmd.extend(sys.argv[1:])

    env = os.environ.copy()
    return subprocess.call(cmd, cwd=str(root), env=env)


if __name__ == "__main__":
    raise SystemExit(main())
