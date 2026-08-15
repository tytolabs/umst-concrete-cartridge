// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
//
// FLEET-COMPOSER-Z Z106 — umst-mcp L4/L5 verify entry (`cargo test mcp_l4_l5`).
//
// Thin facade over `l4_l5` close predicates. Absorbs Z29 owner slice; does not redo census.

pub use crate::l5::{
    l4_l5_close_eligible, l4_l5_probe, l4_l5_production_wired, l4_l5_surface_wired,
    L4L5SurfaceProbe, L4_STDIO_WIRE_CLOSED, L5_ADDITIVE_TOOL_COUNT, L5_FULL_PROFILE_TOOLS_LIST_COUNT,
    L5_SURFACE_WIRE_HOPS,
};

/// FLEET-COMPOSER-Z Z106 card id.
pub const COMPOSER_Z106_JOB_ID: &str = "FLEET-COMPOSER-Z106-L4-L5";

/// FLEET-COMPOSER-Z Z106 completion receipt cross-ref.
pub const COMPOSER_Z106_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_Z106_1232.md";

/// FLEET-COMPOSER-Z Z106 wave slot number.
pub const COMPOSER_Z106_WAVE_SLOT: &str = "Z106";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_l4_l5_z106_metadata_pins() {
        assert_eq!(COMPOSER_Z106_JOB_ID, "FLEET-COMPOSER-Z106-L4-L5");
        assert_eq!(COMPOSER_Z106_WAVE_SLOT, "Z106");
        assert!(COMPOSER_Z106_RECEIPT_PATH.contains("COMPOSER_Z106_1232"));
        assert_eq!(L5_SURFACE_WIRE_HOPS.len(), L5_ADDITIVE_TOOL_COUNT);
    }

    #[test]
    fn mcp_l4_l5_l4_stdio_wire_closed_witness() {
        assert!(L4_STDIO_WIRE_CLOSED);
        assert_eq!(L5_FULL_PROFILE_TOOLS_LIST_COUNT, 18);
    }

    #[test]
    fn mcp_l4_l5_surface_wired_default_profile() {
        let probe = l4_l5_probe();
        assert!(probe.l4_stdio_wire_closed);
        assert!(probe.constitutional_tools_wired);
        assert_eq!(probe.l5_wire_hop_count, 5);
        assert!(l4_l5_surface_wired(&probe));
    }

    #[test]
    fn mcp_l4_l5_close_eligible_without_production_flip() {
        let probe = l4_l5_probe();
        assert!(!probe.production_wired);
        assert!(!l4_l5_production_wired());
        assert!(l4_l5_close_eligible(&probe));
    }

    #[test]
    fn mcp_l4_l5_l5_wire_hops_ordinal_pin() {
        for (idx, hop) in L5_SURFACE_WIRE_HOPS.iter().enumerate() {
            assert_eq!(hop.ordinal, (idx + 1) as u8);
            assert!(!hop.surface.is_empty());
        }
    }
}
