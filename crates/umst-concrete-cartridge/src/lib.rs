// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! UMST differentiable concrete cartridge: calibration profiles, homogeneous routing, coupled tensor engines.

#![allow(clippy::doc_lazy_continuation)]

pub mod calibration;
pub mod calibration_fit;
pub mod calibration_metrics;
pub mod core;
pub mod formulas;
pub mod homogeneous;
pub mod mix_layout;
pub mod physics;
/// Tensor engine orchestration (`compute_all`) and MCP/CLI capsules.
pub mod pipeline;
#[cfg(feature = "virtual-proxies")]
pub mod proxies;

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports Striatus shell artefact helpers (symmetry gather) without new physics.
pub mod print_ready;

/// Wire DTOs and pure `predict` / schema bytes without `serde_json` in this crate.
pub mod facade;
pub mod gate_policy;

#[cfg(feature = "manifold-gate")]
pub mod gate_evidence;

#[cfg(feature = "agent-layer")]
/// Gate-validated research memory (Physical Reasoning Layer).
pub mod research;

#[cfg(feature = "manifold-gate")]
pub mod material_transition;

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports concrete transition gate cartridge when `manifold-gate` is enabled.
#[cfg(feature = "manifold-gate")]
pub use gate_evidence::ConcreteTransitionCartridge;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports W9 Tier-2c cement closure SSOT when `tier2c-handshake` is enabled.
#[cfg(feature = "tier2c-handshake")]
pub use material_transition::{
    cement_reaction_extent_kinetics_spec, CementMaterialParams, CEMENT_DEFAULT_S_INTRINSIC_MPA,
    CEMENT_REACTION_ENTHALPY_J_PER_KG,
};

mod burn_compat;

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports manifold façade symbols for ergonomics only.
pub use core::{
    apply_physics_to_umst, ConcreteCartridge, IScienceCartridge, PhysicalResult, StatePoint,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports cartridge HTTP gate policy evaluator.
pub use gate_policy::ConcretePolicyEvaluator;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Stable import path for MCP/CLI integration tests.
pub use pipeline::run_full_physics_pipeline;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: JSON envelope for staged tensor outputs.
pub use pipeline::PhysicsPipelineReport;

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Scalar digest accompanying report JSON.
pub use pipeline::PhysicsPipelineSummary;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Stage record type embedded in [`PhysicsPipelineReport`].
pub use pipeline::PipelineStageRecord;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Serialized stage disposition enum for MCP/CLI audit trails.
pub use pipeline::PipelineStageStatus;
/// formal_anchor: literature://wire-schema-physics-pipeline-v1
/// formal_status: Literature
/// formal_citation: "physics_pipeline schema tag (`physics_pipeline.v1`)"
/// formal_form: "`schema_version` string on serde `PhysicsPipelineReport` — bump tag when breaking report shape."
/// formal_anchor_rationale: Wire consumers pin report JSON against this version field.
pub use pipeline::PHYSICS_PIPELINE_SCHEMA_VERSION;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports host transition gate traits when `manifold-gate` is enabled.
#[cfg(feature = "manifold-gate")]
pub use umst_manifold::gate::{
    GateEvaluator, ThermodynamicTransitionEvaluator, TransitionGateEvaluator,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports manifold deployment manifest when `manifold-manifest` is enabled.
#[cfg(feature = "manifold-manifest")]
pub use umst_manifold::manifest::UmstManifest;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-exports manifold ROS serde DTOs when `ros2-contract` is enabled.
#[cfg(feature = "ros2-contract")]
pub use umst_manifold::ros;
