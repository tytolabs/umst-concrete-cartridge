#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""Bulk-import contribution.v1 JSONL into SQLite memory store via `umst` gate re-check.

Each line must be valid contribution.v1 JSON. Rows failing gate re-check are skipped
(reject rows are NOT written to memory — run gate_check separately for audit).

Usage:
  python3 scripts/ingest_contributions.py contributions.jsonl --db .umst-memory/memory.db
  python3 scripts/bootstrap_memory_from_audit.py fixtures/bootstrap_audit_slice.csv | \\
    python3 scripts/ingest_contributions.py --db .umst-memory/memory.db
"""

from __future__ import annotations

import argparse
import json
import sqlite3
import subprocess
import sys
from pathlib import Path

SCHEMA_SQL = """
PRAGMA journal_mode=WAL;
CREATE TABLE IF NOT EXISTS memory_records (
  memory_id TEXT PRIMARY KEY,
  content_id TEXT UNIQUE NOT NULL,
  idempotency_key TEXT UNIQUE,
  record_json TEXT NOT NULL
) STRICT;
CREATE TRIGGER IF NOT EXISTS memory_records_no_update
  BEFORE UPDATE ON memory_records
BEGIN
  SELECT RAISE(ABORT, 'memory_records are immutable');
END;
CREATE TRIGGER IF NOT EXISTS memory_records_no_delete
  BEFORE DELETE ON memory_records
BEGIN
  SELECT RAISE(ABORT, 'memory_records are immutable');
END;
"""


def content_id_from_contribution(obj: dict) -> str:
    import hashlib

    preimage = {
        "schema_version": obj.get("schema_version"),
        "canon_version": obj.get("canon_version"),
        "mix_spec": obj.get("mix_spec"),
        "process": obj.get("process"),
        "outcome": obj.get("outcome"),
        "gate_summary": obj.get("gate_summary"),
        "catalog_hash": obj.get("catalog_hash"),
        "observed_at": obj.get("observed_at"),
    }
    digest = hashlib.sha256(
        json.dumps(preimage, separators=(",", ":"), sort_keys=True).encode()
    ).hexdigest()
    return f"sha256:{digest}"


def gate_check_admissible(contribution: dict, profile: str) -> bool:
  mix = contribution.get("mix_spec")
  if not isinstance(mix, dict):
      return False
  proc = subprocess.run(
      [
          "cargo",
          "run",
          "-q",
          "-p",
          "umst-mcp",
          "--features",
          "agent-layer",
      ],
      input=json.dumps(
          {
              "jsonrpc": "2.0",
              "id": 1,
              "method": "tools/call",
              "params": {
                  "name": "umst_gate_check",
                  "arguments": {"mix": mix, "profile": profile},
              },
          }
      ).encode(),
      capture_output=True,
      cwd=Path(__file__).resolve().parents[1],
  )
  if proc.returncode != 0:
      return bool(contribution.get("gate_summary", {}).get("admissible"))
  try:
      frame = json.loads(proc.stdout.decode().splitlines()[-1])
      text = frame["result"]["content"][0]["text"]
      summary = json.loads(text)
      gate = summary.get("gate_summary", summary)
      return bool(gate.get("admissible"))
  except (KeyError, json.JSONDecodeError, IndexError):
      return bool(contribution.get("gate_summary", {}).get("admissible"))


def ingest_line(
    conn: sqlite3.Connection | None,
    line: str,
    profile: str,
    memory_jsonl: Path | None,
    *,
    skip_gate: bool,
    dry_run: bool,
) -> str:
    obj = json.loads(line)
    if obj.get("schema_version") != "contribution.v1":
        return "skip_schema"
    if not skip_gate and not gate_check_admissible(obj, profile):
        return "skip_gate"
    if not obj.get("gate_summary", {}).get("admissible"):
        return "skip_not_admissible"

    from validate_contribution_inbox import credit_admits_promotion

    if not credit_admits_promotion(obj):
        return "skip_low_credit"

    import uuid

    memory_id = str(uuid.uuid4())
    content_id = content_id_from_contribution(obj)
    idem = obj.get("idempotency_key")
    record = {
        "schema_version": "memory_record.v1",
        "canon_version": obj.get("canon_version", "jcs-rfc8785-v1"),
        "content_id": content_id,
        "observed_at": obj.get("observed_at"),
        "payload": {
            "mix_spec": obj.get("mix_spec"),
            "process": obj.get("process"),
            "outcome": obj.get("outcome"),
            "gate_summary": obj.get("gate_summary"),
        },
        "catalog_hash": obj.get("catalog_hash"),
        "catalog_ids": obj.get("gate_summary", {}).get("catalog_ids", []),
        "memory_id": memory_id,
    }
    record_json = json.dumps(record, separators=(",", ":"))
    if dry_run:
        return "would_insert"
    assert conn is not None and memory_jsonl is not None
    try:
        conn.execute(
            "INSERT INTO memory_records (memory_id, content_id, idempotency_key, record_json) VALUES (?, ?, ?, ?)",
            (memory_id, content_id, idem, record_json),
        )
    except sqlite3.IntegrityError:
        return "skip_duplicate"
    memory_jsonl.parent.mkdir(parents=True, exist_ok=True)
    with memory_jsonl.open("a", encoding="utf-8") as fh:
        fh.write(record_json + "\n")
    return "inserted"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("jsonl", nargs="?", help="contribution JSONL (default stdin)")
    parser.add_argument("--db", default=".umst-memory/memory.db", help="SQLite path")
    parser.add_argument("--profile", default="default")
    parser.add_argument(
        "--skip-gate",
        action="store_true",
        help="Trust gate_summary.admissible without MCP gate_check (bootstrap only)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Validate and gate-check only; do not write SQLite or JSONL",
    )
    args = parser.parse_args()

    db_path = Path(args.db)
    conn: sqlite3.Connection | None
    if args.dry_run:
        conn = None
    else:
        db_path.parent.mkdir(parents=True, exist_ok=True)
        conn = sqlite3.connect(db_path)
        conn.executescript(SCHEMA_SQL)

    fh = open(args.jsonl, encoding="utf-8") if args.jsonl else sys.stdin
    memory_jsonl = None if args.dry_run else db_path.parent / "memory.jcs.jsonl"
    counts: dict[str, int] = {}
    for line in fh:
        line = line.strip()
        if not line:
            continue
        status = ingest_line(
            conn,
            line,
            args.profile,
            memory_jsonl,
            skip_gate=args.skip_gate,
            dry_run=args.dry_run,
        )
        counts[status] = counts.get(status, 0) + 1
    if not args.dry_run and conn is not None:
        conn.commit()
    if args.jsonl:
        fh.close()
    print(json.dumps({"counts": counts, "db": str(db_path), "dry_run": args.dry_run}, indent=2))
    if args.dry_run:
        bad = sum(
            counts.get(k, 0)
            for k in (
                "skip_schema",
                "skip_gate",
                "skip_not_admissible",
            )
        )
        return 1 if bad else 0
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
