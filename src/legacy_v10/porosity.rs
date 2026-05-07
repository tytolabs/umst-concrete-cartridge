// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: LicenseRef-Proprietary
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct PorosityResult {
    pub porosity: f32,         // 0.0 - 1.0
    pub permeability: f32,     // m² (Intrinsic Permeability, Kozeny-Carman)
    pub formation_factor: f32, // Archie's Law F = 1/φ^m
}

#[wasm_bindgen]
pub struct PorosityEngine;

#[wasm_bindgen]
impl PorosityEngine {
    /// Computes permeability using the Kozeny-Carman equation.
    ///
    /// # Arguments
    /// * `porosity` (phi): The void fraction of the material (0.0 - 1.0).
    /// * `specific_surface` (S): Surface area per unit volume (m2/m3).
    /// * `tortuosity`: Path tortuosity (typically 1.5 - 2.5 for paste).
    pub fn compute_kozeny_carman(
        porosity: f32,
        specific_surface: f32,
        tortuosity: f32,
    ) -> PorosityResult {
        if porosity <= 0.0 || specific_surface <= 0.0 {
            return PorosityResult {
                porosity,
                permeability: 0.0,
                formation_factor: 1.0,
            };
        }

        // K = (phi^3) / (c * (1-phi)^2 * S^2)
        // c is Kozeny constant (~5, dependent on tortuosity).
        let k_constant = 5.0 * tortuosity; // Approx

        let numerator = porosity.powi(3);
        let denominator = k_constant * (1.0 - porosity).powi(2) * specific_surface.powi(2);

        let permeability = numerator / denominator;

        // Archie's Law Formation Factor (F = a / phi^m)
        // For concrete, m ~ 2.0
        let formation_factor = 1.0 / porosity.powi(2);

        PorosityResult {
            porosity,
            permeability,
            formation_factor,
        }
    }

    /// Calculate Darcy flux (q) given gradient.
    /// q = -K * dh/dx
    pub fn compute_flux(permeability: f32, head_diff: f32, length: f32) -> f32 {
        if length <= 0.0 {
            return 0.0;
        }
        permeability * (head_diff / length)
    }
}
