#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
"""SVG → PNG → GIF pipeline for RC beam topology optimization.

Renders each frame as a crisp SVG (vector text), rasterises via cairosvg
at configurable supersample for retina-quality anti-aliasing, then assembles
an animated GIF + animated WebP with full 24-bit colour.

Environment (optional, read in ``main()``):

* ``UMST_BEAM_GIF_FRAME_MS`` — per-frame duration for GIF/WebP (default **200**).
* ``UMST_BEAM_GIF_HOLD_FRAMES`` — duplicate copies of the final frame at the end (default **8**).
* ``UMST_BEAM_GIF_HOLD_MS`` — if set, duplicated hold frames use this duration (ms); else they use ``UMST_BEAM_GIF_FRAME_MS``.
* ``UMST_BEAM_GIF_SUPERSAMPLE`` — integer ≥1 passed to cairosvg as width/height multiplier vs SVG px (default **2**).
* ``UMST_BEAM_GIF_MAX_SIDE`` — if set, downscale each raster frame so max(width,height) ≤ this value (aspect preserved).

Usage:
    python notebooks/render_beam_gif.py
"""
from __future__ import annotations
import io, json, math, os, sys, tempfile
from pathlib import Path
import numpy as np

try:
    import svgwrite
    import cairosvg
    from PIL import Image
except ImportError as e:
    print(f"Missing dep: {e}\n  .venv/bin/pip install svgwrite cairosvg Pillow")
    sys.exit(1)

REPO    = Path(__file__).resolve().parents[1]
ART_DIR = REPO / "crates/umst-concrete-cartridge/examples/_artifacts/beam"
OUT_DIR = REPO / "notebooks/_artifacts"

# ── Layout (SVG units = px at 1×; cairosvg renders at 2×) ────────────────────
CELL    = 40
PAD_L   = 110
PAD_R   = 260
PAD_T   = 115
PAD_B   = 95
INFO_H  = 150
SCALE   = 2        # cairosvg output_width multiplier

FONT    = "Helvetica, Arial, sans-serif"

# ── Colours ──────────────────────────────────────────────────────────────────
CONC = [
    "#12141c","#191e2a","#233044","#324664","#46648c",
    "#5f87af","#82a8cd","#a5c3dc","#c8d7e8","#e4e8f0","#f8f8fc",
]
STEEL      = "#ffb932"
STEEL_HI   = "#ffdc78"
ACCENT     = "#50c8ff"
LOAD       = "#ff5a46"
GREEN      = "#64dc8c"
PURPLE     = "#b48cff"
WARM       = "#ffc864"
CENTROID   = "#ff6450"
BG_TOP     = "#08080f"
BG_BOT     = "#10141e"
TEXT_MAIN  = "#c8cdd7"
TEXT_DIM   = "#646e7d"
TEXT_HI    = "#f0f0f5"
GRID       = "#1e2230"
BORDER     = "#323746"
SUPPORT    = "#828794"
PANEL_BG   = "#0c0e16"

def _density_hex(d):
    d = max(0.0, min(1.0, d))
    idx = d * (len(CONC) - 1)
    lo = int(idx); hi = min(lo + 1, len(CONC) - 1); t = idx - lo
    c1 = tuple(int(CONC[lo][i:i+2], 16) for i in (1, 3, 5))
    c2 = tuple(int(CONC[hi][i:i+2], 16) for i in (1, 3, 5))
    r = int(c1[0] + t*(c2[0]-c1[0]))
    g = int(c1[1] + t*(c2[1]-c1[1]))
    b = int(c1[2] + t*(c2[2]-c1[2]))
    return f"#{r:02x}{g:02x}{b:02x}"

def _phase(it, total):
    f = it / max(total, 1)
    if f < 0.12:
        return ("Phase 1 — Exploration",
                "Near-uniform density. The optimizer hasn't learned where stress concentrates.",
                ACCENT)
    elif f < 0.40:
        return ("Phase 2 — Load Path Discovery",
                "Dense material concentrates near the top and load point. Compression fan forming.",
                GREEN)
    elif f < 0.70:
        return ("Phase 3 — Material Removal",
                "Rows near the steel lose density — dead weight carved away by the optimizer.",
                WARM)
    else:
        return ("Phase 4 — Converged Strut-and-Tie",
                "Classical compression fan visible. Dense top, steel bottom — textbook strut-and-tie.",
                PURPLE)

# ── SVG frame builder ────────────────────────────────────────────────────────

def build_svg(rho, nx, ny, iteration, total, c_init, c_now, dx_m):
    beam_w = nx * CELL
    beam_h = ny * CELL
    span_m = nx * dx_m
    depth_m = ny * dx_m
    w = beam_w + PAD_L + PAD_R
    h = beam_h + PAD_T + PAD_B + INFO_H

    dwg = svgwrite.Drawing(size=(f"{w}px", f"{h}px"), profile="full")
    dwg.defs.add(dwg.linearGradient(
        id="bg", start=("0%","0%"), end=("0%","100%"),
        gradientUnits="userSpaceOnUse",
    ).add_stop_color(0, BG_TOP).add_stop_color(1, BG_BOT))
    dwg.add(dwg.rect((0,0), (w,h), fill="url(#bg)"))

    bx, by = PAD_L, PAD_T
    phase_name, phase_desc, phase_col = _phase(iteration, total)

    # ── Title block ──
    dwg.add(dwg.text("UMST — Reinforced Concrete Topology Optimization",
        insert=(bx, 22), fill=ACCENT, font_size="22px", font_family=FONT, font_weight="bold"))
    dwg.add(dwg.text(f"Simply-Supported Beam  ·  {span_m:.1f} m × {depth_m:.1f} m  ·  Neural SIMP + Adjoint Compliance",
        insert=(bx, 42), fill=TEXT_DIM, font_size="15px", font_family=FONT))
    dwg.add(dwg.text("C30/37 concrete (E = 30 GPa, ν = 0.2)  ·  B500B rebar (E = 200 GPa, bottom chord)",
        insert=(bx, 60), fill=TEXT_DIM, font_size="11px", font_family=FONT))
    dwg.add(dwg.text(phase_name,
        insert=(bx, 82), fill=phase_col, font_size="17px", font_family=FONT, font_weight="bold"))

    # Progress bar
    prog = iteration / max(total, 1)
    dwg.add(dwg.rect((bx, 92), (beam_w, 4), fill=GRID))
    if prog > 0:
        dwg.add(dwg.rect((bx, 92), (int(beam_w * prog), 4), fill=phase_col, opacity=0.9))
    dwg.add(dwg.text(f"{int(prog*100)}%",
        insert=(bx + beam_w + 8, 97), fill=TEXT_DIM, font_size="11px", font_family=FONT))

    # ── Beam cells ──
    rho_2d = np.array(rho).reshape(ny, nx)
    for iy in range(ny):
        for ix in range(nx):
            data_row = ny - 1 - iy
            d = float(rho_2d[data_row, ix])
            is_steel = data_row == 0
            col = STEEL if is_steel else _density_hex(d)
            x0, y0 = bx + ix * CELL, by + iy * CELL
            dwg.add(dwg.rect((x0, y0), (CELL, CELL), fill=col))

    # Grid
    for ix in range(nx + 1):
        dwg.add(dwg.line((bx + ix*CELL, by), (bx + ix*CELL, by + beam_h), stroke=GRID, stroke_width=0.5))
    for iy in range(ny + 1):
        dwg.add(dwg.line((bx, by + iy*CELL), (bx + beam_w, by + iy*CELL), stroke=GRID, stroke_width=0.5))
    dwg.add(dwg.rect((bx, by), (beam_w, beam_h), fill="none", stroke=BORDER, stroke_width=1.5))

    # Steel glow line
    dwg.add(dwg.line((bx, by + (ny-1)*CELL), (bx + beam_w, by + (ny-1)*CELL),
        stroke=STEEL_HI, stroke_width=2))

    # ── Structural centroid line ──
    centroid_pts = []
    for ix in range(nx):
        col_data = rho_2d[1:, ix]
        rows = np.arange(1, ny)
        total_rho = col_data.sum()
        ctr_row = np.sum(rows * col_data) / total_rho if total_rho > 1e-6 else ny / 2.0
        screen_y = by + (ny - 1 - ctr_row) * CELL + CELL / 2
        centroid_pts.append((bx + ix * CELL + CELL / 2, screen_y))

    cy_vals = [p[1] for p in centroid_pts]
    cy_range = max(cy_vals) - min(cy_vals)
    if cy_range > 4:
        pts_str = " ".join(f"{x:.1f},{y:.1f}" for x, y in centroid_pts)
        dwg.add(dwg.polyline(points=centroid_pts, fill="none",
            stroke=CENTROID, stroke_width=3, stroke_linecap="round",
            stroke_linejoin="round", opacity=0.85))
        # Glow
        dwg.add(dwg.polyline(points=centroid_pts, fill="none",
            stroke=CENTROID, stroke_width=6, stroke_linecap="round",
            opacity=0.25))
        # Dots
        for j in range(0, len(centroid_pts), 3):
            cx, cy = centroid_pts[j]
            dwg.add(dwg.circle((cx, cy), 3, fill=CENTROID))

        lx, ly = centroid_pts[1]
        dwg.add(dwg.text("Support →", insert=(lx + 6, ly - 8),
            fill=CENTROID, font_size="10px", font_family=FONT))
        rx, ry = centroid_pts[-2]
        dwg.add(dwg.text("← Load", insert=(rx - 50, ry - 8),
            fill=CENTROID, font_size="10px", font_family=FONT))
        mx, my = centroid_pts[nx // 2]
        dwg.add(dwg.text("Structural centroid", insert=(mx - 45, my + 16),
            fill="#c87864", font_size="10px", font_family=FONT))

    # ── Row-averaged density profile ──
    prof_x = bx + beam_w + 20
    prof_w = 90
    dwg.add(dwg.text("Row avg ρ", insert=(prof_x, by - 6),
        fill=TEXT_MAIN, font_size="12px", font_family=FONT, font_weight="bold"))
    for iy in range(ny):
        data_row = ny - 1 - iy
        y0 = by + iy * CELL + 2
        bh = CELL - 4
        if data_row == 0:
            dwg.add(dwg.rect((prof_x, y0), (prof_w, bh), fill=STEEL, stroke=STEEL_HI, stroke_width=0.5))
            dwg.add(dwg.text("steel", insert=(prof_x + prof_w + 4, y0 + bh/2 + 4),
                fill=STEEL, font_size="10px", font_family=FONT))
        else:
            rm = float(rho_2d[data_row, :].mean())
            bw = int(rm * prof_w)
            dwg.add(dwg.rect((prof_x, y0), (prof_w, bh), fill="none", stroke=GRID, stroke_width=0.5))
            dwg.add(dwg.rect((prof_x, y0), (bw, bh), fill=_density_hex(rm)))
            dwg.add(dwg.text(f"{rm:.2f}", insert=(prof_x + prof_w + 4, y0 + bh/2 + 4),
                fill=TEXT_DIM, font_size="10px", font_family=FONT))
    # Ticks
    for tick in [0.0, 0.5, 1.0]:
        tx = prof_x + int(tick * prof_w)
        dwg.add(dwg.line((tx, by + beam_h), (tx, by + beam_h + 4), stroke=TEXT_DIM, stroke_width=0.5))
        dwg.add(dwg.text(f"{tick:.1f}", insert=(tx - 6, by + beam_h + 14),
            fill=TEXT_DIM, font_size="9px", font_family=FONT))

    # ── Legend chips ──
    cy_l = by + beam_h + 26
    for i, (col, label) in enumerate([(STEEL, "Steel rebar (fixed)"),
                                       (_density_hex(0.85), "Dense concrete (strut)"),
                                       ("#12141c", "Void (dead weight)")]):
        yy = cy_l + i * 18
        dwg.add(dwg.rect((prof_x, yy), (12, 10), fill=col, stroke=BORDER, stroke_width=0.5))
        dwg.add(dwg.text(label, insert=(prof_x + 16, yy + 9),
            fill=TEXT_DIM, font_size="10px", font_family=FONT))
    if cy_range > 4:
        yy = cy_l + 3 * 18
        dwg.add(dwg.line((prof_x, yy + 5), (prof_x + 12, yy + 5), stroke=CENTROID, stroke_width=2.5))
        dwg.add(dwg.text("Structural centroid", insert=(prof_x + 16, yy + 9),
            fill=CENTROID, font_size="10px", font_family=FONT))

    # ── Supports ──
    sy = by + beam_h + 6; tri = 16
    # Pin
    px_l = bx + CELL / 2
    dwg.add(dwg.polygon([(px_l, sy), (px_l - 10, sy + tri), (px_l + 10, sy + tri)], fill=SUPPORT))
    dwg.add(dwg.line((px_l - 14, sy + tri + 2), (px_l + 14, sy + tri + 2), stroke=SUPPORT, stroke_width=2))
    dwg.add(dwg.text("PIN", insert=(px_l - 8, sy + tri + 16),
        fill=TEXT_DIM, font_size="10px", font_family=FONT, font_weight="bold"))
    # Roller
    px_r = bx + (nx - 1) * CELL + CELL / 2
    dwg.add(dwg.polygon([(px_r, sy), (px_r - 10, sy + tri), (px_r + 10, sy + tri)], fill=SUPPORT))
    dwg.add(dwg.circle((px_r, sy + tri + 5), 4, fill=SUPPORT))
    dwg.add(dwg.line((px_r - 14, sy + tri + 11), (px_r + 14, sy + tri + 11), stroke=SUPPORT, stroke_width=2))
    dwg.add(dwg.text("ROLLER", insert=(px_r - 14, sy + tri + 25),
        fill=TEXT_DIM, font_size="10px", font_family=FONT, font_weight="bold"))

    # ── Load arrow ──
    ax = bx + (nx - 1) * CELL + CELL / 2
    dwg.add(dwg.line((ax, by - 40), (ax, by - 6), stroke=LOAD, stroke_width=3))
    dwg.add(dwg.polygon([(ax, by - 3), (ax - 7, by - 14), (ax + 7, by - 14)], fill=LOAD))
    dwg.add(dwg.text("F = 50 kN", insert=(ax - 70, by - 35),
        fill=LOAD, font_size="13px", font_family=FONT, font_weight="bold"))

    # ── Dimension lines ──
    dim_y = by + beam_h + 70
    dwg.add(dwg.line((bx, dim_y), (bx + beam_w, dim_y), stroke=TEXT_DIM, stroke_width=0.7))
    dwg.add(dwg.line((bx, dim_y - 4), (bx, dim_y + 4), stroke=TEXT_DIM))
    dwg.add(dwg.line((bx + beam_w, dim_y - 4), (bx + beam_w, dim_y + 4), stroke=TEXT_DIM))
    dwg.add(dwg.text(f"L = {span_m:.1f} m", insert=(bx + beam_w/2 - 25, dim_y + 14),
        fill=TEXT_DIM, font_size="11px", font_family=FONT))
    # Depth
    dx = bx - 28
    dwg.add(dwg.line((dx, by), (dx, by + beam_h), stroke=TEXT_DIM, stroke_width=0.7))
    dwg.add(dwg.line((dx - 4, by), (dx + 4, by), stroke=TEXT_DIM))
    dwg.add(dwg.line((dx - 4, by + beam_h), (dx + 4, by + beam_h), stroke=TEXT_DIM))
    dwg.add(dwg.text(f"h = {depth_m:.1f} m", insert=(bx - 105, by + beam_h/2 + 4),
        fill=TEXT_DIM, font_size="11px", font_family=FONT))

    # ── Bottom panel ──
    py = h - INFO_H
    dwg.add(dwg.rect((0, py), (w, INFO_H), fill=PANEL_BG))
    dwg.add(dwg.line((0, py), (w, py), stroke="#232834", stroke_width=1))

    c1, c2, c3 = PAD_L, PAD_L + 260, PAD_L + 500
    c_drop = (1 - c_now / max(c_init, 1e-12)) * 100 if c_init > 0 else 0
    comp_col = GREEN if c_drop > 20 else TEXT_MAIN
    rho_span = float(np.max(rho) - np.min(rho))
    vf = float(np.mean(rho))
    span_col = ACCENT if rho_span > 0.3 else LOAD

    dwg.add(dwg.text(f"Iteration  {iteration} / {total}",
        insert=(c1, py + 20), fill=TEXT_HI, font_size="14px", font_family=FONT))
    dwg.add(dwg.text(f"Compliance:  {c_now:.0f}",
        insert=(c2, py + 20), fill=comp_col, font_size="14px", font_family=FONT))
    dwg.add(dwg.text(f"Stiffness gain:  {c_drop:.1f}%",
        insert=(c3, py + 20), fill=comp_col, font_size="14px", font_family=FONT))
    dwg.add(dwg.text(f"ρ span:  {rho_span:.3f}",
        insert=(c1, py + 42), fill=span_col, font_size="14px", font_family=FONT))
    dwg.add(dwg.text(f"Volume fraction:  {vf:.3f}  ({vf*100:.0f}% material)",
        insert=(c2, py + 42), fill=TEXT_MAIN, font_size="14px", font_family=FONT))

    dwg.add(dwg.text(phase_desc,
        insert=(c1, py + 70), fill=TEXT_MAIN, font_size="13px", font_family=FONT))

    dwg.add(dwg.text("Dense rows = load-carrying compression (top).  Light rows above steel = removed dead weight.",
        insert=(c1, py + 100), fill=TEXT_DIM, font_size="10px", font_family=FONT))
    dwg.add(dwg.text("The centroid line traces where structural mass concentrates — the compression resultant in a Strut-and-Tie model.",
        insert=(c1, py + 115), fill=TEXT_DIM, font_size="10px", font_family=FONT))

    return dwg, w, h


def svg_to_pil(dwg, w, h, supersample: int):
    """Render SVG to PIL Image at ``supersample``× resolution via cairosvg."""
    ss = max(1, int(supersample))
    svg_bytes = dwg.tostring().encode("utf-8")
    png_bytes = cairosvg.svg2png(
        bytestring=svg_bytes,
        output_width=w * ss,
        output_height=h * ss,
    )
    return Image.open(io.BytesIO(png_bytes)).convert("RGB")


def _env_int(name: str, default: int, *, minimum: int = 0) -> int:
    raw = os.environ.get(name)
    if raw is None or not str(raw).strip():
        return max(minimum, default)
    try:
        return max(minimum, int(raw))
    except ValueError:
        return max(minimum, default)


def _maybe_cap_max_side(im: Image.Image, max_side: int | None) -> Image.Image:
    if max_side is None or max_side <= 0:
        return im
    w, h = im.size
    m = max(w, h)
    if m <= max_side:
        return im
    scale = max_side / float(m)
    nw = max(1, int(round(w * scale)))
    nh = max(1, int(round(h * scale)))
    return im.resize((nw, nh), Image.LANCZOS)


def main():
    manifest_path = ART_DIR / "manifest.json"
    if not manifest_path.is_file():
        print(f"ERROR: {manifest_path} not found. Run optimize_rc_beam first.")
        sys.exit(1)
    with open(manifest_path) as f:
        m = json.load(f)

    frame_ms = _env_int("UMST_BEAM_GIF_FRAME_MS", 200, minimum=1)
    hold_frames = _env_int("UMST_BEAM_GIF_HOLD_FRAMES", 8, minimum=0)
    hold_ms_raw = os.environ.get("UMST_BEAM_GIF_HOLD_MS")
    if hold_ms_raw is not None and str(hold_ms_raw).strip():
        try:
            hold_ms = max(1, int(hold_ms_raw))
        except ValueError:
            hold_ms = frame_ms
    else:
        hold_ms = frame_ms
    supersample = _env_int("UMST_BEAM_GIF_SUPERSAMPLE", SCALE, minimum=1)
    max_side_raw = os.environ.get("UMST_BEAM_GIF_MAX_SIDE")
    max_side = None
    if max_side_raw is not None and str(max_side_raw).strip():
        try:
            max_side = max(1, int(max_side_raw))
        except ValueError:
            max_side = None

    nx, ny = int(m["nx"]), int(m["ny"])
    total = int(m["iters"]); n = nx * ny
    dx_m = float(m.get("dx", 0.1))
    c_init = float(m.get("compliance_initial", 1.0))
    c_final = float(m.get("compliance_final", c_init))

    files = sorted(ART_DIR.glob("iter_*.npy"))
    if not files:
        print(f"ERROR: no iter_*.npy in {ART_DIR}"); sys.exit(1)

    out_w = (nx * CELL + PAD_L + PAD_R) * supersample
    print(
        f"SVG→PNG→GIF pipeline: {len(files)} frames, {out_w}px wide ({supersample}×), "
        f"frame_ms={frame_ms}, hold_frames={hold_frames}, hold_ms={hold_ms}"
    )

    frames = []
    for i, fpath in enumerate(files):
        it = int(fpath.stem.replace("iter_", ""))
        arr = np.load(fpath, allow_pickle=False).astype(np.float32).reshape(-1)
        if arr.size != n:
            continue
        t = it / max(total, 1)
        c_now = c_init + t * (c_final - c_init)
        dwg, w, h = build_svg(arr, nx, ny, it, total, c_init, c_now, dx_m)
        pil = _maybe_cap_max_side(svg_to_pil(dwg, w, h, supersample), max_side)
        frames.append(pil)
        sys.stdout.write(f"\r  rendered {i+1}/{len(files)}")
        sys.stdout.flush()
    print()

    n_content = len(frames)
    # Hold final (duplicate raster frames for end pause)
    for _ in range(hold_frames):
        frames.append(frames[-1])

    OUT_DIR.mkdir(parents=True, exist_ok=True)

    # Save final frame SVG
    svg_path = OUT_DIR / "beam_strut_and_tie_final.svg"
    last_arr = np.load(files[-1], allow_pickle=False).astype(np.float32).reshape(-1)
    last_dwg, lw, lh = build_svg(last_arr, nx, ny, total, total, c_init, c_final, dx_m)
    last_dwg.saveas(str(svg_path), pretty=True)
    print(f"  SVG: {svg_path}")

    # Save final frame PNG (hi-res)
    png_path = OUT_DIR / "beam_strut_and_tie_final.png"
    frames[-1].save(str(png_path))
    print(f"  PNG: {png_path} ({frames[-1].size[0]}×{frames[-1].size[1]})")

    n_body = n_content
    n_hold = hold_frames
    webp_durations = [frame_ms] * n_body + [hold_ms] * n_hold
    if len(webp_durations) != len(frames):
        webp_durations = [frame_ms] * len(frames)

    # Animated WebP (full colour)
    webp_path = OUT_DIR / "beam_strut_and_tie.webp"
    try:
        frames[0].save(
            str(webp_path),
            save_all=True,
            append_images=frames[1:],
            duration=webp_durations,
            loop=0,
            quality=90,
        )
    except TypeError:
        frames[0].save(
            str(webp_path),
            save_all=True,
            append_images=frames[1:],
            duration=frame_ms,
            loop=0,
            quality=90,
        )
    print(f"  WebP: {webp_path} ({webp_path.stat().st_size // 1024} KB)")

    # Animated GIF (quantised, but from hi-res source)
    gif_path = OUT_DIR / "beam_strut_and_tie.gif"
    gif_frames = [
        f.quantize(colors=256, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.FLOYDSTEINBERG)
        for f in frames
    ]
    gif_durations = [frame_ms] * n_body + [hold_ms] * n_hold
    if len(gif_durations) != len(gif_frames):
        gif_durations = [frame_ms] * len(gif_frames)
    gif_frames[0].save(
        str(gif_path),
        save_all=True,
        append_images=gif_frames[1:],
        duration=gif_durations,
        loop=0,
        optimize=True,
    )
    print(f"  GIF: {gif_path} ({gif_path.stat().st_size // 1024} KB)")

    print("Done.")


if __name__ == "__main__":
    main()
