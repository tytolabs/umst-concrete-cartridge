#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"
export NB_ROOT="${ROOT}/notebooks"
STRICT=0
for arg in "$@"; do
  if [[ "${arg}" == "--strict" ]]; then
    STRICT=1
  fi
done

if ! command -v jupyter >/dev/null 2>&1; then
  if [[ "${STRICT}" -eq 1 ]]; then
    echo "FAIL: jupyter not installed (--strict requires nbconvert)" >&2
    exit 1
  fi
  echo "SKIP: jupyter not installed"
  exit 0
fi
jupyter nbconvert --execute --inplace "${NB_ROOT}/sustainable_mix_audit.ipynb"
echo "OK: notebooks executed"
