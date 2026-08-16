#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

export PROFILE="${PROFILE:-debug}"

echo "=== [1] fmt check (dry) ==="
cargo fmt --all -- --check

echo "=== [2] workspace tests ==="
cargo test --workspace --verbose

echo "=== [3] examples ==="
cargo build --workspace --examples --verbose

echo "=== [4] all-features (empty features must stay green) ==="
cargo check --workspace --all-features --verbose

echo "=== [5] umst-canonical stdin smoke ==="
_canon_out="$(printf '%s\n' '{"b":2,"a":1}' | "target/${PROFILE}/umst-canonical")"
[[ "${_canon_out}" == *'"a":1'* ]] || { echo "canonical output: ${_canon_out}"; exit 1; }

echo "=== [6] MCP smoke ==="
python3 scripts/mcp_smoke.py

echo "=== [7] predict determinism (CLI canonical bytes == Python canonical bytes) ==="
if ! python3 -c "import umst_concrete_cartridge" 2>/dev/null; then
  python3 -m pip install -q maturin
  (cd crates/umst-py && maturin develop -q)
fi
python3 scripts/check_predict_determinism.py --profile uci_d1 \
  --mix-json '{"w_c":0.4,"temperature_k":293.15}'

echo "=== ACCEPTANCE DONE ==="
