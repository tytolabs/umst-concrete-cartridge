#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# Public CI sibling checkout — no credentials (W-63). SHAs from `.umst-pins.toml`.
set -euo pipefail
args=()
while IFS= read -r line; do
  [ -n "${line}" ] && args+=("${line}")
done < <(python3 - <<'PY'
import tomllib
with open(".umst-pins.toml", "rb") as f:
    pins = tomllib.load(f)
for name, cfg in pins.items():
    if name == "umst-supercap-cartridge":
        continue
    print(f"{name}@{cfg['sha']}")
PY
)
bash .github/scripts/checkout-umst-siblings.sh "${args[@]}"
