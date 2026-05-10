// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

use burn::tensor::{backend::Backend, Tensor};

/// Pure tensor implementation of the Rheology Engine.
/// Computes yield stress and plastic viscosity using YODEL and Chateau-Ovarlez models.
/// formal_anchor: empirical://datasets/dataset_d1.csv
/// formal_status: Empirical
/// formal_axioms: NONE
/// formal_dataset: "uci_concrete_yeh_1998"
/// formal_citation: "Yeh (1998), UCI ML Repository, doi:10.24432/C5PK67"
/// formal_envelope: "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel/YODEL pathway exercised under tests/rheology.rs + adversarial harness"
pub struct RheologyEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend> RheologyEngine<B> {
    /// Computes relative viscosity using the Chateau-Ovarlez model.
    ///
    /// η_r = (1 - phi / phi_m)^(-[η] * phi_m)
    ///
    /// # Arguments
    /// * `solid_fraction` (phi) - Volume fraction of solid particles [Batch, Depth, Height, Width]
    /// * `max_packing` (phi_m) - Maximum packing fraction [Batch, Depth, Height, Width]
    /// * `intrinsic_viscosity` ([η]) - Shape factor (2.5 for spheres) [Batch, Depth, Height, Width]
    /// * `fluid_viscosity` - Viscosity of the suspending fluid [Batch, Depth, Height, Width]
    /// formal_anchor: empirical://datasets/dataset_d1.csv
    /// formal_status: Empirical
    /// formal_axioms: NONE
    /// formal_dataset: "uci_concrete_yeh_1998"
    /// formal_citation: "Yeh (1998), UCI ML Repository, doi:10.24432/C5PK67"
    /// formal_envelope: "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel/YODEL pathway exercised under tests/rheology.rs + adversarial harness"
    pub fn compute_chateau_ovarlez(
        solid_fraction: Tensor<B, 4>,
        max_packing: Tensor<B, 4>,
        intrinsic_viscosity: Tensor<B, 4>,
        fluid_viscosity: Tensor<B, 4>,
    ) -> Tensor<B, 4> {
        // Guard against division by zero and unphysical packing
        // phi_m must be > 0.01
        let valid_pack_mask = max_packing.clone().greater_elem(0.01_f32);
        let safe_pack = max_packing
            .clone()
            .mask_fill(valid_pack_mask.clone().bool_not(), 1.0_f32);

        // phi / phi_m
        let phi_ratio = solid_fraction.div(safe_pack.clone());

        // (1 - phi/phi_m) -> clamp to minimum 0.001 to prevent negative bases
        let one_minus_ratio = phi_ratio
            .mul_scalar(-1.0_f32)
            .add_scalar(1.0_f32)
            .clamp_min(0.001_f32);

        // Exponent: -[η] * phi_m
        let exponent = intrinsic_viscosity.mul(safe_pack).mul_scalar(-1.0_f32);

        // Relative viscosity
        let rel_visc = one_minus_ratio.powf(exponent);

        // Final viscosity = rel_visc * fluid_viscosity
        let final_visc = rel_visc.mul(fluid_viscosity);

        // Apply validity mask
        final_visc.mask_fill(valid_pack_mask.bool_not(), 0.0_f32)
    }

    /// Computes yield stress using the YODEL (Yield Stress Model for Suspensions).
    ///
    /// tau_y = m1 * (phi^2 * f_sigma) / (d50 * (phi_m - phi))
    ///
    /// # Arguments
    /// * `solid_fraction` (phi)
    /// * `max_packing` (phi_m)
    /// * `particle_size_d50` - Median particle size
    /// * `interparticle_force` (f_sigma)
    /// formal_anchor: empirical://datasets/dataset_d1.csv
    /// formal_status: Empirical
    /// formal_axioms: NONE
    /// formal_dataset: "uci_concrete_yeh_1998"
    /// formal_citation: "Yeh (1998), UCI ML Repository, doi:10.24432/C5PK67"
    /// formal_envelope: "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel/YODEL pathway exercised under tests/rheology.rs + adversarial harness"
    pub fn compute_yield_stress_yodel(
        solid_fraction: Tensor<B, 4>,
        max_packing: Tensor<B, 4>,
        particle_size_d50: Tensor<B, 4>,
        interparticle_force: Tensor<B, 4>,
    ) -> Tensor<B, 4> {
        let m1 = 1.8_f32; // Percolation threshold parameter

        let phi_sq = solid_fraction.clone().powf_scalar(2.0);
        let numerator = phi_sq.mul(interparticle_force).mul_scalar(m1);

        let packing_diff = max_packing.sub(solid_fraction);
        let denominator = particle_size_d50.clone().mul(packing_diff);

        // Guard against denominator <= 0 (physical jamming or size=0)
        let valid_den_mask = denominator.clone().greater_elem(1e-6_f32);
        let safe_den = denominator.mask_fill(valid_den_mask.clone().bool_not(), 1.0_f32);

        let tau_y = numerator.div(safe_den);

        tau_y.mask_fill(valid_den_mask.bool_not(), 0.0_f32)
    }
}
