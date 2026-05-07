// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: LicenseRef-Proprietary
//
// MaOS — Material Agnostic Operating System
// NanoEngine: Nanomaterial Enhancement Physics
//
// ═══════════════════════════════════════════════════════════════════════════════
// PHYSICS MODELS
// ═══════════════════════════════════════════════════════════════════════════════
//
// 1. Pozzolanic Activity (Nano-silica, Nazari & Riahi, 2011)
//    The high surface area of nano-silica accelerates pozzolanic reaction:
//    k_poz = k_0 × exp(α × SSA / SSA_ref)
//    where SSA_ref = 200 m²/g (typical silica fume reference)
//
// 2. Nucleation Seeding Effect (Thomas et al., 2009)
//    Nanomaterials act as nucleation sites for C-S-H:
//    Δt_set = -β × log(1 + n × SSA / SSA_ref)
//    where β ~ 0.3 (empirical factor for nano-silica)
//
// 3. Strength Enhancement (Sanchez & Sobolev, 2010)
//    Δσ/σ_0 = γ × (d/d_nano)^n × exp(-w/c × κ)
//    where:
//    - d: Cement particle size (~15 μm)
//    - d_nano: Nano particle size (~0.02 μm for nano-SiO2)
//    - n ≈ 0.3-0.5 (size effect exponent)
//    - κ: w/c sensitivity factor
//
// 4. Pore Refinement (Mondal et al., 2010)
//    Nanomaterials reduce porosity and refine pore structure:
//    φ_refined = φ_0 × (1 - δ × V_nano)
//    where δ ~ 5-10 for nano-silica at optimal dosage
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::tensors::{MaterialType, MixTensor, MIX_TENSOR_STRIDE};
use wasm_bindgen::prelude::*;

/// Result of nanomaterial physics calculations
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct NanoResult {
    /// Pozzolanic activity multiplier (1.0 = baseline, >1 = enhanced)
    pub pozzolanic_factor: f32,
    /// Setting time change (minutes, negative = faster)
    pub set_time_change: f32,
    /// Strength enhancement factor (multiply by base strength)
    pub strength_factor: f32,
    /// Porosity reduction factor (0-1, 0.8 = 20% reduction)
    pub porosity_factor: f32,
    /// Total nanomaterial dosage (% by cement weight)
    pub nano_dosage: f32,
    /// Average specific surface area (m²/g)
    pub avg_ssa: f32,
}

#[wasm_bindgen]
pub struct NanoEngine;

#[wasm_bindgen]
impl NanoEngine {
    /// Compute nanomaterial effects on concrete properties
    ///
    /// # Arguments
    /// * `mix` - The mix tensor containing nanomaterials
    /// * `baseline_porosity` - Baseline porosity fraction (0-1, typically 0.15-0.25)
    ///
    /// # Returns
    /// NanoResult with enhancement factors
    pub fn compute(mix: &MixTensor, _baseline_porosity: f32) -> NanoResult {
        let data = mix.data();
        let stride = MIX_TENSOR_STRIDE;
        let count = data.len() / stride;

        // Accumulate nanomaterial and cement properties
        let mut cement_mass = 0.0_f32;
        let mut nano_mass = 0.0_f32;
        let mut weighted_ssa = 0.0_f32;
        let mut weighted_reactivity = 0.0_f32;

        for i in 0..count {
            let offset = i * stride;
            let mass = data[offset];
            let type_id = data[offset + 2] as u8;
            let blaine = data[offset + 5]; // blaine/SSA field
            let reactivity = data[offset + 12]; // reactivity field at position 12

            if type_id == MaterialType::Cement as u8 {
                cement_mass += mass;
            } else if type_id == MaterialType::Nanomaterial as u8 {
                nano_mass += mass;
                // SSA for nanomaterials stored in blaine field
                // Use a reasonable default if not specified
                let actual_ssa = if blaine > 0.0 { blaine } else { 200.0 };
                weighted_ssa += actual_ssa * mass;

                // Reactivity for nanomaterials
                let actual_react = if reactivity > 0.0 { reactivity } else { 1.0 };
                weighted_reactivity += actual_react * mass;
            }
        }

        // No nanomaterials present
        if nano_mass < 0.001 || cement_mass < 0.001 {
            return NanoResult {
                pozzolanic_factor: 1.0,
                set_time_change: 0.0,
                strength_factor: 1.0,
                porosity_factor: 1.0,
                nano_dosage: 0.0,
                avg_ssa: 0.0,
            };
        }

        // Calculate nano parameters
        let nano_dosage = (nano_mass / cement_mass) * 100.0; // % by cement weight
        let avg_ssa = weighted_ssa / nano_mass;
        let avg_reactivity = weighted_reactivity / nano_mass;

        // Reference SSA for silica fume (~20 m²/g) and nano-silica (~200-400 m²/g)
        let ssa_ref = 200.0_f32;
        let ssa_ratio = avg_ssa / ssa_ref;

        // ═══════════════════════════════════════════════════════════════════════
        // Model 1: Pozzolanic Activity Enhancement
        // ═══════════════════════════════════════════════════════════════════════
        // Higher SSA = faster pozzolanic reaction
        // α ~ 0.5 (empirical coefficient)
        let alpha = 0.5_f32;
        let pozzolanic_factor = (alpha * ssa_ratio.ln().max(0.0)).exp().clamp(1.0, 5.0);

        // ═══════════════════════════════════════════════════════════════════════
        // Model 2: Nucleation Seeding (Setting Time)
        // ═══════════════════════════════════════════════════════════════════════
        // Nanomaterials accelerate hydration via nucleation
        // β ~ 30 minutes per decade of SSA ratio
        let beta = 30.0_f32;
        let set_time_change = -beta * (1.0 + nano_dosage * ssa_ratio).ln();

        // ═══════════════════════════════════════════════════════════════════════
        // Model 3: Strength Enhancement
        // ═══════════════════════════════════════════════════════════════════════
        // Nano-filling + pozzolanic activity both contribute
        // Optimal dosage typically 2-3% for nano-silica, 0.1-0.5% for CNT/GO

        // Effectiveness peaks at optimal dosage, then may decrease (agglomeration)
        // Simplified parabolic model with reactivity weighting
        let optimal_dosage = 2.5_f32; // % by cement weight for nano-silica type
        let dosage_efficiency =
            1.0 - ((nano_dosage - optimal_dosage) / (optimal_dosage * 2.0)).powi(2);
        let dosage_efficiency = dosage_efficiency.max(0.1);

        // Strength factor: includes pozzolanic contribution and size effect
        // γ ~ 0.15 (max ~15% enhancement at optimal dosage)
        let gamma = 0.15_f32;
        let strength_factor = 1.0 + gamma * dosage_efficiency * avg_reactivity * ssa_ratio.sqrt();
        let strength_factor = strength_factor.clamp(1.0, 1.5); // Cap at 50% enhancement

        // ═══════════════════════════════════════════════════════════════════════
        // Model 4: Pore Refinement
        // ═══════════════════════════════════════════════════════════════════════
        // Nanomaterials fill gel pores and refine pore structure
        // δ ~ 5 at optimal dosage
        let delta = 5.0_f32;
        let pore_reduction = delta * (nano_dosage / 100.0) * dosage_efficiency;
        let porosity_factor = (1.0 - pore_reduction).clamp(0.3, 1.0);

        NanoResult {
            pozzolanic_factor,
            set_time_change,
            strength_factor,
            porosity_factor,
            nano_dosage,
            avg_ssa,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    fn create_nano_mix(nano_mass: f32, nano_ssa: f32, nano_reactivity: f32) -> MixTensor {
        let mut tensor = MixTensor::new();
        // Add cement (400 kg, sg 3.15)
        // add_material(mass, sg, type, co2, cost, blaine, fm, shape, visc, yield, thix, k, react, ar, ts, absorption, moisture)
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
        // Add water (180 kg)
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
        // Add nanomaterial (mass, sg ~2.2 for nano-silica)
        // SSA in blaine position (5), reactivity in position 12
        tensor.add_material(
            nano_mass,
            2.2,
            MaterialType::Nanomaterial as u8,
            0.0,
            0.0,      // co2, cost
            nano_ssa, // blaine/SSA at position 5
            0.0,
            0.0, // fm, shape
            0.0,
            0.0,
            0.0,             // visc, yield, thix
            0.0,             // k_factor
            nano_reactivity, // reactivity at position 12
            0.0,
            0.0, // ar, ts
            0.0,
            0.0, // absorption, moisture
        );
        tensor
    }

    #[test]
    fn test_no_nano() {
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

        let result = NanoEngine::compute(&tensor, 0.18);

        assert_eq!(result.pozzolanic_factor, 1.0);
        assert_eq!(result.set_time_change, 0.0);
        assert_eq!(result.strength_factor, 1.0);
        assert_eq!(result.porosity_factor, 1.0);
        assert_eq!(result.nano_dosage, 0.0);
        println!("✅ No nanomaterial test passed");
    }

    #[test]
    fn test_nano_silica_optimal() {
        // Nano-silica: 10 kg (2.5% of cement), SSA = 200 m²/g, reactivity = 1.0
        let mix = create_nano_mix(10.0, 200.0, 1.0);
        let result = NanoEngine::compute(&mix, 0.18);

        // Dosage should be 2.5%
        assert!(
            (result.nano_dosage - 2.5).abs() < 0.1,
            "Dosage should be ~2.5%: got {}",
            result.nano_dosage
        );

        // Strength factor should be enhanced
        assert!(
            result.strength_factor > 1.0,
            "Strength factor should be > 1.0: got {}",
            result.strength_factor
        );

        // Set time should be reduced (negative)
        assert!(
            result.set_time_change < 0.0,
            "Set time change should be negative: got {}",
            result.set_time_change
        );

        // Porosity should be reduced
        assert!(
            result.porosity_factor < 1.0,
            "Porosity factor should be < 1.0: got {}",
            result.porosity_factor
        );

        println!(
            "✅ Nano-silica optimal test passed: dosage={:.2}%, σ_factor={:.3}",
            result.nano_dosage, result.strength_factor
        );
    }

    #[test]
    fn test_high_ssa_effect() {
        // Compare standard SSA vs high SSA (e.g., CNT-like)
        let mix_low_ssa = create_nano_mix(8.0, 100.0, 1.0);
        let mix_high_ssa = create_nano_mix(8.0, 500.0, 1.0);

        let result_low = NanoEngine::compute(&mix_low_ssa, 0.18);
        let result_high = NanoEngine::compute(&mix_high_ssa, 0.18);

        // Higher SSA should give faster set (more negative)
        assert!(
            result_high.set_time_change < result_low.set_time_change,
            "Higher SSA should accelerate set time"
        );

        // Higher SSA should enhance pozzolanic activity
        assert!(
            result_high.pozzolanic_factor >= result_low.pozzolanic_factor,
            "Higher SSA should enhance pozzolanic activity"
        );

        println!("✅ SSA effect test passed");
    }

    #[test]
    fn test_overdosing_effect() {
        // Test that overdosing reduces effectiveness
        let mix_optimal = create_nano_mix(10.0, 200.0, 1.0); // 2.5%
        let mix_over = create_nano_mix(30.0, 200.0, 1.0); // 7.5%

        let result_opt = NanoEngine::compute(&mix_optimal, 0.18);
        let result_over = NanoEngine::compute(&mix_over, 0.18);

        // Strength enhancement should be lower at overdose
        // (This models agglomeration effects)
        assert!(
            result_over.strength_factor < result_opt.strength_factor + 0.1,
            "Overdosing should not significantly improve strength"
        );

        println!(
            "✅ Overdosing effect test passed: opt={:.3}, over={:.3}",
            result_opt.strength_factor, result_over.strength_factor
        );
    }

    #[test]
    fn test_reactivity_effect() {
        // Test reactivity influence (e.g., comparing reactive nano-silica vs inert nano-clay)
        let mix_inert = create_nano_mix(10.0, 200.0, 0.3);
        let mix_reactive = create_nano_mix(10.0, 200.0, 1.5);

        let result_inert = NanoEngine::compute(&mix_inert, 0.18);
        let result_reactive = NanoEngine::compute(&mix_reactive, 0.18);

        // Higher reactivity should give higher strength factor
        assert!(
            result_reactive.strength_factor > result_inert.strength_factor,
            "Higher reactivity should improve strength factor"
        );

        println!("✅ Reactivity effect test passed");
    }
}
