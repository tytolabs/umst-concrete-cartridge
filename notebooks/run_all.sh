#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"
export NB_ROOT="${ROOT}/notebooks"
if ! command -v jupyter >/dev/null 2>&1; then
  echo "SKIP: jupyter not installed"
  exit 0
fi
jupyter nbconvert --execute --inplace "${NB_ROOT}/sustainable_mix_audit.ipynb"
echo "OK: notebooks executed"
