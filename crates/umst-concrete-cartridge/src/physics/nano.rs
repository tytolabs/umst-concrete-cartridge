// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
use burn::tensor::{backend::Backend, Tensor};

use crate::chem_adapter::{nano_cartridge_calibration, nano_deferred_kinetics_pins};

/// Pure tensor implementation of the Nanomaterial Engine.
/// Computes nucleation seeding (C-S-H), pozzolanic acceleration, and pore refinement.
/// formal_anchor: empirical://datasets/csh-nano-calibration-grid.v1.csv
/// formal_status: Empirical
/// formal_dataset: "Pellenq et al. (2009) C-S-H grid calibration lineage"
/// formal_citation: "Pellenq et al. (2009) PNAS 106, 16102"
/// formal_envelope: "tests/realism/adversarial_physics.rs"
pub struct NanoEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend> NanoEngine<B> {
    /// Computes the enhancements provided by nanomaterial inclusions
    /// (e.g., nano-silica, graphene oxide, C-S-H seeds).
    ///
    /// # Arguments
    /// * `nano_dosage` - Dosage of nanomaterials as % of cement weight [Batch, Depth, Height, Width]
    /// * `nano_ssa` - Specific Surface Area of the nanomaterial in m2/g (e.g. 200 for nano-SiO2)
    /// * `nano_reactivity` - Empirical reactivity multiplier (1.0 for standard nano-silica)
    /// formal_anchor: empirical://datasets/csh-nano-calibration-grid.v1.csv
    /// formal_status: Empirical
    /// formal_dataset: "Pellenq et al. (2009) C-S-H grid calibration lineage"
    /// formal_citation: "Pellenq et al. (2009) PNAS 106, 16102"
    /// formal_envelope: "tests/realism/adversarial_physics.rs"
    pub fn compute_enhancements(
        nano_dosage: Tensor<B, 4>,
        nano_ssa: Tensor<B, 4>,
        nano_reactivity: Tensor<B, 4>,
    ) -> (Tensor<B, 4>, Tensor<B, 4>, Tensor<B, 4>, Tensor<B, 4>) {
        let pins = nano_deferred_kinetics_pins();
        let cal = nano_cartridge_calibration();
        let ssa_ref = pins.ssa_ref_m2_per_g;
        let ssa_ratio = nano_ssa.div_scalar(ssa_ref);

        // 1. Pozzolanic Activity Factor (Nazari & Riahi, 2011)
        let alpha = pins.pozzolanic_alpha;
        let ssa_ln = ssa_ratio.clone().log().clamp_min(0.0_f32);
        let pozzolanic_factor = ssa_ln.mul_scalar(alpha).exp().clamp(1.0_f32, 5.0_f32);

        // 2. Nucleation Seeding: Set Time Acceleration (Thomas et al., 2009)
        // dt = -beta * ln(1 + dosage * ssa_ratio)
        let beta = pins.nucleation_beta_min_per_decade;
        let dosage_ssa = nano_dosage.clone().mul(ssa_ratio.clone());
        let set_time_change = dosage_ssa.add_scalar(1.0_f32).log().mul_scalar(-beta);

        // 3. Strength Enhancement (Sanchez & Sobolev, 2010)
        // Parabolic efficiency curve: peaks at ~2.5% dosage, drops due to agglomeration
        let optimal_dosage = cal.optimal_dosage_pct;
        let dosage_diff = nano_dosage
            .clone()
            .sub_scalar(optimal_dosage)
            .div_scalar(optimal_dosage * 2.0_f32);
        let dosage_efficiency = dosage_diff
            .powf_scalar(2.0_f32)
            .mul_scalar(-1.0_f32)
            .add_scalar(1.0_f32)
            .clamp_min(0.1_f32);

        let gamma = cal.strength_gamma;
        let strength_boost = dosage_efficiency
            .clone()
            .mul(nano_reactivity)
            .mul(ssa_ratio.clone().sqrt())
            .mul_scalar(gamma);
        let strength_factor = strength_boost.add_scalar(1.0_f32).clamp(1.0_f32, 1.5_f32);

        // 4. Pore Refinement (Mondal et al., 2010)
        let delta = cal.pore_refinement_delta;
        let pore_reduction = nano_dosage
            .clone()
            .div_scalar(100.0_f32)
            .mul(dosage_efficiency)
            .mul_scalar(delta);
        let porosity_factor = pore_reduction
            .mul_scalar(-1.0_f32)
            .add_scalar(1.0_f32)
            .clamp(0.3_f32, 1.0_f32);

        // Mask out areas with 0 nano dosage to prevent baseline drifting
        let active_mask = nano_dosage.greater_elem(0.001_f32);

        let final_pozzolanic = pozzolanic_factor.mask_fill(active_mask.clone().bool_not(), 1.0_f32);
        let final_set_change = set_time_change.mask_fill(active_mask.clone().bool_not(), 0.0_f32);
        let final_strength = strength_factor.mask_fill(active_mask.clone().bool_not(), 1.0_f32);
        let final_porosity = porosity_factor.mask_fill(active_mask.bool_not(), 1.0_f32);

        (
            final_pozzolanic,
            final_set_change,
            final_strength,
            final_porosity,
        )
    }
}
