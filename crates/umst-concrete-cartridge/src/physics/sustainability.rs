// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
use burn::tensor::{backend::Backend, Tensor};

/// Pure tensor implementation of the Sustainability & Economics Engine.
/// Computes Embodied Carbon (CO2e) and Financial Cost across the manifold.
/// This acts as the critical negative-reward penalty for the optimization engine's topology exploration.
/// formal_anchor: literature://EN-15804+A2-GWP-and-unit-costs
/// formal_status: Literature
/// formal_axioms: NONE
/// formal_citation: "EN 15804+A2 (2019) cradle-to-gate / modules A2 — indicative EPD-style CO₂e intensities; financial row uses linear $/kg mass factors"
/// formal_form: "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3)"
pub struct SustainabilityEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend> SustainabilityEngine<B> {
    /// Computes the total embodied carbon (kg CO2e) per voxel.
    ///
    /// Total CO2 = sum(mass_i * co2_factor_i)
    ///
    /// # Arguments
    /// * `mass_cement` - Tensor [Batch, Depth, Height, Width]
    /// * `mass_scm` - Tensor [Batch, Depth, Height, Width]
    /// * `mass_aggregate` - Tensor [Batch, Depth, Height, Width]
    /// * `mass_water` - Tensor [Batch, Depth, Height, Width]
    /// * `co2_factors` - Tuple (cement_f, scm_f, agg_f, water_f) as scalars
    /// formal_anchor: literature://EN-15804+A2-GWP-and-unit-costs
    /// formal_status: Literature
    /// formal_axioms: NONE
    /// formal_citation: "EN 15804+A2 (2019) cradle-to-gate / modules A2 — indicative EPD-style CO₂e intensities; financial row uses linear $/kg mass factors"
    /// formal_form: "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3)"
    pub fn compute_embodied_carbon(
        mass_cement: Tensor<B, 4>,
        mass_scm: Tensor<B, 4>,
        mass_aggregate: Tensor<B, 4>,
        mass_water: Tensor<B, 4>,
        co2_factors: (f32, f32, f32, f32),
    ) -> Tensor<B, 4> {
        let (f_cem, f_scm, f_agg, f_wat) = co2_factors;

        let co2_cem = mass_cement.mul_scalar(f_cem);
        let co2_scm = mass_scm.mul_scalar(f_scm);
        let co2_agg = mass_aggregate.mul_scalar(f_agg);
        let co2_wat = mass_water.mul_scalar(f_wat);

        co2_cem.add(co2_scm).add(co2_agg).add(co2_wat)
    }

    /// Computes the total financial cost ($/m3) per voxel.
    /// formal_anchor: literature://EN-15804+A2-GWP-and-unit-costs
    /// formal_status: Literature
    /// formal_axioms: NONE
    /// formal_citation: "EN 15804+A2 (2019) cradle-to-gate / modules A2 — indicative EPD-style CO₂e intensities; financial row uses linear $/kg mass factors"
    /// formal_form: "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3)"
    pub fn compute_financial_cost(
        mass_cement: Tensor<B, 4>,
        mass_scm: Tensor<B, 4>,
        mass_aggregate: Tensor<B, 4>,
        mass_water: Tensor<B, 4>,
        cost_factors: (f32, f32, f32, f32),
    ) -> Tensor<B, 4> {
        let (c_cem, c_scm, c_agg, c_wat) = cost_factors;

        let cost_cem = mass_cement.mul_scalar(c_cem);
        let cost_scm = mass_scm.mul_scalar(c_scm);
        let cost_agg = mass_aggregate.mul_scalar(c_agg);
        let cost_wat = mass_water.mul_scalar(c_wat);

        cost_cem.add(cost_scm).add(cost_agg).add(cost_wat)
    }
}
