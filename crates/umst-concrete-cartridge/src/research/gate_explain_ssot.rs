// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Gate explain vocabulary — re-exports [`umst_manifold::runtime::gate::explain_codes`] SSOT.

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Cartridge mirror of manifold gate reject codes and field/remediation SSOT for MCP explain parity.
pub use umst_manifold::runtime::gate::{
    fields_for_code, remediation_for_code, MANIFEST_BRIDGE_DISABLED, MIX_SPEC_RATIONAL_PARSE_FAIL,
    MIX_SPEC_WIRE_INVALID, THERMODYNAMIC_CD_FAIL, THERMODYNAMIC_FAIL, TOP_GATE_EXPLAIN_CODES,
};
