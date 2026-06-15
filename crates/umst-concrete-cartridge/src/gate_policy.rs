// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Cartridge-owned HTTP gate policy marker (`GateEvaluator` only — orthogonal to [`crate::core::ConcreteCartridge`] `IScienceCartridge`).

use umst_manifold::gate::{default_gate_manifest, GateEvaluator, HttpGateManifest as GateManifest};

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Zero-sized policy evaluator; HTTP gate catalog row without spatial physics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConcretePolicyEvaluator;

impl ConcretePolicyEvaluator {
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Powers closure defaults for HTTP manifest shim.
    #[must_use]
    pub fn default_gate_manifest() -> GateManifest {
        default_gate_manifest()
    }
}

impl GateEvaluator for ConcretePolicyEvaluator {
    fn catalog_id(&self) -> &'static str {
        "umst.cartridge.concrete.policy"
    }

    fn gate_family(&self) -> &'static str {
        "concrete_powers_manifest_defaults"
    }
}
