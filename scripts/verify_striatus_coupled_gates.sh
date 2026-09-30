#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
#
# Reproducible gate check for Striatus / Track L + B6 quick + print_ready pytest (m1-l / m1-b8 / B6 CI).
# Run from anywhere:  bash scripts/verify_striatus_coupled_gates.sh
#
# Steps:
#   1) `cargo test -p umst-concrete-cartridge` — default package (constitutive / pipeline); does **not**
#      compile `shell_*` tests (those are `#![cfg(feature = "solver-experimental")]`).
#   2) `cargo test … --features solver-experimental` — includes `shell_demo_smoke` + B6 **quick** harness.
#   3) `pytest notebooks/tests/test_print_ready.py` — Track L sidecar / STL feasibility.
#   4) `python3 scripts/check_solver_status.py` — cartridge shim forwards to
#      `../umst-manifold/scripts/check_solver_status.py` with this repo's `docs/Solver-Status.md` and
#      manifold `--root` (exits 0 with stderr note if sibling missing).
#      § *Cartridge ↔ manifold contract* in `umst-manifold/docs/`.
#
# Optional: UMST_REQUIRE_B8=1  →  pytest fails if gates_track_b8_all_pass is false (Ring‑1 honesty).
# Full ignored B6 (`shell_topology_rib_pattern_full_v04`, hours) is **not** invoked here — see docs/Solver-Status.md.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"
: "${CARGO:=cargo}"
if [[ -x "${ROOT}/.venv/bin/python" ]]; then
  PY="${ROOT}/.venv/bin/python"
else
  PY="${PYTHON:-python3}"
fi

echo "== umst-concrete-cartridge coupled gate verify (repo: ${ROOT})"
echo "== (1/4) cargo test -p umst-concrete-cartridge (default; no shell_* integration)"
"${CARGO}" test -p umst-concrete-cartridge

echo "== (2/4) cargo test -p umst-concrete-cartridge --features solver-experimental (shell + B6 quick)"
"${CARGO}" test -p umst-concrete-cartridge --features solver-experimental

echo "== (3/4) pytest notebooks/tests/test_print_ready.py (UMST_REQUIRE_B8=${UMST_REQUIRE_B8:-unset})"
"${PY}" -m pytest "${ROOT}/notebooks/tests/test_print_ready.py" -v

JSON="${ROOT}/notebooks/_artifacts/striatus_shell_v0.4.print_ready.json"
if [[ -f "${JSON}" ]]; then
  echo "== print_ready B8 rollup (jq optional)"
  if command -v jq >/dev/null 2>&1; then
    jq '{gates_track_b8_all_pass, gate_topo_complexity_b7, gate_volume_fraction_mesh_b7, gate_density_xy_variance_b8, nodal_volume_fraction, mesh_volume_fraction_in_bbox}' "${JSON}"
  else
    "${PY}" -c "import json; d=json.load(open('${JSON}')); print('gates_track_b8_all_pass', d.get('gates_track_b8_all_pass')); print('gates', {k:d[k] for k in ('gate_topo_complexity_b7','gate_volume_fraction_mesh_b7','gate_density_xy_variance_b8','nodal_volume_fraction','mesh_volume_fraction_in_bbox') if k in d})"
  fi
fi

echo "== (4/4) scripts/check_solver_status.py (skip if no sibling umst-manifold)"
"${PY}" "${ROOT}/scripts/check_solver_status.py"

echo "== OK (full B6: UMST_SHELL_RIB_PATTERN=1 … shell_topology_rib_pattern_full_v04 --release -- --ignored — not run here)"
