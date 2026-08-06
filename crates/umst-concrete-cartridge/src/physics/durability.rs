// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
//
// FLEET-COMPOSER-ACCEL-D AC105 — composite concrete durability owner.
// Canonical implementation lives in `durability/engine.rs`; wire via `physics/mod.rs` on integrate.

#[path = "durability/engine.rs"]
mod engine;

pub use engine::{
    DurabilityEngine, DurabilityOutcome, PathwayBreakdown, PathwayLeg,
    ORCHESTRATOR_PIN_AIR_FRACTION, ORCHESTRATOR_PIN_AIR_VOID_SURFACE, ORCHESTRATOR_PIN_ALPHA,
    ORCHESTRATOR_PIN_INTERNAL_RH, ORCHESTRATOR_PIN_PASTE_FRACTION, ORCHESTRATOR_PIN_REF_DIFFUSIVITY,
    ORCHESTRATOR_PIN_REQUIRED_AIR_PCT, ORCHESTRATOR_PIN_WC,
};
