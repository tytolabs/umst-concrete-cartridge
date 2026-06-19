#!/usr/bin/env python3
"""Validate contribution inbox JSONL for CI (schema, admissible flag, duplicates, optional gate).

Usage:
  python3 scripts/validate_contribution_inbox.py contributions/inbox/*.jsonl
  python3 scripts/validate_contribution_inbox.py --gate-check contributions/inbox/foo.jsonl
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from ingest_contributions import content_id_from_contribution, gate_check_admissible

REPO_ROOT = Path(__file__).resolve().parents[1]
SCHEMA = REPO_ROOT / "schemas" / "contribution.v1.json"
MAX_LINES_DEFAULT = 500


def load_known_content_ids(merged_dir: Path) -> set[str]:
    known: set[str] = set()
    if not merged_dir.is_dir():
        return known
    for path in sorted(merged_dir.rglob("*.jsonl")):
        if path.name == "MANIFEST.jsonl":
            continue
        for line in path.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if not line:
                continue
            try:
                known.add(content_id_from_contribution(json.loads(line)))
            except (json.JSONDecodeError, TypeError):
                continue
    return known


def check_jsonschema(instance_path: Path) -> bool:
    if shutil.which("check-jsonschema") is None:
        try:
            obj = json.loads(instance_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            return False
        return obj.get("schema_version") == "contribution.v1" and isinstance(
            obj.get("mix_spec"), dict
        )
    proc = subprocess.run(
        [
            "check-jsonschema",
            "--schemafile",
            str(SCHEMA),
            str(instance_path),
        ],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        print(proc.stdout + proc.stderr, file=sys.stderr)
    return proc.returncode == 0


def validate_file(
    path: Path,
    *,
    known_ids: set[str],
    max_lines: int,
    gate_check: bool,
    profile: str,
) -> int:
    errors = 0
    lines = [ln.strip() for ln in path.read_text(encoding="utf-8").splitlines() if ln.strip()]
    if len(lines) > max_lines:
        print(f"error: {path}: {len(lines)} lines exceeds max {max_lines}", file=sys.stderr)
        errors += 1
        return errors

    tmp_dir = path.parent / ".validate_tmp"
    tmp_dir.mkdir(exist_ok=True)

    for i, line in enumerate(lines, start=1):
        try:
            obj = json.loads(line)
        except json.JSONDecodeError as e:
            print(f"error: {path}:{i}: invalid JSON: {e}", file=sys.stderr)
            errors += 1
            continue

        tmp = tmp_dir / f"{path.stem}-line{i}.json"
        tmp.write_text(json.dumps(obj, indent=2), encoding="utf-8")
        if not check_jsonschema(tmp):
            print(f"error: {path}:{i}: schema validation failed", file=sys.stderr)
            errors += 1
            continue

        if obj.get("schema_version") != "contribution.v1":
            print(f"error: {path}:{i}: schema_version must be contribution.v1", file=sys.stderr)
            errors += 1
            continue

        if not obj.get("gate_summary", {}).get("admissible"):
            print(f"error: {path}:{i}: gate_summary.admissible must be true", file=sys.stderr)
            errors += 1
            continue

        cid = content_id_from_contribution(obj)
        if cid in known_ids:
            print(f"error: {path}:{i}: duplicate content_id {cid}", file=sys.stderr)
            errors += 1
            continue
        known_ids.add(cid)

        if gate_check and not gate_check_admissible(obj, profile):
            print(f"error: {path}:{i}: umst_gate_check re-check failed", file=sys.stderr)
            errors += 1

    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("files", nargs="+", type=Path)
    parser.add_argument("--merged-dir", type=Path, default=Path("contributions/merged"))
    parser.add_argument("--max-lines", type=int, default=MAX_LINES_DEFAULT)
    parser.add_argument("--gate-check", action="store_true")
    parser.add_argument("--profile", default="default")
    args = parser.parse_args()

    known = load_known_content_ids(args.merged_dir)
    total_errors = 0
    for path in args.files:
        if not path.is_file():
            print(f"error: not a file: {path}", file=sys.stderr)
            total_errors += 1
            continue
        total_errors += validate_file(
            path,
            known_ids=known,
            max_lines=args.max_lines,
            gate_check=args.gate_check,
            profile=args.profile,
        )

    if total_errors:
        print(f"validate_contribution_inbox: {total_errors} error(s)", file=sys.stderr)
        return 1
    print("validate_contribution_inbox: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
