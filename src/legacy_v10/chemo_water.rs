// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ChemoWaterResult {
    pub diffusivity: f32, // D(theta)
    pub suction: f32,     // Psi (MPa)
}

#[wasm_bindgen]
pub struct ChemoWaterEngine;

#[wasm_bindgen]
impl ChemoWaterEngine {
    /// Computes soil/concrete water diffusivity using the van Genuchten model.
    /// D(theta) = ((1-m)K_s / (alpha*m*phi)) * Theta^(1/2-1/m) * ... (Simplified)
    ///
    /// MVP: D = D0 * exp(n * theta)
    pub fn compute_diffusivity(water_content: f32, theta_sat: f32) -> ChemoWaterResult {
        if water_content <= 0.0 {
            return ChemoWaterResult {
                diffusivity: 0.0,
                suction: 1000.0,
            };
        }

        let rel_sat = water_content / theta_sat;

        // Exponential diffusivity model typical for cement paste
        // D(S) = D_sat * exp(beta * (S - 1))
        let d_sat = 1.6e-11; // m^2/s typical
        let beta = 6.0;

        let diffusivity = d_sat * (beta * (rel_sat - 1.0)).exp();

        // Suction (Psi) calculation - Kelvin-Laplace
        // Psi = - (RT/Vm) * ln(RH)
        // Or simple power law: Psi = A * S^(-b)
        let suction = 10.0 * rel_sat.powf(-2.5); // MPa approx

        ChemoWaterResult {
            diffusivity,
            suction,
        }
    }
}
