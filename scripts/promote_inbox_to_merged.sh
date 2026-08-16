#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
# Maintainer: move one inbox JSONL shard to contributions/merged/YYYY-MM/ and append MANIFEST.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
INBOX="${1:?usage: promote_inbox_to_merged.sh contributions/inbox/<file>.jsonl}"

cd "$ROOT"
if [[ ! -f "$INBOX" ]]; then
  echo "missing inbox file: $INBOX" >&2
  exit 1
fi
case "$INBOX" in
  contributions/inbox/*.jsonl) ;;
  *)
    echo "path must be under contributions/inbox/: $INBOX" >&2
    exit 1
    ;;
esac

YM="$(date -u +%Y-%m)"
DEST_DIR="contributions/merged/${YM}"
mkdir -p "$DEST_DIR"
BASENAME="$(basename "$INBOX")"
DEST="${DEST_DIR}/${BASENAME}"

git mv "$INBOX" "$DEST"
python3 scripts/update_contribution_manifest.py --append "$DEST"
echo "promoted → $DEST (manifest updated)"
