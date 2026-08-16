// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//
// FLEET-COMPOSER-X X05 — WEB-009 stdio smoke harden retick integration battery.
//
// Absorbs H08 4-slot stdio subprocess smoke; pins WEB-009 honest production boundary.

use umst_mcp::stdio_smoke::NATIVE_STDIO_SMOKE_SLOT_COUNT;
use umst_mcp::web_009::{
    web_009_stdio_smoke_x05_honest, web_009_stdio_smoke_x05_probe, COMPOSER_X05_JOB_ID,
    COMPOSER_X05_WAVE_SLOT, WEB_009_STDIO_WIRE_HOPS,
};

/// X05 metadata pin — job id, wave slot, wire hop count.
#[test]
fn x05_web_009_stdio_smoke_metadata_pin() {
    assert_eq!(COMPOSER_X05_JOB_ID, "FLEET-COMPOSER-X05-STDIO-SMOKE-RETICK");
    assert_eq!(COMPOSER_X05_WAVE_SLOT, "X05");
    assert_eq!(WEB_009_STDIO_WIRE_HOPS.len(), 4);
    assert_eq!(NATIVE_STDIO_SMOKE_SLOT_COUNT, 4);
}

/// X05 honesty gate — stdio reproducible; prod false; H08 fold honest.
#[test]
fn x05_web_009_stdio_smoke_honest_without_production_flip() {
    let probe = web_009_stdio_smoke_x05_probe();
    assert_eq!(probe.smoke_slot_count, 4);
    assert_eq!(probe.wire_hop_count, 4);
    assert!(probe.stdio_smoke_reproducible);
    assert!(!probe.web_009_production_closed);
    assert!(!probe.web_009_stdio_production_wired);
    assert!(probe.h08.stdio_smoke_reproducible);
    assert!(!probe.h08.web_009_production_closed);
    assert!(!probe.h08.native_stdio_smoke_production_wired);
    assert!(web_009_stdio_smoke_x05_honest(&probe));
}

/// X05 wire map ordinal pin — 4 hops mirror 4-slot battery.
#[test]
fn x05_web_009_stdio_wire_hops_mirror_slot_battery() {
    for (idx, hop) in WEB_009_STDIO_WIRE_HOPS.iter().enumerate() {
        assert_eq!(hop.ordinal, (idx + 1) as u8);
        assert!(!hop.surface.is_empty());
        assert!(!hop.role.is_empty());
    }
}
