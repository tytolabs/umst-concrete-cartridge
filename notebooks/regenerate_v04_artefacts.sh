#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
#
# Track L — regenerate Striatus v0.4 STL / print_ready / auxiliary assets after an
# `optimize_shell_3d` run (see docs/Striatus.md). Requires Python deps from notebooks/requirements.txt.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
ART="$ROOT/notebooks/_artifacts"
mkdir -p "$ART"

echo "1. Run the shell optimiser (example) to populate $ART/*.npy — see docs/Striatus.md"
echo "   Example: cargo run -p umst-concrete-cartridge --example optimize_shell_3d --features solver-experimental"
echo ""
echo "2. Export print-ready geometry + JSON gates (B7/B8)"
python3 notebooks/export_print_ready.py
echo ""
echo "3. (Optional) GIF: compose from optimisation frames if your renderer pipeline is configured."
echo "   Done. Inspect striatus_shell_v0.4.* under notebooks/_artifacts/"
