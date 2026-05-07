use burn::tensor::{Tensor, backend::Backend};
use umst_manifold::core::tensors::MixTensor;

/// Pure function to calculate the hydration degree alpha over time.
/// 
/// Because it is composed purely of `burn` tensor operations, the gradient of the hydration
/// degree with respect to the input mix fractions can be computed natively.
pub fn compute_hydration_degree<B: Backend>(mix: &MixTensor<B>, age_days: Tensor<B, 2>, temperature_c: Tensor<B, 2>) -> Tensor<B, 2> {
    let dims = mix.fractions.dims();
    let batch_size = dims[0];
    
    // Physical indices
    let cement = mix.fractions.clone().slice([0..batch_size, 1..2]);
    let slag = mix.fractions.clone().slice([0..batch_size, 5..6]); // Assume index 5
    let fly_ash = mix.fractions.clone().slice([0..batch_size, 6..7]); // Assume index 6
    
    let binder = cement.clone().add(slag.clone()).add(fly_ash.clone()).clamp_min(1e-6);
    let scm_ratio = slag.add(fly_ash).div(binder);
    
    // alpha_max = 0.95 - scm_ratio * 0.15
    let alpha_max = scm_ratio.clone().mul_scalar(-0.15).add_scalar(0.95);
    
    // k_ref = 0.55 * k_ref_multiplier (assume multiplier is 1.0 for now)
    let k_ref = 0.55f32;
    
    // Arrhenius temperature correction
    let t_ref_k = 293.15f32;
    let t_k = temperature_c.add_scalar(273.15);
    let e_over_r = 5000.0f32;
    
    // temp_factor = exp(E/R * (1/T_ref - 1/T))
    let inv_t_ref = 1.0 / t_ref_k;
    let inv_t = t_k.powf_scalar(-1.0);
    let temp_factor = inv_t.mul_scalar(-1.0).add_scalar(inv_t_ref).mul_scalar(e_over_r).exp();
    
    // scm_factor = 1.0 - scm_ratio * 0.4
    let scm_factor = scm_ratio.mul_scalar(-0.4).add_scalar(1.0);
    
    // k = k_ref * temp_factor * scm_factor
    let k = temp_factor.mul(scm_factor).mul_scalar(k_ref);
    
    // alpha = alpha_max * (1.0 - exp(-k * sqrt(age)))
    let age_sqrt = age_days.clone().sqrt();
    let decay = k.mul(age_sqrt).mul_scalar(-1.0).exp();
    let alpha = alpha_max.mul(decay.mul_scalar(-1.0).add_scalar(1.0));
    
    alpha.clamp(0.0, 1.0)
}
