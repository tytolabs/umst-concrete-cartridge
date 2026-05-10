<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Canonical headline metrics (`table_per_dataset_metrics.csv`)

This directory holds a **deterministic**, cartridge-native CSV of compressive-strength residuals
versus the mirrored datasets under [`datasets/`](../datasets). Authoritative dataset row counts,
file digests, and bibliographic citations are documented in **[`datasets/PROVENANCE.md`](../datasets/PROVENANCE.md)**.

## Columns

| Column | Meaning |
|--------|---------|
| `profile_id` | Bundled calibration bundle id (`calibration/profiles/*.v1.toml`). |
| `dataset_csv` | CSV filename evaluated (same stem as shipped under `datasets/`). |
| `n_rows` | Number of evaluated mix rows with valid predictions. |
| `mae` | Mean absolute error \(\mathrm{MPa}\) vs recorded strength (column index 8). |
| `rmse` | Root mean square error \(\mathrm{MPa}\). |
| `r2` | Coefficient of determination \(R^2\) on the CSV slice (ordinary least-squares definition). |
| `max_abs_error` | Largest absolute residual \(\mathrm{MPa}\). |
| `verification_status` | `Contract` profiles carry asserted `[acceptance]` gates via `tests/calibration/dataset_metrics.rs`; `Boundary` profiles omit those assertions. |

## Regeneration

Run (from the cartridge repository root):

```bash
cargo run -q --bin calibration_report --features "cli,calibration" > docs/Calibration.md
```

The binary writes **both** Markdown (stdout) **and** this CSV/README pair.

## Manuscript alignment

Aggregate row counts cited in manuscripts or ancillary materials should reconcile with
[`datasets/PROVENANCE.md`](../datasets/PROVENANCE.md) and **`docs/SSOT.json`** in this crate.
Totals may differ when an external excerpt omits subsets of these CSV mirrors—the manifest here
describes exactly what ships in **`datasets/*.csv`**.

