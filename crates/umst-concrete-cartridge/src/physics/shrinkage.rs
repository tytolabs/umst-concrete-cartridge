// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

use burn::tensor::{backend::Backend, Tensor};

use crate::burn_compat::bool_and;
use crate::chem_adapter::critical_wc_f32;

/// Cartridge-calibration drying time scale [days] — B4 / fib MC2010 order (~30–50 d).
/// Class: **Primitive-fact** (cartridge envelope; not lab-fitted).
const DRYING_TAU_DAYS: f32 = 35.0;
/// Drying onset age [days] — routes orchestrator `ORCHESTRATOR_T_LOAD_DAYS` pin.
/// Class: **Primitive-fact** (routing contract).
const DRYING_T0_DAYS: f32 = 7.0;
/// Ultimate drying scale [µε] at RH→0, w/c=0.45 — cartridge-calibration envelope.
/// Class: **Primitive-fact** (cartridge envelope; not lab-fitted).
const DRYING_EPS_ULT_SCALE: f32 = -450.0;

/// Honest residue — orchestrator shrinkage stage wires autogenous scalar only.
pub const SHRINKAGE_ORCHESTRATOR_WIRE: &str = "autogenous_only";
/// B2 `umst-cartridge-solid-inelastic` has no drying extract yet.
pub const SHRINKAGE_B2_DRYING_OPEN: bool = true;

/// RH deficit drive for drying ultimate strain — mirrors creep Pickett `rh_effect` exponent.
fn drying_rh_deficit<B: Backend>(ambient_rh: Tensor<B, 4>) -> Tensor<B, 4> {
    ambient_rh
        .mul_scalar(-1.0_f32)
        .add_scalar(1.0_f32)
        .clamp_min(0.0_f32)
        .powf_scalar(1.2_f32)
}

/// B4 / fib MC2010 tanh time development — Constitutive-Equations §D.4.
fn b4_tanh_development(age_days: f32, tau_days: f32) -> f32 {
    if age_days <= 0.0 {
        0.0_f32
    } else {
        (age_days / tau_days).sqrt().tanh().min(1.0_f32)
    }
}

/// Pure tensor implementation of the Shrinkage Engine.
/// Computes Autogenous and Drying shrinkage strain using fib Model Code 2010 / B4 model approximations.
/// formal_anchor: empirical://datasets/dataset_d1.csv
/// formal_status: Empirical
/// formal_axioms: NONE
/// formal_dataset: "uci_concrete_yeh_1998"
/// formal_citation: "Bažant et al. (2015) Mater. Struct. 48, 753 (B4 shrinkage model)"
/// formal_envelope: "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Bažant–Baweja shrinkage pathway exercised under tests/shrinkage.rs + adversarial harness"
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
    /// formal_anchor: empirical://datasets/dataset_d1.csv
    /// formal_status: Empirical
    /// formal_axioms: NONE
    /// formal_dataset: "uci_concrete_yeh_1998"
    /// formal_citation: "Bažant et al. (2015) Mater. Struct. 48, 753 (B4 shrinkage model)"
    /// formal_envelope: "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Bažant–Baweja shrinkage pathway exercised under tests/shrinkage.rs + adversarial harness"
    pub fn compute_autogenous_shrinkage(
        wc_ratio: Tensor<B, 4>,
        degree_hydration: Tensor<B, 4>,
        cement_content_kg: Tensor<B, 4>,
        scm_ratio: Tensor<B, 4>,
    ) -> Tensor<B, 4> {
        let critical_wc = critical_wc_f32();

        // 1. Ultimate shrinkage as a function of w/c (empirical B4 fit)
        // High shrinkage at low w/c, low shrinkage at high w/c
        let low_wc_mask = wc_ratio.clone().lower_equal_elem(0.30_f32);
        let mid_wc_mask = bool_and(
            wc_ratio.clone().lower_equal_elem(critical_wc),
            wc_ratio.clone().greater_elem(0.30_f32),
        );
        let high_wc_mask = bool_and(
            wc_ratio.clone().lower_equal_elem(0.50_f32),
            wc_ratio.clone().greater_elem(critical_wc),
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
            .add_scalar(critical_wc)
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

    /// Computes drying shrinkage strain [µε] from ambient RH and exposure age.
    ///
    /// Uses B4 tanh time law (Constitutive-Equations §D.4) with cartridge-calibration ultimate
    /// envelope coupled to RH deficit (Pickett pathway, consistent with `creep.rs`).
    ///
    /// # Arguments
    /// * `wc_ratio` - Water/cement ratio
    /// * `ambient_rh` - Ambient relative humidity (0.0 to 1.0)
    /// * `cement_content_kg` - Cement content [kg/m³]
    /// * `age_days` - Member age since casting [days]
    /// formal_anchor: empirical://datasets/dataset_d1.csv
    /// formal_status: Empirical
    /// formal_axioms: NONE
    /// formal_dataset: "uci_concrete_yeh_1998"
    /// formal_citation: "Bažant et al. (2015) Mater. Struct. 48, 753 (B4 shrinkage model)"
    /// formal_envelope: "Drying shrinkage tanh pathway exercised under shrinkage_engine tests; orchestrator wire OPEN"
    pub fn compute_drying_shrinkage(
        wc_ratio: Tensor<B, 4>,
        ambient_rh: Tensor<B, 4>,
        cement_content_kg: Tensor<B, 4>,
        age_days: f32,
    ) -> Tensor<B, 4> {
        let elapsed_days = (age_days - DRYING_T0_DAYS).max(0.0_f32);
        let beta_t = b4_tanh_development(elapsed_days, DRYING_TAU_DAYS);

        let rh_drive = drying_rh_deficit(ambient_rh);
        let wc_factor = wc_ratio.div_scalar(0.45_f32).powf_scalar(1.3_f32);
        let paste_factor = cement_content_kg.div_scalar(350.0_f32).sqrt();

        let eps_ds_ult = wc_factor
            .mul(paste_factor)
            .mul(rh_drive)
            .mul_scalar(DRYING_EPS_ULT_SCALE);

        eps_ds_ult.mul_scalar(beta_t)
    }

    /// Superposes autogenous and drying shrinkage [µε] — total strain for topology / durability gates.
    ///
    /// Orchestrator currently reports autogenous magnitude only; this total path is tensor-deepened
    /// for downstream consumers (`SHRINKAGE_ORCHESTRATOR_WIRE` residue).
    pub fn compute_total_shrinkage(
        wc_ratio: Tensor<B, 4>,
        degree_hydration: Tensor<B, 4>,
        cement_content_kg: Tensor<B, 4>,
        scm_ratio: Tensor<B, 4>,
        ambient_rh: Tensor<B, 4>,
        age_days: f32,
    ) -> Tensor<B, 4> {
        let autogenous = Self::compute_autogenous_shrinkage(
            wc_ratio.clone(),
            degree_hydration,
            cement_content_kg.clone(),
            scm_ratio,
        );
        let drying = Self::compute_drying_shrinkage(
            wc_ratio,
            ambient_rh,
            cement_content_kg,
            age_days,
        );
        autogenous.add(drying)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::tensor::{Data, Shape, Tensor};
    use burn_ndarray::{NdArray, NdArrayDevice};

    type B = NdArray<f32>;

    /// Orchestrator shrinkage pin — matches `pipeline/b2_orchestrator_delegate` call site.
    /// Class: **Primitive-fact** (routing contract, not fitted from shrinkage output).
    const PIN_WC: f32 = 0.45;
    const PIN_ALPHA: f32 = 0.7;
    const PIN_CEMENT_KG: f32 = 350.0;
    const PIN_SCM_RATIO: f32 = 0.1;
    const PIN_AMBIENT_RH: f32 = 0.55;
    const PIN_AGE_DAYS: f32 = 28.0;

    /// Measured golden autogenous shrinkage [µε] at orchestrator pin (Burn tensor path).
    /// Class: **Measured** — witness 2026-07-21 AC104 · `cargo test --lib shrinkage_engine`.
    const GOLDEN_AUTOGENOUS_MICROSTRAIN: f32 = -406.741_5_f32;

    /// Measured golden drying shrinkage [µε] at orchestrator pin (Burn tensor path).
    /// Class: **Measured** — witness 2026-07-21 AC104 · `cargo test --lib shrinkage_engine`.
    const GOLDEN_DRYING_MICROSTRAIN: f32 = -112.126_90_f32;

    fn scalar_rank4(v: f32) -> Tensor<B, 4> {
        let dev = NdArrayDevice::default();
        Tensor::from_data(Data::new(vec![v], Shape::new([1, 1, 1, 1])), &dev)
    }

    fn autogenous_at_pin(wc: f32, alpha: f32, cement_kg: f32, scm_ratio: f32) -> f32 {
        let strain = ShrinkageEngine::<B>::compute_autogenous_shrinkage(
            scalar_rank4(wc),
            scalar_rank4(alpha),
            scalar_rank4(cement_kg),
            scalar_rank4(scm_ratio),
        );
        strain.into_data().value[0]
    }

    fn drying_at_pin(wc: f32, rh: f32, cement_kg: f32, age_days: f32) -> f32 {
        let strain = ShrinkageEngine::<B>::compute_drying_shrinkage(
            scalar_rank4(wc),
            scalar_rank4(rh),
            scalar_rank4(cement_kg),
            age_days,
        );
        strain.into_data().value[0]
    }

    fn total_at_pin(
        wc: f32,
        alpha: f32,
        cement_kg: f32,
        scm_ratio: f32,
        rh: f32,
        age_days: f32,
    ) -> f32 {
        let strain = ShrinkageEngine::<B>::compute_total_shrinkage(
            scalar_rank4(wc),
            scalar_rank4(alpha),
            scalar_rank4(cement_kg),
            scalar_rank4(scm_ratio),
            scalar_rank4(rh),
            age_days,
        );
        strain.into_data().value[0]
    }

    /// Monolith golden vector — pins Burn tensor path at orchestrator mix contract.
    #[test]
    fn shrinkage_engine_measured_golden_vector_at_orchestrator_pin() {
        let measured = autogenous_at_pin(PIN_WC, PIN_ALPHA, PIN_CEMENT_KG, PIN_SCM_RATIO);
        assert!(
            measured.is_finite() && measured < 0.0,
            "orchestrator-pin autogenous shrinkage must be finite and negative; got {measured}"
        );
        let rel_err =
            (measured - GOLDEN_AUTOGENOUS_MICROSTRAIN).abs() / GOLDEN_AUTOGENOUS_MICROSTRAIN.abs();
        assert!(
            rel_err < 1e-5,
            "shrinkage golden drift: measured={measured} golden={GOLDEN_AUTOGENOUS_MICROSTRAIN} rel_err={rel_err}"
        );
    }

    /// Admissibility: shrinkage magnitude non-decreases with hydration at fixed pin.
    #[test]
    fn shrinkage_engine_magnitude_increases_with_hydration() {
        let early = autogenous_at_pin(PIN_WC, 0.3, PIN_CEMENT_KG, PIN_SCM_RATIO);
        let late = autogenous_at_pin(PIN_WC, PIN_ALPHA, PIN_CEMENT_KG, PIN_SCM_RATIO);
        assert!(
            late.abs() >= early.abs(),
            "autogenous shrinkage magnitude must rise with hydration: early={early} late={late}"
        );
    }

    /// B4 envelope: lower w/c ⇒ higher autogenous shrinkage magnitude (self-desiccation).
    #[test]
    fn shrinkage_engine_magnitude_increases_as_wc_decreases() {
        let high_wc = autogenous_at_pin(0.50, PIN_ALPHA, PIN_CEMENT_KG, PIN_SCM_RATIO);
        let low_wc = autogenous_at_pin(0.35, PIN_ALPHA, PIN_CEMENT_KG, PIN_SCM_RATIO);
        assert!(
            low_wc.abs() > high_wc.abs(),
            "B4 envelope must penalize low w/c harder: low_wc={low_wc} high_wc={high_wc}"
        );
    }

    /// Monolith golden vector — pins drying tanh pathway at orchestrator mix contract.
    #[test]
    fn shrinkage_engine_measured_golden_drying_at_orchestrator_pin() {
        let measured = drying_at_pin(PIN_WC, PIN_AMBIENT_RH, PIN_CEMENT_KG, PIN_AGE_DAYS);
        assert!(
            measured.is_finite() && measured < 0.0,
            "orchestrator-pin drying shrinkage must be finite and negative; got {measured}"
        );
        let rel_err =
            (measured - GOLDEN_DRYING_MICROSTRAIN).abs() / GOLDEN_DRYING_MICROSTRAIN.abs();
        assert!(
            rel_err < 1e-5,
            "drying golden drift: measured={measured} golden={GOLDEN_DRYING_MICROSTRAIN} rel_err={rel_err}"
        );
    }

    /// Admissibility: drying magnitude rises as ambient RH falls.
    #[test]
    fn shrinkage_engine_drying_magnitude_increases_as_rh_decreases() {
        let humid = drying_at_pin(PIN_WC, 0.90, PIN_CEMENT_KG, PIN_AGE_DAYS);
        let dry = drying_at_pin(PIN_WC, 0.40, PIN_CEMENT_KG, PIN_AGE_DAYS);
        assert!(
            dry.abs() > humid.abs(),
            "drying shrinkage must rise as RH falls: humid={humid} dry={dry}"
        );
    }

    /// Admissibility: drying magnitude non-decreases with exposure age.
    #[test]
    fn shrinkage_engine_drying_magnitude_increases_with_age() {
        let early = drying_at_pin(PIN_WC, PIN_AMBIENT_RH, PIN_CEMENT_KG, DRYING_T0_DAYS + 1.0);
        let late = drying_at_pin(PIN_WC, PIN_AMBIENT_RH, PIN_CEMENT_KG, PIN_AGE_DAYS);
        assert!(
            late.abs() >= early.abs(),
            "drying shrinkage magnitude must rise with age: early={early} late={late}"
        );
    }

    /// Total shrinkage superposition is algebraic sum of components at pin.
    #[test]
    fn shrinkage_engine_total_superposes_autogenous_and_drying() {
        let auto = autogenous_at_pin(PIN_WC, PIN_ALPHA, PIN_CEMENT_KG, PIN_SCM_RATIO);
        let dry = drying_at_pin(PIN_WC, PIN_AMBIENT_RH, PIN_CEMENT_KG, PIN_AGE_DAYS);
        let total = total_at_pin(
            PIN_WC,
            PIN_ALPHA,
            PIN_CEMENT_KG,
            PIN_SCM_RATIO,
            PIN_AMBIENT_RH,
            PIN_AGE_DAYS,
        );
        assert!(
            (total - (auto + dry)).abs() < 1e-4,
            "total shrinkage must superpose: total={total} auto+dry={}",
            auto + dry
        );
    }

    /// B2 scalar autogenous parity at orchestrator pin — tensor path vs `solid-inelastic`.
    #[test]
    fn shrinkage_engine_b2_autogenous_parity_at_orchestrator_pin() {
        use umst_cartridge_solid_inelastic::{
            try_autogenous_shrinkage_microstrain, AutogenousShrinkageInput,
        };

        let tensor = autogenous_at_pin(PIN_WC, PIN_ALPHA, PIN_CEMENT_KG, PIN_SCM_RATIO);
        let b2 = try_autogenous_shrinkage_microstrain(AutogenousShrinkageInput {
            wc_ratio: f64::from(PIN_WC),
            degree_hydration: f64::from(PIN_ALPHA),
            cement_content_kg: f64::from(PIN_CEMENT_KG),
            scm_ratio: f64::from(PIN_SCM_RATIO),
        })
        .expect("B2 shrinkage pin");

        let rel_err = (f64::from(tensor) - b2).abs() / b2.abs();
        assert!(
            rel_err < 1e-5,
            "B2/tensor autogenous parity drift: tensor={tensor} b2={b2} rel_err={rel_err}"
        );
    }

    /// Honest residue ledger — orchestrator and B2 drying wire stay open.
    #[test]
    fn shrinkage_engine_ac104_residue_ledger_honest() {
        assert_eq!(SHRINKAGE_ORCHESTRATOR_WIRE, "autogenous_only");
        assert!(SHRINKAGE_B2_DRYING_OPEN);
    }
}
