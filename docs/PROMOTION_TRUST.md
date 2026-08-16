SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
# Promotion bundle trust — operator runbook

**Audience:** Human calibration reviewers (never MCP-exposed)  
**Policy:** [`governance/promotion_policy.yaml`](../governance/promotion_policy.yaml)

---

## Flow

```text
umst promote-contribution MEMORY_ID --approval-file approval.json
  → promotion_record.v1.json sidecar
  → (optional) RFC 3161 TSA countersign
  → (optional) Sigstore cosign sign-blob
  → pending_calibration/ handoff
```

When `UMST_UCRS_WITNESS=live`, the memory row must carry `stamp_tier: UcrsTier2` (`validate_track_a_stamp_tier` in `promotion.rs`).

---

## RFC 3161 timestamp (external anchor)

```bash
# Production: FreeTSA or your org TSA
export TSA_URL=https://freetsa.org/tsr
./scripts/promotion_tsa_timestamp.sh .umst-memory/promotions/MEM123.promotion_record.v1.json

# CI / offline: stub sidecar only
SKIP_TSA=1 ./scripts/promotion_tsa_timestamp.sh path/to/record.json
```

Outputs beside the record:

| File | Role |
|------|------|
| `*.tsr` | DER timestamp token |
| `*.tsa.meta.json` | Operator metadata |
| `*.tsa.stub.json` | Honest skip when `SKIP_TSA=1` |

---

## Sigstore (bundle signing)

```bash
# Keyless (requires cosign + OIDC)
./scripts/cosign_promotion_bundle.sh path/to/promotion_record.v1.json

# Keyed
export COSIGN_KEY=/path/to/cosign.key
./scripts/cosign_promotion_bundle.sh path/to/promotion_record.v1.json

# CI / offline
SKIP_COSIGN=1 ./scripts/cosign_promotion_bundle.sh path/to/record.json
```

Per-contribute Sigstore is **not** in scope — only the human promotion bundle.

---

## Related

- [`AGENT_MCP.md`](AGENT_MCP.md) — promotion never via MCP  
- [`MEMORY_REPLICATION.md`](MEMORY_REPLICATION.md) — SQLite durability  
- [`CLI.md`](CLI.md) — `umst promote-contribution`
