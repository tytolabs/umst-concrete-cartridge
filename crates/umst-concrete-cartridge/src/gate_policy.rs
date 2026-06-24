// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Cartridge-owned HTTP gate policy marker (`GateEvaluator` only — orthogonal to [`crate::core::ConcreteCartridge`] `IScienceCartridge`).

use umst_manifold::gate::{GateEvaluator, HttpGateManifest as GateManifest};
use umst_manifold::manifest::UmstManifest;

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Zero-sized policy evaluator; HTTP gate catalog row without spatial physics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConcretePolicyEvaluator;

impl ConcretePolicyEvaluator {
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Injection-only HTTP gate manifest from explicit [`UmstManifest`] (see `docs/RUNTIME_TOPOLOGY.md` § HTTP gate defaults).
    #[must_use]
    pub fn gate_manifest_from(manifest: &UmstManifest) -> GateManifest {
        GateManifest::from(manifest)
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
