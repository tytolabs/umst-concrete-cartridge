#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
# Smoke: restore UMST_MEMORY_DB copy and verify row count (Litestream operator checklist).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="${1:?usage: verify_litestream_restore.sh <restored-memory.db> [expected_count]}"
EXPECTED="${2:-}"

if [[ ! -f "$SRC" ]]; then
  echo "missing restored db: $SRC" >&2
  exit 1
fi

COUNT="$(sqlite3 "$SRC" 'SELECT COUNT(*) FROM memory_records;' 2>/dev/null || echo "")"
if [[ -z "$COUNT" ]]; then
  echo "FAIL: memory_records table missing or unreadable in $SRC" >&2
  exit 1
fi

echo "restored memory_records: $COUNT"
if [[ -n "$EXPECTED" && "$COUNT" != "$EXPECTED" ]]; then
  echo "FAIL: expected $EXPECTED rows, got $COUNT" >&2
  exit 1
fi

JOBS="$(sqlite3 "$SRC" 'SELECT COUNT(*) FROM contribute_jobs;' 2>/dev/null || echo "0")"
echo "restored contribute_jobs: $JOBS"
echo "verify_litestream_restore: OK"
