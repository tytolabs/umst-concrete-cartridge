# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""Stitch PNG frames into the Striatus emergence GIF (palette quantisation)."""
from __future__ import annotations

from pathlib import Path

import imageio.v2 as imageio
import numpy as np

REPO = Path(__file__).resolve().parents[1]
FRAMES = REPO / "notebooks" / "_artifacts" / "frames"
OUT = REPO / "notebooks" / "_artifacts" / "striatus_emergence.gif"


def main() -> None:
    FRAMES.mkdir(parents=True, exist_ok=True)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    paths = sorted(FRAMES.glob("frame_*.png"))
    if not paths:
        raise FileNotFoundError("no frames; run render_shell_gif.py first")
    ims = [imageio.imread(str(p)) for p in paths]
    durations = []
    n = len(ims)
    for i in range(n):
        if i < min(50, n):
            durations.append(0.12)
        elif i < min(150, n):
            durations.append(0.08)
        else:
            durations.append(0.2)
    if durations:
        durations[-1] = 1.5
    rgba = [np.asarray(im) for im in ims]
    imageio.mimsave(
        str(OUT),
        rgba,
        format="GIF",
        duration=durations,
        quantize=256,
        palettesize=256,
    )


if __name__ == "__main__":
    main()
