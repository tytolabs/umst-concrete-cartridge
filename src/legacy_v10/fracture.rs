// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: Apache-2.0
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct FractureResult {
    pub k_ic: f32,               // Stress Intensity Factor (MPa.m^0.5)
    pub critical_crack_len: f32, // mm
    #[wasm_bindgen(getter_with_clone)]
    pub failure_mode: String, // "Safe", "Brittle Fracture"
}

#[wasm_bindgen]
pub struct FractureEngine;

#[wasm_bindgen]
impl FractureEngine {
    /// Computes stress intensity factor K_I for a center crack in an infinite plate.
    /// K_I = sigma * sqrt(pi * a)
    pub fn compute_lefm(
        stress_mpa: f32,
        crack_length_mm: f32,        // 2a
        fracture_toughness_kic: f32, // Material property typ 0.5-1.5 for concrete
    ) -> FractureResult {
        let a = (crack_length_mm / 1000.0) / 2.0; // half-crack length in meters

        // K_I calculation
        let k_i = stress_mpa * (std::f32::consts::PI * a).sqrt();

        // Critical crack size for this stress layout
        // a_c = (1/pi) * (K_Ic / sigma)^2
        let a_c_m = if stress_mpa > 0.0 {
            (1.0 / std::f32::consts::PI) * (fracture_toughness_kic / stress_mpa).powi(2)
        } else {
            999.0
        };

        let mode = if k_i > fracture_toughness_kic {
            "Brittle Fracture"
        } else {
            "Safe"
        };

        FractureResult {
            k_ic: k_i,
            critical_crack_len: a_c_m * 2000.0, // convert back to full crack length mm
            failure_mode: mode.into(),
        }
    }
}
