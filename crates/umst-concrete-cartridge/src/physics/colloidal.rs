// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

use burn::tensor::{backend::Backend, Tensor};

use crate::chem_adapter::{
    dlvo_boltzmann_f32, dlvo_collapse_separation_f32, dlvo_debye_prefactor_f32,
    dlvo_dielectric_water_f32, dlvo_hamaker_f32, dlvo_reference_temperature_f32,
    dlvo_tensor_collapse_sentinel_kt_f32, dlvo_tensor_ionic_floor_m_f32, dlvo_tensor_mv_to_v_f32,
    dlvo_tensor_sep_floor_nm_f32, dlvo_vacuum_permittivity_f32, flocculation_barrier_kt_f32,
    flocculation_multiplier_base_f32, flocculation_multiplier_max_f32,
    flocculation_yield_stress_slope_f32,
};

/// Pure tensor implementation of the Colloidal Engine.
/// Computes DLVO (Derjaguin, Landau, Verwey, Overbeek) theory
/// to determine particle flocculation, dispersion, and structural stability.
/// formal_anchor: empirical://datasets/dataset_d1.csv
/// formal_status: Empirical
/// formal_axioms: NONE
/// formal_dataset: "uci_concrete_yeh_1998"
/// formal_citation: "Flatt & Bowen (2007) J. Am. Ceram. Soc. 89, 1244 (YODEL)"
/// formal_envelope: "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); DLVO pathway exercised under tests/realism/adversarial_physics.rs"
pub struct ColloidalEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend> ColloidalEngine<B> {
    /// Computes the interaction potential energy (kT) between particles.
    /// This directly influences yield stress and early-age stiffening.
    ///
    /// # Arguments
    /// * `separation_nm` - Average particle separation distance (nm)
    /// * `zeta_potential_mv` - Zeta potential of particles in suspension (mV)
    /// * `ionic_strength_m` - Molar ionic strength of the pore solution (M)
    /// formal_anchor: empirical://datasets/dataset_d1.csv
    /// formal_status: Empirical
    /// formal_axioms: NONE
    /// formal_dataset: "uci_concrete_yeh_1998"
    /// formal_citation: "Flatt & Bowen (2007) J. Am. Ceram. Soc. 89, 1244 (YODEL)"
    /// formal_envelope: "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); DLVO pathway exercised under tests/realism/adversarial_physics.rs"
    pub fn compute_dlvo_potential(
        separation_nm: Tensor<B, 4>,
        zeta_potential_mv: Tensor<B, 4>,
        ionic_strength_m: Tensor<B, 4>,
    ) -> Tensor<B, 4> {
        let hamaker = dlvo_hamaker_f32();
        let epsilon = dlvo_dielectric_water_f32() * dlvo_vacuum_permittivity_f32();
        let kt_joules = dlvo_boltzmann_f32() * dlvo_reference_temperature_f32();

        let safe_separation = separation_nm
            .clone()
            .clamp_min(dlvo_tensor_sep_floor_nm_f32());
        let sep_m = safe_separation.clone().mul_scalar(1e-9_f32);

        // 1. Van der Waals Attraction (simplified sphere-plate or close range flat plates)
        // V_A = -A / (12 * h)
        let v_vdw = sep_m.powf_scalar(-1.0_f32).mul_scalar(-hamaker / 12.0_f32);

        // 2. Electrostatic Repulsion (Double Layer)
        // Debye length: kappa^-1 (nm) approx 0.304 / sqrt(I)
        let safe_ionic = ionic_strength_m.clamp_min(dlvo_tensor_ionic_floor_m_f32());
        let debye_len_nm = safe_ionic
            .sqrt()
            .powf_scalar(-1.0_f32)
            .mul_scalar(dlvo_debye_prefactor_f32());

        let zeta_v = zeta_potential_mv.div_scalar(dlvo_tensor_mv_to_v_f32());
        let zeta_sq = zeta_v.clone().mul(zeta_v);

        // Decay term: exp(-h / debye_len)
        let decay_exponent = separation_nm.clone().div(debye_len_nm).mul_scalar(-1.0_f32);
        let decay = decay_exponent.exp();

        // Repulsion magnitude approx: V_R = eps * zeta^2 * exp(-kappa * h)
        let v_repulsion = zeta_sq.mul(decay).mul_scalar(epsilon);

        // 3. Total Potential Energy in kT
        let total_joules = v_vdw.add(v_repulsion);
        let total_kt = total_joules.div_scalar(kt_joules);

        // Mask out physical anomalies
        let collapse_mask = separation_nm.lower_elem(dlvo_collapse_separation_f32());
        let mut final_kt = total_kt;
        // Collapse sentinel routed through cartridge tensor witness (not umst-chem SSOT).
        final_kt = final_kt.mask_fill(collapse_mask, dlvo_tensor_collapse_sentinel_kt_f32());

        final_kt
    }

    /// Translates DLVO potential into a flocculation multiplier for Rheology yield stress.
    /// Highly negative potential = strong flocculation = higher yield stress.
    /// formal_anchor: empirical://datasets/dataset_d1.csv
    /// formal_status: Empirical
    /// formal_axioms: NONE
    /// formal_dataset: "uci_concrete_yeh_1998"
    /// formal_citation: "Flatt & Bowen (2007) J. Am. Ceram. Soc. 89, 1244 (YODEL)"
    /// formal_envelope: "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); DLVO pathway exercised under tests/realism/adversarial_physics.rs"
    pub fn compute_flocculation_multiplier(dlvo_potential_kt: Tensor<B, 4>) -> Tensor<B, 4> {
        // Barrier + clamp policy routed through cartridge witnesses (not umst-chem SSOT).
        let negative_barrier = dlvo_potential_kt
            .clone()
            .clamp_max(flocculation_barrier_kt_f32());
        negative_barrier
            .mul_scalar(flocculation_yield_stress_slope_f32())
            .add_scalar(flocculation_multiplier_base_f32())
            .clamp(
                flocculation_multiplier_base_f32(),
                flocculation_multiplier_max_f32(),
            )
    }
}
