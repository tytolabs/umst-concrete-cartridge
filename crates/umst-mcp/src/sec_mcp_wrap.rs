// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
//
// FLEET-COMPOSER-Z Z86 — umst-mcp SEC-MCP-WRAP typed trust refuse close predicate.
//
// **Policy:** HCOM-029 semantic agent MCP tools enforce typed `trust_refused` via
// `mcp_trust_gate`; gateway stdio exec pre-check and production ceremony remain
// honestly **open** (`production_wired=false`).
//
// **Absorbs:** Y52 gateway refuse matrix authority · Y51 SEC-GW-AUDIT adjacent · Z29 L4/L5.

/// AGAP-2033 card id.
pub const JOB_ID: &str = "AGAP-2033-SEC-MCP-WRAP";

/// AGAP-2350 deepen card id.
pub const DEEPEN_JOB_ID: &str = "AGAP-2350-SEC-MCP-WRAP";

/// FLEET-COMPOSER-Z Z86 card id.
pub const COMPOSER_Z86_JOB_ID: &str = "FLEET-COMPOSER-Z86-SEC-MCP-WRAP";

/// FLEET-COMPOSER-Z Z86 completion receipt cross-ref.
pub const COMPOSER_Z86_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_Z86_1232.md";

/// FLEET-COMPOSER-Z Z86 wave slot.
pub const COMPOSER_Z86_WAVE_SLOT: &str = "Z86";

/// FLEET-COMPOSER-Y Y52 receipt cross-ref (gateway owner — absorb, do not redo).
pub const PRIOR_Y52_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_Y52_0808.md";

/// FLEET-COMPOSER-Y Y51 receipt cross-ref (SEC-GW-AUDIT adjacent).
pub const PRIOR_Y51_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_Y51_0808.md";

/// Gateway stdio exec trust pre-check owner (honest open residue).
pub const GATEWAY_STDIO_EXEC_OWNER: &str =
    "umst-gateway/crates/umst-gateway/src/sec_mcp_wrap.rs::mcp_stdio_exec_trust_pre_check_wired";

/// Typed refuse-path matrix row count (umst-mcp HCOM-029 quartet).
pub const REFUSE_PATH_MATRIX_ROW_COUNT: usize = 6;

/// Honest SEC-MCP-WRAP closure posture at umst-mcp boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecMcpWrapClosure {
    /// Typed refuse wired; gateway stdio exec pre-check + production open.
    Partial,
    /// Honesty gate failed.
    Failed,
}

/// One hop on the umst-mcp SEC-MCP-WRAP wire map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecMcpWrapWireHop {
    /// Ordinal (1-based).
    pub ordinal: u8,
    /// Module or symbol surface.
    pub surface: &'static str,
    /// Role in the trust delegate chain.
    pub role: &'static str,
}

/// umst-mcp trust refuse wire map — semantic agent MCP surface only.
pub const WIRE_HOPS: &[SecMcpWrapWireHop] = &[
    SecMcpWrapWireHop {
        ordinal: 1,
        surface: "umst-mcp/src/mcp_trust_gate.rs::trust_gate_for_semantic_agent_tool",
        role: "HCOM-029 quartet → TrustGate mapping",
    },
    SecMcpWrapWireHop {
        ordinal: 2,
        surface: "umst-mcp/src/mcp_trust_gate.rs::trust_from_wire",
        role: "Parse trust.scope witness from MCP args",
    },
    SecMcpWrapWireHop {
        ordinal: 3,
        surface: "umst-mcp/src/mcp_trust_gate.rs::check_semantic_agent_trust",
        role: "umst-trust admit gate — ScopeNotPermitted refuse",
    },
    SecMcpWrapWireHop {
        ordinal: 4,
        surface: "umst-mcp/src/mcp_trust_gate.rs::trust_refuse_wire",
        role: "Typed agent_error.v1 trust_refused envelope",
    },
    SecMcpWrapWireHop {
        ordinal: 5,
        surface: "umst-mcp/src/semantic_hcom.rs::trust_pre_gate",
        role: "Pre-exec hook on HCOM-029 semantic handlers",
    },
    SecMcpWrapWireHop {
        ordinal: 6,
        surface: "umst-mcp/src/sec_mcp_wrap.rs::sec_mcp_wrap_close_eligible",
        role: "Z86 umst-mcp owner close predicate",
    },
];

/// Gateway stdio exec trust pre-check — owned by umst-gateway; honest open.
#[must_use]
pub const fn gateway_stdio_exec_trust_pre_check_wired() -> bool {
    false
}

/// Production ceremony — delegate/mock only; no live gateway measured.
#[must_use]
pub const fn sec_mcp_wrap_production_wired() -> bool {
    false
}

/// FLEET-COMPOSER-Z Z86 typed probe — umst-mcp SEC-MCP-WRAP surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecMcpWrapProbe {
    /// Z86 card id.
    pub composer_z86_job_id: &'static str,
    /// Model slug for receipt attribution.
    pub composer_model_slug: &'static str,
    /// Z86 wave slot.
    pub composer_z86_wave_slot: &'static str,
    /// Wire hop count.
    pub wire_hop_count: u8,
    /// Typed refuse-path matrix honest (feature-gated live eval).
    pub refuse_path_matrix_honest: bool,
    /// Refuse-path matrix row count.
    pub refuse_path_matrix_row_count: u8,
    /// Gateway stdio exec pre-check wired (external — honest false).
    pub gateway_stdio_exec_trust_pre_check_wired: bool,
    /// Production wired (honest false).
    pub production_wired: bool,
    /// Combined closure — Partial max at umst-mcp boundary.
    pub closure: SecMcpWrapClosure,
}

/// Build the FLEET-COMPOSER-Z Z86 SEC-MCP-WRAP probe.
#[must_use]
pub fn sec_mcp_wrap_probe() -> SecMcpWrapProbe {
    let refuse_honest = sec_mcp_wrap_refuse_path_matrix_honest();
    SecMcpWrapProbe {
        composer_z86_job_id: COMPOSER_Z86_JOB_ID,
        composer_model_slug: crate::mcp_spine::COMPOSER_MODEL_SLUG,
        composer_z86_wave_slot: COMPOSER_Z86_WAVE_SLOT,
        wire_hop_count: WIRE_HOPS.len() as u8,
        refuse_path_matrix_honest: refuse_honest,
        refuse_path_matrix_row_count: REFUSE_PATH_MATRIX_ROW_COUNT as u8,
        gateway_stdio_exec_trust_pre_check_wired: gateway_stdio_exec_trust_pre_check_wired(),
        production_wired: sec_mcp_wrap_production_wired(),
        closure: if refuse_honest {
            SecMcpWrapClosure::Partial
        } else {
            SecMcpWrapClosure::Failed
        },
    }
}

/// umst-mcp SEC-MCP-WRAP surface wired — wire map + typed refuse matrix honest.
#[must_use]
pub fn sec_mcp_wrap_surface_wired(probe: &SecMcpWrapProbe) -> bool {
    probe.wire_hop_count == WIRE_HOPS.len() as u8
        && probe.refuse_path_matrix_row_count == REFUSE_PATH_MATRIX_ROW_COUNT as u8
        && probe.refuse_path_matrix_honest
}

/// Z86 close eligibility — surface wired; gateway stdio exec + production honestly open.
#[must_use]
pub fn sec_mcp_wrap_close_eligible(probe: &SecMcpWrapProbe) -> bool {
    sec_mcp_wrap_surface_wired(probe)
        && !probe.gateway_stdio_exec_trust_pre_check_wired
        && !probe.production_wired
        && matches!(probe.closure, SecMcpWrapClosure::Partial)
        && probe.composer_z86_job_id == COMPOSER_Z86_JOB_ID
        && probe.composer_model_slug == crate::mcp_spine::COMPOSER_MODEL_SLUG
}

#[cfg(feature = "tool-propose-communicative-act")]
mod refuse_matrix {
    use super::REFUSE_PATH_MATRIX_ROW_COUNT;
    use crate::mcp_trust_gate::check_semantic_agent_trust;
    use serde_json::{json, Value};
    use umst_trust::TrustError;

    /// One typed refuse-path row for umst-mcp HCOM-029 semantic agent tools.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct McpTrustRefusePathRow {
        /// Row id (R1..R6).
        pub row_id: &'static str,
        /// Semantic agent MCP tool name.
        pub tool_name: &'static str,
        /// trust.scope wire value.
        pub scope: &'static str,
        /// Whether trust is revoked.
        pub revoked: bool,
        /// Expected admit (Ok).
        pub expect_admit: bool,
    }

    pub const REFUSE_PATH_MATRIX: &[McpTrustRefusePathRow] = &[
        McpTrustRefusePathRow {
            row_id: "R1",
            tool_name: "propose_communicative_act",
            scope: "ephemeral",
            revoked: false,
            expect_admit: false,
        },
        McpTrustRefusePathRow {
            row_id: "R2",
            tool_name: "propose_communicative_act",
            scope: "device",
            revoked: false,
            expect_admit: true,
        },
        McpTrustRefusePathRow {
            row_id: "R3",
            tool_name: "refine_shape",
            scope: "ephemeral",
            revoked: false,
            expect_admit: false,
        },
        McpTrustRefusePathRow {
            row_id: "R4",
            tool_name: "refine_shape",
            scope: "device",
            revoked: false,
            expect_admit: true,
        },
        McpTrustRefusePathRow {
            row_id: "R5",
            tool_name: "map_to_geometry",
            scope: "ephemeral",
            revoked: false,
            expect_admit: true,
        },
        McpTrustRefusePathRow {
            row_id: "R6",
            tool_name: "map_to_geometry",
            scope: "device",
            revoked: true,
            expect_admit: false,
        },
    ];

    fn args_for_row(row: &McpTrustRefusePathRow) -> Value {
        let mut args = json!({ "intent": "chair", "surface": "chair" });
        if row.revoked {
            args["trust"] = json!({ "scope": row.scope, "revoked": true });
        } else {
            args["trust"] = json!({ "scope": row.scope });
        }
        args
    }

    /// Evaluate one typed refuse-path matrix row against live `check_semantic_agent_trust`.
    #[must_use]
    pub fn refuse_path_row_honest(row: &McpTrustRefusePathRow) -> bool {
        let args = args_for_row(row);
        let result = check_semantic_agent_trust(row.tool_name, &args);
        if row.expect_admit {
            result.is_ok()
        } else {
            matches!(result, Err(TrustError::ScopeNotPermitted { .. }))
                || matches!(result, Err(TrustError::AttestationRevoked { .. }))
        }
    }

    /// Whether all 6 typed refuse-path rows match expected admit/refuse posture.
    #[must_use]
    pub fn sec_mcp_wrap_refuse_path_matrix_honest() -> bool {
        REFUSE_PATH_MATRIX.len() == REFUSE_PATH_MATRIX_ROW_COUNT
            && REFUSE_PATH_MATRIX.iter().all(refuse_path_row_honest)
    }
}

#[cfg(feature = "tool-propose-communicative-act")]
pub use refuse_matrix::{
    refuse_path_row_honest, sec_mcp_wrap_refuse_path_matrix_honest, McpTrustRefusePathRow,
    REFUSE_PATH_MATRIX,
};

#[cfg(not(feature = "tool-propose-communicative-act"))]
/// Feature-off stub — live matrix requires `tool-propose-communicative-act`.
#[must_use]
pub fn sec_mcp_wrap_refuse_path_matrix_honest() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sec_mcp_wrap_z86_metadata_pins() {
        assert_eq!(JOB_ID, "AGAP-2033-SEC-MCP-WRAP");
        assert_eq!(DEEPEN_JOB_ID, "AGAP-2350-SEC-MCP-WRAP");
        assert_eq!(COMPOSER_Z86_JOB_ID, "FLEET-COMPOSER-Z86-SEC-MCP-WRAP");
        assert_eq!(COMPOSER_Z86_WAVE_SLOT, "Z86");
        assert!(COMPOSER_Z86_RECEIPT_PATH.contains("COMPOSER_Z86_1232"));
        assert!(PRIOR_Y52_RECEIPT_PATH.contains("COMPOSER_Y52_0808"));
        assert!(PRIOR_Y51_RECEIPT_PATH.contains("COMPOSER_Y51_0808"));
        assert_eq!(REFUSE_PATH_MATRIX_ROW_COUNT, 6);
        assert_eq!(WIRE_HOPS.len(), 6);
    }

    #[test]
    fn sec_mcp_wrap_wire_hops_ordinal_pin() {
        for (idx, hop) in WIRE_HOPS.iter().enumerate() {
            assert_eq!(hop.ordinal, (idx + 1) as u8);
            assert!(!hop.surface.is_empty());
            assert!(!hop.role.is_empty());
        }
    }

    #[test]
    fn sec_mcp_wrap_gateway_stdio_exec_honest_open() {
        assert!(!gateway_stdio_exec_trust_pre_check_wired());
        assert!(!sec_mcp_wrap_production_wired());
        assert!(GATEWAY_STDIO_EXEC_OWNER.contains("mcp_stdio_exec_trust_pre_check_wired"));
    }

    #[cfg(feature = "tool-propose-communicative-act")]
    #[test]
    fn sec_mcp_wrap_refuse_path_matrix_six_rows_honest() {
        assert_eq!(REFUSE_PATH_MATRIX.len(), REFUSE_PATH_MATRIX_ROW_COUNT);
        for row in REFUSE_PATH_MATRIX {
            assert!(refuse_path_row_honest(row), "row {} failed", row.row_id);
        }
        assert!(sec_mcp_wrap_refuse_path_matrix_honest());
    }

    #[cfg(feature = "tool-propose-communicative-act")]
    #[test]
    fn sec_mcp_wrap_surface_wired_default_profile() {
        let probe = sec_mcp_wrap_probe();
        assert!(probe.refuse_path_matrix_honest);
        assert_eq!(probe.wire_hop_count, 6);
        assert!(sec_mcp_wrap_surface_wired(&probe));
        assert!(matches!(probe.closure, SecMcpWrapClosure::Partial));
    }

    #[cfg(feature = "tool-propose-communicative-act")]
    #[test]
    fn sec_mcp_wrap_close_eligible_without_production_flip() {
        let probe = sec_mcp_wrap_probe();
        assert!(!probe.gateway_stdio_exec_trust_pre_check_wired);
        assert!(!probe.production_wired);
        assert!(sec_mcp_wrap_close_eligible(&probe));
    }

    #[cfg(not(feature = "tool-propose-communicative-act"))]
    #[test]
    fn sec_mcp_wrap_feature_off_matrix_stub_honest() {
        assert!(!sec_mcp_wrap_refuse_path_matrix_honest());
        let probe = sec_mcp_wrap_probe();
        assert!(matches!(probe.closure, SecMcpWrapClosure::Failed));
        assert!(!sec_mcp_wrap_close_eligible(&probe));
    }
}
