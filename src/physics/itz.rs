// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO


use burn::tensor::{Tensor, backend::Backend};
use umst_manifold::core::tensors::MixTensor;

/// Pure function to calculate the Interfacial Transition Zone (ITZ) properties.
/// Essential for accurately modeling Recycled Aggregate Concrete (RAC).
/// 
/// Because it is composed purely of `burn` tensor operations, the gradient of the ITZ
/// with respect to the input mix fractions can be computed natively.
/// formal_anchor: NONE
/// formal_status: Library
/// formal_axioms: NONE
/// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
pub fn compute_itz_thickness_microns<B: Backend>(max_aggregate_size_mm: Tensor<B, 2>) -> Tensor<B, 2> {
    // ITZ thickness roughly scales with agg size and w/c
    // Helper model: t = 10um + 2.5 * agg_size
    max_aggregate_size_mm.mul_scalar(2.5).add_scalar(10.0)
}

/// formal_anchor: NONE
/// formal_status: Library
/// formal_axioms: NONE
/// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
pub fn compute_itz_porosity<B: Backend>(wc_ratio: Tensor<B, 2>) -> Tensor<B, 2> {
    // ITZ porosity is typically 1.5x - 2x bulk paste porosity
    // Paste porosity ~ wc - 0.2
    let bulk_porosity = wc_ratio.sub_scalar(0.2).clamp_min(0.05);
    
    // Max porosity capped at 0.9 physically
    bulk_porosity.mul_scalar(1.8).clamp_max(0.9)
}

/// formal_anchor: NONE
/// formal_status: Library
/// formal_axioms: NONE
/// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
pub fn compute_itz_percolation_factor<B: Backend>(itz_vol_frac: Tensor<B, 2>) -> Tensor<B, 2> {
    // Percolation threshold approx 20%-30% ITZ volume
    // factor = 1.0 + max(0, itz_vol - 0.3) * 10.0
    let excess_itz = itz_vol_frac.sub_scalar(0.3).clamp_min(0.0);
    excess_itz.mul_scalar(10.0).add_scalar(1.0)
}
