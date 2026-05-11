# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
"""Wrapper: run `pytest notebooks/tests/test_print_ready.py` from repo root."""
from __future__ import annotations

import sys
from pathlib import Path

import pytest

_HERE = Path(__file__).resolve().parent
_TESTS = _HERE / "tests" / "test_print_ready.py"

if __name__ == "__main__":
    raise SystemExit(pytest.main([str(_TESTS), "-v", *sys.argv[1:]]))
