#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""Maintain contributions/merged/MANIFEST.jsonl — append-only ledger of merged inbox shards.

Manifest line schema (minimal):
  {"path", "sha256", "merged_at", "rows", "lab", "content_ids"?}

Usage (maintainer, after moving inbox → merged/YYYY-MM/):
  python3 scripts/update_contribution_manifest.py --append contributions/merged/2026-06/lab-foo.jsonl

CI / local verify (manifest matches merged shards on disk):
  python3 scripts/update_contribution_manifest.py --check

Rescan and print missing manifest rows (dry-run append preview):
  python3 scripts/update_contribution_manifest.py --scan
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_MANIFEST = REPO_ROOT / "contributions" / "merged" / "MANIFEST.jsonl"
DEFAULT_MERGED_DIR = REPO_ROOT / "contributions" / "merged"
LAB_DATE_RE = re.compile(r"^([a-z0-9][a-z0-9-]*)-(\d{8})-")


def _rel(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def file_sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def lab_from_filename(path: Path) -> str:
    m = LAB_DATE_RE.match(path.name)
    if m:
        return m.group(1)
    return path.stem.split("-")[0] or "unknown"


def scan_shard(path: Path) -> dict:
    sys.path.insert(0, str(REPO_ROOT / "scripts"))
    from ingest_contributions import content_id_from_contribution

    lines = [ln.strip() for ln in path.read_text(encoding="utf-8").splitlines() if ln.strip()]
    content_ids: list[str] = []
    for line in lines:
        obj = json.loads(line)
        content_ids.append(content_id_from_contribution(obj))
    return {
        "path": _rel(path),
        "sha256": file_sha256(path),
        "merged_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat(),
        "rows": len(lines),
        "lab": lab_from_filename(path),
        "content_ids": content_ids,
    }


def load_manifest(path: Path) -> list[dict]:
    if not path.is_file():
        return []
    rows: list[dict] = []
    for i, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        line = line.strip()
        if not line:
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError as e:
            print(f"error: {path}:{i}: invalid JSON: {e}", file=sys.stderr)
    return rows


def iter_merged_shards(merged_dir: Path) -> list[Path]:
    if not merged_dir.is_dir():
        return []
    return sorted(
        p
        for p in merged_dir.rglob("*.jsonl")
        if p.name != "MANIFEST.jsonl" and p.is_file()
    )


def manifest_index(rows: list[dict]) -> dict[str, dict]:
    return {row["path"]: row for row in rows if row.get("path")}


def check_manifest(manifest_path: Path, merged_dir: Path) -> int:
    errors = 0
    manifest_rows = load_manifest(manifest_path)
    by_path = manifest_index(manifest_rows)
    shards = iter_merged_shards(merged_dir)

    for shard in shards:
        rel = _rel(shard)
        scanned = scan_shard(shard)
        entry = by_path.get(rel)
        if entry is None:
            print(f"error: missing manifest entry for {rel}", file=sys.stderr)
            errors += 1
            continue
        if entry.get("sha256") != scanned["sha256"]:
            print(
                f"error: sha256 mismatch for {rel}: manifest={entry.get('sha256')} disk={scanned['sha256']}",
                file=sys.stderr,
            )
            errors += 1
        if entry.get("rows") != scanned["rows"]:
            print(
                f"error: row count mismatch for {rel}: manifest={entry.get('rows')} disk={scanned['rows']}",
                file=sys.stderr,
            )
            errors += 1
        manifest_ids = set(entry.get("content_ids") or [])
        disk_ids = set(scanned["content_ids"])
        if manifest_ids and manifest_ids != disk_ids:
            print(f"error: content_ids mismatch for {rel}", file=sys.stderr)
            errors += 1

    manifest_paths = {row["path"] for row in manifest_rows if row.get("path")}
    for shard in shards:
        rel = _rel(shard)
        if rel not in manifest_paths:
            print(f"error: shard on disk without manifest row: {rel}", file=sys.stderr)
            errors += 1

    for path in sorted(manifest_paths - {_rel(s) for s in shards}):
        print(f"error: manifest references missing file: {path}", file=sys.stderr)
        errors += 1

    if errors:
        print(f"update_contribution_manifest --check: {errors} error(s)", file=sys.stderr)
        return 1
    print("update_contribution_manifest --check: ok")
    return 0


def append_entry(manifest_path: Path, shard_path: Path, *, merged_at: str | None) -> int:
    shard = shard_path.resolve()
    if not shard.is_file():
        print(f"error: not a file: {shard}", file=sys.stderr)
        return 1
    rel = _rel(shard)
    existing = manifest_index(load_manifest(manifest_path))
    if rel in existing:
        print(f"error: manifest already has entry for {rel}", file=sys.stderr)
        return 1

    row = scan_shard(shard)
    if merged_at:
        row["merged_at"] = merged_at
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    with manifest_path.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(row, separators=(",", ":"), sort_keys=True) + "\n")
    print(json.dumps(row, indent=2))
    return 0


def scan_missing(manifest_path: Path, merged_dir: Path) -> int:
    by_path = manifest_index(load_manifest(manifest_path))
    missing = []
    for shard in iter_merged_shards(merged_dir):
        rel = _rel(shard)
        if rel not in by_path:
            missing.append(scan_shard(shard))
    if not missing:
        print("scan: all merged shards have manifest entries")
        return 0
    print(json.dumps({"missing": missing}, indent=2))
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--merged-dir", type=Path, default=DEFAULT_MERGED_DIR)
    parser.add_argument("--check", action="store_true", help="verify manifest matches merged shards")
    parser.add_argument("--scan", action="store_true", help="list shards missing manifest rows")
    parser.add_argument("--append", type=Path, metavar="SHARD", help="append one merged shard to manifest")
    parser.add_argument(
        "--merged-at",
        help="ISO8601 merged_at for --append (default: now UTC)",
    )
    args = parser.parse_args()

    if args.check:
        return check_manifest(args.manifest, args.merged_dir)
    if args.scan:
        return scan_missing(args.manifest, args.merged_dir)
    if args.append:
        return append_entry(args.manifest, args.append, merged_at=args.merged_at)

    parser.print_help()
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
