// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO


use burn::tensor::{Tensor, backend::Backend};
use umst_manifold::core::tensors::MixTensor;

/// Pure function to calculate the bulk capillary porosity of the cement paste over time.
/// 
/// Because it is composed purely of `burn` tensor operations, the gradient of the porosity
/// with respect to the input mix fractions allows the agent to naturally step towards denser mixes.
/// formal_anchor: NONE
/// formal_status: Library
/// formal_axioms: NONE
/// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
pub fn compute_capillary_porosity<B: Backend>(wc_ratio: Tensor<B, 2>, hydration_degree: Tensor<B, 2>) -> Tensor<B, 2> {
    // Powers-Brownyard model for capillary porosity
    // p_c = (w/c - 0.36 * alpha) / (w/c + 0.32)
    // Water volume consumed by hydration is 0.36 * alpha
    
    let consumed_water = hydration_degree.mul_scalar(0.36);
    let capillary_water = wc_ratio.clone().sub(consumed_water).clamp_min(0.0);
    
    let total_paste_volume = wc_ratio.add_scalar(0.32);
    
    let porosity = capillary_water.div(total_paste_volume);
    
    porosity
}
