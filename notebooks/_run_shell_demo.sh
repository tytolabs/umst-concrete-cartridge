#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
# End-to-end Striatus shell demo: Rust NPY dumps -> PNG frames -> overlay -> GIF -> STL.
# Optional: UMST_SHELL_ITERS=5 for CI/smoke; UMST_SHELL_DUMP_ITER=1 for iter_*.npy frames (large).
# Optional: UMST_SHELL_HELM=1 enables graph Helmholtz on the AD tape (example default is off).
# render_shell_gif.py interpolates from uniform when iter_*.npy are absent.
# Default UMST_SHELL_SELF_WEIGHT=0 (traction-only): default optimize_shell_3d self-weight often gives NaN
# compliance on f32 adjoint at 40×40×4; smaller UMST_SHELL_ITERS does not fix that.
# Remove stale iter_*.npy before a run if grid changed (e.g. find …/shell -name 'iter_*.npy' -delete).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"
export UMST_SHELL_SELF_WEIGHT="${UMST_SHELL_SELF_WEIGHT:-0}"
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
