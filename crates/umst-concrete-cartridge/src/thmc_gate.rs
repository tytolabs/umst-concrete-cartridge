// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! THMC solver gate wiring — inject [`ConcreteTransitionCartridge`] SSOT at post-step evidence hook.

use umst_manifold::physics::solvers::ThmcSolver;
use umst_manifold::runtime::gate::GateCartridge;

use crate::gate_evidence::ConcreteTransitionCartridge;
use crate::material_transition::CEMENT_DEFAULT_S_INTRINSIC_MPA;

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Configures [`ThmcSolver`] intrinsic strength from cement SSOT before coupled step.
/// Configure a [`ThmcSolver`] to lift gate snapshots with cartridge cement constants.
#[must_use]
pub fn with_gate_cartridge(solver: ThmcSolver, cartridge: &ConcreteTransitionCartridge) -> ThmcSolver {
    let _ = cartridge;
    solver.with_gate_intrinsic_strength_mpa(CEMENT_DEFAULT_S_INTRINSIC_MPA)
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Routes post-step evidence through cartridge [`GateCartridge`] witness.
/// Resolve the static cartridge witness for THMC post-step telemetry.
#[must_use]
pub fn gate_cartridge_witness(cartridge: &ConcreteTransitionCartridge) -> &'static dyn GateCartridge {
    let _ = cartridge;
    &ConcreteTransitionCartridge
}
