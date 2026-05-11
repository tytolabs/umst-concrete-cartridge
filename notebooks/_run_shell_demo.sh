#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
# End-to-end Striatus shell demo: Rust NPY dumps -> PNG frames -> overlay -> GIF -> STL.
#
# Python env (see notebooks/README.md): use one venv; install NumPy + render stack with the *same*
# interpreter you use here (`PYTHON=python3` or `.venv/bin/python`). Prefer `uv venv` + `uv pip install`
# or `python3 -m venv` + `pip install -r notebooks/requirements-shell-demo.txt` then
# `pip install './crates/umst-py[render]'` (or `maturin develop --extras render` from crates/umst-py).
#
# Optional: UMST_SHELL_ITERS=5 for CI/smoke; UMST_SHELL_DUMP_ITER=1 for iter_*.npy frames (large).
# Optional: UMST_SHELL_HELM=1 enables graph Helmholtz on the AD tape (example default is off).
# Repro / conditioning (optimize_shell_3d, same names): UMST_SHELL_NX, UMST_SHELL_NY, UMST_SHELL_NZ, UMST_SHELL_VF,
# UMST_SHELL_INIT_SCALE, UMST_SHELL_SYMMETRY, UMST_SHELL_SYMM_PERIOD, UMST_SHELL_PCG, UMST_SHELL_MAX_CG,
# UMST_SHELL_E_MIN_REL, UMST_SHELL_SELF_WEIGHT (this script defaults SELF_WEIGHT to 0), UMST_SHELL_VOL_LOOP.
#
# CI-fast / smoke (Rust step only — not brief-aligned): example clamps grid to nx,ny in [6,40] and nz in [2,8], e.g.
#   UMST_SHELL_NX=6 UMST_SHELL_NY=6 UMST_SHELL_NZ=2 UMST_SHELL_ITERS=2 UMST_SHELL_DUMP_ITER=0 UMST_SHELL_MAX_CG=200 \
#     cargo run --release -p umst-concrete-cartridge --example optimize_shell_3d --features 'solver-experimental render'
# CPU BLAS threads (if linked): macOS Accelerate — VECLIB_MAXIMUM_THREADS; OpenBLAS — OPENBLAS_NUM_THREADS
# (see repo README “CPU matmul”).
#
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
export UMST_SHELL_DUMP_STRIDE="${UMST_SHELL_DUMP_STRIDE:-10}"
: "${CARGO:=cargo}"
: "${PYTHON:=python3}"
"${CARGO}" run --release -p umst-concrete-cartridge --example optimize_shell_3d --features 'solver-experimental render'
"${PYTHON}" "${ROOT}/notebooks/render_shell_gif.py"
"${PYTHON}" "${ROOT}/notebooks/overlay_final_isostatics.py"
"${PYTHON}" "${ROOT}/notebooks/stitch_gif.py"
"${PYTHON}" "${ROOT}/notebooks/export_print_ready.py"
