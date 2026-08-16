#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""Export memory_record rows from SQLite as contribution.v1 JSONL for git inbox PRs.

Usage:
  python3 scripts/export_contributions_jsonl.py \\
    --db .umst-memory/memory.db \\
    --lab tyto \\
    --out contributions/inbox/tyto-20260619-a1b2c3.jsonl

  python3 scripts/export_contributions_jsonl.py --db .umst-memory/memory.db --lab tyto --stdout
"""

from __future__ import annotations

import argparse
import json
import sqlite3
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from ingest_contributions import content_id_from_contribution


def record_to_contribution(record: dict) -> dict | None:
    if record.get("schema_version") != "memory_record.v1":
        return None
    payload = record.get("payload")
    if not isinstance(payload, dict):
        return None
    gate = payload.get("gate_summary")
    if not isinstance(gate, dict) or not gate.get("admissible"):
        return None
    return {
        "schema_version": "contribution.v1",
        "canon_version": record.get("canon_version", "jcs-rfc8785-v1"),
        "mix_spec": payload.get("mix_spec"),
        "process": payload.get("process"),
        "outcome": payload.get("outcome"),
        "gate_summary": gate,
        "catalog_hash": record.get("catalog_hash"),
        "observed_at": record.get("observed_at"),
    }


def load_known_content_ids(merged_dir: Path | None, manifest: Path | None) -> set[str]:
    known: set[str] = set()
    if manifest and manifest.is_file():
        for line in manifest.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if not line:
                continue
            try:
                row = json.loads(line)
                for cid in row.get("content_ids", []):
                    known.add(cid)
            except json.JSONDecodeError:
                continue
    if merged_dir and merged_dir.is_dir():
        for path in sorted(merged_dir.rglob("*.jsonl")):
            if path.name == "MANIFEST.jsonl":
                continue
            for line in path.read_text(encoding="utf-8").splitlines():
                line = line.strip()
                if not line:
                    continue
                try:
                    obj = json.loads(line)
                    known.add(content_id_from_contribution(obj))
                except json.JSONDecodeError:
                    continue
    return known


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--db", default=".umst-memory/memory.db")
    parser.add_argument("--lab", required=True, help="lab-slug for export metadata")
    parser.add_argument("--out", type=Path, help="output JSONL path")
    parser.add_argument("--stdout", action="store_true")
    parser.add_argument(
        "--since-content-id",
        help="export rows with content_id lexicographically after this value",
    )
    parser.add_argument(
        "--merged-dir",
        type=Path,
        default=Path("contributions/merged"),
        help="skip content_ids already present in merged JSONL shards",
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        default=Path("contributions/merged/MANIFEST.jsonl"),
    )
    args = parser.parse_args()

    db_path = Path(args.db)
    if not db_path.is_file():
        print(f"error: database not found: {db_path}", file=sys.stderr)
        return 1

    known = load_known_content_ids(args.merged_dir, args.manifest)
    conn = sqlite3.connect(db_path)
    rows = conn.execute(
        "SELECT record_json FROM memory_records ORDER BY content_id"
    ).fetchall()
    conn.close()

    exported = 0
    skipped = 0
    lines: list[str] = []
    for (record_json,) in rows:
        record = json.loads(record_json)
        content_id = record.get("content_id", "")
        if args.since_content_id and content_id <= args.since_content_id:
            skipped += 1
            continue
        if content_id in known:
            skipped += 1
            continue
        contrib = record_to_contribution(record)
        if contrib is None:
            skipped += 1
            continue
        cid = content_id_from_contribution(contrib)
        if cid in known:
            skipped += 1
            continue
        lines.append(json.dumps(contrib, separators=(",", ":"), sort_keys=True))
        exported += 1

    if args.stdout:
        out_fh = sys.stdout
    else:
        if not args.out:
            print("error: --out or --stdout required", file=sys.stderr)
            return 1
        args.out.parent.mkdir(parents=True, exist_ok=True)
        out_fh = args.out.open("w", encoding="utf-8")

    for line in lines:
        out_fh.write(line + "\n")

    if not args.stdout and args.out:
        out_fh.close()

    summary = {
        "lab": args.lab,
        "exported_at": datetime.now(timezone.utc).isoformat(),
        "exported": exported,
        "skipped": skipped,
        "out": str(args.out) if args.out else "stdout",
    }
    print(json.dumps(summary, indent=2), file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
