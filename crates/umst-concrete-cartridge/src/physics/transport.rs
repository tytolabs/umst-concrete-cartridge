// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

use burn::tensor::{backend::Backend, Tensor};

use crate::chem_adapter::{
    powers_non_evap_water_coeff_f32, powers_paste_denominator_offset_f32,
};

/// Pure tensor implementation of the Transport Engine.
/// Models capillary porosity, tortuosity, and chloride diffusivity
/// across the material manifold.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Tensor facade grouping porosity and chloride diffusivity kernels documented on methods.
pub struct TransportEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend> TransportEngine<B> {
    /// Computes Capillary Porosity based on water/cement ratio and hydration degree.
    ///
    /// phi_c = (w/c - 0.36 * alpha) / (w/c + 0.32)
    ///
    /// # Arguments
    /// * `wc_ratio` - Water/Cement ratio tensor [Batch, Depth, Height, Width]
    /// * `degree_hydration` (alpha) - Degree of hydration [Batch, Depth, Height, Width]
    /// formal_anchor: lean://umst-formal/Lean/Concrete/Powers.lean#PowersState
    /// catalog_id: thermodynamic_mix
    /// formal_status: Mechanised
    /// formal_axioms: NONE
    pub fn compute_capillary_porosity(
        wc_ratio: Tensor<B, 4>,
        degree_hydration: Tensor<B, 4>,
    ) -> Tensor<B, 4> {
        let alpha_036 = degree_hydration.mul_scalar(powers_non_evap_water_coeff_f32());
        let numerator = wc_ratio.clone().sub(alpha_036);

        let denominator = wc_ratio.add_scalar(powers_paste_denominator_offset_f32());

        // Denominator is always positive for valid W/C, but clamp for safety
        let safe_den = denominator.clamp_min(0.01_f32);

        let porosity = numerator.div(safe_den);

        // Porosity cannot be negative
        porosity.clamp_min(0.0_f32)
    }

    /// Computes apparent chloride diffusivity using the empirical Life-365 / Nernst-Planck model.
    ///
    /// D = D_ref * (phi_c)^n
    ///
    /// # Arguments
    /// * `capillary_porosity` - Computed porosity tensor
    /// * `ref_diffusivity` - Reference diffusivity scalar tensor
    /// formal_anchor: lean://umst-formal/Lean/MeasurementCost.lean#zero_info_zero_energy
    /// catalog_id: umst.gate.landauer_cbf
    /// formal_status: Mechanised
    /// formal_axioms: NONE
    pub fn compute_chloride_diffusivity(
        capillary_porosity: Tensor<B, 4>,
        ref_diffusivity: Tensor<B, 4>,
    ) -> Tensor<B, 4> {
        // Typical exponent for concrete diffusivity vs porosity is ~3.0 to 4.0
        let exponent = 3.5_f32;

        let pore_network = capillary_porosity.powf_scalar(exponent);

        pore_network.mul(ref_diffusivity)
    }
}
