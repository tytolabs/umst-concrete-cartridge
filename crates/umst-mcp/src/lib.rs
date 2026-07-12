// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! `umst-mcp` library surface — pure helpers referenced by agent docs and tests.

pub mod handlers;
pub mod parity;

/// Thin facade — implementation lives in `umst-agent-mcp-core` (S5).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Re-export facade; soft_gate morphisms documented in umst-agent-mcp-core.
pub use umst_agent_mcp_core::soft_gate;

#[cfg(feature = "agent-layer")]
pub mod agent_layer;

#[cfg(feature = "rmcp-wire")]
pub mod rmcp_server;

#[cfg(feature = "tool-manifest")]
pub mod manifest;
