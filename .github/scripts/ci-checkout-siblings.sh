#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
set -euo pipefail
if [ -z "${UMST_PRIVATE_CHECKOUT:-}" ]; then
  echo "::error::UMST_PRIVATE_CHECKOUT secret missing. Operator: gh secret set UMST_PRIVATE_CHECKOUT -R tytolabs/umst-concrete-cartridge --body YOUR_PAT (see workspace/ops/G14_CI.json OPERATOR_ACTION)"
  exit 1
fi
git config --global url."https://x-access-token:${UMST_PRIVATE_CHECKOUT}@github.com/".insteadOf "https://github.com/"
bash .github/scripts/checkout-umst-siblings.sh
