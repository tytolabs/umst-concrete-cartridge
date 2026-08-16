#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
# After a shell demo run: assert hero GIF and STL stay within README clone budgets (C9).
# Exits 0 with SKIP lines when files are absent (e.g. fresh clone before mini-run).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GIF="${ROOT}/notebooks/_artifacts/striatus_emergence.gif"
STL="${ROOT}/notebooks/_artifacts/striatus_shell_v0.4.stl"
MAX_GIF=5242880
MAX_STL=8388608
ok=1
if [[ -f "${GIF}" ]]; then
  sz=$(wc -c < "${GIF}" | tr -d ' ')
  if [[ "${sz}" -gt ${MAX_GIF} ]]; then
    echo "FAIL: ${GIF} is ${sz} bytes (budget ${MAX_GIF})" >&2
    ok=0
  else
    echo "OK: GIF ${sz} bytes (<= ${MAX_GIF})"
  fi
else
  echo "SKIP: no ${GIF} (run notebooks/_run_shell_demo.sh first)"
fi
if [[ -f "${STL}" ]]; then
  sz=$(wc -c < "${STL}" | tr -d ' ')
  if [[ "${sz}" -gt ${MAX_STL} ]]; then
    echo "FAIL: ${STL} is ${sz} bytes (budget ${MAX_STL})" >&2
    ok=0
  else
    echo "OK: STL ${sz} bytes (<= ${MAX_STL})"
  fi
else
  echo "SKIP: no ${STL} (run notebooks/_run_shell_demo.sh first)"
fi
if [[ "${ok}" -ne 1 ]]; then
  exit 1
fi
