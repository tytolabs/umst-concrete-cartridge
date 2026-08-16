#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
# Two back-to-back mini demo runs; STL SHA-256 must match (acceptance #5 subset).
# Override iteration count: UMST_SHELL_ITERS=5 (default here).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"
export UMST_SHELL_ITERS="${UMST_SHELL_ITERS:-5}"
export UMST_SHELL_DUMP_ITER="${UMST_SHELL_DUMP_ITER:-0}"
H1="$(mktemp)"
H2="$(mktemp)"
cleanup() {
  rm -f "${H1}" "${H2}"
}
trap cleanup EXIT

rm -rf "${ROOT}/notebooks/_artifacts"
bash "${ROOT}/notebooks/_run_shell_demo.sh" >/dev/null
( cd "${ROOT}" && sha256sum notebooks/_artifacts/striatus_shell_v0.4.stl ) >"${H1}"

rm -rf "${ROOT}/notebooks/_artifacts"
bash "${ROOT}/notebooks/_run_shell_demo.sh" >/dev/null
( cd "${ROOT}" && sha256sum notebooks/_artifacts/striatus_shell_v0.4.stl ) >"${H2}"

if ! diff "${H1}" "${H2}"; then
  echo "FAIL: STL hash mismatch between two runs" >&2
  exit 1
fi
echo "DETERMINISTIC OK (STL sha256 match, UMST_SHELL_ITERS=${UMST_SHELL_ITERS})"
