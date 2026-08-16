#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
# V4 — blocked until V3-gate opens (real compressive strength measurements).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"
if [[ "${UMST_V4_ALLOW_REHEARSAL:-0}" != "1" ]]; then
  echo "SKIP: V4 calibration pin — V3-gate CLOSED (no physical measurements yet)."
  echo "Set UMST_V4_ALLOW_REHEARSAL=1 to run placeholder rehearsal checks only."
  exit 0
fi
python3 << 'PY'
import json
from pathlib import Path
p = Path("calibration/v3_pipeline_rehearsal/placeholder_points.json")
m = json.loads(p.read_text())
assert m.get("WARNING_not_measurements"), "missing WARNING_not_measurements"
for pt in m["points"]:
    assert "placeholder" in str(pt.keys()), "points must be labeled placeholder"
print("OK: V4 rehearsal placeholder structure (not validation)")
PY
