// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO


use burn::tensor::{Tensor, backend::Backend};

/// Pure function for the Compressible Packing Model (CPM) approximation.
/// Calculates the packing density of the aggregate skeleton.
/// 
/// Because it is composed purely of `burn` tensor operations, the gradient of the packing density
/// with respect to the aggregate fractions allows the agent to perfectly grade the mix.
pub fn compute_packing_density<B: Backend>(coarse_fraction: Tensor<B, 2>, fine_fraction: Tensor<B, 2>) -> Tensor<B, 2> {
    // Empirical interaction model (de Larrard approximation)
    let total_agg = coarse_fraction.clone().add(fine_fraction.clone()).clamp_min(1e-6);
    let fine_ratio = fine_fraction.div(total_agg);
    
    // Ideal packing usually occurs around 0.40 - 0.45 fine ratio for standard aggregates
    // We use a differentiable inverted parabola: P = P_max - k*(r - r_opt)^2
    let p_max = 0.74f32; // Maximum theoretical packing
    let r_opt = 0.42f32; // Optimal sand ratio
    let k = 1.2f32;      // Curvature penalty
    
    let diff = fine_ratio.sub_scalar(r_opt);
    let diff_sq = diff.powf_scalar(2.0);
    
    // Packing density drops off as we move away from optimal ratio
    let packing_density = diff_sq.mul_scalar(-k).add_scalar(p_max).clamp_min(0.5);
    
    packing_density
}
