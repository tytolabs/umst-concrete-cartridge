use burn::tensor::{backend::Backend, Tensor};

/// Pure tensor implementation of the Thermodynamic Engine
/// Computes hydration heat evolution using the Arrhenius law across a differentiable manifold.
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
    pub fn compute_heat_rate(
        temp_c: Tensor<B, 4>,
        alpha: Tensor<B, 4>,
        activation_energy: Tensor<B, 4>,
    ) -> (Tensor<B, 4>, Tensor<B, 4>) {
        let r_gas = 8.314_f32;
        let temp_k = temp_c.add_scalar(273.15_f32);

        // Affinity term: (1 - alpha)^1.5
        // Using clamp_min(0.0) to replicate max(0.0) and prevent NaN in backward pass for negative bases
        let one_minus_alpha = alpha
            .clone()
            .mul_scalar(-1.0)
            .add_scalar(1.0)
            .clamp_min(0.0);
        let chem_affinity = one_minus_alpha.powf_scalar(1.5);

        // Arrhenius: k = exp(-E / RT)
        let rt = temp_k.mul_scalar(r_gas);
        let negative_e_over_rt = activation_energy.mul_scalar(-1.0).div(rt);
        let rate_constant = negative_e_over_rt.exp();

        // Reference rate at 20C approx
        let ref_rate = 1e6_f32;

        let heat_rate = rate_constant.mul(chem_affinity).mul_scalar(ref_rate);

        // Adiabatic temp rise: Simplistic alpha * 50.0
        let adiabatic_temp_rise = alpha.mul_scalar(50.0_f32);

        (heat_rate, adiabatic_temp_rise)
    }
}
