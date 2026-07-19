// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

use burn::tensor::{backend::Backend, Tensor};

use crate::chem_adapter::{
    chemo_diffusion_weight_scale_f32, critical_wc_f32, desiccation_rh_drop_scale_f32,
    kelvin_capillary_scale_mpa_f32,
};

/// Pure tensor implementation of the Chemo-Mechanical Water Transport Engine.
/// Computes moisture diffusion and capillary tension gradients inside the pore network.
/// formal_anchor: lean://umst-formal/Lean/Concrete/Powers.lean#PowersState
/// catalog_id: thermodynamic_mix
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
pub struct ChemoWaterEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend> ChemoWaterEngine<B> {
    /// Computes the internal Relative Humidity profile and resulting capillary tension.
    /// This causes drying shrinkage and limits late-stage hydration.
    /// formal_anchor: lean://umst-formal/Lean/Concrete/Powers.lean#PowersState
    /// catalog_id: thermodynamic_mix
    /// formal_status: Mechanised
    /// formal_axioms: physicalSecondLaw
    pub fn compute_moisture_transport(
        wc_ratio: Tensor<B, 4>,
        degree_hydration: Tensor<B, 4>,
        ambient_rh: Tensor<B, 4>,
        porosity: Tensor<B, 4>,
    ) -> (Tensor<B, 4>, Tensor<B, 4>) {
        // 1. Desorption Isotherm (Kelvin-Laplace approximation)
        // Internal RH is bounded by ambient RH over time, but controlled by self-desiccation.
        let critical_wc = critical_wc_f32();
        let desiccation_potential = wc_ratio
            .clone()
            .powf_scalar(-1.0_f32)
            .mul_scalar(critical_wc)
            .mul_scalar(-1.0_f32)
            .add_scalar(1.0_f32)
            .clamp_min(0.0_f32)
            .sqrt();
        let internal_rh_drop = degree_hydration
            .mul(desiccation_potential)
            .mul_scalar(desiccation_rh_drop_scale_f32());

        let mut internal_rh = internal_rh_drop.mul_scalar(-1.0_f32).add_scalar(1.0_f32);

        // As time -> inf, internal RH converges to ambient RH, weighted by porosity.
        let diffusion_weight = porosity
            .mul_scalar(chemo_diffusion_weight_scale_f32())
            .clamp_max(1.0_f32);
        internal_rh = internal_rh
            .clone()
            .mul(
                diffusion_weight
                    .clone()
                    .mul_scalar(-1.0_f32)
                    .add_scalar(1.0_f32),
            )
            .add(ambient_rh.mul(diffusion_weight));

        // 2. Capillary Tension (MPa)
        // P_cap = - (R * T / V_m) * ln(RH)
        // At 293K, (R*T/V_m) approx 135 MPa
        let capillary_tension = internal_rh
            .clone()
            .clamp_min(0.1_f32)
            .log()
            .mul_scalar(-kelvin_capillary_scale_mpa_f32());

        (internal_rh, capillary_tension)
    }
}
