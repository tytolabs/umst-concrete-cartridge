// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! A lightweight MCP (Model Context Protocol) server that exposes the
//! UMST concrete cartridge to AI agents via JSON-RPC 2.0 over stdin/stdout.
//!
//! Start with `cargo run -p umst-mcp` and point Claude Desktop (or any
//! MCP-compatible agent) at the resulting binary.

use std::io::{self, BufRead, Write};

use anyhow::Result;
use burn_ndarray::{NdArray, NdArrayDevice};
use serde_json::{json, Value};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::homogeneous::{
    compressive_strength_mpa, degree_of_hydration_alpha, embodied_co2_kg_per_m3, safety_margin,
    yield_stress_pa, MixRow,
};
use umst_concrete_cartridge::mix_layout::{fractions_from_mix_row, mix_tensor_from_layout};
use umst_concrete_cartridge::run_full_physics_pipeline;

type Backend = NdArray<f32>;

// ---------------------------------------------------------------------------
// Entry-point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    // Tracing goes to stderr so it never contaminates the JSON-RPC channel.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("UMST MCP server started (stdio transport).");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Value>(&line) {
            Ok(req) => dispatch(&req),
            Err(e) => {
                tracing::error!("Bad JSON-RPC frame: {e}");
                continue;
            }
        };

        let out = serde_json::to_string(&response)? + "\n";
        stdout.write_all(out.as_bytes())?;
        stdout.flush()?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// JSON-RPC dispatcher
// ---------------------------------------------------------------------------

fn dispatch(req: &Value) -> Value {
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

    match method {
        // ── MCP discovery ──────────────────────────────────────────────
        "initialize" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "umst-concrete-cartridge",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        }),

        "tools/list" => handle_tools_list(id),
        "tools/call" => handle_tools_call(id, req.get("params")),

        _ => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("Method not found: {method}") }
        }),
    }
}

// ---------------------------------------------------------------------------
// Tool catalogue
// ---------------------------------------------------------------------------

fn handle_tools_list(id: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": [
                {
                    "name": "evaluate_mix",
                    "description": "Run tensor physics pipeline (`run_full_physics_pipeline`) plus homogeneous strength; optional `compare_homogeneous` adds legacy scalar envelope for regressions.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "cement_kg_m3": {
                                "type": "number",
                                "description": "Cement content in kg/m³ (e.g. 350)"
                            },
                            "water_kg_m3": {
                                "type": "number",
                                "description": "Water content in kg/m³ (e.g. 160)"
                            },
                            "slag_kg_m3": {
                                "type": "number",
                                "description": "Ground granulated blast-furnace slag in kg/m³",
                                "default": 0.0
                            },
                            "fly_ash_kg_m3": {
                                "type": "number",
                                "description": "Fly ash in kg/m³",
                                "default": 0.0
                            },
                            "superplasticizer_kg_m3": {
                                "type": "number",
                                "description": "Superplasticizer dosage in kg/m³",
                                "default": 0.0
                            },
                            "temperature_c": {
                                "type": "number",
                                "description": "Curing temperature in °C",
                                "default": 20.0
                            },
                            "age_days": {
                                "type": "number",
                                "description": "Age at evaluation in days",
                                "default": 28.0
                            },
                            "profile": {
                                "type": "string",
                                "description": "Calibration profile id (default, uci_d1, uhpc, …)",
                                "default": "default"
                            },
                            "compare_homogeneous": {
                                "type": "boolean",
                                "description": "When true, include `homogeneous_compare` block (legacy scalar envelope) alongside tensor pipeline JSON.",
                                "default": false
                            },
                            "aggregate_volume_fraction": {
                                "type": "number",
                                "description": "Solids volume fraction of aggregate in the mix [0, 0.85]; must match CLI `MixSpec` for parity (default 0.65).",
                                "default": 0.65
                            }
                        },
                        "required": ["cement_kg_m3", "water_kg_m3"]
                    }
                },
                {
                    "name": "list_profiles",
                    "description": "List available calibration profiles with descriptions.",
                    "inputSchema": { "type": "object", "properties": {} }
                }
            ]
        }
    })
}

// ---------------------------------------------------------------------------
// Tool execution
// ---------------------------------------------------------------------------

fn handle_tools_call(id: Value, params: Option<&Value>) -> Value {
    let name = params
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");

    let args = params
        .and_then(|p| p.get("arguments"))
        .cloned()
        .unwrap_or(json!({}));

    match name {
        "evaluate_mix" => tool_evaluate_mix(id, &args),
        "list_profiles" => tool_list_profiles(id),
        other => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("Unknown tool: {other}") }
        }),
    }
}

fn tool_evaluate_mix(id: Value, args: &Value) -> Value {
    let cement = args
        .get("cement_kg_m3")
        .and_then(|v| v.as_f64())
        .unwrap_or(350.0) as f32;
    let water = args
        .get("water_kg_m3")
        .and_then(|v| v.as_f64())
        .unwrap_or(160.0) as f32;
    let slag = args
        .get("slag_kg_m3")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;
    let fly_ash = args
        .get("fly_ash_kg_m3")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;
    let sp = args
        .get("superplasticizer_kg_m3")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;
    let temp_c = args
        .get("temperature_c")
        .and_then(|v| v.as_f64())
        .unwrap_or(20.0) as f32;
    let age_d = args
        .get("age_days")
        .and_then(|v| v.as_f64())
        .unwrap_or(28.0) as f32;
    let profile_id = args
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let compare_homogeneous = args
        .get("compare_homogeneous")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let agg_raw = args
        .get("aggregate_volume_fraction")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.65) as f32;
    if !(0.0..=0.85).contains(&agg_raw) {
        return json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{ "type": "text", "text": format!(
                    "aggregate_volume_fraction must be in [0, 0.85] (same as CLI MixSpec); got {agg_raw}"
                ) }],
                "isError": true
            }
        });
    }
    let agg_vf = agg_raw;

    // Load calibration profile
    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": format!("Error loading profile '{profile_id}': {e}") }],
                    "isError": true
                }
            });
        }
    };

    // Build the homogeneous mix row using the actual MixRow struct fields
    let row = MixRow {
        cement_kg_m3: cement,
        slag_kg_m3: slag,
        fly_ash_kg_m3: fly_ash,
        water_kg_m3: water,
        superplasticizer_kg_m3: sp,
        temperature_c: temp_c,
        age_days: age_d,
    };

    let wc = if cement > 0.0 {
        water / cement
    } else {
        f32::NAN
    };

    let frac = fractions_from_mix_row(&row, agg_vf);
    let dev = NdArrayDevice::default();
    let mix_tensor = mix_tensor_from_layout::<Backend>(&frac, &dev);
    let pipe = run_full_physics_pipeline::<Backend>(&profile, &mix_tensor);
    let pipe_json = serde_json::to_value(&pipe).unwrap_or_else(|_| json!({}));

    let mut result = json!({
        "profile": profile_id,
        "cement_kg_m3": cement,
        "water_kg_m3": water,
        "water_cement_ratio": wc,
        "slag_kg_m3": slag,
        "fly_ash_kg_m3": fly_ash,
        "superplasticizer_kg_m3": sp,
        "temperature_c": temp_c,
        "age_days": age_d,
        "aggregate_volume_fraction": agg_raw,
        "predicted_compressive_strength_mpa_tensor_jennings": pipe.summary.strength_jennings_mpa,
        "physics_pipeline": pipe_json,
        "engine": "umst-concrete-cartridge",
        "engine_version": env!("CARGO_PKG_VERSION")
    });

    if compare_homogeneous {
        let binder = (cement + slag + fly_ash).max(1.0);
        let w_c_row = (water / cement.max(1e-6)).clamp(0.05, 0.95);
        let sp_pct = (sp / binder) * 100.0;
        
        if let Ok(strength) = compressive_strength_mpa(&profile, &row) {
            if let Ok(alpha_h) = degree_of_hydration_alpha(&profile, &row) {
                let tau_h = yield_stress_pa(&profile, w_c_row, sp_pct, agg_vf);
                let agg_mass = 2_600.0_f32 * agg_vf;
                let gwp_h =
                    embodied_co2_kg_per_m3(&profile, cement, slag + fly_ash, agg_mass, water);
                let margin_h = safety_margin(&profile, w_c_row, alpha_h);
                let h = json!({
                    "compressive_strength_mpa": f64::from(strength),
                    "yield_stress_pa": f64::from(tau_h),
                    "degree_of_hydration": f64::from(alpha_h),
                    "gwp_kg_co2_eq_per_m3": f64::from(gwp_h),
                    "safety_margin": f64::from(margin_h),
                });
                if let Value::Object(ref mut m) = result {
                    m.insert("homogeneous_compare".into(), h);
                }
            }
        }
    }

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [{ "type": "text", "text": serde_json::to_string_pretty(&result).unwrap_or_default() }]
        }
    })
}

fn tool_list_profiles(id: Value) -> Value {
    let profiles: Vec<Value> = umst_concrete_cartridge::calibration::BUNDLED_PROFILE_IDS
        .iter()
        .map(|pid| {
            let desc = umst_concrete_cartridge::calibration::profile_descriptions()
                .get(pid)
                .copied()
                .unwrap_or("no description");
            json!({ "id": pid, "description": desc })
        })
        .collect();

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [{ "type": "text", "text": serde_json::to_string_pretty(&profiles).unwrap_or_default() }]
        }
    })
}
