// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
//
// FLEET-COMPOSER-X X05 — WEB-009 native stdio smoke harden retick.
//
// **Policy:** stdio subprocess 4-slot GREEN is reproducible via `cargo test -p umst-mcp stdio_smoke`;
// WEB-009 production fold and gateway native wrap remain honestly **open** (`production_wired=false`).
//
// **Absorbs:** H08 native stdio battery · W-wave sustain · SWARM-0831-30 owner pin.

use crate::stdio_smoke;

/// FLEET-COMPOSER-X X05 card id.
pub const COMPOSER_X05_JOB_ID: &str = "FLEET-COMPOSER-X05-STDIO-SMOKE-RETICK";

/// FLEET-COMPOSER-X X05 completion receipt cross-ref.
pub const COMPOSER_X05_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_X05_0734.md";

/// FLEET-COMPOSER-X X05 wave slot number.
pub const COMPOSER_X05_WAVE_SLOT: &str = "X05";

/// FLEET-COMPOSER-X manifest cross-ref.
pub const FLEET_X_MANIFEST_PATH: &str = "outputs/.tmp/FLEET_COMPOSER_X_100_0734.md";

/// Prior H08 stdio smoke harden receipt.
pub const PRIOR_H08_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_H08_2242.md";

/// WEB-009 lane id.
pub const WEB_009_JOB_ID: &str = "WEB-009";

/// WEB-009 production closure owner — not claimed by X05.
pub const WEB_009_PRODUCTION_OWNER: &str = "1836-spawn";

/// One hop on the umst-mcp WEB-009 stdio retick wire map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Web009StdioWireHop {
    /// Ordinal (1-based).
    pub ordinal: u8,
    /// Surface path.
    pub surface: &'static str,
    /// Role on the honest close path.
    pub role: &'static str,
}

/// umst-mcp WEB-009 stdio retick wire map (4 hops — mirrors 4-slot battery).
pub const WEB_009_STDIO_WIRE_HOPS: &[Web009StdioWireHop] = &[
    Web009StdioWireHop {
        ordinal: 1,
        surface: "umst-mcp/tests/stdio_smoke_h08.rs::h08_stdio_initialize_jsonrpc_handshake_green",
        role: "initialize → protocolVersion 2024-11-05",
    },
    Web009StdioWireHop {
        ordinal: 2,
        surface: "umst-mcp/tests/stdio_smoke_h08.rs::h08_stdio_tools_list_census_matches_ssot",
        role: "tools/list count ≡ tool_census SSOT",
    },
    Web009StdioWireHop {
        ordinal: 3,
        surface: "umst-mcp/tests/stdio_smoke_h08.rs::h08_stdio_umst_predict_call_green",
        role: "tools/call umst_predict → result.v2",
    },
    Web009StdioWireHop {
        ordinal: 4,
        surface: "umst-mcp/tests/stdio_smoke_h08.rs::h08_stdio_unknown_method_error_frame_green",
        role: "unknown method → -32601",
    },
];

/// WEB-009 production closed — 1836-spawn exclusive; X05 does not flip.
pub const fn web_009_production_closed() -> bool {
    false
}

/// Native stdio production wiring — delegate smoke only; no live wasm claim.
pub const fn web_009_stdio_production_wired() -> bool {
    false
}

/// Stdio subprocess 4-slot battery reproducible via cargo test.
pub const fn web_009_stdio_smoke_reproducible() -> bool {
    stdio_smoke::native_stdio_smoke_reproducible()
}

/// Receipt authority chain for X05 absorb (H08 + X manifest + stdio smoke module).
#[must_use]
pub fn web_009_stdio_smoke_x05_authority_chain_honest() -> bool {
    PRIOR_H08_RECEIPT_PATH.contains("COMPOSER_H08_2242")
        && FLEET_X_MANIFEST_PATH.contains("FLEET_COMPOSER_X_100_0734")
        && COMPOSER_X05_RECEIPT_PATH.contains("COMPOSER_X05_0734")
        && stdio_smoke::native_stdio_smoke_h08_authority_chain_honest()
}

/// FLEET-COMPOSER-X X05 typed probe — folds H08 stdio battery + WEB-009 honest boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Web009StdioSmokeX05Probe {
    /// FLEET-COMPOSER-X05 card id.
    pub composer_x05_job_id: &'static str,
    /// Model slug for receipt attribution.
    pub composer_model_slug: &'static str,
    /// X05 wave slot.
    pub composer_x05_wave_slot: &'static str,
    /// WEB-009 lane id.
    pub web_009_job_id: &'static str,
    /// Folded H08 stdio smoke probe.
    pub h08: stdio_smoke::NativeStdioSmokeH08Probe,
    /// Reproducible stdio subprocess battery slot count.
    pub smoke_slot_count: usize,
    /// Wire hop count (mirrors slot count).
    pub wire_hop_count: u8,
    /// Stdio smoke reproducible via cargo test.
    pub stdio_smoke_reproducible: bool,
    /// WEB-009 production closed (honest false).
    pub web_009_production_closed: bool,
    /// WEB-009 stdio production wired (honest false).
    pub web_009_stdio_production_wired: bool,
    /// Receipt authority chain honest.
    pub authority_chain_honest: bool,
}

/// Build the FLEET-COMPOSER-X X05 WEB-009 stdio smoke retick probe.
#[must_use]
pub fn web_009_stdio_smoke_x05_probe() -> Web009StdioSmokeX05Probe {
    Web009StdioSmokeX05Probe {
        composer_x05_job_id: COMPOSER_X05_JOB_ID,
        composer_model_slug: crate::mcp_spine::COMPOSER_MODEL_SLUG,
        composer_x05_wave_slot: COMPOSER_X05_WAVE_SLOT,
        web_009_job_id: WEB_009_JOB_ID,
        h08: stdio_smoke::native_stdio_smoke_h08_probe(),
        smoke_slot_count: stdio_smoke::NATIVE_STDIO_SMOKE_SLOT_COUNT,
        wire_hop_count: WEB_009_STDIO_WIRE_HOPS.len() as u8,
        stdio_smoke_reproducible: web_009_stdio_smoke_reproducible(),
        web_009_production_closed: web_009_production_closed(),
        web_009_stdio_production_wired: web_009_stdio_production_wired(),
        authority_chain_honest: web_009_stdio_smoke_x05_authority_chain_honest(),
    }
}

/// X05 honesty gate — partial max; stdio GREEN without production flip invent.
#[must_use]
pub fn web_009_stdio_smoke_x05_honest(probe: &Web009StdioSmokeX05Probe) -> bool {
    probe.composer_x05_job_id == COMPOSER_X05_JOB_ID
        && probe.composer_model_slug == crate::mcp_spine::COMPOSER_MODEL_SLUG
        && probe.composer_x05_wave_slot == COMPOSER_X05_WAVE_SLOT
        && probe.web_009_job_id == WEB_009_JOB_ID
        && probe.smoke_slot_count == stdio_smoke::NATIVE_STDIO_SMOKE_SLOT_COUNT
        && probe.wire_hop_count == WEB_009_STDIO_WIRE_HOPS.len() as u8
        && probe.stdio_smoke_reproducible
        && !probe.web_009_production_closed
        && !probe.web_009_stdio_production_wired
        && probe.authority_chain_honest
        && stdio_smoke::native_stdio_smoke_h08_honest(&probe.h08)
        && web_009_stdio_smoke_x05_authority_chain_honest()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fleet_composer_x05_web_009_stdio_smoke_metadata() {
        assert_eq!(COMPOSER_X05_JOB_ID, "FLEET-COMPOSER-X05-STDIO-SMOKE-RETICK");
        assert_eq!(COMPOSER_X05_WAVE_SLOT, "X05");
        assert_eq!(WEB_009_STDIO_WIRE_HOPS.len(), 4);
        assert!(web_009_stdio_smoke_x05_authority_chain_honest());
    }

    #[test]
    fn fleet_composer_x05_web_009_stdio_smoke_honest_partial() {
        let probe = web_009_stdio_smoke_x05_probe();
        assert_eq!(probe.smoke_slot_count, 4);
        assert_eq!(probe.wire_hop_count, 4);
        assert!(probe.stdio_smoke_reproducible);
        assert!(!probe.web_009_production_closed);
        assert!(!probe.web_009_stdio_production_wired);
        assert!(probe.authority_chain_honest);
        assert!(web_009_stdio_smoke_x05_honest(&probe));
    }
}
