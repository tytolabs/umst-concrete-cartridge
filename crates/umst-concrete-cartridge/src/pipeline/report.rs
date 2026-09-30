// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Serialized multi-stage physics envelope for CLI / MCP tooling.
//! formal_anchor: NONE
//! formal_status: NONE
//! formal_anchor_rationale: Wire schema sidecar adjacent to manifold `PhysicalResult`; no standalone Lean witness.

use serde::{Deserialize, Serialize};

/// Wire schema tag for [`PhysicsPipelineReport`].
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Version discriminator for additive JSON fields.
pub const PHYSICS_PIPELINE_SCHEMA_VERSION: &str = "physics_pipeline.v1";

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Enumerates audited stage outcomes (`Executed` vs honest skips/failures).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStageStatus {
    Executed,
    SkippedMissingInputs,
    SkippedUnsupportedSignature,
    Failed,
}

/// One manifest entry outcome.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Serialized evidence that a stage ran, skipped, or failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStageRecord {
    pub id: String,
    pub status: PipelineStageStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl PipelineStageRecord {
    #[must_use]
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Successful stage marker helper.
    pub fn ok(id: &'static str) -> Self {
        Self {
            id: id.to_string(),
            status: PipelineStageStatus::Executed,
            detail: None,
        }
    }

    #[must_use]
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Honest skip when inputs/constants are absent by design.
    pub fn skip_missing(id: &'static str, reason: impl Into<String>) -> Self {
        Self {
            id: id.to_string(),
            status: PipelineStageStatus::SkippedMissingInputs,
            detail: Some(reason.into()),
        }
    }

    #[must_use]
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Propagates panics-avoiding error strings for observability.
    pub fn fail(id: &'static str, reason: impl Into<String>) -> Self {
        Self {
            id: id.to_string(),
            status: PipelineStageStatus::Failed,
            detail: Some(reason.into()),
        }
    }
}

/// Collapsible numeric summary extracted from tensors (already host-scalars for batch-collapse mode).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Human/tooling digest; not a substitute for full tensor fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsPipelineSummary {
    pub effective_water_cement_ratio: f32,
    pub hydration_alpha: f32,
    pub porosity_capillary: f32,
    pub strength_jennings_mpa: f32,
    pub rheology_yield_stress_pa: f32,
    pub thermo_adiabatic_rise_proxy_c: f32,
    pub chloride_diffusivity_m2_s: f32,
    pub printability_buildability: f32,
    pub printability_extrudability: f32,
    pub rheology_plastic_viscosity_pa_s: f32,
    pub itz_thickness_microns: f32,
    pub fracture_toughness_k_ic_mpa_sqrt_m: f32,
    pub sustainability_gwp_kg_co2_m3: f32,
    pub sustainability_cost_usd_per_m3: f32,
    pub dlvo_potential_kt_minimum: f32,
    pub shrinkage_microstrain_proxy: f32,
    pub freeze_thaw_durability_factor: f32,
    pub creep_compliance_1_over_gpa: f32,
}

/// Serializable multi-physics capsule returned by [`super::run_full_physics_pipeline`].
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Cartridge-local rich JSON envelope parallel to manifold tensors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsPipelineReport {
    pub schema_version: String,
    pub representation: &'static str,
    pub stages: Vec<PipelineStageRecord>,
    pub summary: PhysicsPipelineSummary,
}
