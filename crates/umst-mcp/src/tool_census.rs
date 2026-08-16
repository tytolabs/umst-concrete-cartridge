// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//
// MCP `tools/list` census — honest per-profile tool counts (MCP-082 deepen).
//
// **Gap closed (SWARM-C25-0831-40):** S7 proposed-tools row — default-off features
// (`tool-dry-run`, `tool-promote`, `tool-arena-session-unified`) that grow `tools/list`
// beyond the constitutional 13 without changing S0 gate golden bytes.
// `gate-explain-v2` patches `umst_gate_check` schema only — count unchanged.

/// GO-LIVE Step 3 constitutional count (`default = ["agent-layer"]`).
pub const CONSTITUTIONAL_COUNT: usize = 13;

/// Historical base-four surface (`--no-default-features`).
pub const BASE_FOUR_COUNT: usize = 4;

/// S7 proposed tool names (P2–P4) — one tool per enabled default-off feature.
pub const S7_PROPOSED_TOOL_NAMES: &[&str] = &[
    "umst_dry_run",
    "umst_promote_contribution",
    "umst_arena_session",
];

/// Maximum S7 proposed tools when all three list-growing features are enabled.
pub const S7_PROPOSED_MAX_ADDS: usize = 3;

/// Semantic stub (`tool-semantic-hcom` without `tool-propose-communicative-act`).
pub const SEMANTIC_HCOM_STUB_ADDS: usize = 1;

/// Full HCOM-029 semantic agent surface (`tool-propose-communicative-act`).
pub const SEMANTIC_AGENT_FULL_ADDS: usize = 4;

/// WEB-009 additive informational tool (`tool-web-propose-delta`).
pub const WEB_PROPOSE_DELTA_ADDS: usize = 1;

/// Documented combined profiles from `docs/AGENT_MCP.md` §12.4 (measured, not invented).
pub const PROFILE_SEMANTIC_HCOM_WEB_COUNT: usize =
    CONSTITUTIONAL_COUNT + SEMANTIC_HCOM_STUB_ADDS + WEB_PROPOSE_DELTA_ADDS;

/// `agent-layer` + `tool-propose-communicative-act` + `tool-web-propose-delta`.
pub const PROFILE_SEMANTIC_FULL_WEB_COUNT: usize =
    CONSTITUTIONAL_COUNT + SEMANTIC_AGENT_FULL_ADDS + WEB_PROPOSE_DELTA_ADDS;

/// `agent-layer` + all three S7 list-growing features.
pub const PROFILE_S7_ALL_ON_COUNT: usize = CONSTITUTIONAL_COUNT + S7_PROPOSED_MAX_ADDS;

/// Count additive tools for the **current** build profile.
#[must_use]
pub fn additive_tool_count_for_build() -> usize {
    // `mut` is required when S7 additive features are enabled; otherwise unused.
    #[allow(unused_mut)]
    let mut n = 0usize;
    #[cfg(feature = "tool-dry-run")]
    {
        n += 1;
    }
    #[cfg(feature = "tool-promote")]
    {
        n += 1;
    }
    #[cfg(feature = "tool-arena-session-unified")]
    {
        n += 1;
    }
    #[cfg(all(
        feature = "tool-semantic-hcom",
        not(feature = "tool-propose-communicative-act")
    ))]
    {
        n += SEMANTIC_HCOM_STUB_ADDS;
    }
    #[cfg(feature = "tool-propose-communicative-act")]
    {
        n += SEMANTIC_AGENT_FULL_ADDS;
    }
    #[cfg(feature = "tool-web-propose-delta")]
    {
        n += WEB_PROPOSE_DELTA_ADDS;
    }
    n
}

/// Expected `tools/list` length for the **current** build profile.
#[must_use]
pub fn expected_tools_list_count_for_build() -> usize {
    #[cfg(feature = "agent-layer")]
    {
        CONSTITUTIONAL_COUNT + additive_tool_count_for_build()
    }
    #[cfg(not(feature = "agent-layer"))]
    {
        BASE_FOUR_COUNT
    }
}

/// Operator-facing census row for S7 proposed tools (markdown table cell).
#[must_use]
pub fn s7_proposed_census_markdown_row() -> &'static str {
    "| S7 proposed (P2–P4) | `gate-explain-v2` (schema patch, +0) · `tool-dry-run` · `tool-promote` · `tool-arena-session-unified` | +0..+3 | 13→16 max | `tests/proposed_tools.rs` |"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constitutional_default_build_is_thirteen() {
        #[cfg(feature = "agent-layer")]
        {
            assert_eq!(expected_tools_list_count_for_build(), 13);
            assert_eq!(additive_tool_count_for_build(), 0);
        }
    }

    #[test]
    fn s7_proposed_arithmetic_honest() {
        assert_eq!(S7_PROPOSED_TOOL_NAMES.len(), S7_PROPOSED_MAX_ADDS);
        assert_eq!(PROFILE_S7_ALL_ON_COUNT, 16);
        assert_eq!(PROFILE_SEMANTIC_HCOM_WEB_COUNT, 15);
        assert_eq!(PROFILE_SEMANTIC_FULL_WEB_COUNT, 18);
    }

    #[test]
    fn s7_census_row_documents_gap_closure() {
        let row = s7_proposed_census_markdown_row();
        assert!(row.contains("S7 proposed"));
        assert!(row.contains("tool-dry-run"));
        assert!(row.contains("13→16"));
    }
}
