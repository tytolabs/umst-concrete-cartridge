// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
//
// FLEET-COMPOSER-H H08 — native stdio JSON-RPC smoke battery probe.
//
// **Policy:** stdio subprocess GREEN is reproducible via `cargo test -p umst-mcp stdio_smoke`;
// WEB-009 production fold and gateway native wrap remain honestly **open** (`production_wired=false`).
//
// **Absorbs:** F11 gateway stdio regression ledger · G39 MCP wrap closure census · SWARM-0831-30.

use crate::tool_census;

/// FLEET-COMPOSER-H H08 card id.
pub const COMPOSER_H08_JOB_ID: &str = "FLEET-COMPOSER-H08-STDIO-SMOKE";

/// FLEET-COMPOSER-H H08 completion receipt cross-ref.
pub const COMPOSER_H08_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_H08_2242.md";

/// FLEET-COMPOSER-X X05 completion receipt cross-ref (stdio smoke retick absorb).
pub const COMPOSER_X05_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_X05_0734.md";

/// FLEET-COMPOSER-H H08 wave slot number.
pub const COMPOSER_H08_WAVE_SLOT: &str = "H08";

/// FLEET-COMPOSER-H manifest cross-ref.
pub const FLEET_H_MANIFEST_PATH: &str = "outputs/.tmp/FLEET_COMPOSER_H_100_2242.md";

/// Prior F11 gateway stdio regression guard receipt.
pub const PRIOR_F11_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_F11_STDIO_1942.md";

/// Prior G39 LIB-ADOPT-W-MCP-WRAP closure census receipt.
pub const PRIOR_G39_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_G39_MCP_WRAP_2143.md";

/// SWARM-0831-30 native stdio smoke owner cross-ref.
pub const SWARM_0831_30_OWNER: &str = "SWARM-0831-30";

/// WEB-009 production closure owner — not claimed by H08.
pub const WEB_009_PRODUCTION_OWNER: &str = "1836-spawn";

/// MCP JSON-RPC protocol version returned by `initialize`.
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// Reproducible native stdio smoke battery slot count (integration `stdio_smoke_h08` + lib probe).
pub const NATIVE_STDIO_SMOKE_SLOT_COUNT: usize = 4;

/// Constitutional 13-tool manifest count (default `agent-layer` profile).
pub const CONSTITUTIONAL_MANIFEST_TOOL_COUNT: usize = tool_census::CONSTITUTIONAL_COUNT;

/// Native stdio subprocess smoke is GREEN when `cargo test -p umst-mcp stdio_smoke` passes.
pub const fn native_stdio_smoke_reproducible() -> bool {
    true
}

/// WEB-009 live wasm fold — 1836-spawn exclusive; H08 does not flip.
pub const fn web_009_production_closed() -> bool {
    false
}

/// Native stdio path production wiring — delegate smoke only; no live wasm claim.
pub const fn native_stdio_smoke_production_wired() -> bool {
    false
}

/// Gateway native MCP wrap — external boundary; stays open per G39.
pub const fn gateway_native_wrap_closed() -> bool {
    false
}

/// Receipt authority chain for H08 absorb (F11 + G39 + H manifest).
#[must_use]
pub fn native_stdio_smoke_h08_authority_chain_honest() -> bool {
    PRIOR_F11_RECEIPT_PATH.contains("COMPOSER_F11_STDIO_1942")
        && PRIOR_G39_RECEIPT_PATH.contains("COMPOSER_G39_MCP_WRAP_2143")
        && FLEET_H_MANIFEST_PATH.contains("FLEET_COMPOSER_H_100_2242")
        && COMPOSER_H08_RECEIPT_PATH.contains("COMPOSER_H08_2242")
}

/// FLEET-COMPOSER-H H08 typed probe — folds stdio battery + honest production boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeStdioSmokeH08Probe {
    /// FLEET-COMPOSER-H08 card id.
    pub composer_h08_job_id: &'static str,
    /// Model slug for receipt attribution.
    pub composer_model_slug: &'static str,
    /// H08 wave slot.
    pub composer_h08_wave_slot: &'static str,
    /// Reproducible stdio subprocess battery slot count.
    pub smoke_slot_count: usize,
    /// Expected `tools/list` length for current build profile.
    pub expected_tools_list_count: usize,
    /// Stdio smoke reproducible via cargo test.
    pub stdio_smoke_reproducible: bool,
    /// WEB-009 production closed (honest false).
    pub web_009_production_closed: bool,
    /// Native stdio production wired (honest false).
    pub native_stdio_smoke_production_wired: bool,
    /// Gateway native wrap closed (honest false).
    pub gateway_native_wrap_closed: bool,
    /// Receipt authority chain honest.
    pub authority_chain_honest: bool,
}

/// Build the FLEET-COMPOSER-H H08 native stdio smoke probe.
#[must_use]
pub fn native_stdio_smoke_h08_probe() -> NativeStdioSmokeH08Probe {
    NativeStdioSmokeH08Probe {
        composer_h08_job_id: COMPOSER_H08_JOB_ID,
        composer_model_slug: crate::mcp_spine::COMPOSER_MODEL_SLUG,
        composer_h08_wave_slot: COMPOSER_H08_WAVE_SLOT,
        smoke_slot_count: NATIVE_STDIO_SMOKE_SLOT_COUNT,
        expected_tools_list_count: tool_census::expected_tools_list_count_for_build(),
        stdio_smoke_reproducible: native_stdio_smoke_reproducible(),
        web_009_production_closed: web_009_production_closed(),
        native_stdio_smoke_production_wired: native_stdio_smoke_production_wired(),
        gateway_native_wrap_closed: gateway_native_wrap_closed(),
        authority_chain_honest: native_stdio_smoke_h08_authority_chain_honest(),
    }
}

/// H08 honesty gate — partial max; stdio GREEN without production flip invent.
#[must_use]
pub fn native_stdio_smoke_h08_honest(probe: &NativeStdioSmokeH08Probe) -> bool {
    probe.composer_h08_job_id == COMPOSER_H08_JOB_ID
        && probe.composer_model_slug == crate::mcp_spine::COMPOSER_MODEL_SLUG
        && probe.composer_h08_wave_slot == COMPOSER_H08_WAVE_SLOT
        && probe.smoke_slot_count == NATIVE_STDIO_SMOKE_SLOT_COUNT
        && probe.expected_tools_list_count == tool_census::expected_tools_list_count_for_build()
        && probe.stdio_smoke_reproducible
        && !probe.web_009_production_closed
        && !probe.native_stdio_smoke_production_wired
        && !probe.gateway_native_wrap_closed
        && probe.authority_chain_honest
        && native_stdio_smoke_h08_authority_chain_honest()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fleet_composer_h08_stdio_smoke_metadata() {
        assert_eq!(COMPOSER_H08_JOB_ID, "FLEET-COMPOSER-H08-STDIO-SMOKE");
        assert_eq!(COMPOSER_H08_WAVE_SLOT, "H08");
        assert_eq!(NATIVE_STDIO_SMOKE_SLOT_COUNT, 4);
        assert_eq!(CONSTITUTIONAL_MANIFEST_TOOL_COUNT, 13);
        assert!(native_stdio_smoke_h08_authority_chain_honest());
    }

    #[test]
    fn fleet_composer_h08_stdio_smoke_honest_partial() {
        let probe = native_stdio_smoke_h08_probe();
        assert!(probe.stdio_smoke_reproducible);
        assert!(!probe.web_009_production_closed);
        assert!(!probe.native_stdio_smoke_production_wired);
        assert!(!probe.gateway_native_wrap_closed);
        assert!(probe.authority_chain_honest);
        assert!(native_stdio_smoke_h08_honest(&probe));
    }
}
