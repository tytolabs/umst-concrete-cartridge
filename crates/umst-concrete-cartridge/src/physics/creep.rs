// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! # S6_RETIRE — Tier A.2 Burn tensor path (card `g_spawn_i_creep_s6_2054`)
//!
//! B2 scalar SSOT: `umst-cartridge-solid-inelastic::try_creep_compliance` (`g_spawn_i_creep_1947`).
//! Inventory: `s6_creep_tensor_inventory.rs`. **Delete this module** after orchestrator reroute
//! (`pipeline/orchestrator.rs` L479) delegates exclusively to B2 scalar.

use burn::tensor::{backend::Backend, Tensor};

use crate::burn_compat::bool_and;

/// Pure tensor implementation of the Creep Engine.
/// Computes basic and drying creep compliance (Extended Microprestress Solidification theory / fib Model Code 2010).
/// formal_anchor: empirical://datasets/dataset_d1.csv
/// formal_status: Empirical
/// formal_axioms: NONE
/// formal_dataset: "uci_concrete_yeh_1998"
/// formal_citation: "Bažant et al. (2015) Mater. Struct. 48, 753 (RILEM B4)"
/// formal_envelope: "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); RILEM B4 creep pathway exercised under tests/creep.rs + adversarial harness"
pub struct CreepEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend> CreepEngine<B> {
    /// Computes the total creep compliance (elastic + basic + drying creep).
    /// Used by the orchestrator to evaluate long-term viscoelastic stability.
    ///
    /// # Arguments
    /// * `compressive_strength` - 28-day compressive strength (MPa)
    /// * `wc_ratio` - Water/Cement ratio
    /// * `ambient_rh` - Relative humidity (0.0 to 1.0)
    /// * `t_load_days` - Age at loading (days)
    /// * `t_current_days` - Current age (days)
    /// formal_anchor: empirical://datasets/dataset_d1.csv
    /// formal_status: Empirical
    /// formal_axioms: NONE
    /// formal_dataset: "uci_concrete_yeh_1998"
    /// formal_citation: "Bažant et al. (2015) Mater. Struct. 48, 753 (RILEM B4)"
    /// formal_envelope: "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); RILEM B4 creep pathway exercised under tests/creep.rs + adversarial harness"
    pub fn compute_compliance(
        compressive_strength: Tensor<B, 4>,
        wc_ratio: Tensor<B, 4>,
        ambient_rh: Tensor<B, 4>,
        t_load_days: f32,
        t_current_days: f32,
    ) -> Tensor<B, 4> {
        let duration = (t_current_days - t_load_days).max(0.1_f32);

        // 1. Elastic Modulus
        let fc_safe = compressive_strength.clone().clamp_min(1.0_f32);
        let e_28 = fc_safe
            .clone()
            .div_scalar(10.0_f32)
            .powf_scalar(0.3_f32)
            .mul_scalar(22.0_f32); // GPa

        // β_cc load factor
        let high_strength_mask = fc_safe.clone().greater_elem(50.0_f32);
        let mid_strength_mask = bool_and(
            fc_safe.clone().lower_equal_elem(50.0_f32),
            fc_safe.clone().greater_elem(35.0_f32),
        );

        let mut s_factor = fc_safe.clone().zeros_like().add_scalar(0.38_f32);
        s_factor = s_factor
            .mask_fill(high_strength_mask, 0.20_f32)
            .mask_fill(mid_strength_mask, 0.25_f32);

        let t_ratio = (28.0_f32 / t_load_days).sqrt();
        let beta_cc_load = s_factor.mul_scalar(1.0_f32 - t_ratio).exp();
        let e_load = e_28.clone().mul(beta_cc_load.sqrt());

        let elastic_compliance = e_load.powf_scalar(-1.0_f32);

        // 2. Aging Factor
        let aging_factor = 1.0_f32 / (0.1_f32 + t_load_days.powf(0.2_f32));

        // 3. Basic Creep Compliance
        let lambda_0 = 10.0_f32;
        let c0_base = wc_ratio.clone().mul_scalar(0.40_f32).add_scalar(0.30_f32);
        let strength_factor = fc_safe.powf_scalar(-1.0_f32).mul_scalar(40.0_f32).sqrt();

        let c0 = c0_base
            .mul(strength_factor.clone())
            .mul_scalar(aging_factor);
        let time_func = (1.0_f32 + duration / lambda_0).ln();
        let basic_creep = c0.mul_scalar(time_func).div(e_28.clone());

        // 4. Drying Creep Compliance (Pickett Effect)
        let rh_effect = ambient_rh
            .clone()
            .mul_scalar(-1.0_f32)
            .add_scalar(1.0_f32)
            .clamp_min(0.0_f32)
            .powf_scalar(1.5_f32);
        let drying_time_factor = (duration / (duration + 100.0_f32)).min(1.0_f32);

        let cd = wc_ratio
            .div_scalar(0.45_f32)
            .powf_scalar(1.5_f32)
            .mul(strength_factor)
            .mul_scalar(0.15_f32);
        let drying_creep = cd.mul(rh_effect).mul_scalar(drying_time_factor).div(e_28);

        // 5. Total Compliance
        elastic_compliance.add(basic_creep).add(drying_creep)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::tensor::{Data, Shape, Tensor};
    use burn_ndarray::{NdArray, NdArrayDevice};

    type B = NdArray<f32>;

    /// Orchestrator creep pin — matches `pipeline/mechanics_delegate` call site.
    /// Class: **Primitive-fact** (routing contract, not fitted from compliance output).
    const PIN_FC_MPA: f32 = 40.0;
    const PIN_WC: f32 = 0.45;
    const PIN_AMBIENT_RH: f32 = 0.55;
    const PIN_T_LOAD_DAYS: f32 = 7.0;
    const PIN_T_CURRENT_DAYS: f32 = 28.0;

    /// Measured golden compliance [1/GPa] at orchestrator pin (Burn tensor path).
    /// Class: **Measured** — witness 2026-07-21 AC13 · `cargo test --lib creep_engine`.
    const GOLDEN_COMPLIANCE_1_OVER_GPA: f32 = 0.044_552_77_f32;

    fn scalar_rank4(v: f32) -> Tensor<B, 4> {
        let dev = NdArrayDevice::default();
        Tensor::from_data(Data::new(vec![v], Shape::new([1, 1, 1, 1])), &dev)
    }

    fn compliance_at_pin(
        fc_mpa: f32,
        wc: f32,
        rh: f32,
        t_load_days: f32,
        t_current_days: f32,
    ) -> f32 {
        let compliance = CreepEngine::<B>::compute_compliance(
            scalar_rank4(fc_mpa),
            scalar_rank4(wc),
            scalar_rank4(rh),
            t_load_days,
            t_current_days,
        );
        compliance.into_data().value[0]
    }

    /// Monolith golden vector — pins Burn tensor path at orchestrator mix contract.
    #[test]
    fn creep_engine_measured_golden_vector_at_orchestrator_pin() {
        let measured = compliance_at_pin(
            PIN_FC_MPA,
            PIN_WC,
            PIN_AMBIENT_RH,
            PIN_T_LOAD_DAYS,
            PIN_T_CURRENT_DAYS,
        );
        assert!(
            measured.is_finite() && measured > 0.0,
            "orchestrator-pin compliance must be finite and positive; got {measured}"
        );
        let rel_err =
            (measured - GOLDEN_COMPLIANCE_1_OVER_GPA).abs() / GOLDEN_COMPLIANCE_1_OVER_GPA;
        assert!(
            rel_err < 1e-5,
            "creep golden drift: measured={measured} golden={GOLDEN_COMPLIANCE_1_OVER_GPA} rel_err={rel_err}"
        );
    }

    /// Admissibility: compliance non-decreases with elapsed time at fixed pin.
    #[test]
    fn creep_engine_compliance_monotone_in_duration() {
        let early = compliance_at_pin(
            PIN_FC_MPA,
            PIN_WC,
            PIN_AMBIENT_RH,
            PIN_T_LOAD_DAYS,
            PIN_T_LOAD_DAYS + 1.0,
        );
        let late = compliance_at_pin(
            PIN_FC_MPA,
            PIN_WC,
            PIN_AMBIENT_RH,
            PIN_T_LOAD_DAYS,
            PIN_T_CURRENT_DAYS,
        );
        assert!(
            late >= early,
            "creep compliance must not decrease with time: early={early} late={late}"
        );
    }

    /// Drying term: lower RH ⇒ higher compliance (Pickett pathway exercised).
    #[test]
    fn creep_engine_compliance_increases_as_rh_decreases() {
        let humid = compliance_at_pin(
            PIN_FC_MPA,
            PIN_WC,
            0.90,
            PIN_T_LOAD_DAYS,
            PIN_T_CURRENT_DAYS,
        );
        let dry = compliance_at_pin(
            PIN_FC_MPA,
            PIN_WC,
            0.40,
            PIN_T_LOAD_DAYS,
            PIN_T_CURRENT_DAYS,
        );
        assert!(
            dry > humid,
            "drying creep must rise as RH falls: humid={humid} dry={dry}"
        );
    }
}
