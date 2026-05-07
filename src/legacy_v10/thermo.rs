// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: Apache-2.0
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ThermoResult {
    pub heat_rate: f32,           // W/m3
    pub adiabatic_temp_rise: f32, // deg C
}

#[wasm_bindgen]
pub struct ThermoEngine;

#[wasm_bindgen]
impl ThermoEngine {
    /// Computes rate of heat evolution (q) using Arrhenius law.
    pub fn compute_heat_rate(
        temp_c: f32,
        alpha: f32,
        activation_energy: f32, // J/mol (typ 40000)
    ) -> ThermoResult {
        let r_gas = 8.314;
        let temp_k = temp_c + 273.15;

        // Affinity term (1-alpha)^n
        // Nucleation term not modeled here, assuming post-acceleration
        let chem_affinity = (1.0 - alpha).max(0.0).powf(1.5);

        // Arrhenius: k = A * exp(-E / RT)
        let rate_constant = (-activation_energy / (r_gas * temp_k)).exp();

        // Reference rate at 20C approx
        let ref_rate = 1e6; // arbitrary scale factor for W/m3

        let heat_rate = ref_rate * rate_constant * chem_affinity;

        ThermoResult {
            heat_rate,
            adiabatic_temp_rise: alpha * 50.0, // Simplistic: 50C max rise at alpha=1
        }
    }
}
