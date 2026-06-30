# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
"""Re-render the last PNG frames with principal-compression streamlines (PyVista).

Reads `final.npy` and optional `final_sigma.npy` from the shell artefact directory
(same layout as `examples/optimize_shell_3d.rs`). If stress is absent, uses the
negative density gradient as a stand-in vector field so streamlines still read as
compression-leaning ribs on the iso-surface.
"""
from __future__ import annotations

import json
import os
from pathlib import Path

import numpy as np
import pyvista as pv

REPO = Path(__file__).resolve().parents[1]
WORKSPACE = REPO.parent
TO_ARCHIVED = Path(os.environ.get("UMST_TO_ARCHIVED_CRATE", WORKSPACE / "crates" / "umst-topology-opt-archived"))
SHELL = TO_ARCHIVED / "examples" / "_artifacts" / "shell"
FRAMES = REPO / "notebooks" / "_artifacts" / "frames"


def _load_manifest() -> dict:
    p = SHELL / "manifest.json"
    if not p.is_file():
        raise FileNotFoundError(f"missing {p}; run optimize_shell_3d first")
    return json.loads(p.read_text())


def _rho_grid(m: dict) -> tuple[np.ndarray, int, int, int, float, float, float]:
    nx1, ny1, nz1 = int(m["nx"]) + 1, int(m["ny"]) + 1, int(m["nz"]) + 1
    dx, dy, dz = float(m["dx"]), float(m["dy"]), float(m["dz"])
    arr = np.load(SHELL / "final.npy", allow_pickle=False).astype(np.float32)
    flat = arr.reshape(-1)
    n = nx1 * ny1 * nz1
    if flat.size != n:
        raise ValueError(f"final.npy length {flat.size} != grid nodes {n}")
    rho = flat.reshape((nx1, ny1, nz1), order="F")
    return rho, nx1, ny1, nz1, dx, dy, dz


def _node_ids(nx1: int, ny1: int, nz1: int) -> np.ndarray:
    ix = np.arange(nx1, dtype=np.int64)[:, None, None]
    iy = np.arange(ny1, dtype=np.int64)[None, :, None]
    iz = np.arange(nz1, dtype=np.int64)[None, None, :]
    return (ix + nx1 * iy + nx1 * ny1 * iz).astype(np.int64)


def _compression_vectors_from_voigt(
    voigt: np.ndarray, nx1: int, ny1: int, nz1: int
) -> np.ndarray:
    """Unit vectors (nx1, ny1, nz1, 3) along smallest principal stress."""
    n = nx1 * ny1 * nz1
    v = voigt.reshape(-1)
    if v.size != n * 6:
        raise ValueError(f"final_sigma.npy length {v.size} != {n * 6}")
    mat = v.reshape(n, 6)
    nid = _node_ids(nx1, ny1, nz1)
    g = mat[nid, :].astype(np.float64)
    sxx, syy, szz, sxy, syz, sxz = (g[..., i] for i in range(6))
    t = np.stack(
        [
            np.stack([sxx, sxy, sxz], axis=-1),
            np.stack([sxy, syy, syz], axis=-1),
            np.stack([sxz, syz, szz], axis=-1),
        ],
        axis=-2,
    )
    _, vecs = np.linalg.eigh(t)
    comp = vecs[..., 0]
    norms = np.linalg.norm(comp, axis=-1, keepdims=True) + 1e-12
    return (comp / norms).astype(np.float32)


def _compression_vectors_from_rho(rho: np.ndarray, dx: float, dy: float, dz: float) -> np.ndarray:
    """Proxy: negative gradient of density (toward higher ρ), normalised."""
    gx, gy, gz = np.gradient(rho.astype(np.float64), dx, dy, dz)
    comp = np.stack([-gx, -gy, -gz], axis=-1)
    norms = np.linalg.norm(comp, axis=-1, keepdims=True) + 1e-12
    return (comp / norms).astype(np.float32)


def main() -> None:
    paths = sorted(FRAMES.glob("frame_*.png"))
    if not len(paths):
        raise FileNotFoundError("no frames; run render_shell_gif.py first")
    tail = paths[-5:] if len(paths) >= 5 else paths

    m = _load_manifest()
    rho, nx1, ny1, nz1, dx, dy, dz = _rho_grid(m)
    sig_path = SHELL / "final_sigma.npy"
    if sig_path.is_file():
        voigt = np.load(sig_path, allow_pickle=False).astype(np.float32)
        if np.isfinite(voigt).all():
            try:
                vectors = _compression_vectors_from_voigt(voigt, nx1, ny1, nz1)
            except np.linalg.LinAlgError:
                vectors = _compression_vectors_from_rho(rho, dx, dy, dz)
        else:
            vectors = _compression_vectors_from_rho(rho, dx, dy, dz)
    else:
        vectors = _compression_vectors_from_rho(rho, dx, dy, dz)

    pv.OFF_SCREEN = True
    plotter = pv.Plotter(off_screen=True, window_size=(1280, 720))

    grid = pv.ImageData(
        dimensions=(nx1, ny1, nz1),
        spacing=(dx, dy, dz),
        origin=(0.0, 0.0, 0.0),
    )
    grid.point_data["rho"] = np.ascontiguousarray(rho.reshape(-1, order="F"))
    grid.point_data["vectors"] = np.ascontiguousarray(vectors.reshape(-1, 3, order="F"))

    span = float(np.max(rho) - np.min(rho))
    if span < 1e-3:
        thr = float(np.mean(rho)) - 1e-3
        blk = grid.threshold(thr, scalars="rho", invert=False)
        surf = blk.extract_surface(algorithm="dataset_surface")
    else:
        lo, hi = float(np.min(rho)), float(np.max(rho))
        # Match `export_print_ready.py`: SIMP 0.5 when the band brackets it; else band midpoint.
        iso = 0.5 if lo < 0.5 < hi else 0.5 * (lo + hi)
        surf = grid.contour(isosurfaces=[iso], scalars="rho")
    if surf.n_points == 0:
        plotter.close()
        raise RuntimeError("empty density iso-surface; cannot overlay streamlines")

    cx = 0.5 * (nx1 - 1) * dx
    cy = 0.5 * (ny1 - 1) * dy
    ztop = (nz1 - 1) * dz
    try:
        slines = grid.streamlines(
            vectors="vectors",
            max_time=2.8,
            integration_direction="both",
            source_radius=0.55 * min(dx * nx1, dy * ny1),
            source_center=(cx, cy, ztop * 0.92),
            max_steps=400,
            initial_step_length=0.08 * min(dx, dy, dz),
        )
    except Exception:
        slines = pv.PolyData()

    for out in tail:
        plotter.clear()
        plotter.add_mesh(surf, color="#c8d8ec", opacity=1.0, smooth_shading=True)
        if slines.n_points > 0:
            plotter.add_mesh(slines, color="#dc5a28", line_width=2.5)
        plotter.add_axes()
        plotter.camera_position = "iso"
        plotter.add_text("principal-compression streamlines", font_size=11)
        plotter.screenshot(str(out))

    plotter.close()


if __name__ == "__main__":
    main()
