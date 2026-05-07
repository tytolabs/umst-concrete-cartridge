// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

pub mod core;
pub mod physics;

mod burn_compat;

// Expose the core cartridge interface
pub use umst_manifold::core::{IScienceCartridge, MixTensor, PhysicalResult};
