// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! `umst-mcp` library surface — pure helpers referenced by agent docs and tests.

pub mod handlers;
pub mod l5;
pub mod mcp_l4_l5;
pub mod mcp_spine;
pub mod parity;
pub mod sec_mcp_wrap;
pub mod stdio_smoke;
pub mod tool_census;
pub mod web_009;

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

#[cfg(any(
    feature = "gate-explain-v2",
    feature = "tool-dry-run",
    feature = "tool-promote",
    feature = "tool-arena-session-unified"
))]
pub mod proposed_tools;

#[cfg(feature = "tool-semantic-hcom")]
pub mod semantic_hcom_schema;

#[cfg(feature = "tool-propose-communicative-act")]
pub mod mcp_trust_gate;

#[cfg(feature = "tool-propose-communicative-act")]
pub mod semantic_hcom;

#[cfg(feature = "tool-web-propose-delta")]
pub mod web_propose_delta;
