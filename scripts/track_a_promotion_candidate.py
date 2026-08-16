#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""Track A → promotion candidate report (JSON only; never writes calibration TOML).

Reads `proposed_next_mix.json` sidecar from Track A optimize and emits a
promotion_candidate.v1 report for human review.

Usage:
  python3 scripts/track_a_promotion_candidate.py proposed_next_mix.json > candidate.json
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("sidecar", type=Path, help="proposed_next_mix.json from Track A")
    parser.add_argument("--memory-id", default=None, help="Optional linked memory row id")
    parser.add_argument("--memory-json", type=Path, default=None, help="Optional memory_record.v1 for stamp_tier check")
    args = parser.parse_args()

    witness_live = os.environ.get("UMST_UCRS_WITNESS", "synthetic") == "live"
    stamp_tier = None
    if args.memory_json:
        memory = json.loads(args.memory_json.read_text(encoding="utf-8"))
        stamp_tier = (memory.get("observed_at") or {}).get("stamp_tier")
        if witness_live and stamp_tier != "UcrsTier2":
            print(
                f"error: UMST_UCRS_WITNESS=live requires observed_at.stamp_tier UcrsTier2, got {stamp_tier!r}",
                file=sys.stderr,
            )
            return 2

    data = json.loads(args.sidecar.read_text(encoding="utf-8"))
    dual = data.get("dual_gate", {})
    report = {
        "schema_version": "promotion_candidate.v1",
        "canon_version": "jcs-rfc8785-v1",
        "source": "track_a_optimize",
        "memory_id": args.memory_id,
        "calibration_profile": data.get("calibration_profile"),
        "proposed_mix": data.get("proposed_mix"),
        "dual_gate": dual,
        "eligible_for_propose_promotion": bool(dual.get("passes")),
        "policy_id": "governance/promotion_policy.yaml",
        "witness_mode": "live" if witness_live else "synthetic",
        "observed_stamp_tier": stamp_tier,
        "note": "Human must run umst propose-promotion + promote-contribution; no silent theta.",
    }
    json.dump(report, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
