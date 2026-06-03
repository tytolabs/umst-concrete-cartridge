// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Virtual lab proxies composing manifold solvers (feature `virtual-proxies`).
//!
//! Each scoring fn is a pure map **τ₀ / extrudability → [0, 1]** used as a literature surrogate
//! inside the printability leg of [`crate::pipeline::dual_gate::evaluate_dual_gate`]. These are
//! **not** Lean witnesses — they sit below R1 on the ladder and compose lazily with
//! `W_print` before R1 CD (`umst.gate.cd_transition`) runs on the thermodynamic leg.

#[cfg(feature = "virtual-proxies")]
pub mod virtual_extrusion;
#[cfg(feature = "virtual-proxies")]
pub mod virtual_stack;
