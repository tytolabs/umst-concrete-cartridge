// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

use burn::tensor::{backend::Backend, Tensor};

/// Pure tensor implementation of the 3D Printability Engine.
/// Computes extrudability, buildability, and open time using Roussel's models.
/// This acts as the geometric constraint mapping for the optimization engine.
/// formal_anchor: NONE
/// formal_status: Library
/// formal_axioms: NONE
/// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
pub struct PrintabilityEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend> PrintabilityEngine<B> {
    /// Computes Buildability Score (Critical Height) using Roussel's model.
    /// h_crit = tau_0 / (rho * g) * thixotropy_factor
    ///
    /// # Arguments
    /// * `yield_stress` (tau_0) - Static yield stress [Batch, Depth, Height, Width]
    /// * `thixotropy_index` (A_thix) - Structural buildup rate [Batch, Depth, Height, Width]
    /// * `target_height_mm` - Constant scalar target height for the layer or part.
    /// formal_anchor: NONE
    /// formal_status: Library
    /// formal_axioms: NONE
    /// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
    pub fn compute_buildability(
        yield_stress: Tensor<B, 4>,
        thixotropy_index: Tensor<B, 4>,
        target_height_mm: f32,
    ) -> Tensor<B, 4> {
        let rho = 2400.0_f32; // kg/m^3
        let g = 9.81_f32; // m/s^2
        let t_char = 60.0_f32; // seconds characteristic time

        // Effective yield stress after structural buildup
        // effective_tau = tau_0 + A_thix * t_char
        let structural_buildup = thixotropy_index.clone().mul_scalar(t_char);
        let effective_tau = yield_stress.clone().add(structural_buildup);

        // h_crit (m) = effective_tau / (rho * g)
        let rho_g = rho * g;
        let h_crit_m = effective_tau.div_scalar(rho_g);
        let h_crit_mm = h_crit_m.mul_scalar(1000.0_f32);

        // Continuous stiffening factor
        // stiffening_ratio = A_thix * t_layer / max(tau_0, 1.0)
        let t_layer = 30.0_f32; // 30s per layer
        let safe_tau = yield_stress.clone().clamp_min(1.0_f32);
        let stiffening_ratio = thixotropy_index.clone().mul_scalar(t_layer).div(safe_tau);

        // thixo_factor = 1.0 + 3.0 * ln(1.0 + stiffening_ratio)
        // burn log is natural log (ln)
        let ln_term = stiffening_ratio
            .add_scalar(1.0_f32)
            .log()
            .clamp_max(2.0_f32);
        let thixo_factor = ln_term.mul_scalar(3.0_f32).add_scalar(1.0_f32);

        // Apply factor if thixotropy > 0.1
        let thixo_mask = thixotropy_index.greater_elem(0.1_f32);
        let active_thixo_factor = thixo_factor.mask_fill(thixo_mask.bool_not(), 1.0_f32);

        let effective_h_crit_mm = h_crit_mm.mul(active_thixo_factor);

        // Score ratio
        let buildability = effective_h_crit_mm
            .div_scalar(target_height_mm)
            .clamp_max(1.0_f32);

        // Layer stability (shape retention at nozzle exit)
        let soft_mask = yield_stress.clone().lower_elem(50.0_f32);
        let stability_penalty = yield_stress.div_scalar(50.0_f32).clamp_min(0.1_f32);
        let layer_stability = stability_penalty.mask_fill(soft_mask.bool_not(), 1.0_f32);

        buildability.mul(layer_stability)
    }

    /// Computes Extrudability based on the Bingham number (tau_0 / (eta * gamma_dot))
    /// formal_anchor: NONE
    /// formal_status: Library
    /// formal_axioms: NONE
    /// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
    pub fn compute_extrudability(
        yield_stress: Tensor<B, 4>,
        plastic_viscosity: Tensor<B, 4>,
        nozzle_mm: f32,
        speed_mm_s: f32,
    ) -> Tensor<B, 4> {
        let nozzle_m = nozzle_mm / 1000.0;
        let speed_m_s = speed_mm_s / 1000.0;
        let shear_rate = 8.0 * speed_m_s / nozzle_m;

        // safe visocisty
        let safe_visc = plastic_viscosity.clone().clamp_min(0.001_f32);
        let bingham_number = yield_stress.clone().div(safe_visc.mul_scalar(shear_rate));

        // Simplified continuous piecewise scoring mapped to tensors
        // Bn < 1 -> penalty (too fluid)
        // Bn 1-5 -> optimal (1.0)
        // Bn > 10 -> severe penalty (plug flow)

        let mut score = yield_stress.clone().zeros_like();
        score = score.add_scalar(1.0_f32);

        // If bn > 5.0, score decreases
        let high_bn_mask = bingham_number.clone().greater_elem(5.0_f32);
        let high_penalty = bingham_number
            .clone()
            .sub_scalar(5.0_f32)
            .div_scalar(5.0_f32)
            .mul_scalar(0.3_f32);
        let reduced_score = score.clone().sub(high_penalty);
        score = score
            .mask_fill(high_bn_mask.clone(), 0.0_f32)
            .add(reduced_score.mask_fill(high_bn_mask.clone().bool_not(), 0.0_f32));

        // Very high yield penalty (> 1000 Pa)
        let yield_penalty_mask = yield_stress.clone().greater_elem(1000.0_f32);
        score = score.mask_fill(yield_penalty_mask, 0.2_f32);

        score.clamp(0.0_f32, 1.0_f32)
    }
}
