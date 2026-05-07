// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: LicenseRef-Proprietary
//
// MaOS — Material Agnostic Operating System
// FiberEngine: Fiber Reinforcement Physics for FRCC
//
// ═══════════════════════════════════════════════════════════════════════════════
// PHYSICS MODELS
// ═══════════════════════════════════════════════════════════════════════════════
//
// 1. Tensile Strength Enhancement (Naaman & Reinhardt, 2006)
//    Δσ_t = η_θ × η_l × V_f × τ × (L/d)
//    where:
//    - η_θ: Fiber orientation factor (0.5 for 3D random, 0.64 for 2D random)
//    - η_l: Fiber length efficiency (1 - tanh(βL/2) / (βL/2))
//    - V_f: Fiber volume fraction
//    - τ: Interfacial bond strength (MPa)
//    - L/d: Fiber aspect ratio
//
// 2. Toughness Enhancement (JSCE Model)
//    I_5 = 1.0 + 0.045 × V_f × (L/d) × σ_fu / τ
//    where:
//    - I_5: Toughness index (ratio of area under load-deflection)
//    - σ_fu: Fiber ultimate tensile strength
//
// 3. Crack Bridging Stress (Fib Model Code 2010)
//    σ_bridge = 0.45 × σ_f × V_f × (L/d) × η_θ
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::tensors::{MaterialType, MixTensor, MIX_TENSOR_STRIDE};
use wasm_bindgen::prelude::*;

/// Result of fiber physics calculations
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct FiberResult {
    /// Tensile strength enhancement (MPa)
    pub tensile_boost: f32,
    /// Toughness index I_5 (dimensionless, 1.0 = no fiber)
    pub toughness_index: f32,
    /// Crack bridging stress (MPa)
    pub bridging_stress: f32,
    /// Fiber volume fraction (%)
    pub volume_fraction: f32,
    /// Average fiber aspect ratio
    pub avg_aspect_ratio: f32,
}

#[wasm_bindgen]
pub struct FiberEngine;

#[wasm_bindgen]
impl FiberEngine {
    /// Compute fiber contribution to concrete mechanics
    ///
    /// # Arguments
    /// * `mix` - The mix tensor containing fiber materials
    /// * `matrix_strength` - Matrix compressive strength (MPa) for bond estimation
    ///
    /// # Returns
    /// FiberResult with tensile boost, toughness, and bridging stress
    pub fn compute(mix: &MixTensor, matrix_strength: f32) -> FiberResult {
        let data = mix.data();
        let stride = MIX_TENSOR_STRIDE;
        let count = data.len() / stride;

        // Accumulate fiber properties
        let mut total_fiber_mass = 0.0_f32;
        #[allow(unused_variables)]
        let mut _total_mass = 0.0_f32;
        let mut total_fiber_vol = 0.0_f32;
        let mut total_vol = 0.0_f32;
        let mut weighted_aspect_ratio = 0.0_f32;
        let mut weighted_tensile = 0.0_f32;

        for i in 0..count {
            let offset = i * stride;
            let mass = data[offset];
            let sg = data[offset + 1];
            let type_id = data[offset + 2] as u8;
            let aspect_ratio = data[offset + 13];
            let tensile_strength = data[offset + 14];

            if mass > 0.0 && sg > 0.0 {
                let vol = mass / (sg * 1000.0); // m³
                _total_mass += mass;
                total_vol += vol;

                if type_id == MaterialType::Fiber as u8 {
                    total_fiber_mass += mass;
                    total_fiber_vol += vol;
                    weighted_aspect_ratio += aspect_ratio * mass;
                    weighted_tensile += tensile_strength * mass;
                }
            }
        }

        // No fibers present
        if total_fiber_mass < 0.001 {
            return FiberResult {
                tensile_boost: 0.0,
                toughness_index: 1.0, // No enhancement
                bridging_stress: 0.0,
                volume_fraction: 0.0,
                avg_aspect_ratio: 0.0,
            };
        }

        // Calculate fiber parameters
        let v_f = if total_vol > 0.0 {
            total_fiber_vol / total_vol
        } else {
            0.0
        };
        let v_f_percent = v_f * 100.0;
        let avg_l_d = weighted_aspect_ratio / total_fiber_mass;
        let avg_tensile = weighted_tensile / total_fiber_mass;

        // ═══════════════════════════════════════════════════════════════════════
        // Model 1: Tensile Strength Enhancement (Naaman & Reinhardt)
        // ═══════════════════════════════════════════════════════════════════════

        // Orientation factor (3D random assumption)
        let eta_theta = 0.5_f32;

        // Length efficiency factor (simplified)
        // For typical fibers with good bond, η_l ≈ 0.7-0.9
        let eta_l = 0.8_f32;

        // Interfacial bond strength estimation
        // τ ≈ 0.6 × √(f'c) for steel fibers in good matrix
        // τ ≈ 0.3 × √(f'c) for synthetic fibers
        let tau = 0.5 * matrix_strength.sqrt(); // MPa

        // Tensile boost: Δσ_t = η_θ × η_l × V_f × τ × (L/d)
        let tensile_boost = eta_theta * eta_l * v_f * tau * avg_l_d;

        // ═══════════════════════════════════════════════════════════════════════
        // Model 2: Toughness Enhancement (JSCE Model)
        // ═══════════════════════════════════════════════════════════════════════

        // I_5 = 1.0 + 0.045 × V_f × (L/d) × σ_fu / τ
        let toughness_factor = if tau > 0.0 {
            0.045 * v_f * avg_l_d * avg_tensile / tau
        } else {
            0.0
        };
        let toughness_index = (1.0 + toughness_factor).min(15.0); // Cap at 15x enhancement

        // ═══════════════════════════════════════════════════════════════════════
        // Model 3: Crack Bridging Stress (Fib MC2010)
        // ═══════════════════════════════════════════════════════════════════════

        // σ_bridge = 0.45 × σ_f × V_f × (L/d) × η_θ
        // This represents the stress that can be carried across a crack
        let bridging_stress = 0.45 * avg_tensile * v_f * avg_l_d * eta_theta;

        FiberResult {
            tensile_boost,
            toughness_index,
            bridging_stress,
            volume_fraction: v_f_percent,
            avg_aspect_ratio: avg_l_d,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    fn create_fiber_mix(
        fiber_mass: f32,
        fiber_sg: f32,
        aspect_ratio: f32,
        tensile: f32,
    ) -> MixTensor {
        let mut tensor = MixTensor::new();
        // add_material(mass, sg, type, co2, cost, blaine, fm, shape, visc, yield, thix, k, react, ar, ts)
        // Add cement (400 kg, sg 3.15)
        tensor.add_material(
            400.0,
            3.15,
            MaterialType::Cement as u8,
            0.12,
            0.9,
            350.0,
            0.0,
            0.5, // co2, cost, blaine, fm, shape
            0.0,
            200.0,
            0.0, // visc, yield, thix
            0.0,
            0.0,
            0.0,
            0.0, // k, react, ar, ts
            0.25,
            1.0, // absorption, moisture
        );
        // Add aggregate (1200 kg, sg 2.65)
        tensor.add_material(
            1200.0,
            2.65,
            MaterialType::Aggregate as u8,
            0.01,
            0.05,
            0.0,
            3.0,
            0.6, // co2, cost, blaine, fm, shape
            0.0,
            0.0,
            0.0, // visc, yield, thix
            0.0,
            0.0,
            0.0,
            0.0, // k, react, ar, ts
            5.0,
            2.0, // absorption, moisture
        );
        // Add water (180 kg, sg 1.0)
        tensor.add_material(
            180.0,
            1.0,
            MaterialType::Water as u8,
            0.001,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0, // absorption, moisture
        );
        // Add fiber
        tensor.add_material(
            fiber_mass,
            fiber_sg,
            MaterialType::Fiber as u8,
            0.0,
            5.0,
            0.0,
            0.0,
            0.0, // co2, cost, blaine, fm, shape
            0.0,
            0.0,
            0.0, // visc, yield, thix
            0.0,
            0.0,
            aspect_ratio,
            tensile, // k, react, ar, ts
            0.0,
            0.0, // absorption, moisture
        );
        tensor
    }

    #[test]
    fn test_no_fiber() {
        let mut tensor = MixTensor::new();
        // add_material(mass, sg, type, co2, cost, blaine, fm, shape, visc, yield, thix, k, react, ar, ts)
        tensor.add_material(
            400.0,
            3.15,
            MaterialType::Cement as u8,
            0.12,
            0.9,
            350.0,
            0.0,
            0.5,
            0.0,
            200.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.25,
            1.0,
        );
        tensor.add_material(
            180.0,
            1.0,
            MaterialType::Water as u8,
            0.001,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        let result = FiberEngine::compute(&tensor, 40.0);

        assert_eq!(result.tensile_boost, 0.0);
        assert_eq!(result.toughness_index, 1.0);
        assert_eq!(result.bridging_stress, 0.0);
        assert_eq!(result.volume_fraction, 0.0);
        println!("✅ No fiber test passed");
    }

    #[test]
    fn test_steel_fiber() {
        // Steel fiber: 60 kg/m³, sg ~7.85, L/d = 65, tensile = 1300 MPa
        let mix = create_fiber_mix(60.0, 7.85, 65.0, 1300.0);
        let result = FiberEngine::compute(&mix, 40.0);

        // Volume fraction should be around 0.8%
        assert!(
            result.volume_fraction > 0.3 && result.volume_fraction < 2.0,
            "Steel fiber Vf should be reasonable: got {}",
            result.volume_fraction
        );

        // Tensile boost should be positive
        assert!(
            result.tensile_boost > 0.0,
            "Tensile boost should be positive: got {}",
            result.tensile_boost
        );

        // Toughness should be enhanced
        assert!(
            result.toughness_index > 1.0,
            "Toughness index should be > 1.0: got {}",
            result.toughness_index
        );

        println!(
            "✅ Steel fiber test passed: Vf={:.2}%, Δσt={:.2}MPa, I5={:.2}",
            result.volume_fraction, result.tensile_boost, result.toughness_index
        );
    }

    #[test]
    fn test_synthetic_fiber() {
        // PP fiber: 9 kg/m³, sg ~0.91, L/d = 100, tensile = 500 MPa
        let mix = create_fiber_mix(9.0, 0.91, 100.0, 500.0);
        let result = FiberEngine::compute(&mix, 35.0);

        // Volume fraction should be around 1%
        assert!(
            result.volume_fraction > 0.3 && result.volume_fraction < 3.0,
            "PP fiber Vf should be reasonable: got {}",
            result.volume_fraction
        );

        // Tensile boost should be positive but likely lower than steel
        assert!(
            result.tensile_boost > 0.0,
            "Tensile boost should be positive: got {}",
            result.tensile_boost
        );

        // Bridging stress should be positive
        assert!(
            result.bridging_stress > 0.0,
            "Bridging stress should be positive: got {}",
            result.bridging_stress
        );

        println!(
            "✅ Synthetic fiber test passed: Vf={:.2}%, σ_bridge={:.2}MPa",
            result.volume_fraction, result.bridging_stress
        );
    }

    #[test]
    fn test_fiber_dosage_effect() {
        // Test that increasing fiber dosage increases enhancement
        let mix_low = create_fiber_mix(30.0, 7.85, 65.0, 1300.0);
        let mix_high = create_fiber_mix(80.0, 7.85, 65.0, 1300.0);

        let result_low = FiberEngine::compute(&mix_low, 40.0);
        let result_high = FiberEngine::compute(&mix_high, 40.0);

        assert!(
            result_high.volume_fraction > result_low.volume_fraction,
            "Higher dosage should give higher Vf"
        );
        assert!(
            result_high.tensile_boost > result_low.tensile_boost,
            "Higher dosage should give higher tensile boost"
        );
        assert!(
            result_high.toughness_index > result_low.toughness_index,
            "Higher dosage should give higher toughness"
        );

        println!("✅ Fiber dosage effect test passed");
    }

    #[test]
    fn test_aspect_ratio_effect() {
        // Test that higher aspect ratio increases enhancement
        let mix_low_ar = create_fiber_mix(60.0, 7.85, 40.0, 1300.0);
        let mix_high_ar = create_fiber_mix(60.0, 7.85, 80.0, 1300.0);

        let result_low = FiberEngine::compute(&mix_low_ar, 40.0);
        let result_high = FiberEngine::compute(&mix_high_ar, 40.0);

        assert!(
            result_high.tensile_boost > result_low.tensile_boost,
            "Higher L/d should give higher tensile boost"
        );
        assert!(
            result_high.bridging_stress > result_low.bridging_stress,
            "Higher L/d should give higher bridging stress"
        );

        println!("✅ Aspect ratio effect test passed");
    }
}
