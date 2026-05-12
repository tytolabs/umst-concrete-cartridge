#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
"""Render RC beam Strut-and-Tie topology optimization as a premium animated GIF.

Reads iter_*.npy from _artifacts/beam/ and produces a publication-quality
beam_strut_and_tie.gif with a dark engineering aesthetic.

Usage:
    python notebooks/render_beam_gif.py
"""
from __future__ import annotations

import glob
import json
import os
import sys
from pathlib import Path

import numpy as np

try:
    from PIL import Image, ImageDraw
except ImportError:
    print("ERROR: pip install Pillow")
    sys.exit(1)

REPO = Path(__file__).resolve().parents[1]
ART_DIR = REPO / "crates" / "umst-concrete-cartridge" / "examples" / "_artifacts" / "beam"
OUT_DIR = REPO / "notebooks" / "_artifacts"

# --- Premium dark palette ---
BG_TOP = (10, 10, 18)
BG_BOT = (20, 24, 35)

# Steel rebar: warm amber with metallic feel
STEEL_CORE = (255, 185, 55)
STEEL_EDGE = (200, 145, 40)

# Concrete density ramp: dark → warm concrete white
CONCRETE_RAMP = [
    (22, 24, 32),     # 0.00 - void (near-black)
    (30, 35, 48),     # 0.05
    (40, 52, 72),     # 0.15
    (55, 75, 105),    # 0.25
    (75, 105, 145),   # 0.35
    (100, 140, 180),  # 0.45
    (135, 170, 205),  # 0.55
    (170, 195, 220),  # 0.65
    (200, 215, 230),  # 0.75
    (225, 232, 240),  # 0.85
    (245, 245, 248),  # 0.95 - solid concrete (warm white)
]

ACCENT = (80, 200, 255)       # Cyan accent
ACCENT_WARM = (255, 130, 70)  # Warm orange for load
TEXT_COLOR = (195, 200, 210)
TEXT_DIM = (95, 105, 120)
GRID_COLOR = (35, 38, 48)
SUPPORT_COLOR = (140, 145, 155)

# Layout
CELL = 32   # pixels per cell
PAD_L = 80  # left padding
PAD_R = 100 # right padding (for legend)
PAD_T = 90  # top (title + progress)
PAD_B = 75  # bottom (supports + labels)


def lerp_color(c1, c2, t):
    t = max(0.0, min(1.0, t))
    return tuple(int(c1[i] + t * (c2[i] - c1[i])) for i in range(3))


def density_to_color(d, is_steel=False):
    if is_steel:
        return STEEL_CORE
    d = max(0.0, min(1.0, d))
    idx = d * (len(CONCRETE_RAMP) - 1)
    lo = int(idx)
    hi = min(lo + 1, len(CONCRETE_RAMP) - 1)
    t = idx - lo
    return lerp_color(CONCRETE_RAMP[lo], CONCRETE_RAMP[hi], t)


def draw_gradient_bg(draw, w, h):
    for y in range(h):
        t = y / max(h - 1, 1)
        c = lerp_color(BG_TOP, BG_BOT, t)
        draw.line([(0, y), (w, y)], fill=c)


def draw_rounded_rect(draw, box, fill, radius=4):
    """Draw a rounded rectangle."""
    x0, y0, x1, y1 = box
    draw.rectangle([x0 + radius, y0, x1 - radius, y1], fill=fill)
    draw.rectangle([x0, y0 + radius, x1, y1 - radius], fill=fill)
    draw.pieslice([x0, y0, x0 + 2*radius, y0 + 2*radius], 180, 270, fill=fill)
    draw.pieslice([x1 - 2*radius, y0, x1, y0 + 2*radius], 270, 360, fill=fill)
    draw.pieslice([x0, y1 - 2*radius, x0 + 2*radius, y1], 90, 180, fill=fill)
    draw.pieslice([x1 - 2*radius, y1 - 2*radius, x1, y1], 0, 90, fill=fill)


def render_beam_frame(rho_flat, nx, ny, iteration, total_iters):
    beam_w = nx * CELL
    beam_h = ny * CELL
    w = beam_w + PAD_L + PAD_R
    h = beam_h + PAD_T + PAD_B

    img = Image.new("RGB", (w, h))
    draw = ImageDraw.Draw(img)
    draw_gradient_bg(draw, w, h)

    # --- Title area ---
    draw.text((PAD_L // 2, 12), "UMST", fill=ACCENT)
    draw.text((PAD_L // 2 + 50, 12), "— Strut-and-Tie Discovery", fill=TEXT_COLOR)
    draw.text((PAD_L // 2, 32), f"RC Beam  ·  {nx}×{ny} nodes  ·  Simply Supported", fill=TEXT_DIM)

    # Progress bar
    bar_w = w - PAD_L
    bar_x = PAD_L // 2
    bar_y = 56
    progress = iteration / max(total_iters, 1)
    draw.rectangle([(bar_x, bar_y), (bar_x + bar_w, bar_y + 4)], fill=(35, 38, 48))
    fill_w = int(bar_w * progress)
    if fill_w > 0:
        # Gradient fill on progress bar
        for px in range(fill_w):
            t = px / max(fill_w - 1, 1)
            c = lerp_color(ACCENT, ACCENT_WARM, t)
            draw.line([(bar_x + px, bar_y), (bar_x + px, bar_y + 4)], fill=c)
    draw.text((bar_x + bar_w + 8, bar_y - 5), f"{int(progress*100)}%", fill=TEXT_DIM)

    # --- Draw beam cells ---
    bx0 = PAD_L
    by0 = PAD_T

    for iy in range(ny):
        for ix in range(nx):
            idx = (ny - 1 - iy) * nx + ix  # flip y (top of beam at top of image)
            d = float(rho_flat[idx]) if idx < len(rho_flat) else 0.0
            is_steel = (ny - 1 - iy) == 0  # bottom row

            color = density_to_color(d, is_steel)

            x0 = bx0 + ix * CELL
            y0 = by0 + iy * CELL
            draw.rectangle([x0, y0, x0 + CELL - 1, y0 + CELL - 1], fill=color)

            # Subtle inner shadow for depth (top-left highlight on dense cells)
            if d > 0.3 and not is_steel:
                highlight = lerp_color(color, (255, 255, 255), 0.15)
                draw.line([(x0, y0), (x0 + CELL - 2, y0)], fill=highlight)
                draw.line([(x0, y0), (x0, y0 + CELL - 2)], fill=highlight)

    # Steel glow effect on bottom row
    for ix in range(nx):
        x0 = bx0 + ix * CELL
        y0 = by0 + (ny - 1) * CELL
        # Top edge highlight
        draw.line([(x0, y0), (x0 + CELL - 1, y0)], fill=lerp_color(STEEL_CORE, (255, 255, 200), 0.3))

    # Grid lines (very subtle)
    for ix in range(nx + 1):
        x = bx0 + ix * CELL
        draw.line([(x, by0), (x, by0 + beam_h)], fill=GRID_COLOR, width=1)
    for iy in range(ny + 1):
        y = by0 + iy * CELL
        draw.line([(bx0, y), (bx0 + beam_w, y)], fill=GRID_COLOR, width=1)

    # Beam outline glow
    draw.rectangle([bx0 - 2, by0 - 2, bx0 + beam_w + 1, by0 + beam_h + 1], outline=ACCENT, width=1)

    # --- Supports ---
    sy = by0 + beam_h + 6
    # Left pin triangle
    px = bx0 + CELL // 2
    tri_h = 16
    draw.polygon(
        [(px, sy), (px - 10, sy + tri_h), (px + 10, sy + tri_h)],
        fill=SUPPORT_COLOR, outline=(180, 185, 195),
    )
    # Ground line
    draw.line([(px - 14, sy + tri_h + 2), (px + 14, sy + tri_h + 2)], fill=SUPPORT_COLOR, width=2)
    draw.text((px - 8, sy + tri_h + 6), "pin", fill=TEXT_DIM)

    # Right roller
    rx = bx0 + (nx - 1) * CELL + CELL // 2
    draw.polygon(
        [(rx, sy), (rx - 10, sy + tri_h), (rx + 10, sy + tri_h)],
        fill=SUPPORT_COLOR, outline=(180, 185, 195),
    )
    draw.ellipse([(rx - 5, sy + tri_h + 1), (rx + 5, sy + tri_h + 11)], fill=SUPPORT_COLOR)
    draw.text((rx - 12, sy + tri_h + 12), "roller", fill=TEXT_DIM)

    # --- Load arrow at top-right ---
    ax = bx0 + (nx - 1) * CELL + CELL // 2
    ay0 = by0 - 30
    ay1 = by0 - 4
    # Arrow shaft with glow
    for offset in [-1, 0, 1]:
        c = ACCENT_WARM if offset == 0 else lerp_color(ACCENT_WARM, BG_TOP, 0.6)
        draw.line([(ax + offset, ay0), (ax + offset, ay1)], fill=c, width=1)
    # Arrow head
    draw.polygon(
        [(ax, ay1 + 2), (ax - 7, ay1 - 6), (ax + 7, ay1 - 6)],
        fill=ACCENT_WARM,
    )
    draw.text((ax + 10, ay0 - 2), "F = 50 kN", fill=ACCENT_WARM)

    # --- Color legend ---
    leg_x = bx0 + beam_w + 20
    leg_y0 = by0
    leg_h = beam_h
    bar_w_leg = 14
    for py in range(leg_h):
        t = 1.0 - py / max(leg_h - 1, 1)
        c = density_to_color(t)
        draw.line([(leg_x, leg_y0 + py), (leg_x + bar_w_leg, leg_y0 + py)], fill=c)
    draw.rectangle([leg_x - 1, leg_y0 - 1, leg_x + bar_w_leg + 1, leg_y0 + leg_h], outline=GRID_COLOR)
    draw.text((leg_x + bar_w_leg + 4, leg_y0 - 6), "1.0", fill=TEXT_DIM)
    draw.text((leg_x + bar_w_leg + 4, leg_y0 + leg_h - 8), "0.0", fill=TEXT_DIM)
    draw.text((leg_x - 2, leg_y0 - 20), "ρ", fill=TEXT_COLOR)

    # Steel legend
    sy_leg = leg_y0 + leg_h + 15
    draw.rectangle([leg_x, sy_leg, leg_x + bar_w_leg, sy_leg + 12], fill=STEEL_CORE)
    draw.text((leg_x + bar_w_leg + 4, sy_leg - 1), "steel", fill=TEXT_DIM)

    # --- Iteration info ---
    draw.text(
        (PAD_L // 2, h - 22),
        f"iter {iteration:03d}/{total_iters}  ·  ρ span: {float(np.max(rho_flat) - np.min(rho_flat)):.4f}",
        fill=TEXT_COLOR,
    )

    return img


def main():
    manifest_path = ART_DIR / "manifest.json"
    if not manifest_path.is_file():
        print(f"ERROR: {manifest_path} not found. Run optimize_rc_beam first.")
        sys.exit(1)

    with open(manifest_path) as f:
        m = json.load(f)

    nx, ny = int(m["nx"]), int(m["ny"])
    total_iters = int(m["iters"])
    n = nx * ny

    files = sorted(ART_DIR.glob("iter_*.npy"))
    if not files:
        print(f"ERROR: no iter_*.npy in {ART_DIR}")
        sys.exit(1)

    print(f"Rendering {len(files)} frames for {nx}×{ny} beam ({total_iters} iterations)...")

    frames = []
    for fpath in files:
        fname = fpath.name
        it = int(fname.replace("iter_", "").replace(".npy", ""))
        arr = np.load(fpath, allow_pickle=False).astype(np.float32).reshape(-1)
        if arr.size != n:
            print(f"  skip {fname}: size {arr.size} != {n}")
            continue
        frame = render_beam_frame(arr, nx, ny, it, total_iters)
        frames.append(frame)

    # Hold final frame
    if frames:
        for _ in range(8):
            frames.append(frames[-1])

    if not frames:
        print("ERROR: no valid frames")
        sys.exit(1)

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    gif_path = OUT_DIR / "beam_strut_and_tie.gif"
    frames[0].save(
        str(gif_path),
        save_all=True,
        append_images=frames[1:],
        duration=150,
        loop=0,
    )
    print(f"Wrote {gif_path} ({len(frames)} frames)")


if __name__ == "__main__":
    main()
