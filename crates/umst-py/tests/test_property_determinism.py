# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""Property test: CLI and Python ``predict`` canonical bytes match (Hypothesis)."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import unittest
from pathlib import Path

from hypothesis import assume, given, settings
from hypothesis import strategies as st


def _repo_root() -> Path:
    crate_dir = Path(__file__).resolve().parent.parent
    return crate_dir.parent.parent


def _umst_bin(root: Path) -> Path:
    profile = os.environ.get("PROFILE", "debug")
    return root / "target" / profile / "umst"


def _canon_bin(root: Path) -> Path:
    profile = os.environ.get("PROFILE", "debug")
    return root / "target" / profile / "umst-canonical"


def _have_tooling() -> bool:
    try:
        import umst_concrete_cartridge  # noqa: F401
    except ImportError:
        return False
    if not shutil.which("cargo"):
        return False
    root = _repo_root()
    if not _umst_bin(root).is_file():
        subprocess.run(
            ["cargo", "build", "-p", "umst-cli", "--bin", "umst"],
            cwd=str(root),
            check=True,
            capture_output=True,
        )
    if not _canon_bin(root).is_file():
        subprocess.run(
            ["cargo", "build", "-p", "umst-cli", "--bin", "umst-canonical"],
            cwd=str(root),
            check=True,
            capture_output=True,
        )
    return _umst_bin(root).is_file() and _canon_bin(root).is_file()


@unittest.skipUnless(_have_tooling(), "cargo + extension + umst binary required")
class TestPropertyDeterminism(unittest.TestCase):
    @settings(max_examples=20, deadline=None)
    @given(
        w_c=st.floats(min_value=0.38, max_value=0.42, allow_nan=False, allow_infinity=False),
        temperature_k=st.floats(
            min_value=290.0, max_value=296.0, allow_nan=False, allow_infinity=False
        ),
    )
    def test_cli_python_bytes_all_profiles(self, w_c: float, temperature_k: float) -> None:
        from umst_concrete_cartridge import bundled_profile_ids, canonical_json, predict

        root = _repo_root()
        umst = _umst_bin(root)
        canon = _canon_bin(root)
        spec = {"w_c": w_c, "temperature_k": temperature_k}
        payload = json.dumps(spec).encode()

        for profile in bundled_profile_ids():
            proc = subprocess.run(
                [str(umst), "--profile", profile, "predict"],
                input=payload,
                cwd=str(root),
                capture_output=True,
            )
            if proc.returncode != 0:
                assume(False)
            cli_canon = subprocess.run(
                [str(canon)],
                input=proc.stdout,
                cwd=str(root),
                capture_output=True,
            )
            if cli_canon.returncode != 0:
                assume(False)
            try:
                py_dict = predict(spec, profile=profile)
            except Exception:
                assume(False)
            py_bytes = bytes(canonical_json(py_dict))
            self.assertEqual(
                cli_canon.stdout,
                py_bytes,
                msg=f"profile={profile} spec={spec!r}",
            )


if __name__ == "__main__":
    unittest.main()
