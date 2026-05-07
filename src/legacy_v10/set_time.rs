// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: Apache-2.0
//
// MaOS — Material Agnostic Operating System
// SetTimeEngine: Setting Time Prediction for Cementitious Materials
//
// ═══════════════════════════════════════════════════════════════════════════════
// PHYSICS MODELS
// ═══════════════════════════════════════════════════════════════════════════════
//
// 1. Baseline Setting Time (ASTM C403/C191)
//    Initial set: t_i = f(cement fineness, C3S content, w/c)
//    Final set: t_f ≈ 1.5-2.0 × t_i
//
// 2. Temperature Effect (Arrhenius, Tank & Carino, 1991)
//    t_T = t_ref × exp[E_a/R × (1/T - 1/T_ref)]
//    where:
//    - E_a: Activation energy (~33-42 kJ/mol for OPC)
//    - T_ref: Reference temperature (20°C = 293K)
//
// 3. Admixture Effects (ACI 212.3R)
//    - Accelerators: t_acc = t_base × (1 - k_acc × dosage)
//    - Retarders: t_ret = t_base × (1 + k_ret × dosage)
//    - where k depends on admixture type and cement compatibility
//
// 4. SCM Effect (Wang & Lee, 2010)
//    High SCM replacement ratios generally retard setting:
//    t_scm = t_base × (1 + 0.3 × SCM_ratio)
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::tensors::{MaterialType, MixTensor, MIX_TENSOR_STRIDE};
use wasm_bindgen::prelude::*;

/// Result of setting time calculations
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct SetTimeResult {
    /// Initial setting time (minutes from mixing)
    pub initial_set: f32,
    /// Final setting time (minutes from mixing)
    pub final_set: f32,
    /// Open time for working (minutes from mixing)
    pub open_time: f32,
    /// Temperature sensitivity factor (1.0 = baseline)
    pub temp_sensitivity: f32,
    /// Total accelerator effect (negative = faster)
    pub accelerator_effect: f32,
    /// Total retarder effect (positive = slower)
    pub retarder_effect: f32,
}

#[wasm_bindgen]
pub struct SetTimeEngine;

#[wasm_bindgen]
impl SetTimeEngine {
    /// Compute setting times for the mix
    ///
    /// # Arguments
    /// * `mix` - The mix tensor
    /// * `temperature` - Ambient temperature (°C)
    /// * `humidity` - Relative humidity (0-1)
    ///
    /// # Returns
    /// SetTimeResult with setting time predictions
    pub fn compute(mix: &MixTensor, temperature: f32, humidity: f32) -> SetTimeResult {
        let data = mix.data();
        let stride = MIX_TENSOR_STRIDE;
        let count = data.len() / stride;

        // Accumulate material properties
        let mut cement_mass = 0.0_f32;
        let mut water_mass = 0.0_f32;
        let mut scm_mass = 0.0_f32;
        let mut accelerator_effect = 0.0_f32;
        let mut retarder_effect = 0.0_f32;
        let mut cement_blaine = 350.0_f32; // Default fineness
        let mut cement_c3s = 0.55_f32; // Default C3S content

        for i in 0..count {
            let offset = i * stride;
            let mass = data[offset];
            let type_id = data[offset + 2] as u8;
            let blaine = data[offset + 5]; // blaine at position 5
            let shape = data[offset + 7]; // shape can store c3s for cement
            let reactivity = data[offset + 12]; // reactivity can store set_time_change

            match type_id {
                t if t == MaterialType::Cement as u8 => {
                    cement_mass += mass;
                    if blaine > 0.0 {
                        cement_blaine = blaine;
                    }
                    if shape > 0.0 {
                        cement_c3s = shape;
                    } // c3s stored in shape for cement
                }
                t if t == MaterialType::Water as u8 => {
                    water_mass += mass;
                }
                t if t == MaterialType::SCM as u8 => {
                    scm_mass += mass;
                }
                t if t == MaterialType::Accelerator as u8 => {
                    // set_time_change stored in reactivity field for admixtures
                    let effect = if reactivity != 0.0 { reactivity } else { -0.25 };
                    accelerator_effect += effect * mass / cement_mass.max(1.0);
                }
                t if t == MaterialType::Retarder as u8 => {
                    // set_time_change stored in reactivity field for admixtures
                    let effect = if reactivity != 0.0 { reactivity } else { 0.5 };
                    retarder_effect += effect * mass / cement_mass.max(1.0);
                }
                t if t == MaterialType::Admixture as u8 => {
                    // Generic admixtures may have set_time_change in reactivity
                    if reactivity < 0.0 {
                        accelerator_effect += reactivity * mass / cement_mass.max(1.0);
                    } else if reactivity > 0.0 {
                        retarder_effect += reactivity * mass / cement_mass.max(1.0);
                    }
                }
                _ => {}
            }
        }

        // Handle case with no cement
        if cement_mass < 1.0 {
            return SetTimeResult {
                initial_set: 0.0,
                final_set: 0.0,
                open_time: 0.0,
                temp_sensitivity: 1.0,
                accelerator_effect: 0.0,
                retarder_effect: 0.0,
            };
        }

        // ═══════════════════════════════════════════════════════════════════════
        // Model 1: Baseline Setting Time
        // ═══════════════════════════════════════════════════════════════════════

        // w/c ratio
        let w_c = water_mass / cement_mass;
        let w_c_clamped = w_c.clamp(0.25, 0.70);

        // Baseline initial set time (minutes)
        // Higher fineness = faster set, Higher C3S = faster set
        // Reference: 180 minutes at Blaine=350, C3S=0.55, w/c=0.45
        let blaine_factor = 350.0 / cement_blaine.max(200.0);
        let c3s_factor = 0.55 / cement_c3s.clamp(0.40, 0.70);
        let wc_factor = (w_c_clamped / 0.45).powf(0.7);

        let base_initial = 180.0 * blaine_factor * c3s_factor * wc_factor;

        // ═══════════════════════════════════════════════════════════════════════
        // Model 2: Temperature Effect (Arrhenius)
        // ═══════════════════════════════════════════════════════════════════════

        // Activation energy for OPC hydration
        let e_a = 40000.0_f32; // J/mol (typical for OPC)
        let r = 8.314_f32; // J/(mol·K)
        let t_ref = 293.0_f32; // 20°C reference
        let t_kelvin = temperature + 273.15;

        // Arrhenius factor
        let temp_factor = (e_a / r * (1.0 / t_kelvin - 1.0 / t_ref)).exp();
        let temp_factor = temp_factor.clamp(0.2, 5.0); // Reasonable bounds

        // ═══════════════════════════════════════════════════════════════════════
        // Model 3: SCM Effect
        // ═══════════════════════════════════════════════════════════════════════

        let scm_ratio = scm_mass / (cement_mass + scm_mass);
        let scm_factor = 1.0 + 0.3 * scm_ratio;

        // ═══════════════════════════════════════════════════════════════════════
        // Model 4: Admixture Effects
        // ═══════════════════════════════════════════════════════════════════════

        // Accelerator effect (negative = faster set)
        // Clamp total accelerator effect to prevent impossible results
        let acc_factor = (1.0 + accelerator_effect).clamp(0.3, 1.0);

        // Retarder effect (positive = slower set)
        let ret_factor = (1.0 + retarder_effect).clamp(1.0, 4.0);

        // ═══════════════════════════════════════════════════════════════════════
        // Model 5: Humidity Effect
        // ═══════════════════════════════════════════════════════════════════════
        // Low humidity accelerates surface drying but doesn't truly accelerate hydration
        // This affects workability more than actual setting
        let humidity_factor = if humidity < 0.5 {
            0.9 + 0.2 * humidity // Slight acceleration at low humidity
        } else {
            1.0
        };

        // ═══════════════════════════════════════════════════════════════════════
        // Combined Setting Times
        // ═══════════════════════════════════════════════════════════════════════

        let initial_set =
            base_initial * temp_factor * scm_factor * acc_factor * ret_factor * humidity_factor;

        // Final set is typically 1.5-2.0 times initial set
        let final_set = initial_set * 1.7;

        // Open time (working time) is approximately 70-80% of initial set
        let open_time = initial_set * 0.75;

        SetTimeResult {
            initial_set,
            final_set,
            open_time,
            temp_sensitivity: temp_factor,
            accelerator_effect,
            retarder_effect,
        }
    }

    /// Calculate equivalent age at reference temperature (maturity concept)
    ///
    /// # Arguments
    /// * `time_hours` - Actual elapsed time in hours
    /// * `temperature` - Actual temperature (°C)
    ///
    /// # Returns
    /// Equivalent age at 20°C (hours)
    pub fn equivalent_age(time_hours: f32, temperature: f32) -> f32 {
        let e_a = 40000.0_f32;
        let r = 8.314_f32;
        let t_ref = 293.0_f32;
        let t_kelvin = temperature + 273.15;

        // Maturity factor
        let k = (e_a / r * (1.0 / t_ref - 1.0 / t_kelvin)).exp();

        time_hours * k
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
        // Cement (400 kg, blaine=350, c3s=0.55 stored in shape)
        // add_material(mass, sg, type, co2, cost, blaine, fm, shape, visc, yield, thix, k, react, ar, ts, absorption, moisture)
        tensor.add_material(
            400.0,
            3.15,
            MaterialType::Cement as u8,
            0.12,
            0.9,
            350.0,
            0.0,
            0.55, // co2, cost, blaine, fm, shape (c3s)
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
    fn test_baseline_setting() {
        let mix = create_basic_mix();
        let result = SetTimeEngine::compute(&mix, 20.0, 0.60);

        // Initial set should be around 180 minutes at reference conditions
        assert!(
            result.initial_set > 120.0 && result.initial_set < 300.0,
            "Initial set should be 120-300 min: got {}",
            result.initial_set
        );

        // Final set should be > initial set
        assert!(
            result.final_set > result.initial_set,
            "Final set should be > initial set"
        );

        // Open time should be < initial set
        assert!(
            result.open_time < result.initial_set,
            "Open time should be < initial set"
        );

        // Temp sensitivity at 20°C should be ~1.0
        assert!(
            (result.temp_sensitivity - 1.0).abs() < 0.1,
            "Temp sensitivity at 20°C should be ~1.0: got {}",
            result.temp_sensitivity
        );

        println!(
            "✅ Baseline setting test passed: ti={:.1}min, tf={:.1}min",
            result.initial_set, result.final_set
        );
    }

    #[test]
    fn test_temperature_effect() {
        let mix = create_basic_mix();

        let cold = SetTimeEngine::compute(&mix, 5.0, 0.60);
        let normal = SetTimeEngine::compute(&mix, 20.0, 0.60);
        let hot = SetTimeEngine::compute(&mix, 35.0, 0.60);

        // Cold should be slower than normal
        assert!(
            cold.initial_set > normal.initial_set,
            "Cold should delay setting: cold={:.1}, normal={:.1}",
            cold.initial_set,
            normal.initial_set
        );

        // Hot should be faster than normal
        assert!(
            hot.initial_set < normal.initial_set,
            "Hot should accelerate setting: hot={:.1}, normal={:.1}",
            hot.initial_set,
            normal.initial_set
        );

        println!("✅ Temperature effect test passed");
        println!(
            "   5°C: {:.1}min, 20°C: {:.1}min, 35°C: {:.1}min",
            cold.initial_set, normal.initial_set, hot.initial_set
        );
    }

    #[test]
    fn test_accelerator_effect() {
        let mut mix = create_basic_mix();
        // Add calcium chloride accelerator (2% by cement weight)
        // set_time_change of -0.3 stored in reactivity position 12
        mix.add_material(
            8.0,
            2.15,
            MaterialType::Accelerator as u8,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0, // co2, cost, blaine, fm, shape
            0.0,
            0.0,
            0.0, // visc, yield, thix
            0.0,
            -0.3,
            0.0,
            0.0, // k, reactivity (set_time_change), ar, ts
            0.0,
            0.0, // absorption, moisture
        );

        let normal = create_basic_mix();
        let result_normal = SetTimeEngine::compute(&normal, 20.0, 0.60);
        let result_acc = SetTimeEngine::compute(&mix, 20.0, 0.60);

        // Accelerated mix should set faster
        assert!(
            result_acc.initial_set < result_normal.initial_set,
            "Accelerator should reduce setting time"
        );

        // Accelerator effect should be negative
        assert!(
            result_acc.accelerator_effect < 0.0,
            "Accelerator effect should be negative"
        );

        println!(
            "✅ Accelerator effect test passed: normal={:.1}min, acc={:.1}min",
            result_normal.initial_set, result_acc.initial_set
        );
    }

    #[test]
    fn test_retarder_effect() {
        let mut mix = create_basic_mix();
        // Add sugar-based retarder (0.2% by cement weight)
        // set_time_change of 1.0 stored in reactivity position 12
        mix.add_material(
            0.8,
            1.5,
            MaterialType::Retarder as u8,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0, // co2, cost, blaine, fm, shape
            0.0,
            0.0,
            0.0, // visc, yield, thix
            0.0,
            1.0,
            0.0,
            0.0, // k, reactivity (set_time_change), ar, ts
            0.0,
            0.0, // absorption, moisture
        );

        let normal = create_basic_mix();
        let result_normal = SetTimeEngine::compute(&normal, 20.0, 0.60);
        let result_ret = SetTimeEngine::compute(&mix, 20.0, 0.60);

        // Retarded mix should set slower
        assert!(
            result_ret.initial_set > result_normal.initial_set,
            "Retarder should increase setting time"
        );

        // Retarder effect should be positive
        assert!(
            result_ret.retarder_effect > 0.0,
            "Retarder effect should be positive"
        );

        println!(
            "✅ Retarder effect test passed: normal={:.1}min, ret={:.1}min",
            result_normal.initial_set, result_ret.initial_set
        );
    }

    #[test]
    fn test_scm_effect() {
        let mut mix = create_basic_mix();
        // Add fly ash (100 kg, 25% replacement)
        mix.add_material(
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

        let normal = create_basic_mix();
        let result_normal = SetTimeEngine::compute(&normal, 20.0, 0.60);
        let result_scm = SetTimeEngine::compute(&mix, 20.0, 0.60);

        // SCM should slightly delay setting
        assert!(
            result_scm.initial_set >= result_normal.initial_set,
            "SCM should delay or maintain setting time"
        );

        println!(
            "✅ SCM effect test passed: normal={:.1}min, scm={:.1}min",
            result_normal.initial_set, result_scm.initial_set
        );
    }

    #[test]
    fn test_equivalent_age() {
        // At 20°C, equivalent age should equal real time
        let eq_20 = SetTimeEngine::equivalent_age(24.0, 20.0);
        assert!(
            (eq_20 - 24.0).abs() < 1.0,
            "Equivalent age at 20°C should be ~24h: got {}",
            eq_20
        );

        // At higher temp, equivalent age > real time
        let eq_35 = SetTimeEngine::equivalent_age(24.0, 35.0);
        assert!(
            eq_35 > 24.0,
            "Equivalent age at 35°C should be > 24h: got {}",
            eq_35
        );

        // At lower temp, equivalent age < real time
        let eq_5 = SetTimeEngine::equivalent_age(24.0, 5.0);
        assert!(
            eq_5 < 24.0,
            "Equivalent age at 5°C should be < 24h: got {}",
            eq_5
        );

        println!(
            "✅ Equivalent age test passed: 5°C={:.1}h, 20°C={:.1}h, 35°C={:.1}h",
            eq_5, eq_20, eq_35
        );
    }
}
