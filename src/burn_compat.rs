// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Burn-version glue only (no physics).

use burn::tensor::{backend::Backend, Bool, Tensor};

/// Logical AND for boolean tensors (Burn ≤0.12 exposed `bool_and`; later versions use float multiply).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_axioms: NONE
/// formal_anchor_rationale: Burn-version compatibility shim for boolean tensor AND across crate semver skew.
#[inline]
pub fn bool_and<B: Backend, const D: usize>(
    a: Tensor<B, D, Bool>,
    b: Tensor<B, D, Bool>,
) -> Tensor<B, D, Bool> {
    a.float().mul(b.float()).greater_elem(0.5f32)
}
