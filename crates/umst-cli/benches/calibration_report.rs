// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

#![forbid(unsafe_code)]

//! Deterministic Markdown report of bundled calibration profiles (commit to `docs/Calibration.md`).
//! Also writes `results/canonical/table_per_dataset_metrics.csv` and `results/canonical/README.md`.
//! Dataset SHA-256 and row counts are authoritative in `datasets/PROVENANCE.md`.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use sha2::{Digest, Sha256};
use umst_concrete_cartridge::calibration::{Profile, BUNDLED_PROFILE_IDS};
use umst_concrete_cartridge::calibration_metrics::regression_metrics;
use umst_concrete_cartridge::homogeneous::{compressive_strength_mpa, MixRow};

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

fn datasets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../datasets")
}

fn csv_row_to_mix(r: &csv::StringRecord) -> Result<MixRow, Box<dyn Error>> {
    Ok(MixRow {
        cement_kg_m3: r[0].parse()?,
        slag_kg_m3: r[1].parse()?,
        fly_ash_kg_m3: r[2].parse()?,
        water_kg_m3: r[3].parse()?,
        superplasticizer_kg_m3: r[4].parse()?,
        age_days: r[7].parse::<f64>()? as f32,
        temperature_c: r[10].parse::<f64>()? as f32,
    })
}

#[derive(Clone, Debug)]
struct DatasetCsvRow {
    profile_id: String,
    csv_name: String,
    verification_status: String,
    n: usize,
    mae: f64,
    rmse: f64,
    r2: f64,
    max_abs: f64,
}

fn headline_metrics_for_csv(
    profile_id: &str,
    csv_name: &str,
) -> Result<DatasetCsvRow, Box<dyn Error>> {
    let p = Profile::load_bundled(profile_id)?;
    let mut rdr = csv::Reader::from_path(datasets_dir().join(csv_name))?;
    let records: Vec<_> = rdr.records().filter_map(|x| x.ok()).collect();
    let mut preds = Vec::with_capacity(records.len());
    let mut obs = Vec::with_capacity(records.len());
    for rec in records {
        let row = csv_row_to_mix(&rec)?;
        let y: f64 = rec[8].parse()?;
        preds.push(compressive_strength_mpa(&p, &row)? as f64);
        obs.push(y);
    }
    let m = regression_metrics(&preds, &obs);
    Ok(DatasetCsvRow {
        profile_id: profile_id.to_string(),
        csv_name: csv_name.to_string(),
        verification_status: p.contract.verification_status.clone(),
        n: m.n,
        mae: m.mae,
        rmse: m.rmse,
        r2: m.r2,
        max_abs: m.max_abs_error,
    })
}

fn write_canonical_tables(rows: &[DatasetCsvRow]) -> anyhow::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = root.join("results/canonical");
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;

    let csv_path = dir.join("table_per_dataset_metrics.csv");
    let mut wtr = csv::Writer::from_path(&csv_path)
        .with_context(|| format!("open {}", csv_path.display()))?;
    wtr.write_record([
        "profile_id",
        "dataset_csv",
        "n_rows",
        "mae",
        "rmse",
        "r2",
        "max_abs_error",
        "verification_status",
    ])?;
    for r in rows {
        wtr.write_record([
            r.profile_id.as_str(),
            r.csv_name.as_str(),
            &r.n.to_string(),
            &format!("{:.10}", r.mae),
            &format!("{:.10}", r.rmse),
            &format!("{:.10}", r.r2),
            &format!("{:.10}", r.max_abs),
            r.verification_status.as_str(),
        ])?;
    }
    wtr.flush()?;

    let readme = dir.join("README.md");
    let mut body = String::from(
        r#"<!--
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
cargo run -p umst-cli -q --bin calibration_report > docs/Calibration.md
```

The binary writes **both** Markdown (stdout) **and** this CSV/README pair.

## Manuscript alignment

Aggregate row counts cited in manuscripts or ancillary materials should reconcile with
[`datasets/PROVENANCE.md`](../datasets/PROVENANCE.md) and **`docs/SSOT.json`** in this crate.
Totals may differ when an external excerpt omits subsets of these CSV mirrors—the manifest here
describes exactly what ships in **`datasets/*.csv`**.
"#,
    );
    body.push('\n');
    fs::write(&readme, body).with_context(|| format!("write {}", readme.display()))?;
    Ok(())
}

fn dataset_pairs_ordered() -> Vec<(&'static str, &'static str)> {
    vec![
        ("uci_d1", "dataset_d1.csv"),
        ("zenodo_ndt", "dataset_d2.csv"),
        ("zenodo_sonreb", "dataset_d3.csv"),
        ("zenodo_rh", "dataset_d4.csv"),
        ("uhpc", "dataset_uhpc.csv"),
        ("highscm", "dataset_highscm.csv"),
        ("selfheal", "dataset_selfheal.csv"),
    ]
}

fn main() -> anyhow::Result<()> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let profiles_dir = manifest.join("calibration/profiles");

    let mut ids: Vec<&str> = BUNDLED_PROFILE_IDS.to_vec();
    ids.sort_unstable();

    println!("# Calibration report");
    println!();
    println!("Generated by the `calibration_report` binary (`cargo run -p umst-cli --bin calibration_report`).");
    println!("Keys and sections are sorted for stable diffs.");
    println!();

    println!("## Bundled TOML SHA-256 (UTF-8)");
    println!();
    let mut digests: BTreeMap<&str, String> = BTreeMap::new();
    for id in &ids {
        let path = profiles_dir.join(format!("{id}.v1.toml"));
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("read profile file {}", path.display()))?;
        digests.insert(id, sha256_hex(raw.as_bytes()));
    }
    for (id, d) in &digests {
        println!("- `{id}`: `{d}`");
    }
    println!();

    println!("## Parsed profile summaries");
    println!();
    for id in &ids {
        let p = Profile::load_bundled(id).with_context(|| format!("load bundled `{id}`"))?;
        println!("### `{id}`");
        println!();
        println!("| key | value |");
        println!("|-----|-------|");
        println!("| model_kind | {:?} |", p.model_section.kind);
        let wmin = p.regime.w_c_min;
        let wmax = p.regime.w_c_max;
        println!("| regime.w_c_min | {wmin:.4} |");
        println!("| regime.w_c_max | {wmax:.4} |");
        let tmin = p.regime.temperature_k_min;
        let tmax = p.regime.temperature_k_max;
        println!("| regime.temperature_k_min | {tmin:.4} |");
        println!("| regime.temperature_k_max | {tmax:.4} |");
        let amin = p.regime.age_hours_min;
        let amax = p.regime.age_hours_max;
        println!("| regime.age_hours_min | {amin:.4} |");
        println!("| regime.age_hours_max | {amax:.4} |");
        if let Some(x) = p.regime.fly_ash_pct_max {
            println!("| regime.fly_ash_pct_max | {x:.4} |");
        }
        if let Some(x) = p.regime.silica_fume_pct_max {
            println!("| regime.silica_fume_pct_max | {x:.4} |");
        }
        if let Some(x) = p.regime.scm_sum_min_pct {
            println!("| regime.scm_sum_min_pct | {x:.4} |");
        }
        let si = p.powers.s_intrinsic;
        let ksl = p.powers.k_slag;
        let kfa = p.powers.k_fly_ash;
        let kr = p.powers.k_ref;
        let eb = p.powers.early_boost;
        println!("| powers.s_intrinsic | {si:.4} |");
        println!("| powers.k_slag | {ksl:.4} |");
        println!("| powers.k_fly_ash | {kfa:.4} |");
        println!("| powers.k_ref | {kr:.4} |");
        println!("| powers.early_boost | {eb:.4} |");
        if let Some(x) = p.acceptance.strength_mae_max {
            println!("| acceptance.strength_mae_max | {x:.6} |");
        }
        if let Some(x) = p.acceptance.strength_rmse_max {
            println!("| acceptance.strength_rmse_max | {x:.6} |");
        }
        if let Some(x) = p.acceptance.strength_r2_min {
            println!("| acceptance.strength_r2_min | {x:.6} |");
        }
        let sha = &p.provenance.provenance_sha256;
        println!("| provenance.provenance_sha256 | {sha} |");
        let vstat = &p.contract.verification_status;
        println!("| contract.verification_status | {vstat} |");
        println!();
    }

    println!("## Per-dataset headline metrics (CSV artefact)");
    println!();
    println!(
        "See `results/canonical/table_per_dataset_metrics.csv`, regenerated alongside this binary. Row totals for each mirror file are keyed in **`datasets/PROVENANCE.md`** and audited by **`tests/calibration/ssot_row_counts`**."
    );

    let mut canon_rows = Vec::new();
    for (pid, csv) in dataset_pairs_ordered() {
        let row = headline_metrics_for_csv(pid, csv)
            .map_err(|e| anyhow::anyhow!("metrics for `{pid}` / `{csv}`: {e}"))?;
        canon_rows.push(row);
    }

    canon_rows.sort_by(|a, b| a.profile_id.cmp(&b.profile_id));
    write_canonical_tables(&canon_rows).context("write results/canonical")?;

    Ok(())
}
