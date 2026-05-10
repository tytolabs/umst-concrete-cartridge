// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO


use burn::tensor::{Tensor, backend::Backend};
use umst_manifold::core::tensors::MixTensor;

/// Pure function to calculate economic metrics (Pareto Front targets).
/// 
/// Because it is composed purely of `burn` tensor operations, the gradient of the cost
/// with respect to the input mix fractions allows the agent to naturally step towards cheaper mixes.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Auxiliary objective; linear cost vector, no physical claim.
pub fn compute_cost<B: Backend>(mix: &MixTensor<B>, unit_cost_factors: Tensor<B, 2>) -> Tensor<B, 2> {
    // Assumes mix.fractions and unit_cost_factors are [Batch, Features]
    // Computes the dot product per batch item (sum over the features dimension)
    mix.fractions.clone().mul(unit_cost_factors).sum_dim(1)
}
