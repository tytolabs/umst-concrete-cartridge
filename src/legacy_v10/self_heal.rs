// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: Apache-2.0
//
// MaOS — Material Agnostic Operating System
// SelfHealEngine: Self-Healing Concrete Physics
//
// ═══════════════════════════════════════════════════════════════════════════════
// PHYSICS MODELS
// ═══════════════════════════════════════════════════════════════════════════════
//
// Self-healing mechanisms in concrete:
//
// 1. Autogenous Healing (Schlangen & Joseph, 2009)
//    Natural healing through continued hydration of unhydrated cement
//    w_max = α × (c_unhydrated / c_total) × t^0.3
//    - Effective for cracks < 0.15mm
//    - Enhanced by high w/c ratio and presence of SCMs
//
// 2. Bacterial Healing (Jonkers et al., 2010)
//    Bacillus species precipitate CaCO3 in cracks
//    Healing rate: dw/dt = k_bac × N_bac × [substrate] × (1 - w/w_max)
//    - Effective for cracks up to 0.5mm
//    - Requires bacterial capsules + nutrients (Ca-lactate)
//
// 3. Crystalline Admixture Healing (Sisomphon et al., 2012)
//    Proprietary crystalline chemicals react with water in cracks
//    w_heal = β × V_cryst × (w_crack)^0.5 × (t/t_ref)^n
//    - Effective for cracks up to 0.4mm
//    - Requires water ingress for activation
//
// 4. Polymer Capsule Healing (Van Tittelboom et al., 2016)
//    Encapsulated healing agents rupture and release on cracking
//    Healing efficiency: η = f(capsule_content, crack_width, agent_viscosity)
//    - Single-use mechanism
//    - Effective for various crack widths
//
// 5. Shape Memory Alloy (SMA) Healing (Teall et al., 2018)
//    SMA fibers apply closing force when heated
//    F_closure = E_sma × ε_recovery × A_sma
//    - Requires thermal activation
//    - Works with other healing mechanisms
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::tensors::{MaterialType, MixTensor, MIX_TENSOR_STRIDE};
use wasm_bindgen::prelude::*;

/// Self-healing mechanism type
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HealingMechanism {
    Autogenous = 0,  // Natural cement hydration
    Bacterial = 1,   // Bacterial CaCO3 precipitation
    Crystalline = 2, // Crystalline admixtures
    Polymer = 3,     // Polymer microcapsules
    Vascular = 4,    // Vascular network healing
}

/// Result of self-healing calculations
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct SelfHealResult {
    /// Maximum healable crack width (mm)
    pub max_healable_crack: f32,
    /// Healing efficiency for 0.1mm crack (0-1)
    pub healing_efficiency_0_1mm: f32,
    /// Healing efficiency for 0.3mm crack (0-1)
    pub healing_efficiency_0_3mm: f32,
    /// Time to heal 0.1mm crack (days)
    pub heal_time_0_1mm: f32,
    /// Time to heal 0.3mm crack (days)
    pub heal_time_0_3mm: f32,
    /// Healing capacity remaining (0-1, for depletable mechanisms)
    pub remaining_capacity: f32,
    /// Recovery of mechanical strength (%)
    pub strength_recovery: f32,
    /// Primary healing mechanism detected
    pub primary_mechanism: HealingMechanism,
}

#[wasm_bindgen]
pub struct SelfHealEngine;

#[wasm_bindgen]
impl SelfHealEngine {
    /// Compute self-healing potential of the mix
    ///
    /// # Arguments
    /// * `mix` - The mix tensor
    /// * `age_days` - Age of concrete at cracking (days)
    /// * `crack_width` - Characteristic crack width (mm)
    /// * `water_available` - Whether water is available for healing (important for crystalline)
    ///
    /// # Returns
    /// SelfHealResult with healing predictions
    pub fn compute(
        mix: &MixTensor,
        age_days: f32,
        crack_width: f32,
        water_available: bool,
    ) -> SelfHealResult {
        let data = mix.data();
        let stride = MIX_TENSOR_STRIDE;
        let count = data.len() / stride;

        // Accumulate material properties
        let mut cement_mass = 0.0_f32;
        let mut water_mass = 0.0_f32;
        let mut scm_mass = 0.0_f32;
        let mut nanomaterial_mass = 0.0_f32;
        let mut has_crystalline = false;
        let mut crystalline_dosage = 0.0_f32;

        for i in 0..count {
            let offset = i * stride;
            let mass = data[offset];
            let type_id = data[offset + 2] as u8;

            match type_id {
                t if t == MaterialType::Cement as u8 => {
                    cement_mass += mass;
                }
                t if t == MaterialType::Water as u8 => {
                    water_mass += mass;
                }
                t if t == MaterialType::SCM as u8 => {
                    scm_mass += mass;
                }
                t if t == MaterialType::Nanomaterial as u8 => {
                    nanomaterial_mass += mass;
                }
                t if t == MaterialType::Admixture as u8 => {
                    // Check for crystalline healing admixture via reactivity field at position 12
                    let reactivity = data[offset + 12];
                    if reactivity > 1.5 {
                        // High reactivity indicates crystalline healing
                        has_crystalline = true;
                        crystalline_dosage += mass;
                    }
                }
                _ => {}
            }
        }

        // Calculate hydration degree (simplified model)
        // Hydration progresses over time: α(t) = α_ult × (t / (t + t_50))
        let t_50 = 3.0_f32; // Days for 50% hydration
        let alpha_ult = 0.85_f32;
        let hydration_degree = alpha_ult * age_days / (age_days + t_50);

        // Remaining unhydrated cement fraction
        let unhydrated_fraction = (1.0 - hydration_degree).max(0.0);

        // Water-cement ratio
        let w_c = if cement_mass > 0.0 {
            water_mass / cement_mass
        } else {
            0.45
        };

        // SCM ratio (affects autogenous healing via pozzolanic reaction)
        let scm_ratio = if cement_mass > 0.0 {
            scm_mass / (cement_mass + scm_mass)
        } else {
            0.0
        };

        // ═══════════════════════════════════════════════════════════════════════
        // Model 1: Autogenous Healing Potential
        // ═══════════════════════════════════════════════════════════════════════

        // Maximum healable crack width from autogenous healing
        // Increases with unhydrated cement and w/c ratio
        let alpha_auto = 0.3_f32; // Healing coefficient
        let autogenous_heal_max =
            alpha_auto * unhydrated_fraction * (1.0 + w_c) * (1.0 + scm_ratio);
        let autogenous_heal_max = autogenous_heal_max.clamp(0.0, 0.2); // mm

        // ═══════════════════════════════════════════════════════════════════════
        // Model 2: Crystalline Healing Potential
        // ═══════════════════════════════════════════════════════════════════════

        let crystalline_heal_max = if has_crystalline && water_available {
            // Crystalline healing effective up to 0.4mm
            let dosage_factor = (crystalline_dosage / cement_mass.max(1.0) * 100.0).min(3.0) / 3.0;
            0.4 * dosage_factor
        } else {
            0.0
        };

        // ═══════════════════════════════════════════════════════════════════════
        // Model 3: Nanomaterial-Enhanced Healing
        // ═══════════════════════════════════════════════════════════════════════

        // Nanomaterials enhance pozzolanic reaction and accelerate healing
        let nano_factor = if nanomaterial_mass > 0.0 {
            let nano_dosage = nanomaterial_mass / cement_mass.max(1.0);
            1.0 + 2.0 * nano_dosage.min(0.03) / 0.03 // Up to 3x at 3% dosage
        } else {
            1.0
        };

        // ═══════════════════════════════════════════════════════════════════════
        // Combined Healing Potential
        // ═══════════════════════════════════════════════════════════════════════

        let max_healable_crack = (autogenous_heal_max + crystalline_heal_max) * nano_factor;
        let max_healable_crack = max_healable_crack.min(0.5); // Physical limit

        // Determine primary mechanism
        let primary_mechanism = if crystalline_heal_max > autogenous_heal_max {
            HealingMechanism::Crystalline
        } else {
            HealingMechanism::Autogenous
        };

        // ═══════════════════════════════════════════════════════════════════════
        // Healing Efficiency for Specific Crack Widths
        // ═══════════════════════════════════════════════════════════════════════

        fn efficiency_for_crack(crack: f32, max_heal: f32) -> f32 {
            if crack <= 0.0 || max_heal <= 0.0 {
                0.0
            } else if crack <= max_heal {
                // Full healing possible
                (1.0 - crack / max_heal).powf(0.5)
            } else {
                // Partial healing only
                (max_heal / crack).powf(1.5) * 0.5
            }
        }

        let healing_efficiency_0_1mm = efficiency_for_crack(0.1, max_healable_crack);
        let healing_efficiency_0_3mm = efficiency_for_crack(0.3, max_healable_crack);

        // ═══════════════════════════════════════════════════════════════════════
        // Healing Time Estimation
        // ═══════════════════════════════════════════════════════════════════════

        // Base healing time (days) for autogenous healing
        // Smaller cracks heal faster
        fn heal_time(crack: f32, max_heal: f32, nano_factor: f32, water: bool) -> f32 {
            if crack > max_heal || crack <= 0.0 {
                f32::INFINITY
            } else {
                let base_time = 7.0 + 100.0 * (crack / 0.5).powi(2);
                let water_factor = if water { 1.0 } else { 3.0 };
                base_time * water_factor / nano_factor
            }
        }

        let heal_time_0_1mm = heal_time(0.1, max_healable_crack, nano_factor, water_available);
        let heal_time_0_3mm = heal_time(0.3, max_healable_crack, nano_factor, water_available);

        // ═══════════════════════════════════════════════════════════════════════
        // Remaining Capacity (for autogenous, based on unhydrated cement)
        // ═══════════════════════════════════════════════════════════════════════

        // Remaining capacity is based on unhydrated cement available for future healing
        // At t=0, remaining_capacity = 1.0 (all unhydrated)
        // As t→∞, remaining_capacity → 0 (fully hydrated)
        let remaining_capacity = unhydrated_fraction;
        let remaining_capacity = remaining_capacity.clamp(0.0, 1.0);

        // ═══════════════════════════════════════════════════════════════════════
        // Strength Recovery Estimation
        // ═══════════════════════════════════════════════════════════════════════

        // Healed cracks typically recover 70-90% of original strength
        let efficiency = efficiency_for_crack(crack_width, max_healable_crack);
        let strength_recovery = 90.0 * efficiency; // %

        SelfHealResult {
            max_healable_crack,
            healing_efficiency_0_1mm,
            healing_efficiency_0_3mm,
            heal_time_0_1mm,
            heal_time_0_3mm,
            remaining_capacity,
            strength_recovery,
            primary_mechanism,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    fn create_basic_mix() -> MixTensor {
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
        // Water (180 kg, w/c = 0.45)
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
        tensor
    }

    #[test]
    fn test_autogenous_healing_young_concrete() {
        let mix = create_basic_mix();

        // Young concrete (3 days) has more unhydrated cement
        let result = SelfHealEngine::compute(&mix, 3.0, 0.1, true);

        // Should have some autogenous healing potential
        assert!(
            result.max_healable_crack > 0.05,
            "Young concrete should have healing potential: got {}",
            result.max_healable_crack
        );

        // Efficiency for small cracks should be reasonable
        assert!(
            result.healing_efficiency_0_1mm > 0.0,
            "Should heal small cracks: got {}",
            result.healing_efficiency_0_1mm
        );

        // Primary mechanism should be autogenous
        assert_eq!(result.primary_mechanism, HealingMechanism::Autogenous);

        println!(
            "✅ Autogenous healing (young) test passed: max_crack={:.3}mm, eff_0.1={:.2}",
            result.max_healable_crack, result.healing_efficiency_0_1mm
        );
    }

    #[test]
    fn test_autogenous_healing_old_concrete() {
        let mix = create_basic_mix();

        // Old concrete (180 days) has less unhydrated cement
        let result = SelfHealEngine::compute(&mix, 180.0, 0.1, true);

        // Should have reduced healing potential
        assert!(
            result.remaining_capacity < 0.5,
            "Old concrete should have lower remaining capacity"
        );

        println!(
            "✅ Autogenous healing (old) test passed: remaining={:.2}",
            result.remaining_capacity
        );
    }

    #[test]
    fn test_age_effect() {
        let mix = create_basic_mix();

        let young = SelfHealEngine::compute(&mix, 3.0, 0.1, true);
        let old = SelfHealEngine::compute(&mix, 90.0, 0.1, true);

        // Young concrete should have more healing potential
        assert!(
            young.max_healable_crack > old.max_healable_crack,
            "Younger concrete should have more healing potential"
        );
        assert!(
            young.remaining_capacity > old.remaining_capacity,
            "Younger concrete should have more remaining capacity"
        );

        println!(
            "✅ Age effect test passed: young={:.3}mm, old={:.3}mm",
            young.max_healable_crack, old.max_healable_crack
        );
    }

    #[test]
    fn test_scm_enhancement() {
        let mut mix_scm = create_basic_mix();
        // Add fly ash (100 kg)
        mix_scm.add_material(
            100.0,
            2.3,
            MaterialType::SCM as u8,
            0.02,
            0.5,
            0.0,
            0.0,
            0.0, // co2, cost, blaine, fm, shape
            0.0,
            0.0,
            0.0, // visc, yield, thix
            0.0,
            0.0,
            0.0,
            0.0, // k, react, ar, ts
            3.0,
            1.5, // absorption, moisture
        );

        let without_scm = create_basic_mix();
        let result_without = SelfHealEngine::compute(&without_scm, 7.0, 0.1, true);
        let result_with = SelfHealEngine::compute(&mix_scm, 7.0, 0.1, true);

        // SCM should enhance autogenous healing
        assert!(
            result_with.max_healable_crack >= result_without.max_healable_crack,
            "SCM should enhance healing potential"
        );

        println!(
            "✅ SCM enhancement test passed: without={:.3}mm, with={:.3}mm",
            result_without.max_healable_crack, result_with.max_healable_crack
        );
    }

    #[test]
    fn test_crystalline_healing() {
        let mut mix = create_basic_mix();
        // Add crystalline healing admixture (high reactivity = 2.0 in position 12)
        mix.add_material(
            8.0,
            2.5,
            MaterialType::Admixture as u8,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0, // co2, cost, blaine, fm, shape
            0.0,
            0.0,
            0.0, // visc, yield, thix
            0.0,
            2.0,
            0.0,
            0.0, // k, reactivity (high = crystalline), ar, ts
            0.0,
            0.0, // absorption, moisture
        );

        // With water available
        let result_water = SelfHealEngine::compute(&mix, 7.0, 0.1, true);

        // Without water
        let result_dry = SelfHealEngine::compute(&mix, 7.0, 0.1, false);

        // Crystalline healing requires water
        assert!(
            result_water.max_healable_crack > result_dry.max_healable_crack,
            "Crystalline healing requires water"
        );

        // Primary mechanism should be crystalline when water available
        assert_eq!(
            result_water.primary_mechanism,
            HealingMechanism::Crystalline
        );

        println!(
            "✅ Crystalline healing test passed: water={:.3}mm, dry={:.3}mm",
            result_water.max_healable_crack, result_dry.max_healable_crack
        );
    }

    #[test]
    fn test_nanomaterial_enhancement() {
        let mut mix_nano = create_basic_mix();
        // Add nano-silica (2% by cement weight = 8 kg)
        mix_nano.add_material(
            8.0,
            2.2,
            MaterialType::Nanomaterial as u8,
            0.0,
            0.0,
            200.0,
            0.0,
            0.0, // co2, cost, blaine(SSA), fm, shape
            0.0,
            0.0,
            0.0, // visc, yield, thix
            0.0,
            1.0,
            0.0,
            0.0, // k, reactivity, ar, ts
            0.0,
            0.0, // absorption, moisture
        );

        let without_nano = create_basic_mix();
        let result_without = SelfHealEngine::compute(&without_nano, 7.0, 0.1, true);
        let result_with = SelfHealEngine::compute(&mix_nano, 7.0, 0.1, true);

        // Nanomaterials should enhance healing
        assert!(
            result_with.max_healable_crack > result_without.max_healable_crack,
            "Nanomaterials should enhance healing"
        );

        // Nanomaterials should speed up healing
        assert!(
            result_with.heal_time_0_1mm <= result_without.heal_time_0_1mm,
            "Nanomaterials should speed up healing"
        );

        println!(
            "✅ Nanomaterial enhancement test passed: without={:.3}mm, with={:.3}mm",
            result_without.max_healable_crack, result_with.max_healable_crack
        );
    }

    #[test]
    fn test_crack_width_effect() {
        let mix = create_basic_mix();
        let result = SelfHealEngine::compute(&mix, 7.0, 0.2, true);

        // Small cracks should have higher efficiency than large cracks
        assert!(
            result.healing_efficiency_0_1mm > result.healing_efficiency_0_3mm,
            "Small cracks should heal more efficiently"
        );

        // Small cracks should heal faster
        assert!(
            result.heal_time_0_1mm < result.heal_time_0_3mm,
            "Small cracks should heal faster"
        );

        println!(
            "✅ Crack width effect test passed: eff_0.1={:.2}, eff_0.3={:.2}",
            result.healing_efficiency_0_1mm, result.healing_efficiency_0_3mm
        );
    }

    #[test]
    fn test_strength_recovery() {
        let mix = create_basic_mix();

        // Small crack should have high strength recovery
        let small_crack = SelfHealEngine::compute(&mix, 7.0, 0.05, true);
        assert!(
            small_crack.strength_recovery > 50.0,
            "Small cracks should have high strength recovery"
        );

        // Large crack should have low strength recovery
        let large_crack = SelfHealEngine::compute(&mix, 7.0, 0.5, true);
        assert!(
            large_crack.strength_recovery < small_crack.strength_recovery,
            "Large cracks should have lower strength recovery"
        );

        println!(
            "✅ Strength recovery test passed: small={:.1}%, large={:.1}%",
            small_crack.strength_recovery, large_crack.strength_recovery
        );
    }
}
