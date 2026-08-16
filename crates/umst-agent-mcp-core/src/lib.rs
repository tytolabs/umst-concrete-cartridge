// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! `umst-agent-mcp-core` — domain-agnostic agent MCP primitives (MCP build plan S5).
//!
//! Cartridge-specific physics (`gate_check_mix_result`, predict/audit) stays in
//! `umst-concrete-cartridge`; this crate holds reusable cold-boundary helpers.

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Differentiable soft-gate templates; hard admissibility is cartridge path.
pub mod soft_gate;
