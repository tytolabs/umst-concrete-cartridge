#!/usr/bin/env bash
# RFC 3161 timestamp countersign for promotion_record.v1 sidecar (operator-only).
# Usage: promotion_tsa_timestamp.sh PROMOTION_RECORD.json [OUT_DIR]
# Env: TSA_URL (default FreeTSA), SKIP_TSA=1 to write stub sidecar only.

set -euo pipefail

RECORD="${1:?promotion_record.json required}"
OUT_DIR="${2:-$(dirname "$RECORD")}"
TSA_URL="${TSA_URL:-https://freetsa.org/tsr}"
BASENAME="$(basename "$RECORD" .json)"

mkdir -p "$OUT_DIR"

if [[ "${SKIP_TSA:-}" == "1" ]]; then
  cat >"$OUT_DIR/${BASENAME}.tsa.stub.json" <<EOF
{"schema_version":"promotion_tsa_stub.v1","note":"SKIP_TSA=1 — no RFC3161 token","source":"$RECORD"}
EOF
  echo "info: wrote stub TSA sidecar (SKIP_TSA=1)"
  exit 0
fi

command -v openssl >/dev/null || { echo "error: openssl required" >&2; exit 1; }

TSQ="$OUT_DIR/${BASENAME}.tsq"
TSR="$OUT_DIR/${BASENAME}.tsr"

openssl ts -query -data "$RECORD" -sha256 -cert -out "$TSQ"
curl -sSf -H "Content-Type: application/timestamp-query" \
  --data-binary @"$TSQ" "$TSA_URL" -o "$TSR"

openssl ts -reply -in "$TSR" -text >"$OUT_DIR/${BASENAME}.tsa.txt" 2>/dev/null || true

cat >"$OUT_DIR/${BASENAME}.tsa.meta.json" <<EOF
{"schema_version":"promotion_tsa_meta.v1","record":"$RECORD","tsr":"$TSR","tsa_url":"$TSA_URL"}
EOF

echo "info: RFC 3161 token written to $TSR"
