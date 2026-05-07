// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: Apache-2.0
use crate::tensors::{MixTensor, MIX_TENSOR_STRIDE};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct CostResult {
    pub total_cost: f32, // Currency Units (e.g. USD or INR)
    pub cost_per_m3: f32,
}

#[wasm_bindgen]
pub struct CostEngine;

#[wasm_bindgen]
impl CostEngine {
    /// Calculate total cost and unit cost of the mix
    pub fn compute(mix: &MixTensor) -> CostResult {
        let data = mix.data();
        let stride = MIX_TENSOR_STRIDE;
        let count = data.len() / stride;

        let mut total_cost = 0.0;
        let mut total_vol = 0.0;

        for i in 0..count {
            let offset = i * stride;
            let mass = data[offset]; // kg
            let sg = data[offset + 1]; // Specific Gravity
            let cost_factor = data[offset + 4]; // Cost per kg

            total_cost += mass * cost_factor;

            let density = if sg > 0.0 { sg * 1000.0 } else { 1000.0 };
            total_vol += mass / density;
        }

        let cost_per_m3 = if total_vol > 0.0 {
            total_cost / total_vol
        } else {
            0.0
        };

        CostResult {
            total_cost,
            cost_per_m3,
        }
    }
}
