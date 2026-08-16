#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
# Sigstore cosign sign-blob for promotion_record.v1 (operator-only).
# Usage: cosign_promotion_bundle.sh PROMOTION_RECORD.json [OUT_DIR]
# Env: COSIGN_IDENTITY (keyless), or COSIGN_KEY / COSIGN_PASSWORD for keyed mode.
#      SKIP_COSIGN=1 writes stub sidecar only.

set -euo pipefail

RECORD="${1:?promotion_record.json required}"
OUT_DIR="${2:-$(dirname "$RECORD")}"
BASENAME="$(basename "$RECORD" .json)"

mkdir -p "$OUT_DIR"

if [[ "${SKIP_COSIGN:-}" == "1" ]]; then
  cat >"$OUT_DIR/${BASENAME}.sigstore.stub.json" <<EOF
{"schema_version":"promotion_sigstore_stub.v1","note":"SKIP_COSIGN=1 — no cosign signature","source":"$RECORD"}
EOF
  echo "info: wrote stub Sigstore sidecar (SKIP_COSIGN=1)"
  exit 0
fi

command -v cosign >/dev/null || { echo "error: cosign CLI required (https://docs.sigstore.dev)" >&2; exit 1; }

SIG="$OUT_DIR/${BASENAME}.cosign.sig"
CERT="$OUT_DIR/${BASENAME}.cosign.crt"

if [[ -n "${COSIGN_KEY:-}" ]]; then
  cosign sign-blob --key "$COSIGN_KEY" --output-signature "$SIG" --output-certificate "$CERT" "$RECORD"
else
  cosign sign-blob --yes --output-signature "$SIG" --output-certificate "$CERT" "$RECORD"
fi

cat >"$OUT_DIR/${BASENAME}.sigstore.meta.json" <<EOF
{"schema_version":"promotion_sigstore_meta.v1","record":"$RECORD","signature":"$SIG","certificate":"$CERT"}
EOF

echo "info: cosign signature written to $SIG"
