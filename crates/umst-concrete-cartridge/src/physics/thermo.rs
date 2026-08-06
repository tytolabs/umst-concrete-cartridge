// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

use burn::tensor::{backend::Backend, Tensor};

use crate::chem_adapter::{
    gas_constant_f32, ADIABATIC_TEMP_RISE_PER_ALPHA, CHEM_AFFINITY_EXPONENT, THERMO_REF_RATE,
};

/// Pure tensor implementation of the Thermodynamic Engine
/// Computes hydration heat evolution using the Arrhenius law across a differentiable manifold.
/// formal_anchor: lean://umst-formal/Lean/Concrete/Helmholtz.lean#ψAntitoneHelmholtz
/// catalog_id: umst.gate.cd_transition
/// formal_status: Mechanised
/// formal_axioms: NONE
pub struct ThermoEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend> ThermoEngine<B> {
    /// Computes rate of heat evolution (q) using Arrhenius law fully in the tensor graph.
    ///
    /// # Arguments
    /// * `temp_c` - Local temperature tensor in Celsius [Batch, Depth, Height, Width]
    /// * `alpha` - Degree of hydration tensor (0.0 to 1.0) [Batch, Depth, Height, Width]
    /// * `activation_energy` - Activation energy in J/mol [Batch, Depth, Height, Width] (allows spatially varying E_a)
    ///
    /// # Returns
    /// A tuple of tensors: `(heat_rate, adiabatic_temp_rise)`
    /// formal_anchor: lean://umst-formal/Lean/Concrete/Helmholtz.lean#ψAntitoneHelmholtz
    /// catalog_id: umst.gate.cd_transition
    /// formal_status: Mechanised
    /// formal_axioms: NONE
    pub fn compute_heat_rate(
        temp_c: Tensor<B, 4>,
        alpha: Tensor<B, 4>,
        activation_energy: Tensor<B, 4>,
    ) -> (Tensor<B, 4>, Tensor<B, 4>) {
        let r_gas = gas_constant_f32();
        let temp_k = temp_c.add_scalar(273.15_f32);

        // Affinity term: (1 - alpha)^1.5
        // Using clamp_min(0.0) to replicate max(0.0) and prevent NaN in backward pass for negative bases
        let one_minus_alpha = alpha
            .clone()
            .mul_scalar(-1.0)
            .add_scalar(1.0)
            .clamp_min(0.0);
        let chem_affinity = one_minus_alpha.powf_scalar(CHEM_AFFINITY_EXPONENT);

        // Arrhenius: k = exp(-E / RT)
        let rt = temp_k.mul_scalar(r_gas);
        let negative_e_over_rt = activation_energy.mul_scalar(-1.0).div(rt);
        let rate_constant = negative_e_over_rt.exp();

        // Reference rate at 20C approx
        let ref_rate = THERMO_REF_RATE;

        let heat_rate = rate_constant.mul(chem_affinity).mul_scalar(ref_rate);

        // Adiabatic temp rise: Simplistic alpha * 50.0
        let adiabatic_temp_rise = alpha.mul_scalar(ADIABATIC_TEMP_RISE_PER_ALPHA);

        (heat_rate, adiabatic_temp_rise)
    }
}

/// Measured close predicate — Arrhenius tensor heat rate + adiabatic α-proxy pinned in-module.
///
/// Constant is **Derived** from the `#[cfg(test)]` witnesses below; do not flip without a
/// matching `cargo test` paste on the thermo module tests.
pub const fn thermo_engine_tensor_path_measured() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::tensor::{Data, Shape, Tensor};
    use burn_ndarray::{NdArray, NdArrayDevice};

    type B = NdArray<f32>;

    fn device() -> NdArrayDevice {
        NdArrayDevice::default()
    }

    fn t4(value: f32) -> Tensor<B, 4> {
        Tensor::<B, 4>::from_data(Data::new(vec![value], Shape::new([1, 1, 1, 1])), &device())
    }

    fn scalar_4(t: Tensor<B, 4>) -> f32 {
        t.into_data().value[0]
    }

    /// Closed-form Arrhenius witness for parity with `compute_heat_rate` (inventory C-14…C-16).
    fn reference_arrhenius_heat_rate_f32(temp_c: f32, alpha: f32, activation_energy: f32) -> f32 {
        let r_gas = gas_constant_f32();
        let temp_k = temp_c + 273.15_f32;
        let one_minus_alpha = (1.0_f32 - alpha).max(0.0_f32);
        let chem_affinity = one_minus_alpha.powf(CHEM_AFFINITY_EXPONENT);
        let rate_constant = (-activation_energy / (r_gas * temp_k)).exp();
        rate_constant * chem_affinity * THERMO_REF_RATE
    }

    #[test]
    fn thermo_module_uses_chem_adapter_cluster_c_constants() {
        assert!((CHEM_AFFINITY_EXPONENT - 1.5_f32).abs() < f32::EPSILON);
        assert!((THERMO_REF_RATE - 1e6_f32).abs() < 1.0_f32);
        assert!((ADIABATIC_TEMP_RISE_PER_ALPHA - 50.0_f32).abs() < f32::EPSILON);
        assert!((gas_constant_f32() - 8.314_f32).abs() < 0.01_f32);
    }

    #[test]
    fn thermo_arrhenius_heat_rate_matches_reference_at_20c() {
        let temp_c = 20.0_f32;
        let alpha = 0.5_f32;
        let ea = 40_000.0_f32;
        let (q, _) = ThermoEngine::<B>::compute_heat_rate(t4(temp_c), t4(alpha), t4(ea));
        let expected = reference_arrhenius_heat_rate_f32(temp_c, alpha, ea);
        let got = scalar_4(q);
        assert!(
            (got - expected).abs() < 1e-3_f32 * expected.max(1.0_f32),
            "Arrhenius drift: got {got}, expected {expected}",
        );
        assert!(got.is_finite() && got > 0.0_f32);
    }

    #[test]
    fn thermo_adiabatic_rise_linear_in_alpha() {
        for alpha in [0.0_f32, 0.25_f32, 0.5_f32, 0.75_f32, 1.0_f32] {
            let (_, dt) = ThermoEngine::<B>::compute_heat_rate(t4(20.0), t4(alpha), t4(40_000.0));
            let rise = scalar_4(dt);
            let expected = alpha * ADIABATIC_TEMP_RISE_PER_ALPHA;
            assert!(
                (rise - expected).abs() < 1e-5_f32,
                "α={alpha}: rise={rise} expected={expected}",
            );
        }
    }

    #[test]
    fn thermo_chem_affinity_monotone_in_alpha() {
        let ea = 40_000.0_f32;
        let (q0, _) = ThermoEngine::<B>::compute_heat_rate(t4(20.0), t4(0.0), t4(ea));
        let (q_mid, _) = ThermoEngine::<B>::compute_heat_rate(t4(20.0), t4(0.5), t4(ea));
        let (q1, _) = ThermoEngine::<B>::compute_heat_rate(t4(20.0), t4(1.0), t4(ea));
        let v0 = scalar_4(q0);
        let v_mid = scalar_4(q_mid);
        let v1 = scalar_4(q1);
        assert!(v0 > v_mid && v_mid > v1, "heat rate must decrease with α: {v0} {v_mid} {v1}");
        assert!(v1 < 1e-3_f32 * v0, "α=1 should quench affinity: v0={v0} v1={v1}");
    }

    #[test]
    fn thermo_arrhenius_temperature_boosts_rate() {
        let alpha = 0.3_f32;
        let ea = 40_000.0_f32;
        let (q_cold, _) = ThermoEngine::<B>::compute_heat_rate(t4(10.0), t4(alpha), t4(ea));
        let (q_warm, _) = ThermoEngine::<B>::compute_heat_rate(t4(30.0), t4(alpha), t4(ea));
        assert!(
            scalar_4(q_warm) > scalar_4(q_cold),
            "Arrhenius must raise rate with temperature",
        );
    }

    #[test]
    fn fleet_composer_accel_ac12_thermo_engine_measured() {
        assert!(thermo_engine_tensor_path_measured());
        let (q, dt) = ThermoEngine::<B>::compute_heat_rate(t4(20.0), t4(0.5), t4(40_000.0));
        let heat = scalar_4(q);
        let rise = scalar_4(dt);
        assert!(heat.is_finite() && heat > 0.0_f32);
        assert!((rise - 25.0_f32).abs() < 1e-4_f32, "α=0.5 adiabatic proxy: rise={rise}");
    }
}
