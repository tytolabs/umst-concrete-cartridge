// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Integration test: lifted constants from `uci_d1` influence homogeneous strength on a stratified CSV sample.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::error::Error;
use std::path::PathBuf;

use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::homogeneous::{compressive_strength_mpa, MixRow};

fn csv_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../datasets")
        .join(name)
}

fn row_from_d1_record(r: &csv::StringRecord) -> Result<MixRow, Box<dyn Error>> {
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

#[test]
fn uci_d1_mae_on_sample() -> Result<(), Box<dyn Error>> {
    let profile = Profile::load_bundled("uci_d1")?;
    let p = &profile;

    let mut rdr = csv::Reader::from_path(csv_path("dataset_d1.csv"))?;
    let records: Vec<_> = rdr.records().filter_map(|x| x.ok()).collect();
    let mut rng = StdRng::seed_from_u64(0xC0A1_9E11_2026);
    let sample: Vec<_> = records
        .choose_multiple(&mut rng, records.len().min(50))
        .cloned()
        .collect();

    let mut err_sum = 0.0_f64;
    for rec in &sample {
        let row = row_from_d1_record(rec)?;
        let observed: f64 = rec[8].parse()?;
        let pred = compressive_strength_mpa(p, &row)? as f64;
        err_sum += (pred - observed).abs();
    }
    let mae = err_sum / sample.len() as f64;
    let mae_max = p
        .acceptance
        .strength_mae_max
        .expect("uci_d1 profile must define strength_mae_max");
    assert!(
        mae <= mae_max,
        "MAE {mae} exceeds profile bound {mae_max}; see migration policy (no silent retune)"
    );
    Ok(())
}
