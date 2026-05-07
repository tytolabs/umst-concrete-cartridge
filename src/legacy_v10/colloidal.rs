// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: LicenseRef-Proprietary
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct DLVOResult {
    pub potential_energy: f32, // kT
    pub force: f32,            // nN
    #[wasm_bindgen(getter_with_clone)]
    pub stability: String, // "Stable", "Flocculating"
}

#[wasm_bindgen]
pub struct ColloidalEngine;

#[wasm_bindgen]
impl ColloidalEngine {
    /// Computes the total interaction energy between two particles via DLVO theory.
    /// V_total = V_vdW + V_double_layer
    pub fn compute_potential(
        separation_nm: f32,
        zeta_potential_mv: f32,
        ionic_strength_m: f32,
    ) -> DLVOResult {
        if separation_nm <= 0.1 {
            return DLVOResult {
                potential_energy: -999.0,
                force: -999.0,
                stability: "Aggregated".into(),
            };
        }

        // Constants
        let hamaker = 2.0e-20; // J (Cement-Water-Cement approx)
        let epsilon = 78.5 * 8.854e-12; // Dielectric of water
        let k_b = 1.38e-23;
        let temp = 298.0; // K

        // Van der Waals: V_A = -A / (12 * h)  (flat plates approx for close range)
        // Simplified sphere-sphere: V_A = -A*R / (12*h) -> normalized per radius
        let v_vdw = -hamaker / (12.0 * (separation_nm * 1e-9));

        // Electrostatic: V_R ... dependent on Debye length (kappa)
        // kappa prop to sqrt(I)
        // kappa^-1 (nm) approx 0.3 / sqrt(I)
        let debye_len_nm = 0.304 / ionic_strength_m.sqrt();

        // V_R = B * exp(-kappa * h)
        // Low potential approx: V_R = 2 * pi * eps * a * zeta^2 * exp(-h/debye)
        // let's assume normalized per unit radius again? Or just potential per unit area.
        // Let's output energy in kT units roughly.

        // Just a heuristic calc for now:
        let zeta_v = zeta_potential_mv / 1000.0;
        let repulsion_mag = epsilon * zeta_v * zeta_v * (-separation_nm / debye_len_nm).exp(); // Decaying entropic repulsion

        let total_joules = v_vdw + repulsion_mag;
        let total_kt = total_joules / (k_b * temp);

        // Force F = -dV/dh (derivative)
        let force = -total_joules / (separation_nm * 1e-9); // N

        let stability = if total_kt > 15.0 {
            "Stable"
        } else if total_kt < -5.0 {
            "Flocculating"
        } else {
            "Metastable"
        };

        DLVOResult {
            potential_energy: total_kt,
            force: force * 1e9, // nN
            stability: stability.into(),
        }
    }
}
