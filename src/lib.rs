// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy, and Studio Tyto
// SPDX-License-Identifier: Apache-2.0

pub mod core;
pub mod physics;

// Expose the core cartridge interface
pub use umst_manifold::core::{IScienceCartridge, MixTensor, PhysicalResult};
