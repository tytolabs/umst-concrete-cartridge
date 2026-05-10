// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

use serde_json::json;
use std::error::Error;

use umst_cli::cli::{predict, serialize_prediction, MixSpec, PredictionWireVersion};
use umst_concrete_cartridge::calibration::Profile;

fn spec(w: f32, tk: f32, fly: f32, sf: f32, age: f32) -> Result<MixSpec, Box<dyn Error>> {
    Ok(MixSpec::try_from(json!({
        "w_c": w as f64,
        "temperature_k": tk as f64,
        "fly_ash_pct": fly as f64,
        "silica_fume_pct": sf as f64,
        "target_age_hours": age as f64,
    }))?)
}

#[test]
fn regime_warnings_contain_field_tokens() -> Result<(), Box<dyn Error>> {
    let p = Profile::load_bundled("uci_d1")?;
    let on = spec(0.40_f32, 293.15_f32, 0.0, 0.0, 672.0)?;
    let bundle_on = predict(&p, &on)?;
    let wire = serialize_prediction(&bundle_on, PredictionWireVersion::V2)?;
    let warns_on = wire["warnings"].as_array().unwrap();
    assert!(
        warns_on.is_empty(),
        "expected in-regime warnings empty, got {warns_on:?}"
    );

    let off_w = spec(0.56_f32, 293.15_f32, 0.0, 0.0, 672.0)?;
    let bundle_off = predict(&p, &off_w)?;
    let wire_off = serialize_prediction(&bundle_off, PredictionWireVersion::V2)?;
    let wlist = wire_off["warnings"].as_array().unwrap();
    assert!(
        wlist
            .iter()
            .any(|v| v.as_str().unwrap_or("").contains("w_c")),
        "{wlist:?}"
    );

    let off_t = spec(0.40_f32, 340.0_f32, 0.0, 0.0, 672.0)?;
    let bundle_t = predict(&p, &off_t)?;
    let wire_t = serialize_prediction(&bundle_t, PredictionWireVersion::V2)?;
    let tlist = wire_t["warnings"].as_array().unwrap();
    assert!(
        tlist.iter().any(|v| v
            .as_str()
            .unwrap_or("")
            .to_ascii_lowercase()
            .contains("temperature")),
        "{tlist:?}"
    );

    let off_scm = spec(0.42_f32, 293.15_f32, 10.0_f32, 2.0_f32, 672.0)?;
    let p_hs = Profile::load_bundled("highscm")?;
    let bundle_sc = predict(&p_hs, &off_scm)?;
    let wire_sc = serialize_prediction(&bundle_sc, PredictionWireVersion::V2)?;
    assert!(
        wire_sc["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| { v.as_str().unwrap_or("").contains("scm_pct") }),
        "{:?}",
        wire_sc["warnings"]
    );

    Ok(())
}

#[test]
fn three_profiles_in_regime_smoke() -> Result<(), Box<dyn Error>> {
    let base = spec(0.42_f32, 293.15_f32, 22.0, 5.0, 672.0)?;
    for name in ["default", "highscm"] {
        let p = Profile::load_bundled(name)?;
        predict(&p, &base)?;
    }
    let uhpc_spec = spec(0.22_f32, 293.15_f32, 2.0, 8.0, 672.0)?;
    let uhpc_p = Profile::load_bundled("uhpc")?;
    predict(&uhpc_p, &uhpc_spec)?;
    Ok(())
}
