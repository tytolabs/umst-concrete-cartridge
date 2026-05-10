// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Full tensor physics orchestration feeding [`crate::mix_layout`] mixes.
//! formal_anchor: NONE
//! formal_status: NONE
//! formal_anchor_rationale: Cartridge-internal composition layer; manifests map to tooling JSON.

pub mod orchestrator;
pub mod physical_summary;
pub mod report;

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Public entry to staged tensor physics used by `ConcreteCartridge::compute_all`.
pub use orchestrator::run_full_physics_pipeline;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Policy map from rich report to manifold `PhysicalResult`.
pub use physical_summary::physical_result_from_report;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-export pipeline wire types for CLI/MCP consumers.
pub use report::PhysicsPipelineReport;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-export pipeline wire types for CLI/MCP consumers.
pub use report::PhysicsPipelineSummary;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-export pipeline wire types for CLI/MCP consumers.
pub use report::PipelineStageRecord;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-export pipeline wire types for CLI/MCP consumers.
pub use report::PipelineStageStatus;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-export pipeline wire types for CLI/MCP consumers.
pub use report::PHYSICS_PIPELINE_SCHEMA_VERSION;
