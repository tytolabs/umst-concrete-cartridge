// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//
// MaOS — Material Agnostic Operating System
// FreezeThawEngine: Freeze-Thaw Durability Physics
//
// ═══════════════════════════════════════════════════════════════════════════════
// PHYSICS MODELS
// ═══════════════════════════════════════════════════════════════════════════════
//
// 1. Air Content Requirement (ACI 318, Powers' Theory)
//    For freeze-thaw resistance, air content should be:
//    - 6±1% for severe exposure (>25 freeze-thaw cycles/year, saturated)
//    - 5±1% for moderate exposure
//    - 4±1% for mild exposure
//
// 2. Spacing Factor (Powers & Helmuth, 1953)
//    L̄ = (3/α) × [1.4 × (1+p/a)^(1/3) - 1]
//    where:
//    - L̄: Spacing factor (mm) - should be < 0.2mm for durability
//    - α: Specific surface of air voids (mm⁻¹, typically 25-45)
//    - p: Paste volume fraction
//    - a: Air volume fraction
//
// 3. Durability Factor (ASTM C666)
//    DF = (P × N) / M × 100
//    where:
//    - P: Relative dynamic modulus at N cycles (%)
//    - N: Number of cycles at which P falls below 60% or 300 cycles
//    - M: 300 (specified cycles)
//    A DF > 80% indicates good freeze-thaw resistance.
//
// 4. Critical Saturation Degree (Fagerlund, 1977)
//    S_cr = 0.88 - 0.15 × L̄  (for L̄ in mm)
//    Concrete fails when S > S_cr
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::tensors::{MaterialType, MixTensor, MIX_TENSOR_STRIDE};
use wasm_bindgen::prelude::*;

/// Freeze-thaw exposure severity
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FreezeThawExposure {
    Mild = 0,     // <15 cycles/year, rarely saturated
    Moderate = 1, // 15-25 cycles/year, occasionally saturated
    Severe = 2,   // >25 cycles/year, frequently saturated
}

/// Result of freeze-thaw calculations
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct FreezeThawResult {
    /// Total air content (%)
    pub air_content: f32,
    /// Estimated spacing factor (mm)
    pub spacing_factor: f32,
    /// Predicted durability factor (0-100)
    pub durability_factor: f32,
    /// Critical saturation degree (0-1)
    pub critical_saturation: f32,
    /// Required air content for exposure (%)
    pub required_air: f32,
    /// Whether mix meets freeze-thaw requirements
    pub meets_requirements: bool,
}

#[wasm_bindgen]
pub struct FreezeThawEngine;

#[wasm_bindgen]
impl FreezeThawEngine {
    /// Compute freeze-thaw durability parameters
    ///
    /// # Arguments
    /// * `mix` - The mix tensor
    /// * `exposure` - Freeze-thaw exposure severity
    /// * `air_void_specific_surface` - Specific surface of air voids (mm⁻¹, default 25-45)
    ///
    /// # Returns
    /// FreezeThawResult with durability predictions
    pub fn compute(
        mix: &MixTensor,
        exposure: FreezeThawExposure,
        air_void_specific_surface: f32,
    ) -> FreezeThawResult {
        let data = mix.data();
        let stride = MIX_TENSOR_STRIDE;
        let count = data.len() / stride;

        // Accumulate volumes
        let mut total_vol = 0.0_f32;
        let mut air_vol = 0.0_f32;
        let mut cement_vol = 0.0_f32;
        let mut water_vol = 0.0_f32;
        let mut scm_vol = 0.0_f32;
        let mut has_air_entrainer = false;
        let mut entrainer_air = 0.0_f32;

        for i in 0..count {
            let offset = i * stride;
            let mass = data[offset];
            let sg = data[offset + 1];
            let type_id = data[offset + 2] as u8;
            let reactivity = data[offset + 12]; // air_content stored in reactivity for air entrainer

            if mass > 0.0 && sg > 0.0 {
                let vol = mass / (sg * 1000.0); // m³
                total_vol += vol;

                match type_id {
                    t if t == MaterialType::Air as u8 => {
                        air_vol += vol;
                    }
                    t if t == MaterialType::Cement as u8 => {
                        cement_vol += vol;
                    }
                    t if t == MaterialType::Water as u8 => {
                        water_vol += vol;
                    }
                    t if t == MaterialType::SCM as u8 => {
                        scm_vol += vol;
                    }
                    t if t == MaterialType::AirEntrainer as u8 => {
                        has_air_entrainer = true;
                        // target air % stored in reactivity field
                        if reactivity > 0.0 {
                            entrainer_air = reactivity;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Include entrained air if specified
        if has_air_entrainer && entrainer_air > 0.0 {
            // Entrainer provides additional air beyond naturally entrapped
            let target_air_vol = total_vol * entrainer_air / 100.0;
            air_vol = air_vol.max(target_air_vol);
        }

        // Air content as percentage
        let air_content = if total_vol > 0.0 {
            (air_vol / total_vol) * 100.0
        } else {
            0.0
        };

        // Paste volume = cement + water + SCM
        let paste_vol = cement_vol + water_vol + scm_vol;
        let paste_fraction = if total_vol > 0.0 {
            paste_vol / total_vol
        } else {
            0.3
        };
        let air_fraction = if total_vol > 0.0 {
            air_vol / total_vol
        } else {
            0.02
        };

        // Required air content based on exposure
        let required_air = match exposure {
            FreezeThawExposure::Mild => 4.0,
            FreezeThawExposure::Moderate => 5.0,
            FreezeThawExposure::Severe => 6.0,
        };

        // ═══════════════════════════════════════════════════════════════════════
        // Model 1: Spacing Factor (Powers)
        // ═══════════════════════════════════════════════════════════════════════
        // L̄ = (3/α) × [1.4 × (1+p/a)^(1/3) - 1]

        let alpha = air_void_specific_surface.clamp(20.0, 50.0); // mm⁻¹
        let p_over_a = if air_fraction > 0.001 {
            paste_fraction / air_fraction
        } else {
            100.0 // High value indicates poor air content
        };

        let spacing_factor = (3.0 / alpha) * (1.4 * (1.0 + p_over_a).powf(1.0 / 3.0) - 1.0);
        let spacing_factor = spacing_factor.clamp(0.05, 1.5); // mm

        // ═══════════════════════════════════════════════════════════════════════
        // Model 2: Critical Saturation Degree (Fagerlund)
        // ═══════════════════════════════════════════════════════════════════════
        // S_cr = 0.88 - 0.15 × L̄

        let critical_saturation = (0.88 - 0.15 * spacing_factor).clamp(0.5, 0.88);

        // ═══════════════════════════════════════════════════════════════════════
        // Model 3: Durability Factor Prediction
        // ═══════════════════════════════════════════════════════════════════════
        // Empirical model based on spacing factor and air content

        // Air entrainment effectiveness
        let air_effectiveness = if air_content >= required_air {
            1.0
        } else {
            (air_content / required_air).powf(0.5)
        };

        // Spacing factor effectiveness (< 0.2mm is ideal)
        let spacing_effectiveness = if spacing_factor <= 0.2 {
            1.0
        } else if spacing_factor <= 0.4 {
            0.9 - (spacing_factor - 0.2) * 0.5
        } else {
            0.7 - (spacing_factor - 0.4).min(0.5) * 0.6
        };

        // Combined durability factor
        let durability_factor = 100.0 * air_effectiveness * spacing_effectiveness;
        let durability_factor = durability_factor.clamp(0.0, 100.0);

        // Check if requirements are met
        let meets_requirements = air_content >= (required_air - 1.0) && // Within tolerance
            spacing_factor <= 0.25 && // Good spacing
            durability_factor >= 80.0; // Good durability

        FreezeThawResult {
            air_content,
            spacing_factor,
            durability_factor,
            critical_saturation,
            required_air,
            meets_requirements,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    fn create_mix_with_air(air_mass: f32) -> MixTensor {
        let mut tensor = MixTensor::new();
        // add_material(mass, sg, type, co2, cost, blaine, fm, shape, visc, yield, thix, k, react, ar, ts, absorption, moisture)
        // Cement (400 kg)
        tensor.add_material(
            400.0,
            3.15,
            MaterialType::Cement as u8,
            0.12,
            0.9,
            350.0,
            0.0,
            0.55, // co2, cost, blaine, fm, shape
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
        // Water (160 kg)
        tensor.add_material(
            160.0,
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
        // Aggregate (1800 kg)
        tensor.add_material(
            1800.0,
            2.65,
            MaterialType::Aggregate as u8,
            0.01,
            0.01,
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
            2.0,
            1.0, // absorption, moisture
        );
        // Air (entrained/entrapped)
        if air_mass > 0.0 {
            tensor.add_material(
                air_mass,
                0.0012,
                MaterialType::Air as u8, // sg of air ~0.0012
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
                0.0, // absorption, moisture
            );
        }
        tensor
    }

    #[test]
    fn test_no_air() {
        let mix = create_mix_with_air(0.0);
        let result = FreezeThawEngine::compute(&mix, FreezeThawExposure::Severe, 35.0);

        assert!(
            result.air_content < 1.0,
            "Mix with no air should have low air content: got {}",
            result.air_content
        );
        assert!(
            !result.meets_requirements,
            "Mix with no air should not meet freeze-thaw requirements"
        );

        println!(
            "✅ No air test passed: air={:.1}%, DF={:.1}",
            result.air_content, result.durability_factor
        );
    }

    #[test]
    fn test_adequate_air() {
        // Add enough air for 6% content
        // For total ~1m³, we need ~60L = 0.072 kg of air (density ~1.2 kg/m³)
        let mix = create_mix_with_air(0.072);
        let result = FreezeThawEngine::compute(&mix, FreezeThawExposure::Severe, 35.0);

        // Air content should be around 6%
        assert!(
            result.air_content > 4.0,
            "Air content should be adequate: got {}",
            result.air_content
        );

        // Durability factor should be good
        assert!(
            result.durability_factor > 60.0,
            "Durability factor should be reasonable: got {}",
            result.durability_factor
        );

        println!(
            "✅ Adequate air test passed: air={:.1}%, DF={:.1}, L̄={:.3}mm",
            result.air_content, result.durability_factor, result.spacing_factor
        );
    }

    #[test]
    fn test_exposure_levels() {
        let mix = create_mix_with_air(0.05);

        let mild = FreezeThawEngine::compute(&mix, FreezeThawExposure::Mild, 35.0);
        let moderate = FreezeThawEngine::compute(&mix, FreezeThawExposure::Moderate, 35.0);
        let severe = FreezeThawEngine::compute(&mix, FreezeThawExposure::Severe, 35.0);

        // Required air should increase with severity
        assert!(
            mild.required_air < moderate.required_air,
            "Moderate should require more air than mild"
        );
        assert!(
            moderate.required_air < severe.required_air,
            "Severe should require more air than moderate"
        );

        println!(
            "✅ Exposure levels test passed: mild={:.1}%, mod={:.1}%, sev={:.1}%",
            mild.required_air, moderate.required_air, severe.required_air
        );
    }

    #[test]
    fn test_spacing_factor_effect() {
        let mix = create_mix_with_air(0.05);

        // Low specific surface = large bubbles = high spacing factor
        let low_alpha = FreezeThawEngine::compute(&mix, FreezeThawExposure::Severe, 20.0);
        // High specific surface = small bubbles = low spacing factor
        let high_alpha = FreezeThawEngine::compute(&mix, FreezeThawExposure::Severe, 45.0);

        // Higher specific surface should give lower spacing factor
        assert!(
            high_alpha.spacing_factor < low_alpha.spacing_factor,
            "Higher α should give lower spacing factor"
        );

        // Higher specific surface should give better durability
        assert!(
            high_alpha.durability_factor >= low_alpha.durability_factor,
            "Higher α should give better durability"
        );

        println!(
            "✅ Spacing factor effect test passed: α=20→L̄={:.3}mm, α=45→L̄={:.3}mm",
            low_alpha.spacing_factor, high_alpha.spacing_factor
        );
    }

    #[test]
    fn test_critical_saturation() {
        let mix = create_mix_with_air(0.072);
        let result = FreezeThawEngine::compute(&mix, FreezeThawExposure::Severe, 35.0);

        // Critical saturation should be between 0.5 and 0.88
        assert!(
            result.critical_saturation >= 0.5 && result.critical_saturation <= 0.88,
            "Critical saturation should be in valid range: got {}",
            result.critical_saturation
        );

        // Lower spacing factor should give higher critical saturation
        let high_air = create_mix_with_air(0.1);
        let result_high = FreezeThawEngine::compute(&high_air, FreezeThawExposure::Severe, 40.0);

        // More air should improve critical saturation tolerance
        assert!(
            result_high.critical_saturation >= result.critical_saturation - 0.05,
            "Higher air should maintain or improve critical saturation"
        );

        println!(
            "✅ Critical saturation test passed: S_cr={:.3}",
            result.critical_saturation
        );
    }

    #[test]
    fn test_air_entrainer() {
        let mut tensor = MixTensor::new();
        // add_material(mass, sg, type, co2, cost, blaine, fm, shape, visc, yield, thix, k, react, ar, ts)
        // Basic mix components
        tensor.add_material(
            400.0,
            3.15,
            MaterialType::Cement as u8,
            0.12,
            0.9,
            350.0,
            0.0,
            0.55,
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
            160.0,
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
        tensor.add_material(
            1800.0,
            2.65,
            MaterialType::Aggregate as u8,
            0.01,
            0.01,
            0.0,
            3.0,
            0.6,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            2.0,
            1.0,
        );
        // Air entraining agent (dosage 0.3 kg, target 6% air in reactivity field)
        tensor.add_material(
            0.3,
            1.0,
            MaterialType::AirEntrainer as u8,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0, // co2, cost, blaine, fm, shape
            0.0,
            0.0,
            0.0, // visc, yield, thix
            0.0,
            6.0,
            0.0,
            0.0, // k, reactivity (air target %), ar, ts
            0.0,
            0.0, // absorption, moisture
        );

        let result = FreezeThawEngine::compute(&tensor, FreezeThawExposure::Severe, 35.0);

        // Should have entrained air
        assert!(
            result.air_content >= 5.0,
            "Air entrainer should provide adequate air: got {}",
            result.air_content
        );

        println!(
            "✅ Air entrainer test passed: air={:.1}%",
            result.air_content
        );
    }
}
