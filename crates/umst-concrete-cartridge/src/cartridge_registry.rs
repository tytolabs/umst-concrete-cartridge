// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Cartridge-owned gate `catalog_id` / `gate_family` badges (W9 T2d — evicted from kernel traceability).

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: W9 SSOT `catalog_id` badge for [`crate::gate_policy::ConcretePolicyEvaluator`]; traceability only.
pub const DOMAIN_POLICY_CATALOG_ID: &str = "umst.cartridge.domain.policy";

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: HTTP bulk strength closure + Parrott kinetics telemetry `gate_family` (not a `catalog_id`).
pub const MIX_PREDICTION_VS_PHYSICS_GATE_FAMILY: &str = "mix_prediction_vs_physics";

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Default manifest literals `gate_family` for domain HTTP gate ([`crate::gate_policy::ConcretePolicyEvaluator`]).
pub const CONCRETE_POWERS_MANIFEST_GATE_FAMILY: &str = "concrete_powers_manifest_defaults";
