// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Shared MCP tool execution — hand-rolled stdio and `rmcp-wire` parity path.
//!
//! ADDITIVE (Stage S3): physics truth delegated to `umst-cli` / `umst-concrete-cartridge`.

use serde_json::{json, Value};
use umst_cli::canonical::canonical_json_bytes;
use umst_cli::cli::{
    mix_spec_from_json_value, predict_with_options, serialize_prediction, MixSpec, PredictOptions,
    PredictionWireVersion,
};
use umst_concrete_cartridge::calibration::{Profile, BUNDLED_PROFILE_IDS};

#[cfg(feature = "agent-layer")]
use umst_concrete_cartridge::research::{
    append_gate_reject_jsonl, estimate_mi_bits_from_mix, gate_check_mix_result,
    synthetic_observed_at, ProvenanceClock,
};

/// Tool text payload returned by shared handlers (no JSON-RPC framing).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Algebraic sum type for MCP tool body + isError bit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolPayload {
    pub text: String,
    pub is_error: bool,
}

/// Build structured `agent_error.v1` tool payload.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Recoverable tool failure wire; not physics gate.
#[must_use]
pub fn agent_error_payload(
    code: &str,
    message: impl Into<String>,
    remediation: &str,
) -> ToolPayload {
    let body = json!({
        "agent_error": {
            "schema_version": "agent_error.v1",
            "code": code,
            "message": message.into(),
            "remediation": remediation,
        }
    });
    ToolPayload {
        text: serde_json::to_string_pretty(&body).unwrap_or_default(),
        is_error: true,
    }
}

/// `umst_profiles` — bundled calibration ids (sorted).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Read-only catalog listing; physics on predict/gate tools.
#[must_use]
pub fn exec_umst_profiles() -> ToolPayload {
    let mut ids = BUNDLED_PROFILE_IDS.to_vec();
    ids.sort_unstable();

    let profs: Vec<Value> = ids
        .iter()
        .map(|pid| {
            let desc = umst_concrete_cartridge::calibration::profile_descriptions()
                .get(pid)
                .copied()
                .unwrap_or("no description");
            json!({ "id": pid, "description": desc })
        })
        .collect();

    ToolPayload {
        text: serde_json::to_string_pretty(&profs).unwrap_or_default(),
        is_error: false,
    }
}

/// `umst_predict` — constitutive envelope (same path as hand-rolled `main`).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Delegates to cartridge `predict_with_options` / `serialize_prediction`.
#[must_use]
pub fn exec_umst_predict(args: &Value) -> ToolPayload {
    let profile_id = args
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let compare = args
        .get("compare_homogeneous")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    let canon = args
        .get("canonical")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    let schema_version = args.get("schema_version").and_then(|x| x.as_str());
    let wire = match schema_version.unwrap_or("v2") {
        "v1" | "V1" => PredictionWireVersion::V1,
        _ => PredictionWireVersion::V2,
    };

    let mix_v = args.get("mix").cloned().unwrap_or(json!({}));
    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => {
            return agent_error_payload(
                "profile_load_fail",
                format!("profile load error: {e}"),
                "Call umst_profiles for bundled ids (e.g. default, uci_d1) or fix the profile argument.",
            );
        }
    };
    let mut spec: MixSpec = match mix_spec_from_json_value(mix_v.clone()) {
        Ok(s) => s,
        Err(e) => {
            return agent_error_payload(
                "mix_parse_fail",
                format!("mix parse error: {e}"),
                "Use rational strings like \"9/20\" for w_c and temperature_k; see contribution.v1 schema.",
            );
        }
    };
    spec.profile_name = profile_id.to_string();

    let bundle = match predict_with_options(
        &profile,
        &spec,
        PredictOptions {
            compare_homogeneous: compare,
        },
    ) {
        Ok(b) => b,
        Err(e) => {
            return agent_error_payload(
                "predict_fail",
                format!("predict error: {e}"),
                "Verify mix fields and profile calibration; see umst_predict schema.",
            );
        }
    };
    let out = match serialize_prediction(&bundle, wire) {
        Ok(v) => v,
        Err(e) => {
            return agent_error_payload(
                "serialize_fail",
                format!("serialize_prediction: {e}"),
                "Verify mix fields and profile calibration; see umst_predict schema.",
            );
        }
    };

    if canon {
        let bytes = match canonical_json_bytes(&out) {
            Ok(b) => b,
            Err(e) => {
                return agent_error_payload(
                    "canonical_json_fail",
                    format!("canonical JSON: {e}"),
                    "Retry without canonical=true or fix prediction output shape.",
                );
            }
        };
        let escaped =
            serde_json::to_string(&String::from_utf8_lossy(&bytes).to_string()).unwrap_or_default();
        return ToolPayload {
            text: escaped,
            is_error: false,
        };
    }

    ToolPayload {
        text: serde_json::to_string_pretty(&out).unwrap_or_else(|_| "{}".to_string()),
        is_error: false,
    }
}

/// `umst_gate_check` — thermodynamic admissibility (default provenance clock).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Same physics as `AgentSession::gate_check` without session carrier.
#[cfg(feature = "agent-layer")]
#[must_use]
pub fn exec_umst_gate_check_pure(args: &Value) -> ToolPayload {
    let profile_id = args
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let mix = match args.get("mix") {
        Some(m) => m.clone(),
        None => {
            return agent_error_payload(
                "missing_argument",
                "missing mix",
                "Supply mix with rational fields (w_c, temperature_k, …) per contribution.v1.",
            );
        }
    };
    let explain = args
        .get("explain")
        .and_then(|x| x.as_bool())
        .unwrap_or(true);
    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => {
            return agent_error_payload(
                "profile_load_fail",
                format!("profile load error: {e}"),
                "Call umst_profiles for bundled ids or use profile: \"default\".",
            );
        }
    };
    let clock = ProvenanceClock::default();
    let observed = synthetic_observed_at(clock.sequence());
    let result = gate_check_mix_result(&profile, &mix, explain, observed);
    if let Some(ref row) = result.gate_reject {
        let _ = append_gate_reject_jsonl(row, None);
    }
    let is_error = !result.gate_summary.admissible;
    ToolPayload {
        text: serde_json::to_string_pretty(&result).unwrap_or_default(),
        is_error,
    }
}

/// `umst_gate_check` — thermodynamic admissibility (agent-layer session).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Session-threaded gate on `gate_check_mix_result`.
#[cfg(feature = "agent-layer")]
#[must_use]
pub fn exec_umst_gate_check(
    args: &Value,
    session: &crate::agent_layer::AgentSession,
) -> ToolPayload {
    let profile_id = args
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let mix = match args.get("mix") {
        Some(m) => m.clone(),
        None => {
            return agent_error_payload(
                "missing_argument",
                "missing mix",
                "Supply mix with rational fields (w_c, temperature_k, …) per contribution.v1.",
            );
        }
    };
    let explain = args
        .get("explain")
        .and_then(|x| x.as_bool())
        .unwrap_or(true);
    let profile = match Profile::load_bundled(profile_id) {
        Ok(p) => p,
        Err(e) => {
            return agent_error_payload(
                "profile_load_fail",
                format!("profile load error: {e}"),
                "Call umst_profiles for bundled ids or use profile: \"default\".",
            );
        }
    };
    let result = session.gate_check(&profile, &mix, explain);
    let is_error = !result.gate_summary.admissible;
    ToolPayload {
        text: serde_json::to_string_pretty(&result).unwrap_or_default(),
        is_error,
    }
}

/// `umst_mi_estimate` — advisory MI surrogate (no session state).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Advisory enrichment; not admissibility gate.
#[cfg(feature = "agent-layer")]
#[must_use]
pub fn exec_umst_mi_estimate_pure(args: &Value) -> ToolPayload {
    let mix = match args.get("mix") {
        Some(m) => m.clone(),
        None => {
            return agent_error_payload(
                "missing_argument",
                "missing mix",
                "Supply mix with rational fields for MI advisory estimate.",
            );
        }
    };
    let out = json!({
        "mi_bits_est": estimate_mi_bits_from_mix(&mix),
        "advisory": true,
    });
    ToolPayload {
        text: serde_json::to_string_pretty(&out).unwrap_or_default(),
        is_error: false,
    }
}

/// `umst_mi_estimate` — advisory MI surrogate (agent-layer session alias).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Delegates to pure MI path (session carries no MI state).
#[cfg(feature = "agent-layer")]
#[must_use]
pub fn exec_umst_mi_estimate(
    args: &Value,
    session: &crate::agent_layer::AgentSession,
) -> ToolPayload {
    let _ = session;
    exec_umst_mi_estimate_pure(args)
}

/// Golden S0 parity tool names (Stage S3).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Test fixture index; not physics.
pub const PARITY_TOOL_NAMES: &[&str] = &[
    "umst_profiles",
    "umst_gate_check",
    "umst_mi_estimate",
    "umst_predict",
];
