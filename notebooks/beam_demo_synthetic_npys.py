#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Write `manifest.json` + `iter_*.npy` for `render_beam_gif.py` when the Rust example fails.

Manifest keys mirror `optimize_rc_beam.rs` (``umst_beam_dump_v2`` shape) so tooling stays aligned;
the renderer reads grid sizes, iteration cadence, and optional ``compliance_*`` fields for the GIF.
"""

from __future__ import annotations

import json
import os
import struct
import sys
from pathlib import Path


def _parse_positive_int(name: str, raw: str | None, default: int) -> int:
    if raw is None or raw.strip() == "":
        return max(1, default)
    try:
        v = int(raw, 10)
    except ValueError:
        print(f"ERROR: {name} must be a positive integer, got {raw!r}", file=sys.stderr)
        sys.exit(1)
    if v < 1:
        print(f"ERROR: {name} must be >= 1, got {v}", file=sys.stderr)
        sys.exit(1)
    return v


def dump_epochs(epochs: int, stride: int) -> list[int]:
    """Match `optimize_rc_beam.rs`: dump when epoch==1, epoch%stride==0, or epoch==epochs."""
    out: list[int] = []
    for epoch in range(1, epochs + 1):
        if epoch == 1 or epoch % stride == 0 or epoch == epochs:
            out.append(epoch)
    return out


def write_npy_f32(path: Path, data: list[float], shape: tuple[int, ...]) -> None:
    expected = shape[0] * shape[1] * shape[2]
    if len(data) != expected:
        raise ValueError(f"len(data)={len(data)} != product(shape)={expected}")
    path.parent.mkdir(parents=True, exist_ok=True)
    shape_lit = ", ".join(str(s) for s in shape)
    header_dict = "{'descr': '<f4', 'fortran_order': False, 'shape': (%s), }" % shape_lit
    header = bytes(header_dict, "ascii")
    while (len(header) + 10) % 64:
        header += b" "
    header += b"\n"
    if len(header) > 65535:
        raise ValueError("npy header too large")
    out = bytearray()
    out += b"\x93NUMPY"
    out += bytes([1, 0])
    out += struct.pack("<H", len(header))
    out += header
    for x in data:
        out += struct.pack("<f", x)
    path.write_bytes(out)


def main() -> None:
    repo = Path(__file__).resolve().parents[1]
    # Same discrete grid as `optimize_rc_beam.rs`.
    nx, ny = 32, 8
    n_nodes = nx * ny
    dx = 0.1
    batch = 1

    epochs = _parse_positive_int("UMST_BEAM_ITERS", os.environ.get("UMST_BEAM_ITERS"), 90)
    stride = _parse_positive_int("UMST_BEAM_DUMP_STRIDE", os.environ.get("UMST_BEAM_DUMP_STRIDE"), 3)

    art = repo / "crates/umst-concrete-cartridge/examples/_artifacts/beam"
    art.mkdir(parents=True, exist_ok=True)
    for p in art.glob("iter_*.npy"):
        p.unlink(missing_ok=True)

    iterations = dump_epochs(epochs, stride)
    for it in iterations:
        # Smooth fake density field (distinct per frame); bottom row reads as fixed steel in the renderer.
        rho: list[float] = []
        base = it * 0.02
        for j in range(ny):
            for i in range(nx):
                r = float(i) / max(nx - 1, 1)
                c = float(j) / max(ny - 1, 1)
                d = max(0.0, min(1.0, 0.3 + base + 0.4 * r - 0.15 * c))
                if j == 0:
                    d = 1.0
                rho.append(d)
        write_npy_f32(art / f"iter_{it:03}.npy", rho, (1, n_nodes, 1))

    # Plausible compliance endpoints so `render_beam_gif.py` can draw the schematic strip.
    c0, cf, cb = 1.0e4, 6.2e3, 5.9e3
    manifest = {
        "schema": "umst_beam_dump_v2",
        "nx": nx,
        "ny": ny,
        "iters": epochs,
        "dump_stride": stride,
        "n_nodes": n_nodes,
        "dx": dx,
        "batch": batch,
        "compliance_initial": c0,
        "compliance_final": cf,
        "compliance_best": cb,
        "synthetic_demo": True,
    }
    (art / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    print(f"wrote synthetic beam artefacts ({len(iterations)} frames) under {art}")


if __name__ == "__main__":
    main()
