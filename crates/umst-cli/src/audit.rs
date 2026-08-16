// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Parse dataset-style CSV (`dataset_d1` header-compatible) into [`PreparedAuditRow`] payloads.
//!
//! **Aggregate solids fraction** (dimensionless surrogate packed into [`mix_layout`] index 12):
//!
//! \(\phi_{\mathrm{agg}} = \mathrm{clamp}((m_{\mathrm{coarse}} + m_{\mathrm{fine}}) / 2600,\, 10^{-3},\, 0.85)\)
//!
//! Uses **2600 kg/m³** — same illustrative particle density surrogate as homogeneous layout docs.

use std::collections::HashMap;
use std::io::{self, Read};
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use umst_concrete_cartridge::facade::{audit_build_report_v1, PreparedAuditRow};
use umst_concrete_cartridge::{calibration::Profile, homogeneous::MixRow};

const AGG_PARTICLE_DENSITY_KG_M3: f32 = 2_600.0;

/// Normalise permissive CSV header tokens to canonical field keys.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Mechanical header synonym routing for CLI CSV audit ergonomics only.
pub fn canon_header(tok: &str) -> Option<&'static str> {
    let t = tok.trim().to_ascii_lowercase().replace(' ', "_");
    match t.as_str() {
        "cement" | "cement_kg_m3" => Some("cement"),
        "slag" | "blast_furnace_slag" => Some("slag"),
        "fly_ash" | "flyash" => Some("fly_ash"),
        "water" | "water_kg_m3" => Some("water"),
        "superplasticizer" | "superplasticiser" | "plasticizer" | "plasticiser" => {
            Some("superplasticizer")
        }
        "coarse_agg" | "coarse_aggregate" | "coarse" => Some("coarse_agg"),
        "fine_agg" | "fine_aggregate" | "fine" => Some("fine_agg"),
        "age" | "age_days" => Some("age"),
        "temperature" | "temperature_c" | "temp_c" => Some("temperature"),
        "strength" | "compressive_strength_mpa" | "strength_mpa" => Some("strength"),
        _ => None,
    }
}

fn agg_volume_fraction(coarse_kg_m3: f32, fine_kg_m3: f32) -> f32 {
    let v_agg = coarse_kg_m3 / AGG_PARTICLE_DENSITY_KG_M3 + fine_kg_m3 / AGG_PARTICLE_DENSITY_KG_M3;
    v_agg.clamp(1e-3, 0.85)
}

struct OwnedRow {
    mix: MixRow,
    coarse: f32,
    fine: f32,
    strength_obs: Option<f32>,
}

fn parse_owned_row(map: &HashMap<&str, f64>) -> anyhow::Result<OwnedRow> {
    let cement = *map.get("cement").context("missing cement column")?;
    let slag = map.get("slag").copied().unwrap_or(0.0);
    let fly_ash = map.get("fly_ash").copied().unwrap_or(0.0);
    let water = *map.get("water").context("missing water column")?;
    let sp = map.get("superplasticizer").copied().unwrap_or(0.0);
    let coarse = *map.get("coarse_agg").context("missing coarse_agg column")?;
    let fine = *map.get("fine_agg").context("missing fine_agg column")?;
    let age = *map.get("age").context("missing age column")?;
    let temp = *map
        .get("temperature")
        .context("missing temperature column")?;
    let strength_obs = map.get("strength").copied();
    Ok(OwnedRow {
        mix: MixRow {
            cement_kg_m3: cement as f32,
            slag_kg_m3: slug_f32(slag),
            fly_ash_kg_m3: slug_f32(fly_ash),
            water_kg_m3: water as f32,
            superplasticizer_kg_m3: sp as f32,
            age_days: age as f32,
            temperature_c: temp as f32,
        },
        coarse: coarse as f32,
        fine: fine as f32,
        strength_obs: strength_obs.map(slug_f32),
    })
}

#[inline]
fn slug_f32(x: f64) -> f32 {
    x as f32
}

/// Run audit from newline-delimited CSV text (stdin or file contents).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Glue from CSV text to facade [`audit_build_report_v1`] without physical claims.
pub fn audit_csv_buf(profile: &Profile, csv_text: &str, max_rows: Option<usize>) -> Result<Value> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(csv_text.as_bytes());
    let headers = reader.headers()?.clone();
    let mut keys: Vec<Option<&'static str>> = Vec::new();
    for h in headers.iter() {
        keys.push(canon_header(h));
    }

    let mut owned: Vec<(usize, OwnedRow)> = Vec::new();
    for (i, rec) in reader.records().enumerate() {
        if let Some(cap) = max_rows {
            if i >= cap {
                break;
            }
        }
        let rec = rec.with_context(|| format!("csv parse row {}", i + 2))?;
        let mut map: HashMap<&str, f64> = HashMap::new();
        for (col, cell) in rec.iter().enumerate() {
            let Some(can) = keys.get(col).copied().flatten() else {
                continue;
            };
            let v: f64 = cell
                .parse()
                .with_context(|| format!("column {can} numeric parse failed on row {}", i + 2))?;
            map.insert(can, v);
        }
        let row = parse_owned_row(&map)?;
        owned.push((i, row));
    }

    let prepared_storage: Vec<MixRow> = owned.iter().map(|(_, ow)| ow.mix.clone()).collect();
    let mut prep_refs: Vec<PreparedAuditRow> = Vec::with_capacity(owned.len());

    for (slot, (csv_row_ix, ow)) in owned.iter().enumerate() {
        let phi = agg_volume_fraction(ow.coarse, ow.fine);
        prep_refs.push(PreparedAuditRow {
            row_index: *csv_row_ix,
            mix_row: &prepared_storage[slot],
            aggregate_volume_fraction: phi,
            observed_strength_mpa: ow.strength_obs,
        });
    }

    let report = audit_build_report_v1(profile, &prep_refs).map_err(|e| anyhow::anyhow!("{e}"))?;
    serde_json::to_value(&report).context("serialize audit JSON")
}

/// Read stdin to string (used with `--stdin`).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: IO helper for MCP/CLI corpus workflows.
pub fn stdin_to_string() -> Result<String> {
    let stdin = io::stdin();
    let mut buf = String::new();
    stdin.lock().read_to_string(&mut buf)?;
    Ok(buf)
}

/// Load CSV from filesystem path then [`audit_csv_buf`].
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: File IO adapter for corpus audit CLI.
pub fn audit_csv_file(profile: &Profile, path: &Path, max_rows: Option<usize>) -> Result<Value> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    audit_csv_buf(profile, &text, max_rows)
}
