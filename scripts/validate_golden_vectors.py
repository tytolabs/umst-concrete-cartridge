#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""Validate phase8_adversarial golden-vector SSOT manifest and fixture presence.

Usage:
  python3 scripts/validate_golden_vectors.py
  python3 scripts/validate_golden_vectors.py --manifest tests/fixtures/phase8_adversarial.json
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_MANIFEST = REPO_ROOT / "tests" / "fixtures" / "phase8_adversarial.json"
REQUIRED_TOP_KEYS = {
    "schema_version",
    "description",
    "rust_test_binary",
    "fixture_dir",
    "fixtures",
    "inline_vectors",
    "query_page_cases",
}


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError(f"{path}: invalid JSON: {exc}") from exc


def validate_manifest(manifest_path: Path) -> list[str]:
    errors: list[str] = []
    if not manifest_path.is_file():
        return [f"missing manifest: {manifest_path}"]

    data = load_json(manifest_path)
    missing = REQUIRED_TOP_KEYS - data.keys()
    if missing:
        errors.append(f"{manifest_path}: missing keys: {sorted(missing)}")

    if data.get("schema_version") != "phase8_adversarial_golden.v1":
        errors.append(
            f"{manifest_path}: unexpected schema_version {data.get('schema_version')!r}"
        )

    fixture_dir = REPO_ROOT / str(data.get("fixture_dir", ""))
    if not fixture_dir.is_dir():
        errors.append(f"missing fixture_dir: {fixture_dir}")
        return errors

    for entry in data.get("fixtures", []):
        name = entry.get("file")
        if not name:
            errors.append("fixture entry missing 'file'")
            continue
        path = fixture_dir / name
        if not path.is_file():
            errors.append(f"missing fixture file: {path}")

    for vector in data.get("inline_vectors", []):
        if "id" not in vector:
            errors.append("inline_vector missing 'id'")
        mix = vector.get("mix_spec")
        if not isinstance(mix, dict) or not mix:
            errors.append(f"inline_vector {vector.get('id')!r}: mix_spec must be non-empty object")

    for case in data.get("query_page_cases", []):
        if "id" not in case:
            errors.append("query_page_case missing 'id'")

    related = data.get("related_ssot", {})
    verdict_path = related.get("verdict_parity")
    if verdict_path:
        full = REPO_ROOT / verdict_path
        if not full.is_file():
            errors.append(f"missing related_ssot.verdict_parity: {full}")

    return errors


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--manifest",
        type=Path,
        default=DEFAULT_MANIFEST,
        help="path to phase8_adversarial golden manifest",
    )
    args = ap.parse_args()

    errors = validate_manifest(args.manifest.resolve())
    if errors:
        for err in errors:
            print(f"error: {err}", file=sys.stderr)
        sys.exit(1)

    print(json.dumps({"ok": True, "manifest": str(args.manifest)}, indent=2))


if __name__ == "__main__":
    main()
