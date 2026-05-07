// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: Apache-2.0
use crate::tensors::mix::{MixTensor, MIX_TENSOR_STRIDE};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SustainabilityResult {
    pub gwp_total: f32,    // kg CO2e / m3
    pub energy_total: f32, // MJ / m3
    pub score: f32,        // 0.0 - 1.0 (1.0 is net zero or better)
}

#[wasm_bindgen]
pub struct SustainabilityEngine;

#[wasm_bindgen]
impl SustainabilityEngine {
    /// Computes the environmental impact of a mix based on standard LCA factors.
    /// Factors are typically passed in, but we include defaults for MVP.
    pub fn compute_impact(mix: &MixTensor) -> SustainabilityResult {
        // Direct assignment, components unused
        // Direct assignment, components unused
        // let co2 = mix.total_co2(); // This assumes explicit props which are missing in test

        let data = mix.data();
        let stride = MIX_TENSOR_STRIDE;
        let count = data.len() / stride;
        let mut co2_calc = 0.0;
        let mut energy_calc = 0.0;

        for i in 0..count {
            let offset = i * stride;
            let mass = data[offset];
            let type_id = data[offset + 2] as u8;
            let explicit_co2 = data[offset + 3]; // "co2" field

            let factor = if explicit_co2 > 0.0 {
                explicit_co2
            } else {
                // Default Factors (kg CO2 / kg material)
                match type_id {
                    0 => 0.85,  // Cement
                    1 => 0.001, // Water
                    2 => 0.005, // Aggregates (Mining/Transport)
                    3 => 1.5,   // Admixture (Chemicals)
                    4 => 0.1,   // SCM (Fly Ash/Slag)
                    _ => 0.05,
                }
            };

            // Energy Factors (MJ / kg)
            let energy_factor = match type_id {
                0 => 5.0,  // Cement
                1 => 0.0,  // Water
                2 => 0.1,  // Aggregates
                3 => 20.0, // Admixture
                4 => 0.5,  // SCM
                _ => 1.0,
            };

            co2_calc += mass * factor;
            energy_calc += mass * energy_factor;
        }

        // Score Calculation
        // Baseline for concrete is typically ~300-400 kg CO2/m3 (CEM I)
        // Score = 1.0 - (co2 / 500.0) clamped.

        let score = (1.0 - (co2_calc / 600.0)).clamp(0.0, 1.0);

        SustainabilityResult {
            gwp_total: co2_calc,
            energy_total: energy_calc,
            score,
        }
    }

    /// Computes LEED v4 contribution (mock)
    pub fn compute_leed_points(gwp: f32) -> u8 {
        if gwp < 200.0 {
            return 3;
        }
        if gwp < 300.0 {
            return 2;
        }
        if gwp < 400.0 {
            return 1;
        }
        0
    }
}
