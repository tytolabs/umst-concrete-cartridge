// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

#![cfg(feature = "cli")]

//! Headline CSV strength gates apply only to **`[contract].verification_status = "Contract"`** profiles.
//! **Boundary** bundles omit `[acceptance]` (or carry indicative bounds only) and are skipped —
//! see `calibration/SCHEMA.md` and prototype Contract / Boundary doctrine.

use std::collections::BTreeMap;
use std::error::Error;
use std::path::PathBuf;

use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::calibration_metrics::{regression_metrics, RegressionMetrics};
use umst_concrete_cartridge::homogeneous::{compressive_strength_mpa, MixRow};

fn datasets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("datasets")
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

fn metrics(profile_id: &str, csv_name: &str) -> Result<(), Box<dyn Error>> {
    let p = Profile::load_bundled(profile_id)?;
    if p.contract.verification_status == "Boundary" {
        eprintln!(
            "dataset_metrics: skip `{profile_id}` ([contract].verification_status = Boundary — no headline CSV acceptance)"
        );
        return Ok(());
    }

    let mut rdr = csv::Reader::from_path(datasets_dir().join(csv_name))?;
    let records: Vec<_> = rdr.records().filter_map(|x| x.ok()).collect();
    let mut preds = Vec::with_capacity(records.len());
    let mut obs = Vec::with_capacity(records.len());
    for rec in records {
        let row = csv_row_to_mix(&rec)?;
        let y: f64 = rec[8].parse()?;
        let pred = compressive_strength_mpa(&p, &row)? as f64;
        preds.push(pred);
        obs.push(y);
    }
    let RegressionMetrics {
        mae,
        rmse,
        r2,
        max_abs_error: max_err,
        ..
    } = regression_metrics(&preds, &obs);

    let mae_max = p.acceptance.strength_mae_max.unwrap_or(f64::INFINITY);
    let rmse_max = p.acceptance.strength_rmse_max.unwrap_or(f64::INFINITY);
    let r2_min = p.acceptance.strength_r2_min.unwrap_or(f64::NEG_INFINITY);

    eprintln!(
        "{profile_id} ({csv_name}): MAE={mae:.4} RMSE={rmse:.4} R2={r2:.4} max_abs_err={max_err:.4}",
    );

    assert!(
        mae <= mae_max,
        "{profile_id} MAE {mae} exceeds bound {mae_max}"
    );
    assert!(
        rmse <= rmse_max,
        "{profile_id} RMSE {rmse} exceeds bound {rmse_max}"
    );
    assert!(r2 >= r2_min, "{profile_id} R2 {r2} below bound {r2_min}");

    Ok(())
}

#[test]
fn headline_contract_profiles_vs_csv() -> Result<(), Box<dyn Error>> {
    let mut pairs = BTreeMap::new();
    pairs.insert("uci_d1", "dataset_d1.csv");
    pairs.insert("zenodo_ndt", "dataset_d2.csv");
    pairs.insert("zenodo_sonreb", "dataset_d3.csv");
    pairs.insert("zenodo_rh", "dataset_d4.csv");
    pairs.insert("uhpc", "dataset_uhpc.csv");
    pairs.insert("highscm", "dataset_highscm.csv");
    pairs.insert("selfheal", "dataset_selfheal.csv");
    for (id, csv) in pairs {
        metrics(id, csv)?;
    }
    Ok(())
}
