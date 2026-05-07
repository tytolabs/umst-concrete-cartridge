// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct MaturityResult {
    pub temperature_history: Vec<f64>,
    pub equivalent_age: f64, // hours at 20C
    pub strength_pa: f64,
    pub stage: String, // 'DORMANT', 'SETTING', 'HARDENING'
}

#[wasm_bindgen]
pub struct MaturityEngine {
    activation_energy: f64, // J/mol
    #[allow(dead_code)]
    datum_temp: f64, // Kelvin (usually -10C = 263K)
}

#[wasm_bindgen]
impl MaturityEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(activation_energy: Option<f64>) -> MaturityEngine {
        MaturityEngine {
            activation_energy: activation_energy.unwrap_or(41500.0), // ~41.5 kJ/mol for Portland cement
            datum_temp: 263.15,                                      // -10C
        }
    }

    /**
     * Nurse-Saul Maturity Function (simple time-temperature factor)
     * R = (T - T0) / (Tr - T0)
     * We use a simplified Arrhenius for "Equivalent Age" at 20C (293.15K)
     */
    pub fn calculate(&self, temp_c_history: &[f64], interval_hours: f64) -> JsValue {
        let mut equivalent_age = 0.0;
        let ref_temp_k = 293.15; // 20C

        for &temp_c in temp_c_history {
            let temp_k = temp_c + 273.15;

            // Arrhenius function for reaction rate k(T) relative to k(20C)
            // exp( (E / R) * (1/293 - 1/T) )
            // R = 8.314
            let r_gas = 8.314;
            let q = self.activation_energy / r_gas;
            let factor = (q * (1.0 / ref_temp_k - 1.0 / temp_k)).exp();

            equivalent_age += factor * interval_hours;
        }

        // Strength Prediction (Hyperbolic / Carino function)
        // S = S_inf * (k(t - t0)) / (1 + k(t - t0))
        // Simplified: S = S_max * (Age / (Half_Time + Age))
        let s_max = 50_000_000.0; // 50 MPa
        let half_time = 48.0; // hours to reach 50% strength
        let strength = s_max * (equivalent_age / (half_time + equivalent_age));

        let stage = if strength < 1_000_000.0 {
            "DORMANT"
        } else if strength < 10_000_000.0 {
            "SETTING"
        } else {
            "HARDENING"
        };

        let result = MaturityResult {
            temperature_history: temp_c_history.to_vec(),
            equivalent_age,
            strength_pa: strength,
            stage: stage.to_string(),
        };

        serde_wasm_bindgen::to_value(&result).unwrap()
    }
}
