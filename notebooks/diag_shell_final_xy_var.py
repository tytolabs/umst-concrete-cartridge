#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""Recompute `density_xy_plane_variance` on `examples/_artifacts/shell/final.npy` (same as `export_print_ready.py`).

Requires a local solve artefact (file is gitignored). Usage from `umst-concrete-cartridge/`:

  python notebooks/diag_shell_final_xy_var.py

Optional path override:

  python notebooks/diag_shell_final_xy_var.py /path/to/final.npy
"""
from __future__ import annotations

import json
import os
import sys
from pathlib import Path

import numpy as np

REPO = Path(__file__).resolve().parents[1]
WORKSPACE = REPO.parent
TO_ARCHIVED = Path(os.environ.get("UMST_TO_ARCHIVED_CRATE", WORKSPACE / "crates" / "umst-topology-opt-archived"))
SHELL = TO_ARCHIVED / "examples" / "_artifacts" / "shell"


def density_xy_plane_variance(rho: np.ndarray) -> float:
    nx1, ny1, nz1 = rho.shape
    sums = np.sum(rho, axis=2, dtype=np.float64)
    mean_z = sums / float(nz1)
    mean_all = float(mean_z.mean())
    return float(((mean_z - mean_all) ** 2).mean())


def main() -> None:
    npy = Path(sys.argv[1]) if len(sys.argv) > 1 else SHELL / "final.npy"
    man = SHELL / "manifest.json"
    if not npy.is_file():
        print(f"Missing {npy} — run optimize_shell_3d or pass explicit path.", file=sys.stderr)
        sys.exit(1)
    m = json.loads(man.read_text())
    nx1, ny1, nz1 = int(m["nx"]) + 1, int(m["ny"]) + 1, int(m["nz"]) + 1
    raw = np.load(npy, allow_pickle=False).astype(np.float32).reshape(-1)
    n = nx1 * ny1 * nz1
    if raw.size != n:
        print(f"Length {raw.size} != manifest nodes {n}", file=sys.stderr)
        sys.exit(2)
    rho = raw.reshape((nx1, ny1, nz1), order="F")
    v = density_xy_plane_variance(rho)
    nodal_vf = float(np.sum(raw.astype(np.float64)) / float(raw.size))
    print(f"path={npy}")
    print(f"density_xy_plane_variance={v:.12g}")
    print(f"nodal_volume_fraction={nodal_vf:.12g}")
    print(f"rho_min={float(rho.min()):.6g} rho_max={float(rho.max()):.6g}")


if __name__ == "__main__":
    main()
