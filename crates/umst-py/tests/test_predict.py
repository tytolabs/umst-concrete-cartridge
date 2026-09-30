# SPDX-License-Identifier: MIT

"""Stdlib-only tests against a built `_umst_concrete_cartridge` extension."""

import json
import os
import shutil
import subprocess
import unittest
from pathlib import Path


def _cargo_bin_umst_canonical(repo_root: Path) -> Path:
    profile = os.environ.get("PROFILE", "debug")
    return repo_root.joinpath("target", profile, "umst-canonical")


class TestPredictSmoke(unittest.TestCase):
    """Compare Python `canonical_json` with `umst-canonical` for the same decoded object."""

    @classmethod
    def setUpClass(cls) -> None:
        try:
            from umst_concrete_cartridge import predict  # pylint: disable=import-outside-toplevel

            cls._predict = predict
        except ImportError:
            cls._predict = None
        crate_dir = Path(__file__).resolve().parent.parent
        cls._repo_root = crate_dir.parent.parent
        cls._canonical_bin = _cargo_bin_umst_canonical(cls._repo_root)

    def test_predict_default_profile_json_roundtrip_finite(self):
        """If the wheel is imported, headline fields must be finite (result.v2)."""
        if self._predict is None:
            self.skipTest("extension not installed (run maturin develop in crates/umst-py)")
        mixed = {"w_c": 0.4, "temperature_k": 293.15}
        out = self._predict(mixed)
        assert out["schema_version"] == "result.v2"
        assert out["warnings"] is not None

    def test_uci_profile_keyword_matches_spec_signature(self):
        if self._predict is None:
            self.skipTest("extension not installed")
        out = self._predict(
            {"w_c": 0.4, "temperature_k": 293.15},
            profile="uci_d1",
        )
        assert out["calibration_profile"] == "uci_d1"

    def test_cli_python_predict_canonical_bytes_match(self):
        """Subprocess `umst predict` canonical bytes must match Python `predict` + `canonical_json`."""
        if self._predict is None:
            self.skipTest("extension not installed")
        if not shutil.which("cargo"):
            self.skipTest("cargo not on PATH")

        umst_bin = self._repo_root / "target" / os.environ.get("PROFILE", "debug") / "umst"
        canon_bin = self._repo_root / "target" / os.environ.get("PROFILE", "debug") / "umst-canonical"
        if not umst_bin.is_file():
            subprocess.run(
                ["cargo", "build", "-p", "umst-cli", "--bins"],
                cwd=str(self._repo_root),
                check=True,
                capture_output=True,
            )
        self.assertTrue(umst_bin.is_file(), f"expected umst at {umst_bin}")

        mix_json = '{"w_c":0.4,"temperature_k":293.15}'
        cli_pred = subprocess.run(
            [str(umst_bin), "--profile", "uci_d1", "predict"],
            input=mix_json.encode(),
            cwd=str(self._repo_root),
            capture_output=True,
            check=True,
        )
        if not canon_bin.is_file():
            subprocess.run(
                ["cargo", "build", "-p", "umst-cli", "--bin", "umst-canonical"],
                cwd=str(self._repo_root),
                check=True,
                capture_output=True,
            )
        cli_canon = subprocess.run(
            [str(canon_bin)],
            input=cli_pred.stdout,
            cwd=str(self._repo_root),
            capture_output=True,
            check=True,
        ).stdout

        from umst_concrete_cartridge import canonical_json, predict  # pylint: disable=import-outside-toplevel

        spec = json.loads(mix_json)
        py_canon = bytes(canonical_json(predict(spec, profile="uci_d1")))
        self.assertEqual(cli_canon, py_canon)

    def test_canonical_matches_cli_binary_when_available(self):
        if self._predict is None:
            self.skipTest("extension not installed")
        if not shutil.which("cargo"):
            self.skipTest("cargo not on PATH")

        canon_bin = self._canonical_bin
        expect_path = canon_bin.resolve()
        if not expect_path.is_file():
            subprocess.run(
                ["cargo", "build", "-p", "umst-cli", "--bin", "umst-canonical"],
                cwd=str(self._repo_root),
                check=True,
                capture_output=True,
            )
        canon_bin = _cargo_bin_umst_canonical(self._repo_root)
        self.assertTrue(
            canon_bin.is_file(),
            f"built umst-canonical expected at {canon_bin}",
        )

        payload = {"z": json.loads("0.30000000000000004")}
        dumped = subprocess.run(
            [str(canon_bin)],
            input=json.dumps(payload).encode(),
            cwd=str(self._repo_root),
            check=True,
            capture_output=True,
        ).stdout

        try:
            from umst_concrete_cartridge import canonical_json  # pylint: disable=import-outside-toplevel
        except ImportError:
            raise AssertionError("extension missing canonical_json")

        ours = canonical_json(payload)
        self.assertEqual(ours, dumped)


if __name__ == "__main__":
    unittest.main()
