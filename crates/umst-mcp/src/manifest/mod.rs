// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Stage S1 — declarative tool manifest (feature `tool-manifest`, default off).
//!
//! Hand-rolled `base_tools()` / `agent_tools_schema()` remain the live default.
//! Enable dual-emit with `UMST_MCP_MANIFEST=1` when this feature is compiled in.

pub mod descriptor;
pub mod tools_v1;

/// Runtime gate: manifest path only when env is explicitly `1`.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Env IO boundary for optional dual-emit; default remains hand tables.
#[must_use]
pub fn manifest_env_enabled() -> bool {
    std::env::var("UMST_MCP_MANIFEST")
        .map(|v| v == "1")
        .unwrap_or(false)
}
