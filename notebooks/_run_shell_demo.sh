#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
# End-to-end Striatus shell demo: Rust NPY dumps -> PNG frames -> overlay -> GIF -> STL.
# Optional: UMST_SHELL_ITERS=5 for CI/smoke; UMST_SHELL_DUMP_ITER=1 for iter_*.npy frames (large).
# render_shell_gif.py interpolates from uniform when iter_*.npy are absent.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"
export UMST_SHELL_ITERS="${UMST_SHELL_ITERS:-200}"
export UMST_SHELL_DUMP_ITER="${UMST_SHELL_DUMP_ITER:-0}"
export UMST_SHELL_DUMP_STRIDE="${UMST_SHELL_DUMP_STRIDE:-20}"
: "${CARGO:=cargo}"
: "${PYTHON:=python3}"
"${CARGO}" run --release -p umst-concrete-cartridge --example optimize_shell_3d --features 'solver-experimental render'
"${PYTHON}" "${ROOT}/notebooks/render_shell_gif.py"
"${PYTHON}" "${ROOT}/notebooks/overlay_final_isostatics.py"
"${PYTHON}" "${ROOT}/notebooks/stitch_gif.py"
"${PYTHON}" "${ROOT}/notebooks/export_print_ready.py"
