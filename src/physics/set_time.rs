// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

use burn::tensor::{backend::Backend, Tensor};

/// Pure tensor implementation of the Set Time Engine.
/// Computes initial and final setting kinetics (Vicant penetration mapping)
/// dynamically across the spatial manifold based on local temperature and chemistry.
/// formal_anchor: NONE
/// formal_status: Library
/// formal_axioms: NONE
/// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
pub struct SetTimeEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend> SetTimeEngine<B> {
    /// Computes the initial and final set time of the material in minutes.
    /// This defines the kinetic time-domain constraint for 3D printing.
    ///
    /// # Arguments
    /// * `wc_ratio` - Water/Cement ratio tensor [Batch, Depth, Height, Width]
    /// * `temperature_c` - Ambient or internal temperature tensor in Celsius
    /// * `humidity` - Relative humidity (0.0 to 1.0)
    /// * `scm_ratio` - SCM replacement ratio (0.0 to 1.0)
    /// * `accelerator_effect` - Accelerator dosage/effect tensor (-1.0 to 0.0)
    /// * `retarder_effect` - Retarder dosage/effect tensor (0.0 to 3.0)
    /// * `cement_blaine` - Fineness of cement (typically ~350 m2/kg)
    /// * `cement_c3s` - C3S content fraction (typically ~0.55)
    #[allow(clippy::too_many_arguments)]
    /// formal_anchor: NONE
    /// formal_status: Library
    /// formal_axioms: NONE
    /// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
    pub fn compute_setting_time(
        wc_ratio: Tensor<B, 4>,
        temperature_c: Tensor<B, 4>,
        humidity: Tensor<B, 4>,
        scm_ratio: Tensor<B, 4>,
        accelerator_effect: Tensor<B, 4>,
        retarder_effect: Tensor<B, 4>,
        cement_blaine: Tensor<B, 4>,
        cement_c3s: Tensor<B, 4>,
    ) -> (Tensor<B, 4>, Tensor<B, 4>) {
        // 1. Baseline Initial Set (reference: 180 min at wc=0.45, blaine=350, c3s=0.55)
        let safe_wc = wc_ratio.clamp(0.25_f32, 0.70_f32);
        let safe_blaine = cement_blaine.clamp_min(200.0_f32);
        let safe_c3s = cement_c3s.clamp(0.40_f32, 0.70_f32);

        let blaine_factor = safe_blaine.powf_scalar(-1.0_f32).mul_scalar(350.0_f32);
        let c3s_factor = safe_c3s.powf_scalar(-1.0_f32).mul_scalar(0.55_f32);
        let wc_factor = safe_wc.div_scalar(0.45_f32).powf_scalar(0.7_f32);

        let base_initial = blaine_factor
            .mul(c3s_factor)
            .mul(wc_factor)
            .mul_scalar(180.0_f32);

        // 2. Temperature Effect (Arrhenius)
        let e_a = 40000.0_f32; // J/mol
        let r = 8.314_f32;
        let t_ref = 293.0_f32;
        let t_kelvin = temperature_c.add_scalar(273.15_f32);

        let inv_t_diff = t_kelvin.powf_scalar(-1.0_f32).sub_scalar(1.0 / t_ref);
        let temp_exponent = inv_t_diff.mul_scalar(e_a / r);
        let temp_factor = temp_exponent.exp().clamp(0.2_f32, 5.0_f32);

        // 3. SCM Effect
        let scm_factor = scm_ratio.mul_scalar(0.3_f32).add_scalar(1.0_f32);

        // 4. Admixture Effects
        let acc_factor = accelerator_effect
            .add_scalar(1.0_f32)
            .clamp(0.3_f32, 1.0_f32);
        let ret_factor = retarder_effect.add_scalar(1.0_f32).clamp(1.0_f32, 4.0_f32);

        // 5. Humidity Effect
        // if humidity < 0.5 { 0.9 + 0.2*humidity } else { 1.0 }
        let low_humidity_mask = humidity.clone().lower_elem(0.5_f32);
        let low_humidity_val = humidity.clone().mul_scalar(0.2_f32).add_scalar(0.9_f32);
        let mut humidity_factor = humidity.clone().zeros_like().add_scalar(1.0_f32);
        humidity_factor = humidity_factor
            .mask_fill(low_humidity_mask.clone(), 0.0_f32)
            .add(low_humidity_val.mask_fill(low_humidity_mask.bool_not(), 0.0_f32));

        // Combined initial set
        let initial_set = base_initial
            .mul(temp_factor)
            .mul(scm_factor)
            .mul(acc_factor)
            .mul(ret_factor)
            .mul(humidity_factor);

        // Final set is approx 1.7x initial set
        let final_set = initial_set.clone().mul_scalar(1.7_f32);

        (initial_set, final_set)
    }
}
