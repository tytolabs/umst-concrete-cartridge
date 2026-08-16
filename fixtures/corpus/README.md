SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
# Audit corpus fixtures (memory bootstrap)

**Provenance SSOT:** [`umst-prototype-2a/prototype/docs/2_Datasets.md`](../../../umst-prototype-2a/prototype/docs/2_Datasets.md)

| File | Rows | Role |
|------|------|------|
| `audit_corpus.v1.csv` | 18,146 | Full unified corpus (`umst-prototype/data/dataset_full.csv`) |
| `dataset_d1.v1.csv` | 1,030 | UCI D1 slice for fast CI / smoke ingest |
| `provenance.v1.json` | — | SHA-256, dataset tags, calibration pin, Zenodo DOI |

## Column schema

`cement, slag, fly_ash, water, superplasticizer, coarse_agg, fine_agg, age, strength, source[, temperature, humidity]`

Compatible with `umst_audit` / `audit_csv_buf` and `scripts/bootstrap_memory_from_audit.py`.

## Bootstrap ingest

```bash
# Full corpus → contribution JSONL (gate re-check at MCP/CLI ingest still required)
python3 scripts/bootstrap_memory_from_audit.py fixtures/corpus/audit_corpus.v1.csv > /tmp/contributions.jsonl

# CI-sized slice
python3 scripts/bootstrap_memory_from_audit.py fixtures/corpus/dataset_d1.v1.csv --limit 100
```

**Calibration pin:** profiles use `provenance_sha256 = 6ca1128a…` per `calibration/profiles/*.v1.toml`.  
**Corpus file hash:** `audit_corpus.v1.csv` → `86645bd9749bd3b429e7f1b41814686c7c97b39fa9c9ac18edc5825faf6b1e1e` (see `provenance.v1.json`).

## Corpus refresh ritual (after git inbox merges)

When **>100 new rows** land in `contributions/merged/` (or quarterly), refresh the bootstrap snapshot:

```bash
# 1. Ingest all merged shards into maintainer DB
export UMST_MEMORY_DB=.umst-memory/memory.db
cat contributions/merged/*/*.jsonl | python3 scripts/ingest_contributions.py --db "$UMST_MEMORY_DB"

# 2. Export audit-style CSV if needed (optional — for paper/zenodo bundles)
#    Use existing audit export tooling or bootstrap reverse path documented in AGENT_MCP.md

# 3. Update provenance sidecar
sha256sum fixtures/corpus/audit_corpus.v1.csv | awk '{print $1}'  # compare to provenance.v1.json

# 4. Verify row count
sqlite3 "$UMST_MEMORY_DB" 'SELECT COUNT(*) FROM memory_records;'

# 5. Re-run agent-layer CI locally
cargo test -p umst-mcp --features agent-layer
python3 scripts/mcp_smoke.py --agent-layer
```

**Relationship:** `audit_corpus.v1.csv` is the **historical bootstrap** (16,146 gate-validated rows after ingest). Git inbox shards are **forward-looking** federated growth — do not rewrite CSV on every small merge; batch refresh only.

## Licenses

- **D1 (UCI):** public-domain research use  
- **D2–D4:** CC-BY 4.0 (Zenodo slices)  
- **D5–D8:** physics-informed synthetic (documented in prototype `2_Datasets.md`)
