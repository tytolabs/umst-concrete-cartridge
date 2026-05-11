# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
"""Marching-cubes STL export + print-readiness sidecar JSON for the Striatus shell demo.

Consumes `final.npy` (`float32`, shape `[1, N, 1]`) and `manifest.json` written by
`examples/optimize_shell_3d.rs` (node order `ix + nx1*iy + nx1*ny1*iz`, same as
`ExtrudedPlateMechanics`).

Track B7/B8 (v0.4): sidecar includes mesh genus / component counts, density XY variance
(z-averaged, same construction as `shell_topology_rib_pattern` `xy_plane_variance`), marching-cubes
volume fraction in bbox, and boolean gates for non-slab topology. Isosurface uses **0.5** when
`min(ρ) < 0.5 < max(ρ)`; otherwise the midpoint of the observed band (needed while ρ is still
entirely below 0.5). Nearly uniform fields (`max(ρ)−min(ρ) < 1e-3`) abort export instead of the
legacy threshold slab that inflated STL volume to ~100% of bbox.
"""
from __future__ import annotations

import hashlib
import json
import math
import sys
import time
from pathlib import Path

import numpy as np
import pyvista as pv
import trimesh

REPO = Path(__file__).resolve().parents[1]
SHELL = REPO / "crates" / "umst-concrete-cartridge" / "examples" / "_artifacts" / "shell"
OUT_DIR = REPO / "notebooks" / "_artifacts"

# v0.4 artefact names (B8); keep v0.3 filenames as aliases for older scripts until removed.
ART_VERSION = "v0.4"


def _sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def _circumradius(tri_pts: np.ndarray) -> float:
    a = float(np.linalg.norm(tri_pts[1] - tri_pts[0]))
    b = float(np.linalg.norm(tri_pts[2] - tri_pts[1]))
    c = float(np.linalg.norm(tri_pts[0] - tri_pts[2]))
    s = 0.5 * (a + b + c)
    area_sq = s * (s - a) * (s - b) * (s - c)
    if area_sq <= 0.0:
        return 0.0
    area = math.sqrt(area_sq)
    return (a * b * c) / (4.0 * area)


def density_xy_plane_variance(rho: np.ndarray) -> float:
    """Mean z-averaged ρ variance over the XY lattice (full Striatus / `shell_topology_rib_pattern_full_v04` gate); quick Rust CI uses a **top-slice** variance."""
    nx1, ny1, nz1 = rho.shape
    sums = np.sum(rho, axis=2, dtype=np.float64)
    mean_z = sums / float(nz1)
    mean_all = float(mean_z.mean())
    return float(((mean_z - mean_all) ** 2).mean())


def mesh_topology_metrics(tm: trimesh.Trimesh) -> dict[str, float | int | None]:
    """Connected components, Euler χ, closed-orientable genus estimate (largest part by |volume|)."""
    parts = tm.split()
    n_comp = int(len(parts))
    main = max(parts, key=lambda m: abs(float(m.volume)))
    chi: int | None = None
    genus: float | None = None
    if hasattr(main, "euler_number"):
        try:
            chi = int(round(float(main.euler_number)))
            if main.is_watertight and main.is_winding_consistent:
                genus = (2.0 - float(chi)) / 2.0
        except Exception:
            chi = None
            genus = None
    return {
        "mesh_connected_components": n_comp,
        "mesh_euler_characteristic_largest": chi,
        "mesh_genus_closed_orientable_largest": genus,
    }


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    m = json.loads((SHELL / "manifest.json").read_text())
    nx1, ny1, nz1 = int(m["nx"]) + 1, int(m["ny"]) + 1, int(m["nz"]) + 1
    dx, dy, dz = float(m["dx"]), float(m["dy"]), float(m["dz"])
    raw = np.load(SHELL / "final.npy", allow_pickle=False).astype(np.float32).reshape(-1)
    n = nx1 * ny1 * nz1
    if raw.size != n:
        print(f"ERROR: final.npy length {raw.size} != nodes {n}", file=sys.stderr)
        sys.exit(1)
    rho = raw.reshape((nx1, ny1, nz1), order="F")

    grid = pv.ImageData(
        dimensions=(nx1, ny1, nz1),
        spacing=(dx, dy, dz),
        origin=(0.0, 0.0, 0.0),
    )
    grid.point_data["rho"] = np.ascontiguousarray(rho.reshape(-1, order="F"))
    lo, hi = float(rho.min()), float(rho.max())
    span = hi - lo
    # Nearly uniform ρ (e.g. first few Adam outers): old code used `mean(ρ)-1e-3`, which selected
    # almost every voxel when mean(ρ)≈0.15 — Trimesh then saw a solid block (genus 0, VF≈1).
    if span < 1e-3:
        print(
            "ERROR: ρ span {:.3g} < 1e-3 — field is still essentially uniform; "
            "re-run optimize_shell_3d with a larger UMST_SHELL_ITERS (Track L / B8 uses 200 on 40×40×4).".format(
                span
            ),
            file=sys.stderr,
        )
        sys.exit(8)
    # `contour([0.5])` is empty while the projected field stays entirely below 0.5 (common mid-run).
    # Use the standard SIMP isovalue when the range brackets 0.5; otherwise split the observed band.
    iso = 0.5 if lo < 0.5 < hi else 0.5 * (lo + hi)
    surf = grid.contour(isosurfaces=[iso], scalars="rho")
    surf = surf.triangulate()

    if surf.n_points == 0:
        print("ERROR: empty surface from density field", file=sys.stderr)
        sys.exit(4)

    if hasattr(surf, "is_manifold") and not bool(surf.is_manifold):
        print(
            "warn: PyVista surface reports non-manifold before STL export "
            "(common for marching-cubes); continuing — Trimesh watertight check is authoritative.",
            file=sys.stderr,
        )

    stl_path = OUT_DIR / f"striatus_shell_{ART_VERSION}.stl"
    obj_path = OUT_DIR / f"striatus_shell_{ART_VERSION}.obj"

    faces = surf.faces.reshape((-1, 4))[:, 1:4]
    tm = trimesh.Trimesh(surf.points, faces, process=True)
    if not tm.is_watertight:
        print("ERROR: mesh not watertight (trimesh)", file=sys.stderr)
        sys.exit(2)
    if hasattr(tm, "is_winding_consistent") and not tm.is_winding_consistent:
        print("ERROR: mesh winding inconsistent", file=sys.stderr)
        sys.exit(3)

    min_r = float("inf")
    for tri in tm.triangles:
        min_r = min(min_r, _circumradius(tri))
    min_r_mm = min_r * 1000.0
    if min_r_mm < 6.0:
        print(f"ERROR: min circumradius {min_r_mm:.3f} mm < 6 mm", file=sys.stderr)
        sys.exit(6)

    zn = np.array([0.0, 0.0, -1.0], dtype=np.float64)
    max_ang = 0.0
    for nvec in tm.face_normals:
        nn = nvec / (np.linalg.norm(nvec) + 1e-12)
        c = max(-1.0, min(1.0, float(np.dot(nn, zn))))
        ang = math.degrees(math.acos(c))
        if ang > max_ang:
            max_ang = ang
    if max_ang > 30.0 + 1e-3:
        print(f"ERROR: max overhang {max_ang:.3f} deg > 30 deg", file=sys.stderr)
        sys.exit(7)

    voxel_vol_m3 = float(m["dx"]) * float(m["dy"]) * float(m["dz"])
    lx, ly, lz = float(m["lx"]), float(m["ly"]), float(m["lz"])
    domain_m3 = lx * ly * lz
    vf = float(np.mean(rho))
    material_vol_m3 = max(
        float(np.sum(rho > 0.5)) * voxel_vol_m3,
        vf * domain_m3,
    )
    material_vol_cm3 = material_vol_m3 * 1e6
    mesh_vol_cm3 = float(abs(tm.volume) * 1e6) if tm.is_watertight else 0.0

    bbox_mm = (tm.bounds[1] - tm.bounds[0]) * 1000.0
    total_vol_cm3 = float(max(material_vol_cm3, mesh_vol_cm3))

    extents_m = tm.extents.astype(np.float64)
    bbox_vol_m3 = float(np.prod(np.maximum(extents_m, 1e-30)))
    mesh_vol_m3 = float(abs(tm.volume))
    mesh_vf_in_bbox = mesh_vol_m3 / bbox_vol_m3 if bbox_vol_m3 > 0.0 else 0.0

    dens_xy_var = density_xy_plane_variance(rho)
    topo = mesh_topology_metrics(tm)
    genus = topo["mesh_genus_closed_orientable_largest"]
    n_cc = int(topo["mesh_connected_components"])

    # B7: genus ≥ 1 OR ≥ 4 components; reject χ > 1.5 on largest part (sphere/slab-like).
    chi = topo["mesh_euler_characteristic_largest"]
    chi_ok = chi is not None and float(chi) <= 1.5 + 1e-6
    topo_signal = (genus is not None and genus >= 1.0 - 1e-6) or n_cc >= 4
    gate_topo_complexity = bool(chi_ok and topo_signal)

    gate_volume_fraction_mesh = 0.10 <= mesh_vf_in_bbox <= 0.25
    gate_density_xy_variance = dens_xy_var >= 0.1 - 1e-6

    # B8 objective gates (AND): genus, density variance, mesh VF band.
    gates_track_b8_all = bool(
        gate_topo_complexity and gate_density_xy_variance and gate_volume_fraction_mesh
    )

    sidecar = {
        "artefact_version": ART_VERSION,
        "total_volume_cm3": total_vol_cm3,
        "material_volume_cm3": float(material_vol_cm3),
        "surface_area_cm2": float(tm.area * 1e4),
        "bbox_mm": [float(x) for x in bbox_mm.tolist()],
        "min_feature_size_mm": float(min_r_mm),
        "max_overhang_deg": float(max_ang),
        "polygon_count": int(len(tm.faces)),
        "sha256_stl": "",
        "demo_config_hash": hashlib.sha256(json.dumps(m, sort_keys=True).encode()).hexdigest(),
        "timestamp_unix": int(time.time()),
        "density_xy_plane_variance": float(dens_xy_var),
        "mesh_volume_fraction_in_bbox": float(mesh_vf_in_bbox),
        "mesh_connected_components": n_cc,
        "mesh_euler_characteristic_largest": chi,
        "mesh_genus_closed_orientable_largest": genus,
        "contour_isovalue": float(iso),
        "gate_topo_complexity_b7": gate_topo_complexity,
        "gate_volume_fraction_mesh_b7": gate_volume_fraction_mesh,
        "gate_density_xy_variance_b8": gate_density_xy_variance,
        "gates_track_b8_all_pass": gates_track_b8_all,
    }
    tm.export(str(stl_path))
    tm.export(str(obj_path))
    sidecar["sha256_stl"] = _sha256_file(stl_path)
    json_path = OUT_DIR / f"striatus_shell_{ART_VERSION}.print_ready.json"
    json_path.write_text(json.dumps(sidecar, indent=2))

    # Back-compat symlinks / copies for scripts still keyed to v0.3.
    v03_stl = OUT_DIR / "striatus_shell_v0.3.stl"
    v03_json = OUT_DIR / "striatus_shell_v0.3.print_ready.json"
    v03_obj = OUT_DIR / "striatus_shell_v0.3.obj"
    try:
        if v03_stl.exists() or v03_stl.is_symlink():
            v03_stl.unlink()
        v03_stl.symlink_to(stl_path.name)
    except OSError:
        v03_stl.write_bytes(stl_path.read_bytes())
    try:
        if v03_json.exists() or v03_json.is_symlink():
            v03_json.unlink()
        v03_json.symlink_to(json_path.name)
    except OSError:
        v03_json.write_text(json_path.read_text(encoding="utf-8"), encoding="utf-8")
    try:
        if v03_obj.exists() or v03_obj.is_symlink():
            v03_obj.unlink()
        v03_obj.symlink_to(obj_path.name)
    except OSError:
        v03_obj.write_bytes(obj_path.read_bytes())


if __name__ == "__main__":
    main()
