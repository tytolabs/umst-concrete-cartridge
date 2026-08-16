#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""
Continuous spiral / helical toolpath for axis-aligned print volumes.

Designed for concrete extrusion planning (ABB TCP waypoints, GH import, CSV).
One polyline per layer (Archimedean spiral) or one continuous helix (optional).

Geometry model
--------------
- Footprint: axis-aligned rectangle [x0,x1]×[y0,y1] or circle inscribed in it.
- Bead width w ≈ nozzle outer diameter (adjust for overlap).
- Layer pitch p = layer height (Z step between turns in helical mode).
- Archimedean spiral: r(θ) = r_min + (w * (1 - overlap)) * θ / (2π)  (constant radial step per turn)
- Points resampled at arc-length spacing ≤ w/2 for smooth robot motion.

Outputs: CSV (x,y,z,mm/min extrusion flag), JSON metadata. No RAPID emitter (use GH or your post).

Example (50 mm cube, 8 mm nozzle, 4 mm layers):
  python3 scripts/spiral_toolpath.py --size 50 50 40 --nozzle 8 --layer-height 4 --out /tmp/spiral.csv
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterator, List, Sequence, Tuple

Point = Tuple[float, float, float]


@dataclass(frozen=True)
class NozzleProfile:
    """Extrusion / motion knobs (all lengths in mm, speeds in mm/s)."""

    nozzle_diameter_mm: float = 8.0
    layer_height_mm: float = 4.0
    overlap: float = 0.15  # fraction of bead width (0 = tangent beads, 0.2 = 20% overlap)
    feed_rate_mm_s: float = 25.0
  # Optional: cross-section area model for volumetric E (not emitted as G-code E here)
    flow_utilization: float = 0.85  # rectangular bead ≈ w * h * utilization

    @property
    def bead_step_mm(self) -> float:
        w = max(self.nozzle_diameter_mm, 0.1)
        return w * (1.0 - max(0.0, min(0.95, self.overlap)))

    @property
    def bead_area_mm2(self) -> float:
        return (
            self.nozzle_diameter_mm
            * self.layer_height_mm
            * max(0.1, min(1.0, self.flow_utilization))
        )


@dataclass(frozen=True)
class Volume:
    """Axis-aligned print box in mm (robot workobject frame)."""

    origin: Tuple[float, float, float] = (0.0, 0.0, 0.0)
    size: Tuple[float, float, float] = (50.0, 50.0, 40.0)  # lx, ly, lz

    @property
    def x0(self) -> float:
        return self.origin[0]

    @property
    def y0(self) -> float:
        return self.origin[1]

    @property
    def z0(self) -> float:
        return self.origin[2]

    @property
    def lx(self) -> float:
        return self.size[0]

    @property
    def ly(self) -> float:
        return self.size[1]

    @property
    def lz(self) -> float:
        return self.size[2]

    @property
    def center_xy(self) -> Tuple[float, float]:
        return (self.x0 + 0.5 * self.lx, self.y0 + 0.5 * self.ly)

    @property
    def inset_radius(self) -> float:
        """Max spiral radius keeping bead center inside the rectangle."""
        w = 0.0  # set by caller via half-bead inset
        return min(self.lx, self.ly) * 0.5 - w


def _resample_polyline(pts: List[Point], spacing_mm: float) -> List[Point]:
    if len(pts) < 2 or spacing_mm <= 0:
        return pts
    out: List[Point] = [pts[0]]
    carry = 0.0
    for i in range(1, len(pts)):
        x0, y0, z0 = out[-1]
        x1, y1, z1 = pts[i]
        dx, dy, dz = x1 - x0, y1 - y0, z1 - z0
        seg = math.sqrt(dx * dx + dy * dy + dz * dz)
        if seg < 1e-9:
            continue
        ux, uy, uz = dx / seg, dy / seg, dz / seg
        dist = carry
        while dist + spacing_mm <= seg:
            dist += spacing_mm
            t = dist / seg
            out.append((x0 + ux * dist, y0 + uy * dist, z0 + uz * dist))
        carry = seg - dist
        if i == len(pts) - 1 and (out[-1][0] - x1) ** 2 + (out[-1][1] - y1) ** 2 + (out[-1][2] - z1) ** 2 > 1e-6:
            out.append((x1, y1, z1))
    return out


def archimedean_spiral_layer(
    volume: Volume,
    profile: NozzleProfile,
    z: float,
    *,
    shape: str = "rect",
) -> List[Point]:
    """
  One continuous in-plane spiral from outer inset to center (CCW).
  Rect footprint: spiral in polar coords clipped implicitly by max radius.
    """
    cx, cy = volume.center_xy
    half_w = profile.nozzle_diameter_mm * 0.5
    r_max = min(volume.lx, volume.ly) * 0.5 - half_w
    r_min = profile.bead_step_mm * 0.5
    if r_max <= r_min:
        return [(cx, cy, z)]

    step = profile.bead_step_mm
    # θ such that Δr ≈ step: dr/dθ = step/(2π) => b = step/(2π)
    b = step / (2.0 * math.pi)
    pts: List[Point] = []
    theta = 0.0
    r = r_max
    # Start at outer edge on +X from center
    while r >= r_min:
        x = cx + r * math.cos(theta)
        y = cy + r * math.sin(theta)
        pts.append((x, y, z))
        theta += 0.12  # rad; finer → smoother, more points
        r = r_max - b * theta
    pts.append((cx, cy, z))
    return _resample_polyline(pts, spacing_mm=min(step * 0.5, 2.0))


def connect_layers(
    layers: List[List[Point]],
    *,
    ramp_z_mm: float,
    ramp_arc_mm: float = 0.0,
) -> List[Point]:
    """Bridge layer ends to next layer starts with a **diagonal seam ramp**.

    Instead of climbing vertically at the seam (pump-pause blob), the Z-change is
    distributed across the next ``ramp_arc_mm`` of arc length on the new layer.
    The seam apex starts at z_prev; each point along the new layer has its Z linearly
    remapped to ``z_prev + dz · (s/ramp_arc_mm)`` until ramp_arc_mm of arc is consumed,
    then continues at the new z. Matches the Rust ``ramp_first_seam_climb`` policy.

    ``ramp_z_mm`` retained for API compatibility — currently unused; the legacy
    "N vertical micro-steps" behaviour is replaced unconditionally.
    """
    _ = ramp_z_mm  # legacy arg, intentionally unused
    if not layers:
        return []
    path: List[Point] = list(layers[0])
    for nxt in layers[1:]:
        if not nxt:
            continue
        x_prev, y_prev, z_prev = path[-1]
        z_top = nxt[0][2]
        dz = z_top - z_prev
        if abs(dz) < 1e-6 or not nxt:
            path.extend(nxt)
            continue
        # If no arc length was specified, fall back to a sensible default.
        L = ramp_arc_mm if ramp_arc_mm > 0.0 else 20.0
        # Anchor the seam apex at z_prev (instead of jumping to z_top), then ramp Z
        # along the new layer until we have consumed L of in-plane arc length.
        # First waypoint of new layer is held at z_prev (the apex).
        first = nxt[0]
        path.append((first[0], first[1], z_prev))
        s = 0.0
        ramped_through = 0
        for i in range(1, len(nxt)):
            px, py, _ = nxt[i]
            qx, qy, _ = nxt[i - 1]
            s += math.hypot(px - qx, py - qy)
            t = min(s / L, 1.0)
            path.append((px, py, z_prev + dz * t))
            ramped_through = i
            if t >= 1.0:
                break
        # Append the rest of the new layer at its native z_top.
        for i in range(ramped_through + 1, len(nxt)):
            path.append(nxt[i])
    return path


def plan_layered_spiral(
    volume: Volume,
    profile: NozzleProfile,
    *,
    shape: str = "rect",
) -> List[Point]:
    n_layers = max(1, int(volume.lz / max(profile.layer_height_mm, 0.1)))
    layers: List[List[Point]] = []
    for layer in range(n_layers):
        z = volume.z0 + (layer + 0.5) * profile.layer_height_mm
        layers.append(archimedean_spiral_layer(volume, profile, z, shape=shape))
    # Match the Rust core policy: ramp_arc = max(2·nozzle, 20mm). Pump never stop-blobs.
    ramp_arc = max(2.0 * profile.nozzle_diameter_mm, 20.0)
    return connect_layers(
        layers,
        ramp_z_mm=profile.layer_height_mm,
        ramp_arc_mm=ramp_arc,
    )


def continuous_helix(
    volume: Volume,
    profile: NozzleProfile,
) -> List[Point]:
    """
    Single continuous helical path (cylindrical footprint inscribed in volume).
    Good for round beads; square cube uses inscribed circle (wastes corners) unless
    you use layered spiral instead.
    """
    cx, cy = volume.center_xy
    half_w = profile.nozzle_diameter_mm * 0.5
    r_max = min(volume.lx, volume.ly) * 0.5 - half_w
    r_min = profile.bead_step_mm
    step = profile.bead_step_mm
    pitch_z = profile.layer_height_mm
    b = step / (2.0 * math.pi)
    pts: List[Point] = []
    theta = 0.0
    z = volume.z0 + 0.5 * pitch_z
    z_top = volume.z0 + volume.lz - 0.5 * pitch_z
    while z <= z_top and r_max - b * theta >= r_min:
        r = r_max - b * theta
        x = cx + r * math.cos(theta)
        y = cy + r * math.sin(theta)
        pts.append((x, y, z))
        theta += 0.1
        z += (pitch_z / (2.0 * math.pi)) * 0.1  # tie Z advance to θ
    return _resample_polyline(pts, spacing_mm=min(step * 0.5, 2.0))


def cumulative_extrusion_mm(
    path: Sequence[Point], profile: NozzleProfile
) -> List[float]:
    """Volumetric proxy: ΔE ∝ path_length * bead_area / (nozzle diameter) — for relative sync only."""
    e = [0.0]
    area = profile.bead_area_mm2
    w = max(profile.nozzle_diameter_mm, 0.1)
    for i in range(1, len(path)):
        dx = path[i][0] - path[i - 1][0]
        dy = path[i][1] - path[i - 1][1]
        dz = path[i][2] - path[i - 1][2]
        ds = math.sqrt(dx * dx + dy * dy + dz * dz)
        e.append(e[-1] + ds * area / w)
    return e


def write_csv(
    path: Path,
    points: Sequence[Point],
    profile: NozzleProfile,
    e_vals: Sequence[float],
) -> None:
    f_mm_min = profile.feed_rate_mm_s * 60.0
    with path.open("w", encoding="utf-8") as f:
        f.write("x_mm,y_mm,z_mm,e_rel_mm,f_mm_min,extrude_on\n")
        for (x, y, z), e in zip(points, e_vals):
            f.write(f"{x:.4f},{y:.4f},{z:.4f},{e:.4f},{f_mm_min:.2f},1\n")


def write_json_meta(path: Path, meta: dict) -> None:
    path.write_text(json.dumps(meta, indent=2), encoding="utf-8")


def main(argv: Sequence[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--origin", type=float, nargs=3, default=(0.0, 0.0, 0.0), metavar=("X", "Y", "Z"))
    p.add_argument("--size", type=float, nargs=3, default=(50.0, 50.0, 40.0), metavar=("LX", "LY", "LZ"))
    p.add_argument("--nozzle", type=float, default=8.0, help="Nozzle / bead width (mm)")
    p.add_argument("--layer-height", type=float, default=4.0, help="Layer height (mm)")
    p.add_argument("--overlap", type=float, default=0.15, help="Bead overlap fraction 0..0.95")
    p.add_argument("--feed", type=float, default=25.0, help="TCP speed (mm/s)")
    p.add_argument(
        "--mode",
        choices=("layer-spiral", "helix"),
        default="layer-spiral",
        help="layer-spiral: one continuous spiral per layer; helix: single 3D helix (circular footprint)",
    )
    p.add_argument("--out", type=Path, required=True, help="Output .csv path")
    p.add_argument("--meta", type=Path, default=None, help="Optional JSON sidecar")
    args = p.parse_args(list(argv) if argv is not None else None)

    volume = Volume(origin=tuple(args.origin), size=tuple(args.size))
    profile = NozzleProfile(
        nozzle_diameter_mm=args.nozzle,
        layer_height_mm=args.layer_height,
        overlap=args.overlap,
        feed_rate_mm_s=args.feed,
    )
    if volume.lx <= 0 or volume.ly <= 0 or volume.lz <= 0:
        print("error: size must be positive", file=sys.stderr)
        return 1

    if args.mode == "helix":
        path = continuous_helix(volume, profile)
    else:
        path = plan_layered_spiral(volume, profile)

    if len(path) < 2:
        print("error: degenerate path", file=sys.stderr)
        return 1

    e_vals = cumulative_extrusion_mm(path, profile)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    write_csv(args.out, path, profile, e_vals)
    meta = {
        "volume": asdict(volume),
        "profile": asdict(profile),
        "mode": args.mode,
        "n_points": len(path),
        "path_length_mm": sum(
            math.sqrt(
                (path[i][0] - path[i - 1][0]) ** 2
                + (path[i][1] - path[i - 1][1]) ** 2
                + (path[i][2] - path[i - 1][2]) ** 2
            )
            for i in range(1, len(path))
        ),
        "notes": "Import CSV into GH (CSV Read). Map columns to robtargets; e_rel is relative only.",
    }
    meta_path = args.meta or args.out.with_suffix(".json")
    write_json_meta(meta_path, meta)
    print(f"wrote {len(path)} points -> {args.out}")
    print(f"meta -> {meta_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
