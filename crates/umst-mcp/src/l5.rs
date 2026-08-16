// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//
// FLEET-COMPOSER-Z Z29 — umst-mcp L4/L5 semantic + web tool surface close predicate.
//
// **Policy:** L4 composed stdio wire CLOSED (gate_parity witness); L5 MCP schema/handler
// surface WIRED at feature-gated registration; production/live fold and gateway native
// wrap remain honestly **open** (`production_wired=false`).
//
// **Absorbs:** Y11 · X05 WEB-009 stdio · gate_parity L4 close · HCOM-029 semantic surface.

/// FLEET-COMPOSER-Z Z29 card id.
pub const COMPOSER_Z29_JOB_ID: &str = "FLEET-COMPOSER-Z29-L4-L5";

/// FLEET-COMPOSER-Z Z29 completion receipt cross-ref.
pub const COMPOSER_Z29_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_Z29_1015.md";

/// FLEET-COMPOSER-Z Z29 wave slot number.
pub const COMPOSER_Z29_WAVE_SLOT: &str = "Z29";

/// L4 composed stdio wire — CLOSED @ `b1-parity-green` (`7d0ca7b`).
///
/// SSOT witness: `umst-mcp/tests/gate_parity.rs::l4_wire_phase2_inventory::WIRE_OPEN`.
pub const L4_STDIO_WIRE_CLOSED: bool = true;

/// L4 harness slot count (S0 Stage 0f lock).
pub const L4_WIRE_SLOT_COUNT: usize = 6;

/// L4 fixture digest pin — held through wire close.
pub const L4_FIXTURE_DIGEST: &str =
    "7a3d3e5f5d634322474aee76dea9cc79d2cbeb1fe87920c51a4c1a6bdb9e0a87";

/// HCOM-029 semantic agent tool names (L5 semantic family).
pub const L5_SEMANTIC_TOOL_NAMES: &[&str] = &[
    "propose_communicative_act",
    "map_to_geometry",
    "refine_shape",
    "get_audit_digest",
];

/// WEB-009 L5 additive web tool name.
pub const L5_WEB_TOOL_NAME: &str = "web_propose_delta";

/// L5 semantic tool count (HCOM-029 full profile).
pub const L5_SEMANTIC_TOOL_COUNT: usize = 4;

/// L5 web tool count (WEB-009).
pub const L5_WEB_TOOL_COUNT: usize = 1;

/// L5 total additive tools when full semantic + web profile enabled.
pub const L5_ADDITIVE_TOOL_COUNT: usize = L5_SEMANTIC_TOOL_COUNT + L5_WEB_TOOL_COUNT;

/// Expected `tools/list` count for full L4/L5 MCP surface profile.
pub const L5_FULL_PROFILE_TOOLS_LIST_COUNT: usize =
    crate::tool_census::CONSTITUTIONAL_COUNT + L5_ADDITIVE_TOOL_COUNT;

/// One hop on the L5 semantic + web MCP surface wire map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct L5SurfaceWireHop {
    /// Ordinal (1-based).
    pub ordinal: u8,
    /// Tool or surface path.
    pub surface: &'static str,
    /// Owning feature gate.
    pub feature: &'static str,
    /// Role on the honest close path.
    pub role: &'static str,
}

/// L5 semantic + web MCP surface wire map (5 hops).
pub const L5_SURFACE_WIRE_HOPS: &[L5SurfaceWireHop] = &[
    L5SurfaceWireHop {
        ordinal: 1,
        surface: "umst-mcp/src/semantic_hcom.rs::exec_propose_communicative_act",
        feature: "tool-propose-communicative-act",
        role: "HCOM-029 hybrid frontier + local gate",
    },
    L5SurfaceWireHop {
        ordinal: 2,
        surface: "umst-mcp/src/semantic_hcom.rs::exec_map_to_geometry",
        feature: "tool-propose-communicative-act",
        role: "surface → meaning geometry",
    },
    L5SurfaceWireHop {
        ordinal: 3,
        surface: "umst-mcp/src/semantic_hcom.rs::exec_refine_shape",
        feature: "tool-propose-communicative-act",
        role: "HCOM-020 Kleisli stub (honest open)",
    },
    L5SurfaceWireHop {
        ordinal: 4,
        surface: "umst-mcp/src/semantic_hcom.rs::exec_get_audit_digest",
        feature: "tool-propose-communicative-act",
        role: "audit digest witness",
    },
    L5SurfaceWireHop {
        ordinal: 5,
        surface: "umst-mcp/src/web_propose_delta.rs::exec_web_propose_delta_mock",
        feature: "tool-web-propose-delta",
        role: "WEB-009 informational mock fold",
    },
];

/// Gateway L5 native MCP wrap — owned by `umst-gateway` (`R-gateway-wrap-native-mcp`).
pub const fn gateway_l5_native_wrap_closed() -> bool {
    crate::mcp_spine::gateway_native_wrap_closed()
}

/// WEB-009 live wasm fold — honestly open until 1836-spawn.
pub const fn web_propose_delta_live_fold_wired() -> bool {
    crate::mcp_spine::web_propose_delta_live_fold_wired()
}

/// Production wiring — delegate/mock only; no live gateway ceremony measured.
pub const fn l4_l5_production_wired() -> bool {
    false
}

/// FLEET-COMPOSER-Z Z29 typed probe — L4 close + L5 MCP surface wire map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct L4L5SurfaceProbe {
    /// Z29 card id.
    pub composer_z29_job_id: &'static str,
    /// Model slug for receipt attribution.
    pub composer_model_slug: &'static str,
    /// Z29 wave slot.
    pub composer_z29_wave_slot: &'static str,
    /// L4 composed stdio wire closed.
    pub l4_stdio_wire_closed: bool,
    /// L4 harness slot count.
    pub l4_wire_slot_count: usize,
    /// Constitutional 13-tool surface wired.
    pub constitutional_tools_wired: bool,
    /// L5 wire hop count.
    pub l5_wire_hop_count: u8,
    /// L5 full profile tools/list count (when features on).
    pub l5_full_profile_tools_list_count: usize,
    /// Gateway L5 native wrap closed (external boundary).
    pub gateway_l5_native_wrap_closed: bool,
    /// WEB-009 live fold wired (honest false).
    pub web_propose_delta_live_fold_wired: bool,
    /// Production wired (honest false).
    pub production_wired: bool,
}

/// Build the FLEET-COMPOSER-Z Z29 L4/L5 surface probe.
#[must_use]
pub fn l4_l5_probe() -> L4L5SurfaceProbe {
    let spine = crate::mcp_spine::mcp_product_spine_probe();
    L4L5SurfaceProbe {
        composer_z29_job_id: COMPOSER_Z29_JOB_ID,
        composer_model_slug: crate::mcp_spine::COMPOSER_MODEL_SLUG,
        composer_z29_wave_slot: COMPOSER_Z29_WAVE_SLOT,
        l4_stdio_wire_closed: L4_STDIO_WIRE_CLOSED,
        l4_wire_slot_count: L4_WIRE_SLOT_COUNT,
        constitutional_tools_wired: spine.constitutional_tools_wired,
        l5_wire_hop_count: L5_SURFACE_WIRE_HOPS.len() as u8,
        l5_full_profile_tools_list_count: L5_FULL_PROFILE_TOOLS_LIST_COUNT,
        gateway_l5_native_wrap_closed: gateway_l5_native_wrap_closed(),
        web_propose_delta_live_fold_wired: web_propose_delta_live_fold_wired(),
        production_wired: l4_l5_production_wired(),
    }
}

/// umst-mcp L4/L5 MCP surface wired — L4 closed + constitutional + L5 wire map honest.
///
/// Does **not** claim gateway native wrap or live wasm fold closure.
#[must_use]
pub fn l4_l5_surface_wired(probe: &L4L5SurfaceProbe) -> bool {
    probe.l4_stdio_wire_closed
        && probe.constitutional_tools_wired
        && probe.l4_wire_slot_count == L4_WIRE_SLOT_COUNT
        && probe.l5_wire_hop_count == L5_ADDITIVE_TOOL_COUNT as u8
        && probe.l5_full_profile_tools_list_count == L5_FULL_PROFILE_TOOLS_LIST_COUNT
        && L5_SEMANTIC_TOOL_NAMES.len() == L5_SEMANTIC_TOOL_COUNT
        && !L5_WEB_TOOL_NAME.is_empty()
}

/// Z29 close eligibility — surface wired; production + gateway wrap honestly open.
#[must_use]
pub fn l4_l5_close_eligible(probe: &L4L5SurfaceProbe) -> bool {
    l4_l5_surface_wired(probe)
        && !probe.production_wired
        && !probe.web_propose_delta_live_fold_wired
        && !probe.gateway_l5_native_wrap_closed
        && probe.composer_z29_job_id == COMPOSER_Z29_JOB_ID
        && probe.composer_model_slug == crate::mcp_spine::COMPOSER_MODEL_SLUG
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l4_l5_z29_metadata_pins() {
        assert_eq!(COMPOSER_Z29_JOB_ID, "FLEET-COMPOSER-Z29-L4-L5");
        assert_eq!(COMPOSER_Z29_WAVE_SLOT, "Z29");
        assert!(COMPOSER_Z29_RECEIPT_PATH.contains("COMPOSER_Z29_1015"));
        assert_eq!(L5_SURFACE_WIRE_HOPS.len(), L5_ADDITIVE_TOOL_COUNT);
        assert_eq!(L5_FULL_PROFILE_TOOLS_LIST_COUNT, 18);
    }

    #[test]
    fn l4_l5_l4_stdio_wire_closed_witness() {
        assert!(L4_STDIO_WIRE_CLOSED);
        assert_eq!(L4_WIRE_SLOT_COUNT, 6);
        assert!(!L4_FIXTURE_DIGEST.is_empty());
    }

    #[test]
    fn l4_l5_l5_wire_hops_ordinal_pin() {
        for (idx, hop) in L5_SURFACE_WIRE_HOPS.iter().enumerate() {
            assert_eq!(hop.ordinal, (idx + 1) as u8);
            assert!(!hop.surface.is_empty());
            assert!(!hop.feature.is_empty());
            assert!(!hop.role.is_empty());
        }
        assert_eq!(L5_SEMANTIC_TOOL_NAMES.len(), 4);
        assert_eq!(L5_WEB_TOOL_NAME, "web_propose_delta");
    }

    #[test]
    fn l4_l5_surface_wired_default_profile() {
        let probe = l4_l5_probe();
        assert!(probe.l4_stdio_wire_closed);
        assert!(probe.constitutional_tools_wired);
        assert_eq!(probe.l5_wire_hop_count, 5);
        assert_eq!(probe.l5_full_profile_tools_list_count, 18);
        assert!(l4_l5_surface_wired(&probe));
    }

    #[test]
    fn l4_l5_close_eligible_without_production_flip() {
        let probe = l4_l5_probe();
        assert!(!probe.production_wired);
        assert!(!probe.web_propose_delta_live_fold_wired);
        assert!(!probe.gateway_l5_native_wrap_closed);
        assert!(l4_l5_close_eligible(&probe));
        assert!(!l4_l5_production_wired());
    }

    #[cfg(all(
        feature = "tool-propose-communicative-act",
        feature = "tool-web-propose-delta"
    ))]
    #[test]
    fn l4_l5_full_profile_tools_list_count() {
        assert_eq!(
            crate::tool_census::expected_tools_list_count_for_build(),
            L5_FULL_PROFILE_TOOLS_LIST_COUNT
        );
    }

    #[cfg(all(
        feature = "tool-propose-communicative-act",
        feature = "tool-web-propose-delta"
    ))]
    #[test]
    fn l4_l5_web_009_l5_delegate_schema_honest() {
        assert!(crate::web_propose_delta::web_009_l5_stdio_delegate_schema_honest());
    }

    #[cfg(feature = "tool-propose-communicative-act")]
    #[test]
    fn l4_l5_semantic_agent_tools_surface() {
        use crate::semantic_hcom::{hcom_semantic_agent_tool_schemas, HCOM_SEMANTIC_AGENT_TOOLS};
        assert_eq!(
            hcom_semantic_agent_tool_schemas().len(),
            L5_SEMANTIC_TOOL_COUNT
        );
        assert_eq!(HCOM_SEMANTIC_AGENT_TOOLS.len(), L5_SEMANTIC_TOOL_COUNT);
    }
}
