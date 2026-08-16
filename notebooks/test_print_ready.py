# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""Wrapper: run `pytest notebooks/tests/test_print_ready.py` from repo root."""
from __future__ import annotations

import sys
from pathlib import Path

import pytest

_HERE = Path(__file__).resolve().parent
_TESTS = _HERE / "tests" / "test_print_ready.py"

if __name__ == "__main__":
    raise SystemExit(pytest.main([str(_TESTS), "-v", *sys.argv[1:]]))
