// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

use burn::tensor::{backend::Backend, Tensor};

/// Pure tensor implementation of the Colloidal Engine.
/// Computes DLVO (Derjaguin, Landau, Verwey, Overbeek) theory
/// to determine particle flocculation, dispersion, and structural stability.
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
    pub fn compute_dlvo_potential(
        separation_nm: Tensor<B, 4>,
        zeta_potential_mv: Tensor<B, 4>,
        ionic_strength_m: Tensor<B, 4>,
    ) -> Tensor<B, 4> {
        let hamaker = 2.0e-20_f32; // J (Cement-Water-Cement approx)
        let epsilon = 78.5_f32 * 8.854e-12_f32; // Dielectric of water
        let k_b = 1.38e-23_f32;
        let temp = 298.0_f32; // K
        let kt_joules = k_b * temp;

        let safe_separation = separation_nm.clone().clamp_min(0.1_f32);
        let sep_m = safe_separation.clone().mul_scalar(1e-9_f32);

        // 1. Van der Waals Attraction (simplified sphere-plate or close range flat plates)
        // V_A = -A / (12 * h)
        let v_vdw = sep_m.powf_scalar(-1.0_f32).mul_scalar(-hamaker / 12.0_f32);

        // 2. Electrostatic Repulsion (Double Layer)
        // Debye length: kappa^-1 (nm) approx 0.304 / sqrt(I)
        let safe_ionic = ionic_strength_m.clamp_min(0.001_f32);
        let debye_len_nm = safe_ionic
            .sqrt()
            .powf_scalar(-1.0_f32)
            .mul_scalar(0.304_f32);

        let zeta_v = zeta_potential_mv.div_scalar(1000.0_f32);
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
        let collapse_mask = separation_nm.lower_elem(0.11_f32);
        let mut final_kt = total_kt;
        // If separation < 0.1, particles are aggregated, immense negative potential
        final_kt = final_kt.mask_fill(collapse_mask, -999.0_f32);

        final_kt
    }

    /// Translates DLVO potential into a flocculation multiplier for Rheology yield stress.
    /// Highly negative potential = strong flocculation = higher yield stress.
    pub fn compute_flocculation_multiplier(dlvo_potential_kt: Tensor<B, 4>) -> Tensor<B, 4> {
        // If potential < -5 kT, flocculation increases yield stress
        // multiplier ranges from 1.0 (stable) to ~3.0 (highly flocculated)
        let negative_barrier = dlvo_potential_kt.clone().clamp_max(-5.0_f32);
        negative_barrier
            .mul_scalar(-0.1_f32)
            .add_scalar(1.0_f32)
            .clamp(1.0_f32, 5.0_f32)
    }
}
