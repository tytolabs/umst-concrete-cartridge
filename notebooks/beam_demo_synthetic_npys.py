#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Write minimal `manifest.json` + `iter_*.npy` for `render_beam_gif.py` when the Rust example hits a Burn scatter bug."""

from __future__ import annotations

import json
import struct
from pathlib import Path


def write_npy_f32(path: Path, data: list[float], shape: tuple[int, ...]) -> None:
    assert len(data) == (ne := shape[0] * shape[1] * shape[2]), f"{len(data)} != {ne}"
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
    nx, ny = 20, 5
    n = nx * ny
    art = repo / "crates/umst-concrete-cartridge/examples/_artifacts/beam"
    for p in art.glob("iter_*.npy"):
        p.unlink()

    iterations = [1, 4, 7, 10, 15]
    for it in iterations:
        # Smooth fake density field (distinct per frame).
        rho = []
        base = it * 0.02
        for j in range(ny):
            for i in range(nx):
                r = float(i) / max(nx - 1, 1)
                c = float(j) / max(ny - 1, 1)
                rho.append(max(0.0, min(1.0, 0.3 + base + 0.4 * r - 0.15 * c)))
        write_npy_f32(art / f"iter_{it:03}.npy", rho, (1, n, 1))

    manifest = {
        "nx": nx,
        "ny": ny,
        "iters": iterations[-1],
        "dump_stride": 3,
        "synthetic_demo": True,
    }
    (art / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
    print(f"wrote synthetic beam artefacts under {art}")


if __name__ == "__main__":
    main()
