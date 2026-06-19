// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Promotion policy validation (`governance/promotion_policy.yaml`).

use serde::Deserialize;
use thiserror::Error;

/// Promotion policy YAML parse / validation failures.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Human-gated calibration governance; not MCP admissibility.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PolicyError {
    #[error("yaml: {0}")]
    Yaml(String),
    #[error("policy: {0}")]
    Policy(String),
}

/// Parsed `promotion_policy.v1` wire from governance YAML.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Operator policy record; hold-out metrics are empirical gates only.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PromotionPolicy {
    pub schema_version: String,
    pub policy_id: String,
    pub min_holdout_rows: u32,
    pub max_rmse_mpa: f64,
    pub require_human_approval: bool,
    pub allowed_stamp_tiers: Vec<String>,
}

/// Pure: parse and validate promotion policy YAML.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: YAML ingest for propose-promotion CLI; never MCP-exposed.
pub fn parse_promotion_policy_yaml(text: &str) -> Result<PromotionPolicy, PolicyError> {
    let policy: PromotionPolicy =
        serde_yaml::from_str(text).map_err(|e| PolicyError::Yaml(e.to_string()))?;
    validate_promotion_policy(&policy)?;
    Ok(policy)
}

/// Pure: structural validation of promotion policy fields.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Schema shape check on policy wire; not thermodynamic gate.
pub fn validate_promotion_policy(policy: &PromotionPolicy) -> Result<(), PolicyError> {
    if policy.schema_version != "promotion_policy.v1" {
        return Err(PolicyError::Policy(
            "schema_version must be promotion_policy.v1".into(),
        ));
    }
    if policy.min_holdout_rows == 0 {
        return Err(PolicyError::Policy("min_holdout_rows must be > 0".into()));
    }
    if policy.max_rmse_mpa <= 0.0 {
        return Err(PolicyError::Policy("max_rmse_mpa must be > 0".into()));
    }
    if policy.allowed_stamp_tiers.is_empty() {
        return Err(PolicyError::Policy(
            "allowed_stamp_tiers must be non-empty".into(),
        ));
    }
    Ok(())
}

/// Whether live thermodynamic witness is enabled (`UMST_UCRS_WITNESS=live`).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Env IO at session boundary; promotion policy gate only.
#[must_use]
pub fn live_witness_enabled() -> bool {
    std::env::var("UMST_UCRS_WITNESS").as_deref() == Ok("live")
}

/// Track A promotion: when live witness is on, memory rows must carry Tier-2 stamps.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Human-gated calibration policy; not thermodynamic admissibility.
pub fn validate_track_a_stamp_tier(stamp_tier: &str) -> Result<(), PolicyError> {
    if live_witness_enabled() && stamp_tier != "UcrsTier2" {
        return Err(PolicyError::Policy(
            "UMST_UCRS_WITNESS=live requires stamp_tier UcrsTier2 for Track A promotion"
                .into(),
        ));
    }
    if !live_witness_enabled() && stamp_tier == "Absent" {
        return Err(PolicyError::Policy(
            "stamp_tier Absent is not eligible for promotion".into(),
        ));
    }
    Ok(())
}

/// Pure: hold-out RMSE gate for propose-promotion dry-run.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Calibration hold-out metric; human review still required.
#[must_use]
pub fn holdout_rmse_passes(observed: &[f64], predicted: &[f64], max_rmse: f64) -> bool {
    if observed.len() != predicted.len() || observed.is_empty() {
        return false;
    }
    let mse: f64 = observed
        .iter()
        .zip(predicted.iter())
        .map(|(o, p)| {
            let d = o - p;
            d * d
        })
        .sum::<f64>()
        / observed.len() as f64;
    mse.sqrt() <= max_rmse
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static WITNESS_ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn parse_bundled_policy_shape() {
        let text = include_str!("../../../../governance/promotion_policy.yaml");
        let p = parse_promotion_policy_yaml(text).expect("policy");
        assert_eq!(p.schema_version, "promotion_policy.v1");
    }

    #[test]
    fn track_a_requires_ucrs_tier2_when_live_witness() {
        let _guard = WITNESS_ENV_LOCK.lock().expect("witness env lock");
        std::env::set_var("UMST_UCRS_WITNESS", "live");
        assert!(validate_track_a_stamp_tier("UcrsTier2").is_ok());
        assert!(validate_track_a_stamp_tier("Synthetic").is_err());
        std::env::remove_var("UMST_UCRS_WITNESS");
    }

    #[test]
    fn track_a_allows_synthetic_when_not_live() {
        let _guard = WITNESS_ENV_LOCK.lock().expect("witness env lock");
        std::env::set_var("UMST_UCRS_WITNESS", "synthetic");
        assert!(validate_track_a_stamp_tier("Synthetic").is_ok());
        std::env::remove_var("UMST_UCRS_WITNESS");
    }
}
