// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Frugal MI bits surrogate for gate_check (Landauer envelope advisory, not admissibility).

use super::contribution::field_as_rational;
use crate::calibration::Profile;
use serde_json::Value;

const LN2: f64 = std::f64::consts::LN_2;

fn float_to_rational(x: f64) -> String {
    let num = (x * 10_000.0).round() as i64;
    format!("{num}/10000")
}

/// L1 mix displacement → capped MI bits estimate (Landauer advisory, not gate).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Epistemic surrogate for MCP enrichment; not admissibility.
#[must_use]
pub fn estimate_mi_bits_rational(mix_json: &Value, profile: &Profile) -> Option<String> {
    let w_c = field_as_rational(mix_json, "w_c")?;
    let t_k = field_as_rational(mix_json, "temperature_k").unwrap_or(293.15);
    let ref_wc = (profile.regime.w_c_min + profile.regime.w_c_max) / 2.0;
    let ref_t = 293.15_f64;
    let l1 = (w_c - ref_wc).abs() + ((t_k - ref_t) / 100.0).abs();
    let bits = (l1 * 2.0).min(LN2).max(0.0);
    if bits <= f64::EPSILON {
        return None;
    }
    Some(float_to_rational(bits))
}

/// MI bits estimate from mix JSON only (no profile center).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: `umst_mi_estimate` advisory wire; histogram MI on manifold PPO path.
#[must_use]
pub fn estimate_mi_bits_from_mix(mix_json: &Value) -> Option<String> {
    let w_c = field_as_rational(mix_json, "w_c")?;
    let ref_wc = 0.45_f64;
    let bits = ((w_c - ref_wc).abs() * 2.0).min(LN2);
    if bits <= f64::EPSILON {
        return None;
    }
    Some(float_to_rational(bits))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn mi_rational_parses() {
        use crate::research::contribution::rational_to_f64;
        let mix = json!({ "w_c": "11/20", "temperature_k": "29315/100" });
        let s = estimate_mi_bits_from_mix(&mix).expect("bits");
        assert!(rational_to_f64(&s).is_some());
    }
}
