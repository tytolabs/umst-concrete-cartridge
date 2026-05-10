// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! UMST differentiable concrete cartridge: calibration profiles, homogeneous routing, coupled tensor engines.

#![allow(clippy::doc_lazy_continuation)]

pub mod calibration;
pub mod calibration_metrics;
pub mod core;
pub mod formulas;
pub mod homogeneous;
pub mod physics;

#[cfg(feature = "cli")]
pub mod cli;

mod burn_compat;

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-export; classification follows the underlying symbol.
pub use umst_manifold::core::{IScienceCartridge, MixTensor, PhysicalResult};
