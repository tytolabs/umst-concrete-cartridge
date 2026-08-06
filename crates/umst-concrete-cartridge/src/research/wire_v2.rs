// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! `wire.v2` integer-only UCRS observation stamp mapping.

use super::types::ObservedAt;
use serde::{Deserialize, Serialize};

/// observed_at.v2 schema id wire constant.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Schema version string for integer-only UCRS wire.
pub const OBSERVED_AT_V2_SCHEMA: &str = "observed_at.v2";

/// Integer-only UCRS wire (`wire.v2` policy — no f64 on public fields).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: UCRS wire functor; monotonicity checked on v1 `provenance` path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedAtV2 {
    pub schema_version: String,
    pub stamp_tier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ucrs_seq: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_entropy_bits_q: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_entropy_bits_scale: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_head_bits_q: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_head_bits_scale: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wall_ms: Option<u64>,
}

/// Pure: v1 `observed_at` → v2 integer wire.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Field-preserving projection to observed_at.v2; no new stamp semantics.
#[must_use]
pub fn observed_at_to_v2(obs: &ObservedAt) -> ObservedAtV2 {
    ObservedAtV2 {
        schema_version: OBSERVED_AT_V2_SCHEMA.into(),
        stamp_tier: obs.stamp_tier.clone(),
        ucrs_seq: obs.ucrs_seq,
        phase_entropy_bits_q: obs.phase_entropy_bits_q,
        phase_entropy_bits_scale: obs.phase_entropy_bits_scale,
        credit_head_bits_q: obs.credit_head_bits_q,
        credit_head_bits_scale: obs.credit_head_bits_scale,
        wall_ms: obs.wall_ms,
    }
}

#[cfg(feature = "ucrs-provenance")]
/// Map public `UcrsObservedAt` through v2 wire when `ucrs-provenance` is enabled.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: UCRS crate → v2 wire functor; monotonicity on `is_monotonic_after`.
#[must_use]
pub fn ucrs_observed_at_to_v2(u: &umst_ucrs::shared_types::observation::UcrsObservedAt) -> ObservedAtV2 {
    use umst_ucrs::shared_types::observation::WIRE_SCALE;
    ObservedAtV2 {
        schema_version: OBSERVED_AT_V2_SCHEMA.into(),
        stamp_tier: u.stamp_tier.as_wire_str().to_string(),
        ucrs_seq: u.ucrs_seq,
        phase_entropy_bits_q: u.phase_entropy_bits_q,
        phase_entropy_bits_scale: u.phase_entropy_bits_scale.or(Some(WIRE_SCALE)),
        credit_head_bits_q: u.credit_head_bits_q,
        credit_head_bits_scale: u.credit_head_bits_scale.or(Some(WIRE_SCALE)),
        wall_ms: u.wall_ms,
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "ucrs-provenance")]
    use super::*;

    #[cfg(feature = "ucrs-provenance")]
    #[test]
    fn ucrs_observed_at_roundtrip_v2() {
        use umst_ucrs::shared_types::observation::UcrsObservedAt;
        let u = UcrsObservedAt::synthetic(7, 0.5);
        let v2 = ucrs_observed_at_to_v2(&u);
        assert_eq!(v2.ucrs_seq, Some(7));
        assert_eq!(v2.stamp_tier, "Synthetic");
        assert_eq!(v2.phase_entropy_bits_q, u.phase_entropy_bits_q);
        assert_eq!(v2.schema_version, OBSERVED_AT_V2_SCHEMA);
    }
}
