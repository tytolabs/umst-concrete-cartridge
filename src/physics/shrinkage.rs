// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

use burn::tensor::{backend::Backend, Tensor};

use crate::burn_compat::bool_and;

/// Pure tensor implementation of the Shrinkage Engine.
/// Computes Autogenous and Drying shrinkage strain using fib Model Code 2010 / B4 model approximations.
pub struct ShrinkageEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend> ShrinkageEngine<B> {
    /// Computes autogenous shrinkage strain (microstrain) resulting from self-desiccation.
    /// This is a critical penalization metric for topology optimization to prevent self-cracking.
    ///
    /// # Arguments
    /// * `wc_ratio` - Water/Cement ratio
    /// * `degree_hydration` - Current hydration degree (0.0 to 1.0)
    /// * `cement_content_kg` - Cement content in kg/m3
    /// * `scm_ratio` - SCM replacement ratio
    pub fn compute_autogenous_shrinkage(
        wc_ratio: Tensor<B, 4>,
        degree_hydration: Tensor<B, 4>,
        cement_content_kg: Tensor<B, 4>,
        scm_ratio: Tensor<B, 4>,
    ) -> Tensor<B, 4> {
        let critical_wc = 0.42_f32;

        // 1. Ultimate shrinkage as a function of w/c (empirical B4 fit)
        // High shrinkage at low w/c, low shrinkage at high w/c
        let low_wc_mask = wc_ratio.clone().lower_equal_elem(0.30_f32);
        let mid_wc_mask = bool_and(
            wc_ratio.clone().lower_equal_elem(0.42_f32),
            wc_ratio.clone().greater_elem(0.30_f32),
        );
        let high_wc_mask = bool_and(
            wc_ratio.clone().lower_equal_elem(0.50_f32),
            wc_ratio.clone().greater_elem(0.42_f32),
        );

        let mut eps_as_ult = wc_ratio.clone().zeros_like();

        // < 0.30: -1000 - 500 * (0.30 - w/c) / 0.05
        let eps_low = wc_ratio
            .clone()
            .mul_scalar(-1.0_f32)
            .add_scalar(0.30_f32)
            .div_scalar(0.05_f32)
            .mul_scalar(-500.0_f32)
            .sub_scalar(1000.0_f32);
        // 0.30 - 0.42: -600 - 400 * (0.42 - w/c) / 0.12
        let eps_mid = wc_ratio
            .clone()
            .mul_scalar(-1.0_f32)
            .add_scalar(0.42_f32)
            .div_scalar(0.12_f32)
            .mul_scalar(-400.0_f32)
            .sub_scalar(600.0_f32);
        // 0.42 - 0.50: -200 - 400 * (0.50 - w/c) / 0.08
        let eps_high = wc_ratio
            .clone()
            .mul_scalar(-1.0_f32)
            .add_scalar(0.50_f32)
            .div_scalar(0.08_f32)
            .mul_scalar(-400.0_f32)
            .sub_scalar(200.0_f32);
        // > 0.50: -100 * (0.60 - w/c) / 0.10
        let eps_very_high = wc_ratio
            .clone()
            .mul_scalar(-1.0_f32)
            .add_scalar(0.60_f32)
            .clamp_min(0.0_f32)
            .div_scalar(0.10_f32)
            .mul_scalar(-100.0_f32);

        eps_as_ult = eps_as_ult
            .mask_fill(low_wc_mask, 1.0_f32)
            .mul(eps_low)
            .add(
                wc_ratio
                    .clone()
                    .zeros_like()
                    .mask_fill(mid_wc_mask, 1.0_f32)
                    .mul(eps_mid),
            )
            .add(
                wc_ratio
                    .clone()
                    .zeros_like()
                    .mask_fill(high_wc_mask, 1.0_f32)
                    .mul(eps_high),
            )
            .add(
                wc_ratio
                    .clone()
                    .zeros_like()
                    .mask_fill(wc_ratio.clone().greater_elem(0.50_f32), 1.0_f32)
                    .mul(eps_very_high),
            );

        // 2. Development function (exponential decay mapping hydration to strain)
        let alpha_ult = wc_ratio.clone().div_scalar(critical_wc).clamp_max(1.0_f32);
        let active_alpha_mask = alpha_ult.clone().greater_elem(0.01_f32);

        let exponent = degree_hydration
            .clone()
            .mul_scalar(-3.0_f32)
            .div(alpha_ult.clone().clamp_min(0.01_f32));
        let dev_active = exponent
            .exp()
            .mul_scalar(-1.0_f32)
            .add_scalar(1.0_f32)
            .clamp_max(1.0_f32);

        let mut development = degree_hydration.clone().clamp_max(1.0_f32);
        development = development
            .mask_fill(active_alpha_mask.clone(), 0.0_f32)
            .add(dev_active.mask_fill(active_alpha_mask.bool_not(), 0.0_f32));

        // 3. Paste Volume and SCM modifiers
        let paste_factor = cement_content_kg.div_scalar(350.0_f32).sqrt();
        let scm_factor = scm_ratio.mul_scalar(0.3_f32).add_scalar(1.0_f32);

        // Final Autogenous Strain (microstrain, negative value)
        eps_as_ult
            .mul(development)
            .mul(paste_factor)
            .mul(scm_factor)
    }
}
