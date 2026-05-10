// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

pub mod implementation;

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-export; classification follows the underlying symbol.
pub use implementation::ConcreteCartridge;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Forwards manifold UMST write-back helper used after topology physics.
pub use umst_manifold::core::apply_physics_to_umst;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Forwards manifold cartridge façade trait and tensor bundles.
pub use umst_manifold::core::{IScienceCartridge, MixTensor, PhysicalResult};
