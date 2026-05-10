// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Row-count audit: `docs/SSOT.json` must match on-disk `datasets/*.csv` line totals.
//! Authoritative provenance text is [`datasets/PROVENANCE.md`](../../datasets/PROVENANCE.md).

use std::fs;
use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn count_data_rows(csv_path: &PathBuf) -> u64 {
    let raw =
        fs::read_to_string(csv_path).unwrap_or_else(|e| panic!("read {}: {e}", csv_path.display()));
    let nlines = raw.lines().count();
    assert!(nlines > 0, "empty csv {}", csv_path.display());
    (nlines - 1) as u64
}

#[test]
fn ssot_json_matches_csv_lines() {
    let root = manifest_dir();
    let ssot_path = root.join("../../docs/SSOT.json");
    let json = fs::read_to_string(&ssot_path).expect("read docs/SSOT.json");
    let val: serde_json::Value = serde_json::from_str(&json).expect("parse SSOT.json as JSON");

    let sum_ssot = val["total_data_rows"]
        .as_u64()
        .expect("SSOT.total_data_rows must be present as an unsigned integer literal");
    let arr = val["datasets"]
        .as_array()
        .expect("SSOT.datasets must be a JSON array");

    let mut sum_file = 0_u64;

    for d in arr {
        let csv = d["csv"]
            .as_str()
            .expect("dataset entry missing string field `csv`");
        let expected = d["data_rows"].as_u64().expect("`data_rows` must be u64");
        let path = root.join("../../datasets").join(csv);
        let actual = count_data_rows(&path);
        sum_file += actual;
        assert_eq!(
            actual, expected,
            "row mismatch for `{csv}`: SSOT claims {expected} data rows but counted {actual}"
        );
    }

    assert_eq!(
        sum_file, sum_ssot,
        "`total_data_rows` ({sum_ssot}) must equal summed per-file CSV data rows ({sum_file})"
    );
}
