#!/usr/bin/env bash
# Federated git inbox dry-run (no git push).
# Categorical:
#   Path: COLD (scripts / filesystem — not MCP promote)
#   Morphisms: export_contributions_jsonl → validate (Proposed: umst_promote_contribution)
#   Requires UMST_MEMORY_DB with at least one admissible row.
#   Docs: docs/EPISTEMIC_PRIMITIVES.md (human-gated promotion)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

DB="${UMST_MEMORY_DB:-.umst-memory/memory.db}"
LAB="${UMST_LAB_SLUG:-example-lab}"
OUT="contributions/inbox/${LAB}-dryrun.jsonl"

if [[ ! -f "$DB" ]]; then
  echo "Set UMST_MEMORY_DB to a populated SQLite file (run 02_contribute_admissible.py first)." >&2
  exit 1
fi

echo "Exporting from $DB → $OUT"
python3 scripts/export_contributions_jsonl.py --db "$DB" --lab "$LAB" --out "$OUT"
python3 scripts/validate_contribution_inbox.py "$OUT"
python3 scripts/ingest_contributions.py "$OUT" --dry-run --skip-gate
echo "03_export_inbox dry-run: ok (no PR created)"