// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO


use burn::tensor::{backend::Backend, Tensor};

/// Pure tensor implementation of the Freeze-Thaw Durability Engine.
/// Computes air-void spacing factors and critical saturation to prevent cyclic frost damage.
pub struct FreezeThawEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend> FreezeThawEngine<B> {
    /// Computes the spacing factor and relative durability factor for cyclic freeze-thaw.
    /// Returns (spacing_factor_mm, durability_factor).
    ///
    /// # Arguments
    /// * `air_fraction` - Volume fraction of entrained air (0.0 to 0.15)
    /// * `paste_fraction` - Volume fraction of cement paste (0.2 to 0.4)
    /// * `air_void_specific_surface` - Surface area of air voids (mm^-1, typically 25 to 45)
    /// * `required_air` - Target air percentage based on exposure severity (e.g. 6.0 for severe)
    pub fn compute_durability(
        air_fraction: Tensor<B, 4>,
        paste_fraction: Tensor<B, 4>,
        air_void_specific_surface: Tensor<B, 4>,
        required_air: f32,
    ) -> (Tensor<B, 4>, Tensor<B, 4>) {
        let safe_air = air_fraction.clone().clamp_min(0.001_f32);
        let alpha = air_void_specific_surface.clamp(20.0_f32, 50.0_f32);

        // 1. Spacing Factor (Powers & Helmuth, 1953)
        // L = (3/alpha) * [ 1.4 * (1 + p/a)^(1/3) - 1 ]
        let p_over_a = paste_fraction.div(safe_air.clone());
        let cubic_term = p_over_a
            .add_scalar(1.0_f32)
            .powf_scalar(1.0_f32 / 3.0_f32)
            .mul_scalar(1.4_f32)
            .sub_scalar(1.0_f32);
        let spacing_factor = alpha
            .powf_scalar(-1.0_f32)
            .mul_scalar(3.0_f32)
            .mul(cubic_term)
            .clamp(0.05_f32, 1.5_f32);

        // 2. Durability Factor (empirical mapping from spacing to ASTM C666 prediction)
        let air_content_pct = air_fraction.mul_scalar(100.0_f32);

        let adequate_air_mask = air_content_pct.clone().greater_equal_elem(required_air);
        let air_effectiveness = air_content_pct
            .div_scalar(required_air)
            .sqrt()
            .clamp_max(1.0_f32);
        let final_air_eff = air_content_pct
            .clone()
            .zeros()
            .mask_fill(adequate_air_mask.clone(), 1.0_f32)
            .add(air_effectiveness.mask_fill(adequate_air_mask.bool_not(), 0.0_f32));

        // Spacing effectiveness: L <= 0.2 is ideal (1.0). Drops rapidly after.
        let good_spacing = spacing_factor.clone().lower_equal_elem(0.2_f32);
        let mid_spacing = spacing_factor
            .clone()
            .greater_elem(0.2_f32)
            .bool_and(spacing_factor.clone().lower_equal_elem(0.4_f32));

        let eff_mid = spacing_factor
            .clone()
            .mul_scalar(-1.0_f32)
            .add_scalar(0.2_f32)
            .mul_scalar(-0.5_f32)
            .add_scalar(0.9_f32);
        let eff_poor = spacing_factor
            .clone()
            .mul_scalar(-1.0_f32)
            .add_scalar(0.4_f32)
            .clamp_max(0.5_f32)
            .mul_scalar(-0.6_f32)
            .add_scalar(0.7_f32)
            .clamp_min(0.0_f32);

        let spacing_eff = spacing_factor
            .clone()
            .zeros()
            .mask_fill(good_spacing, 1.0_f32)
            .add(
                spacing_factor
                    .clone()
                    .zeros()
                    .mask_fill(mid_spacing, 1.0_f32)
                    .mul(eff_mid),
            )
            .add(
                spacing_factor
                    .clone()
                    .zeros()
                    .mask_fill(spacing_factor.greater_elem(0.4_f32), 1.0_f32)
                    .mul(eff_poor),
            );

        let durability_factor = final_air_eff.mul(spacing_eff).mul_scalar(100.0_f32);

        (spacing_factor, durability_factor)
    }
}
