#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
# End-to-end Striatus shell demo: Rust (`cargo run --release` … optimize_shell_3d) -> PNG frames -> GIF -> STL ->
# `notebooks/export_print_ready.py` (last step in this script).
#
# Python env (see notebooks/README.md): use one venv; install NumPy + render stack with the *same*
# interpreter you use here (`PYTHON=python3` or `.venv/bin/python`). Prefer `uv venv` + `uv pip install`
# or `python3 -m venv` + `pip install -r notebooks/requirements-shell-demo.txt` then
# `pip install './crates/umst-py[render]'` (or `maturin develop --extras render` from crates/umst-py).
# `export_print_ready.py` also needs graph engines: `uv pip install --python .venv/bin/python networkx scipy`
# (see docs/Solver-Status.md — Topology / shell).
#
# --- Long Track L block: 40×40×4 × 200 outers (many CPU hours; wall time is machine-dependent) ---
# From directory that contains `notebooks/` and `crates/` (this repo root = parent of `notebooks/`):
#
#   cd umst-concrete-cartridge
#   rm -f crates/umst-concrete-cartridge/examples/_artifacts/shell/iter_*.npy
#   export UMST_SHELL_NX=40 UMST_SHELL_NY=40 UMST_SHELL_NZ=4 UMST_SHELL_ITERS=200
#   export UMST_SHELL_DUMP_ITER=0 UMST_SHELL_DUMP_STRIDE=10
#   export UMST_SHELL_SELF_WEIGHT=0
#   # Roof ramp is forced `on` inside `_run_shell_demo.sh` (override with a custom script if you need uniform roof).
#   bash notebooks/_run_shell_demo.sh 2>&1 | tee shell_track_l_40cube4_i200.log
#
# Rust writes under crates/umst-concrete-cartridge/examples/_artifacts/shell/ (manifest.json, final.npy, …).
# This script writes under notebooks/_artifacts/: striatus_emergence.gif, striatus_shell_v0.4.stl,
# striatus_shell_v0.4.print_ready.json (via export_print_ready.py). Re-run exporter only from repo root:
#   python3 notebooks/export_print_ready.py
#   # or: .venv/bin/python notebooks/export_print_ready.py
#
# Optional: UMST_SHELL_ITERS=5 for CI/smoke; UMST_SHELL_DUMP_ITER=1 for iter_*.npy frames (large).
#   With dumps on, UMST_SHELL_DUMP_STRIDE defaults to 10 here and in optimize_shell_3d (first / every Nth / last outer).
# Optional: UMST_SHELL_HELM=1 enables graph Helmholtz on the AD tape (example default is off).
#
# Repro / conditioning (optimize_shell_3d reads the same names): UMST_SHELL_VF, UMST_SHELL_INIT_SCALE,
# UMST_SHELL_SYMMETRY, UMST_SHELL_SYMM_PERIOD, UMST_SHELL_PCG, UMST_SHELL_MAX_CG, UMST_SHELL_E_MIN_REL,
# UMST_SHELL_VOL_LOOP, UMST_SHELL_ROOF_RAMP / UMST_SHELL_ROOF_RAMP_F.
#
# CI-fast / smoke (Rust step only — not brief-aligned): example clamps grid to nx,ny in [6,40] and nz in [2,8], e.g.
#   UMST_SHELL_NX=6 UMST_SHELL_NY=6 UMST_SHELL_NZ=2 UMST_SHELL_ITERS=2 UMST_SHELL_DUMP_ITER=0 UMST_SHELL_MAX_CG=200 \
#     cargo run --release -p umst-concrete-cartridge --example optimize_shell_3d --features 'solver-experimental render'
#
# Bounded “ρ span ≥ 1e⁻³” smoke (still not Ring‑1 B8 on full 40³ lattice — export guard only), typical laptop minutes in --release:
#   UMST_SHELL_NX=16 UMST_SHELL_NY=16 UMST_SHELL_NZ=4 UMST_SHELL_ITERS=40 UMST_SHELL_DUMP_ITER=0 \
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
# Track L / B6 roof traction: ramp on unless explicitly `UMST_SHELL_ROOF_RAMP=0` in the *caller*
# environment. Force `=1` here so inherited `ROOF_RAMP=0` (e.g. CI agent shells) cannot silently
# disable the x-ramp load used in `shell_topology_rib_pattern` / Solver-Status recipes.
export UMST_SHELL_ROOF_RAMP=1
# Rib-style planar-texture outer loss (see `shell_topology_rib_pattern` / `m1_b6_closeout`); compliance-only
# Track L runs tend to park post–projection ρ at uniform mean(V*) → B8 `density_xy_plane_variance` ≪ 0.1.
export UMST_SHELL_XY_VAR_LAMBDA="${UMST_SHELL_XY_VAR_LAMBDA:-8}"
export UMST_SHELL_SELF_WEIGHT="${UMST_SHELL_SELF_WEIGHT:-0}"
# Default roof x-ramp on (matches `shell_topology_rib_pattern`); set to 0 only when intentionally uniform roof load.
export UMST_SHELL_ROOF_RAMP="${UMST_SHELL_ROOF_RAMP:-1}"
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
