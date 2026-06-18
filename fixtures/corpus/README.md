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

## Licenses

- **D1 (UCI):** public-domain research use  
- **D2–D4:** CC-BY 4.0 (Zenodo slices)  
- **D5–D8:** physics-informed synthetic (documented in prototype `2_Datasets.md`)
