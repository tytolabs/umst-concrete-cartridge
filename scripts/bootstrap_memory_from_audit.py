#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""Bootstrap contribution.v1 JSONL from dataset_d1-compatible audit CSV (stdin or file).

Each row becomes one contribution line suitable for bulk import.
Gate re-check at ingest time remains mandatory — gate_summary.admissible=true is a
placeholder until umst_gate_check / accept validates each row.

Provenance: fixtures/corpus/provenance.v1.json (lifted from umst-prototype-2a/docs/2_Datasets.md)

Usage:
  python3 scripts/bootstrap_memory_from_audit.py fixtures/corpus/audit_corpus.v1.csv > contributions.jsonl
  python3 scripts/bootstrap_memory_from_audit.py fixtures/corpus/dataset_d1.v1.csv --limit 100
"""

from __future__ import annotations

import argparse
import csv
import json
import sys
from fractions import Fraction
from typing import Any

DEFAULT_CATALOG_HASH = (
    "sha256:0000000000000000000000000000000000000000000000000000000000000001"
)

CORPUS_PROVENANCE_SHA256 = (
    "86645bd9749bd3b429e7f1b41814686c7c97b39fa9c9ac18edc5825faf6b1e1e"
)

HEADER_MAP = {
    "cement": "cement",
    "cement_kg_m3": "cement",
    "water": "water",
    "water_kg_m3": "water",
    "superplasticizer": "superplasticizer",
    "superplasticiser": "superplasticizer",
    "coarse_agg": "coarse_agg",
    "coarse_aggregate": "coarse_agg",
    "fine_agg": "fine_agg",
    "fine_aggregate": "fine_agg",
    "age": "age",
    "age_days": "age",
    "temperature": "temperature",
    "temperature_c": "temperature",
    "strength": "strength",
    "compressive_strength_mpa": "strength",
    "source": "source",
    "humidity": "humidity",
}

SOURCE_TO_REGIME = {
    "D1": "uci_d1",
    "uci": "uci_d1",
    "zenodo_ndt": "zenodo_ndt",
    "zenodo_sun": "zenodo_sun",
    "zenodo_rh": "zenodo_rh",
    "uhpc": "uhpc",
    "lunar": "lunar",
    "selfheal": "selfheal",
    "highscm": "highscm",
}


def canon_header(tok: str) -> str | None:
    return HEADER_MAP.get(tok.strip().lower().replace(" ", "_"))


def rat(x: float) -> str:
    f = Fraction(x).limit_denominator(10_000)
    return f"{f.numerator}/{f.denominator}"


def outcome_source_tag(source: str | None, synthetic: bool = False) -> str:
    if synthetic:
        return "synthetic"
    if source in (None, "", "D1", "uci"):
        return "literature"
    if source and source.startswith("zenodo"):
        return "literature"
    return "bootstrap"


def row_to_contribution(
    row: dict[str, Any],
    row_idx: int,
    *,
    corpus_sha256: str | None,
) -> dict[str, Any]:
    cement = float(row["cement"])
    water = float(row["water"])
    w_c = water / cement if cement else 0.45
    temp_c = float(row.get("temperature", 20.0))
    temp_k = temp_c + 273.15
    coarse = float(row.get("coarse_agg", 1000.0))
    fine = float(row.get("fine_agg", 800.0))
    agg_frac = min(0.85, max(1e-3, (coarse + fine) / 2600.0))
    age_h = float(row.get("age", 28.0)) * 24.0
    strength = row.get("strength")
    source = row.get("source")
    if isinstance(source, str):
        source = source.strip()
    else:
        source = None

    regime = SOURCE_TO_REGIME.get(source or "", "bootstrap_audit")
    exp_id = f"{source or 'audit'}:row_{row_idx}"

    outcome: dict[str, Any] = {
        "source": outcome_source_tag(source),
        "external_experiment_id": exp_id,
    }
    if strength is not None:
        outcome["compressive_strength_mpa"] = rat(float(strength))

    process: dict[str, Any] = {"curing_regime": regime}
    if corpus_sha256:
        process["notes"] = f"corpus_sha256={corpus_sha256}"

    return {
        "schema_version": "contribution.v1",
        "canon_version": "jcs-rfc8785-v1",
        "mix_spec": {
            "w_c": rat(w_c),
            "temperature_k": rat(temp_k),
            "aggregate_volume_fraction": rat(agg_frac),
            "target_age_hours": rat(age_h),
        },
        "process": process,
        "outcome": outcome,
        "gate_summary": {
            "admissible": True,
            "verdict": "PASS",
            "catalog_ids": ["gate.clausius_duhem.v1"],
        },
        "catalog_hash": DEFAULT_CATALOG_HASH,
        "observed_at": {
            "stamp_tier": "Synthetic",
            "ucrs_seq": row_idx,
            "wall_ms": 1718745600000 + row_idx,
        },
    }


def parse_csv(reader: csv.DictReader) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for raw in reader:
        mapped: dict[str, Any] = {}
        for k, v in raw.items():
            if k is None or v in (None, ""):
                continue
            key = canon_header(k)
            if not key:
                continue
            if key == "source":
                mapped[key] = str(v).strip()
            elif key == "humidity":
                mapped[key] = float(v)
            else:
                mapped[key] = float(v)
        if "cement" in mapped and "water" in mapped:
            out.append(mapped)
    return out


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("csv_path", nargs="?", help="CSV file (default stdin)")
    parser.add_argument("--limit", type=int, default=0, help="Max rows (0 = all)")
    parser.add_argument(
        "--corpus-sha256",
        default=CORPUS_PROVENANCE_SHA256,
        help="Embed corpus SHA-256 in process.notes (empty to skip)",
    )
    args = parser.parse_args()

    corpus_sha = args.corpus_sha256 or None

    if args.csv_path:
        fh = open(args.csv_path, newline="", encoding="utf-8")
    else:
        fh = sys.stdin

    reader = csv.DictReader(fh)
    rows = parse_csv(reader)
    if args.limit:
        rows = rows[: args.limit]

    for i, row in enumerate(rows, start=1):
        print(
            json.dumps(
                row_to_contribution(row, i, corpus_sha256=corpus_sha),
                separators=(",", ":"),
            )
        )

    if args.csv_path:
        fh.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
