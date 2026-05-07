// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: Apache-2.0
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ITZResult {
    pub thickness_microns: f32,
    pub porosity_itz: f32,
}

#[wasm_bindgen]
pub struct ITZEngine;

#[wasm_bindgen]
impl ITZEngine {
    /// Computes the estimated thickness and porosity of the Interfacial Transition Zone (ITZ).
    pub fn compute_properties(agg_size_mm: f32, wc_ratio: f32) -> ITZResult {
        // ITZ thickness roughly scales with agg size and w/c
        // Helper model: t = 10um + k * agg_size
        let thickness = 10.0 + (agg_size_mm * 2.5);

        // ITZ porosity is typically 1.5x - 2x bulk paste porosity
        // Paste porosity ~ wc - 0.2
        let bulk_porosity = (wc_ratio - 0.2).max(0.05);
        let porosity_itz = (bulk_porosity * 1.8).min(0.9);

        ITZResult {
            thickness_microns: thickness, // typical 20-50 um
            porosity_itz,
        }
    }

    /// Computes diffusion coefficient boost due to ITZ percolation
    pub fn compute_percolation_factor(itz_vol_frac: f32) -> f32 {
        // Percolation threshold approx 20%-30% ITZ volume
        if itz_vol_frac > 0.3 {
            1.0 + (itz_vol_frac - 0.3) * 10.0
        } else {
            1.0
        }
    }
}
