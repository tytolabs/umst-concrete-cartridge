// SPDX-License-Identifier: MIT

//! Staged tensor physics pipeline and Track A proxy-loop gates.
//!
//! [`run_full_physics_pipeline`] is the cartridge functor root; [`evaluate_dual_gate`] composes
//! printability ⊗ thermodynamic witnesses (see [`dual_gate`] module docs and witness ladder R1).

pub mod b2_orchestrator_delegate;
pub mod canonical_gate;
pub mod cast_phase;
pub mod dual_gate;
pub mod orchestrator;
pub mod physical_summary;
pub mod report;
#[cfg(feature = "proxy-loop")]
pub mod track_a;

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Phase 0d canonical admissibility surface (manifold composed gate).
pub use canonical_gate::thermodynamic_admissible;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: MP3.1 cast lifecycle classifier (α thresholds; orchestrator wiring deferred).
pub use cast_phase::{
    classify_cast_phase, CastLifecycleThresholds, CastPhase, CastPhaseInputs,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-export dual-gate verdict for MCP/CLI Track A.
pub use dual_gate::{
    evaluate_dual_gate, CastGateVerdict, PrintabilityReject, PRINTABLE_TAU_HI, PRINTABLE_TAU_LO,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Thermodynamic leg reject newtype (P2 `GateRejectReason` bridge).
pub use canonical_gate::ThermoReject;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Stable import path for staged tensor physics.
pub use orchestrator::run_full_physics_pipeline;
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Topology / predict policy maps from pipeline report.
pub use physical_summary::{
    nominal_mix_tensor_for_mix_spec, nominal_mix_tensor_for_topology, physical_result_from_report,
    topology_pipeline_headlines, topology_pipeline_report, TopologyNominalMix,
};
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: JSON envelope types for MCP/CLI audit trails.
pub use report::{
    PhysicsPipelineReport, PhysicsPipelineSummary, PipelineStageRecord, PipelineStageStatus,
    PHYSICS_PIPELINE_SCHEMA_VERSION,
};
#[cfg(feature = "proxy-loop")]
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Track A coordinate-descent + proposed mix JSON assembly.
pub use track_a::{
    coordinate_descent_optimize, evaluate_mix_dual_gate, proposed_next_mix_json, ProposedNextMix,
    TrackAObjective,
};
