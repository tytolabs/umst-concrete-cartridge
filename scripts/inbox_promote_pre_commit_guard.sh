#!/usr/bin/env bash
# H5 — inbox-promote bot defense-in-depth.
#
# Re-validate merged shard paths with live gate re-check and manifest consistency
# immediately before `git commit` in the promote bot or maintainer ritual.
#
# Usage (umst-concrete-cartridge repo root):
#   bash scripts/inbox_promote_pre_commit_guard.sh \
#     contributions/merged/2026-06/lab-foo-20260624-a1b2.jsonl

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [ "$#" -lt 1 ]; then
  echo "usage: inbox_promote_pre_commit_guard.sh <merged-shard.jsonl> [...]" >&2
  exit 2
fi

for shard in "$@"; do
  if [ ! -f "$shard" ]; then
    echo "inbox_promote_pre_commit_guard: missing shard: $shard" >&2
    exit 1
  fi
  python3 scripts/validate_contribution_inbox.py --gate-check "$shard"
done

python3 scripts/validate_contribution_inbox.py --check-manifest
echo "inbox_promote_pre_commit_guard: ok (${#@} shard(s))"
