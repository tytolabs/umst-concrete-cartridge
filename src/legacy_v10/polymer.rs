// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: Apache-2.0
//
// MaOS — Material Agnostic Operating System
// PolymerEngine: Polymer-Modified Concrete Physics
//
// ═══════════════════════════════════════════════════════════════════════════════
// PHYSICS MODELS
// ═══════════════════════════════════════════════════════════════════════════════
//
// 1. Film Formation (Ohama Model, 1995)
//    Polymer latexes form continuous films above MFT (min film-forming temp):
//    - Film formation factor: η_film = 1 - exp(-k × P/C × (T - MFT)/10)
//    where P/C is polymer-cement ratio
//
// 2. Flexural Strength Enhancement (ACI 548.3R)
//    σ_flex,mod = σ_flex,0 × (1 + α × P/C)
//    where α ~ 1.5-3.0 depending on polymer type
//
// 3. Adhesion Enhancement (Schulze & Killermann, 2001)
//    τ_bond = τ_0 × (1 + β × P/C × η_film)
//    where β ~ 2.0-4.0 for latex-modified mortars
//
// 4. Permeability Reduction (Justnes & Oyen, 1994)
//    k_mod = k_0 × exp(-γ × P/C × η_film)
//    Polymer films significantly reduce permeability
//
// 5. Polymer Types and Properties:
//    - Styrene-Butadiene Rubber (SBR): Good flexibility, moderate strength
//    - Acrylic: Good UV resistance, moderate adhesion
//    - EVA (Ethylene Vinyl Acetate): Excellent adhesion, moderate cost
//    - Epoxy: Highest strength, but expensive
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::tensors::{MaterialType, MixTensor, MIX_TENSOR_STRIDE};
use wasm_bindgen::prelude::*;

/// Result of polymer modification calculations
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct PolymerResult {
    /// Polymer-cement ratio (by mass)
    pub polymer_cement_ratio: f32,
    /// Film formation effectiveness (0-1)
    pub film_formation: f32,
    /// Flexural strength multiplier (1.0 = no change)
    pub flexural_factor: f32,
    /// Bond/adhesion strength multiplier
    pub adhesion_factor: f32,
    /// Permeability reduction factor (0-1, lower = less permeable)
    pub permeability_factor: f32,
    /// Flexibility/strain capacity improvement (%)
    pub flexibility_gain: f32,
    /// Minimum film-forming temperature (°C)
    pub min_film_temp: f32,
}

#[wasm_bindgen]
pub struct PolymerEngine;

#[wasm_bindgen]
impl PolymerEngine {
    /// Compute polymer modification effects
    ///
    /// # Arguments
    /// * `mix` - The mix tensor containing polymer modifiers
    /// * `temperature` - Curing temperature (°C)
    /// * `humidity` - Curing relative humidity (0-1)
    ///
    /// # Returns
    /// PolymerResult with enhancement factors
    pub fn compute(mix: &MixTensor, temperature: f32, humidity: f32) -> PolymerResult {
        let data = mix.data();
        let stride = MIX_TENSOR_STRIDE;
        let count = data.len() / stride;

        // Accumulate polymer and cement properties
        let mut cement_mass = 0.0_f32;
        let mut polymer_mass = 0.0_f32;
        let mut weighted_mft = 0.0_f32; // Minimum film-forming temperature
        let mut weighted_flexibility = 0.0_f32;

        for i in 0..count {
            let offset = i * stride;
            let mass = data[offset];
            let type_id = data[offset + 2] as u8;

            if type_id == MaterialType::Cement as u8 {
                cement_mass += mass;
            } else if type_id == MaterialType::Polymer as u8 {
                polymer_mass += mass;
                // MFT stored in shape position (7)
                let mft = data[offset + 7];
                let actual_mft = if mft > 0.0 { mft } else { 5.0 }; // Default 5°C
                weighted_mft += actual_mft * mass;

                // Flexibility modifier stored in reactivity position (12)
                let flex = data[offset + 12];
                let actual_flex = if flex > 0.0 { flex } else { 1.0 };
                weighted_flexibility += actual_flex * mass;
            }
        }

        // No polymers present
        if polymer_mass < 0.001 || cement_mass < 0.001 {
            return PolymerResult {
                polymer_cement_ratio: 0.0,
                film_formation: 0.0,
                flexural_factor: 1.0,
                adhesion_factor: 1.0,
                permeability_factor: 1.0,
                flexibility_gain: 0.0,
                min_film_temp: 0.0,
            };
        }

        // Calculate polymer parameters
        let p_c = polymer_mass / cement_mass;
        let avg_mft = weighted_mft / polymer_mass;
        let avg_flexibility = weighted_flexibility / polymer_mass;

        // ═══════════════════════════════════════════════════════════════════════
        // Model 1: Film Formation
        // ═══════════════════════════════════════════════════════════════════════
        // Film forms above MFT, with rate dependent on temperature and humidity

        let temp_above_mft = (temperature - avg_mft).max(0.0);

        // Film formation requires both temperature and humidity
        let humidity_factor = humidity.clamp(0.3, 1.0); // Below 30% RH inhibits film formation

        // Film formation model:
        // - Proportional to P/C ratio (more polymer = better film)
        // - Increases with temperature above MFT (faster coalescence)
        // - At optimal conditions (23°C, 65% RH, 15% P/C), expect ~70% film formation
        let temp_effect = (temp_above_mft / 20.0).min(1.0); // Normalizes at 20°C above MFT
        let pc_effect = (p_c / 0.15).min(1.5); // Normalizes at 15% P/C

        let film_formation = temp_effect * pc_effect * humidity_factor * 0.85;
        let film_formation = film_formation.clamp(0.0, 1.0);

        // ═══════════════════════════════════════════════════════════════════════
        // Model 2: Flexural Strength Enhancement
        // ═══════════════════════════════════════════════════════════════════════
        // Polymers improve flexural strength through film bridging

        let alpha = 2.0_f32; // Flexural enhancement coefficient
        let flexural_factor = 1.0 + alpha * p_c * film_formation;
        let flexural_factor = flexural_factor.clamp(1.0, 3.0); // Cap at 3x improvement

        // ═══════════════════════════════════════════════════════════════════════
        // Model 3: Adhesion Enhancement
        // ═══════════════════════════════════════════════════════════════════════
        // Polymer films dramatically improve substrate adhesion

        let beta = 3.0_f32; // Adhesion enhancement coefficient
        let adhesion_factor = 1.0 + beta * p_c * film_formation;
        let adhesion_factor = adhesion_factor.clamp(1.0, 5.0); // Cap at 5x improvement

        // ═══════════════════════════════════════════════════════════════════════
        // Model 4: Permeability Reduction
        // ═══════════════════════════════════════════════════════════════════════
        // Continuous polymer films block pores and reduce permeability

        let gamma = 3.0_f32; // Permeability reduction coefficient
        let permeability_factor = (-gamma * p_c * film_formation).exp();
        let permeability_factor = permeability_factor.clamp(0.01, 1.0);

        // ═══════════════════════════════════════════════════════════════════════
        // Model 5: Flexibility/Strain Capacity
        // ═══════════════════════════════════════════════════════════════════════
        // Polymers improve strain capacity before cracking

        // Strain improvement as percentage
        let flexibility_gain = 100.0 * p_c * film_formation * avg_flexibility;
        let flexibility_gain = flexibility_gain.clamp(0.0, 500.0); // Max 500% improvement

        PolymerResult {
            polymer_cement_ratio: p_c,
            film_formation,
            flexural_factor,
            adhesion_factor,
            permeability_factor,
            flexibility_gain,
            min_film_temp: avg_mft,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    fn create_polymer_mix(polymer_mass: f32, mft: f32, flexibility: f32) -> MixTensor {
        let mut tensor = MixTensor::new();
        // add_material(mass, sg, type, co2, cost, blaine, fm, shape, visc, yield, thix, k, react, ar, ts)
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
        // Polymer (SBR latex)
        // MFT in shape position (7), flexibility in reactivity position (12)
        tensor.add_material(
            polymer_mass,
            1.05,
            MaterialType::Polymer as u8,
            0.0,
            0.0, // co2, cost
            0.0,
            0.0, // blaine, fm
            mft, // shape position for MFT
            0.0,
            0.0,
            0.0,         // visc, yield, thix
            0.0,         // k_factor
            flexibility, // reactivity position for flexibility
            0.0,
            0.0, // ar, ts
            0.0,
            0.0, // absorption, moisture
        );
        tensor
    }

    #[test]
    fn test_no_polymer() {
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

        let result = PolymerEngine::compute(&tensor, 23.0, 0.65);

        assert_eq!(result.polymer_cement_ratio, 0.0);
        assert_eq!(result.film_formation, 0.0);
        assert_eq!(result.flexural_factor, 1.0);
        assert_eq!(result.adhesion_factor, 1.0);
        assert_eq!(result.permeability_factor, 1.0);

        println!("✅ No polymer test passed");
    }

    #[test]
    fn test_typical_sbr_modification() {
        // SBR at 15% P/C ratio, MFT = 5°C, flexibility = 1.0
        let mix = create_polymer_mix(60.0, 5.0, 1.0); // 60/400 = 0.15
        let result = PolymerEngine::compute(&mix, 23.0, 0.65);

        // P/C ratio should be 0.15
        assert!(
            (result.polymer_cement_ratio - 0.15).abs() < 0.01,
            "P/C should be ~0.15: got {}",
            result.polymer_cement_ratio
        );

        // At 23°C (above MFT of 5°C), film should form
        assert!(
            result.film_formation > 0.3,
            "Film should form above MFT: got {}",
            result.film_formation
        );

        // Flexural factor should be enhanced
        assert!(
            result.flexural_factor > 1.0,
            "Flexural should be enhanced: got {}",
            result.flexural_factor
        );

        // Adhesion should be enhanced
        assert!(
            result.adhesion_factor > 1.0,
            "Adhesion should be enhanced: got {}",
            result.adhesion_factor
        );

        // Permeability should be reduced
        assert!(
            result.permeability_factor < 1.0,
            "Permeability should be reduced: got {}",
            result.permeability_factor
        );

        println!(
            "✅ SBR modification test passed: P/C={:.2}, film={:.2}, flex={:.2}x",
            result.polymer_cement_ratio, result.film_formation, result.flexural_factor
        );
    }

    #[test]
    fn test_temperature_effect() {
        let mix = create_polymer_mix(60.0, 10.0, 1.0); // MFT = 10°C

        // Below MFT - film should not form well
        let cold = PolymerEngine::compute(&mix, 5.0, 0.65);
        // Above MFT - film should form
        let warm = PolymerEngine::compute(&mix, 25.0, 0.65);

        // Warm should have better film formation
        assert!(
            warm.film_formation > cold.film_formation,
            "Warmer temp should improve film formation"
        );

        // Warm should have better enhancements
        assert!(
            warm.flexural_factor > cold.flexural_factor,
            "Warmer temp should improve flexural factor"
        );

        println!(
            "✅ Temperature effect test passed: cold film={:.2}, warm film={:.2}",
            cold.film_formation, warm.film_formation
        );
    }

    #[test]
    fn test_humidity_effect() {
        let mix = create_polymer_mix(60.0, 5.0, 1.0);

        // Low humidity - film may not form well
        let dry = PolymerEngine::compute(&mix, 23.0, 0.35);
        // High humidity - film forms well
        let humid = PolymerEngine::compute(&mix, 23.0, 0.80);

        // Humid should have better film formation
        assert!(
            humid.film_formation >= dry.film_formation,
            "Higher humidity should improve film formation"
        );

        println!(
            "✅ Humidity effect test passed: dry film={:.2}, humid film={:.2}",
            dry.film_formation, humid.film_formation
        );
    }

    #[test]
    fn test_pc_ratio_effect() {
        // Low P/C (5%)
        let low_pc = create_polymer_mix(20.0, 5.0, 1.0);
        let result_low = PolymerEngine::compute(&low_pc, 23.0, 0.65);

        // High P/C (20%)
        let high_pc = create_polymer_mix(80.0, 5.0, 1.0);
        let result_high = PolymerEngine::compute(&high_pc, 23.0, 0.65);

        // Higher P/C should give better enhancements
        assert!(
            result_high.flexural_factor > result_low.flexural_factor,
            "Higher P/C should improve flexural factor"
        );
        assert!(
            result_high.adhesion_factor > result_low.adhesion_factor,
            "Higher P/C should improve adhesion factor"
        );
        assert!(
            result_high.permeability_factor < result_low.permeability_factor,
            "Higher P/C should reduce permeability more"
        );

        println!("✅ P/C ratio effect test passed");
    }

    #[test]
    fn test_flexibility_modifier() {
        // Low flexibility polymer (rigid epoxy)
        let rigid = create_polymer_mix(60.0, 5.0, 0.5);
        let result_rigid = PolymerEngine::compute(&rigid, 23.0, 0.65);

        // High flexibility polymer (SBR)
        let flexible = create_polymer_mix(60.0, 5.0, 2.0);
        let result_flex = PolymerEngine::compute(&flexible, 23.0, 0.65);

        // Flexible polymer should give higher flexibility gain
        assert!(
            result_flex.flexibility_gain > result_rigid.flexibility_gain,
            "Flexible polymer should give higher flexibility gain"
        );

        println!(
            "✅ Flexibility modifier test passed: rigid={:.1}%, flex={:.1}%",
            result_rigid.flexibility_gain, result_flex.flexibility_gain
        );
    }
}
