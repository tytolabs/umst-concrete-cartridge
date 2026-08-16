SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
<!--
-->

# Canonical headline metrics (`table_per_dataset_metrics.csv`)

Deterministic CSV of compressive-strength residuals for bundled calibration profiles against the dataset mirrors under [`datasets/`](../../datasets). Authoritative row counts, file digests, and citations: [`datasets/PROVENANCE.md`](../../datasets/PROVENANCE.md).

## Columns

| Column | Meaning |
|--------|---------|
| `profile_id` | Bundled calibration id (`calibration/profiles/*.v1.toml`). |
| `dataset_csv` | Evaluated CSV filename (same stem as under `datasets/`). |
| `n_rows` | Rows with valid predictions. |
| `mae` | Mean absolute error (MPa) vs recorded strength (column index 8). |
| `rmse` | Root mean square error (MPa). |
| `r2` | Coefficient of determination \(R^2\) on the slice. |
| `max_abs_error` | Largest absolute residual (MPa). |
| `verification_status` | `Contract` profiles participate in `[acceptance]` gates in `tests/calibration/dataset_metrics.rs`; `Boundary` profiles omit those assertions. |

## Regeneration

From the cartridge repository root:

```bash
cd umst-concrete-cartridge
cargo run -p umst-cli -q --bin calibration_report > docs/Calibration.md
```

The binary writes Markdown to **stdout** and refreshes this directory (`table_per_dataset_metrics.csv` and this file).

## Manuscript alignment

Aggregate counts in papers should reconcile with [`datasets/PROVENANCE.md`](../../datasets/PROVENANCE.md) and [`docs/SSOT.json`](../../docs/SSOT.json). External excerpts may subset rows; shipped `datasets/*.csv` define the canonical scope.
