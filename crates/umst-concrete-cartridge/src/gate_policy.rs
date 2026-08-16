// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Cartridge-owned HTTP gate policy marker (`GateEvaluator` only — orthogonal to [`crate::core::ConcreteCartridge`] `IScienceCartridge`).

use umst_manifold::gate::{GateEvaluator, HttpGateManifest as GateManifest};
use umst_manifold::manifest::UmstManifest;

use umst_cartridge_registry::{
    CONCRETE_POWERS_MANIFEST_GATE_FAMILY, DOMAIN_POLICY_CATALOG_ID,
    MIX_PREDICTION_VS_PHYSICS_GATE_FAMILY,
};

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
        DOMAIN_POLICY_CATALOG_ID
    }

    fn gate_family(&self) -> &'static str {
        CONCRETE_POWERS_MANIFEST_GATE_FAMILY
    }
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Zero-sized HTTP shim overlay; supplies cartridge `gate_family` for mix prediction vs physics telemetry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HttpMixDomainEvaluator;

impl GateEvaluator for HttpMixDomainEvaluator {
    fn catalog_id(&self) -> &'static str {
        "umst.gate.http_shim"
    }

    fn gate_family(&self) -> &'static str {
        MIX_PREDICTION_VS_PHYSICS_GATE_FAMILY
    }
}
