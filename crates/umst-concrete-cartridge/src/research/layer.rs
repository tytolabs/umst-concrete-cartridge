// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! `PhysicalReasoningLayer` — per-cartridge memory geometry + contribute schema port.

use super::geometry::{mix_geometry_key, MixGeometryKey};
use super::types::{MemoryQuery, MemoryRecord, CONTRIBUTION_SCHEMA, MEMORY_SCHEMA};
use serde_json::Value;

/// Cartridge-local Physical Reasoning Layer port (memory geometry + schema ids).
///
/// Manifold hosts universal gate law via [`umst_manifold::core::IScienceCartridge`];
/// each cartridge implements this trait for domain-specific locality and wire shapes.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Cartridge port trait; geometry hook defers to `mix_geometry_key`.
pub trait PhysicalReasoningLayer {
    /// Stable cartridge namespace slug (`umst-concrete-cartridge`, …).
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Namespace id for MCP resource URIs; no physics claim.
    fn cartridge_slug(&self) -> &'static str;

    /// Contribution wire schema id.
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Default contribution.v1 schema id re-export.
    fn contribution_schema(&self) -> &'static str {
        CONTRIBUTION_SCHEMA
    }

    /// Memory row wire schema id.
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Default memory_record.v1 schema id re-export.
    fn memory_schema(&self) -> &'static str {
        MEMORY_SCHEMA
    }

    /// Design-coordinate locality index for a mix_spec (concrete: Morton on w_c × T).
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Delegates to Morton geometry; not thermodynamic gate.
    fn mix_geometry(
        &self,
        mix_spec: &Value,
        curing_regime: Option<&str>,
    ) -> Option<MixGeometryKey> {
        mix_geometry_key(mix_spec, curing_regime)
    }

    /// Default memory query (admissible-only).
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Default filter record for agent memory queries.
    fn default_memory_query(&self) -> MemoryQuery {
        MemoryQuery {
            admissible_only: true,
            ..Default::default()
        }
    }

    /// Filter predicate extension hook (default: pass-through).
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Optional cartridge-specific query predicate hook.
    fn matches_query(&self, _record: &MemoryRecord, _query: &MemoryQuery) -> bool {
        true
    }
}

/// Concrete cartridge reference implementation of [`PhysicalReasoningLayer`].
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Zero-sized port impl for umst-concrete-cartridge agent layer.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConcretePhysicalReasoningLayer;

impl PhysicalReasoningLayer for ConcretePhysicalReasoningLayer {
    fn cartridge_slug(&self) -> &'static str {
        "umst-concrete-cartridge"
    }
}
