# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
"""Render PNG frames for the Striatus shell demo (PyVista, offscreen)."""
from __future__ import annotations

import json
from pathlib import Path

import numpy as np
import pyvista as pv

REPO = Path(__file__).resolve().parents[1]
SHELL = REPO / "crates" / "umst-concrete-cartridge" / "examples" / "_artifacts" / "shell"
FRAMES = REPO / "notebooks" / "_artifacts" / "frames"


def _load_manifest() -> dict:
    p = SHELL / "manifest.json"
    if not p.is_file():
        raise FileNotFoundError(f"missing {p}; run optimize_shell_3d first")
    return json.loads(p.read_text())


def _load_rho_final(m: dict) -> np.ndarray:
    p = SHELL / "final.npy"
    if not p.is_file():
        raise FileNotFoundError(f"missing {p}")
    arr = np.load(p, allow_pickle=False).astype(np.float32)
    nx1, ny1, nz1 = int(m["nx"]) + 1, int(m["ny"]) + 1, int(m["nz"]) + 1
    expected = nx1 * ny1 * nz1
    flat = arr.reshape(-1)
    if flat.size != expected:
        raise ValueError(f"final.npy size {flat.size} != grid {expected}")
    # Same node order as `optimize_shell_3d` / `ExtrudedPlateMechanics`:
    # `nid = ix + (nx+1)*iy + (nx+1)*(ny+1)*iz` with ix,iy,iz on the node grid.
    return flat.reshape((nx1, ny1, nz1), order="F")


def _iter_npys():
    return sorted(SHELL.glob("iter_*.npy"))


def main() -> None:
    FRAMES.mkdir(parents=True, exist_ok=True)
    for old in FRAMES.glob("frame_*.png"):
        old.unlink()

    m = _load_manifest()
    nx1, ny1, nz1 = int(m["nx"]) + 1, int(m["ny"]) + 1, int(m["nz"]) + 1
    dx, dy, dz = float(m["dx"]), float(m["dy"]), float(m["dz"])
    rho_final = _load_rho_final(m)
    iters = _iter_npys()

    pv.OFF_SCREEN = True
    plotter = pv.Plotter(off_screen=True, window_size=(1280, 720))

    def render_volume(rho: np.ndarray, tag: str, frame_idx: int) -> None:
        grid = pv.ImageData(
            dimensions=(nx1, ny1, nz1),
            spacing=(dx, dy, dz),
            origin=(0.0, 0.0, 0.0),
        )
        grid.point_data["rho"] = np.ascontiguousarray(rho.reshape(-1, order="F"))
        span = float(np.max(rho) - np.min(rho))
        if span < 1e-3:
            thr = float(np.mean(rho)) - 1e-3
            blk = grid.threshold(thr, scalars="rho", invert=False)
            surf = blk.extract_surface(algorithm="dataset_surface")
        else:
            surf = grid.contour(isosurfaces=[0.5], scalars="rho")
        plotter.clear()
        if surf.n_points == 0:
            plotter.add_text(tag + " (empty surface)", font_size=12)
        else:
            plotter.add_mesh(surf, color="#c4d4e8", opacity=1.0, smooth_shading=True)
        plotter.add_axes()
        plotter.camera_position = "iso"
        plotter.add_text(tag, font_size=12)
        out = FRAMES / f"frame_{frame_idx:04d}.png"
        plotter.screenshot(str(out))

    frame = 0
    if iters:
        for ip, path in enumerate(iters):
            arr = np.load(path, allow_pickle=False).astype(np.float32).reshape((nx1, ny1, nz1), order="F")
            tag = f"{path.name}"
            render_volume(arr, tag, frame)
            frame += 1
    else:
        n_frame = 48
        u = np.full_like(rho_final, 0.08, dtype=np.float32)
        for i in range(n_frame):
            t = (i + 1) / float(n_frame)
            rho = u * (1.0 - t) + rho_final * t
            render_volume(rho, f"morph t={t:.2f}", frame)
            frame += 1

    for _ in range(8):
        render_volume(rho_final, "final density", frame)
        frame += 1

    plotter.close()


if __name__ == "__main__":
    main()
