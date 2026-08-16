#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""CLI vs Python `predict`: byte-identical canonical JSON for the same mix + profile."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--profile", default="uci_d1")
    ap.add_argument(
        "--mix-json",
        default='{"w_c":0.4,"temperature_k":293.15}',
        help="Mix JSON on stdin to umst predict (default UCI-style headline slice)",
    )
    ap.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parent.parent,
    )
    ap.add_argument(
        "--profile-name",
        dest="cargo_profile",
        default=os.environ.get("PROFILE", "debug"),
        help="Cargo profile directory under target/ (default $PROFILE or debug)",
    )
    args = ap.parse_args()

    root: Path = args.repo_root
    umst = root / "target" / args.cargo_profile / "umst"
    canon = root / "target" / args.cargo_profile / "umst-canonical"
    if not umst.is_file():
        print(f"missing {umst}; build workspace first", file=sys.stderr)
        sys.exit(2)
    if not canon.is_file():
        print(f"missing {canon}; cargo build -p umst-cli --bin umst-canonical", file=sys.stderr)
        sys.exit(2)

    mix_bytes = args.mix_json.encode()

    pred = subprocess.run(
        [str(umst), "--profile", args.profile, "predict"],
        input=mix_bytes,
        cwd=str(root),
        capture_output=True,
        check=False,
    )
    if pred.returncode != 0:
        sys.stderr.write(pred.stderr.decode())
        sys.exit(1)

    cli_canon = subprocess.run(
        [str(canon)],
        input=pred.stdout,
        cwd=str(root),
        capture_output=True,
        check=True,
    ).stdout

    try:
        from umst_concrete_cartridge import canonical_json, predict
    except ImportError:
        print(
            "umst_concrete_cartridge not importable; run: "
            "(cd crates/umst-py && maturin develop)",
            file=sys.stderr,
        )
        sys.exit(2)

    spec = json.loads(args.mix_json)
    out = predict(spec, profile=args.profile)
    py_canon = canonical_json(out)

    if cli_canon != py_canon:
        sys.stderr.write(
            f"canonical mismatch CLI len={len(cli_canon)} Python len={len(py_canon)}\n"
        )
        sys.stderr.write(cli_canon[:400].decode(errors="replace") + "\n---\n")
        sys.stderr.write(py_canon[:400].decode(errors="replace") + "\n")
        sys.exit(1)

    print("predict determinism OK", len(cli_canon), "bytes")


if __name__ == "__main__":
    main()
