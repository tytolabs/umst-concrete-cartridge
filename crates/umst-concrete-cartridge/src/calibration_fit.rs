// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Single-parameter θ calibration on yield stress (δ=0 until ≥3 mixes).
//!
//! Pure maps **predicted τ₀ → calibrated τ₀**; no hidden state. Used by Track A dual gate and
//! [`crate::pipeline::physical_summary::physical_result_from_report`] before printability witnesses.

use serde::Deserialize;

/// Rheology bias block in profile TOML (`[rheology_calibration]`).
/// formal_anchor: empirical://datasets/printability-rheology-yield-proxy.v1.csv
/// formal_status: Empirical
/// formal_dataset: "Tyto mortar rheology θ bias (calibration_fit)"
/// formal_citation: "In-house Tyto mortar yield proxy calibration"
/// formal_envelope: "tests/calibration_tyto_mortar.rs"
#[derive(Debug, Clone, Deserialize)]
pub struct RheologyCalibrationBlock {
    #[serde(default = "default_theta")]
    pub theta_tau0_bias: f64,
    #[serde(default)]
    pub delta_structuration: f64,
    /// Optional measured τ₀ band [Pa] for single-mix θ fit (under-identified until ≥3 mixes).
    #[serde(default)]
    pub measured_tau0_lo_pa: Option<f32>,
    #[serde(default)]
    pub measured_tau0_hi_pa: Option<f32>,
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_theta() -> f64 {
    1.0
}

/// Apply θ to a predicted τ₀ (Pa). δ reserved for multi-mix structuration fits.
/// formal_anchor: empirical://datasets/printability-rheology-yield-proxy.v1.csv
/// formal_status: Empirical
/// formal_dataset: "Tyto mortar rheology θ bias (calibration_fit)"
/// formal_citation: "In-house Tyto mortar yield proxy calibration"
/// formal_envelope: "tests/calibration_tyto_mortar.rs"
#[must_use]
pub fn apply_tau0_calibration(predicted_pa: f32, block: Option<&RheologyCalibrationBlock>) -> f32 {
    let theta = block.map(|b| b.theta_tau0_bias as f32).unwrap_or(1.0);
    (predicted_pa * theta).max(0.0)
}

/// Fit θ so `predicted * θ` lies in `[measured_lo, measured_hi]` when band is admissible.
/// formal_anchor: empirical://datasets/printability-rheology-yield-proxy.v1.csv
/// formal_status: Empirical
/// formal_dataset: "Tyto mortar rheology θ bias (calibration_fit)"
/// formal_citation: "In-house Tyto mortar yield proxy calibration"
/// formal_envelope: "tests/calibration_tyto_mortar.rs"
#[must_use]
pub fn fit_theta_tau0_single_mix(predicted_pa: f32, measured_lo: f32, measured_hi: f32) -> f32 {
    if !predicted_pa.is_finite() || predicted_pa <= 1.0 {
        return 1.0;
    }
    if measured_lo <= 0.0 || measured_hi <= measured_lo {
        return 1.0;
    }
    let mid = 0.5 * (measured_lo + measured_hi);
    (mid / predicted_pa).clamp(1e-6, 4.0)
}

/// Effective multiplicative θ: explicit profile bias, or auto-fit from measured band when present.
/// formal_anchor: empirical://datasets/printability-rheology-yield-proxy.v1.csv
/// formal_status: Empirical
/// formal_dataset: "Tyto mortar rheology θ bias (calibration_fit)"
/// formal_citation: "In-house Tyto mortar yield proxy calibration"
/// formal_envelope: "tests/calibration_tyto_mortar.rs"
#[must_use]
pub fn effective_theta_tau0(predicted_pa: f32, block: Option<&RheologyCalibrationBlock>) -> f32 {
    let Some(b) = block else {
        return 1.0;
    };
    if (b.theta_tau0_bias - 1.0).abs() > f64::EPSILON {
        return b.theta_tau0_bias as f32;
    }
    if let (Some(lo), Some(hi)) = (b.measured_tau0_lo_pa, b.measured_tau0_hi_pa) {
        if lo > 0.0 && hi > lo {
            return fit_theta_tau0_single_mix(predicted_pa, lo, hi);
        }
    }
    1.0
}

/// Calibrated τ₀ using [`effective_theta_tau0`].
/// formal_anchor: empirical://datasets/printability-rheology-yield-proxy.v1.csv
/// formal_status: Empirical
/// formal_dataset: "Tyto mortar rheology θ bias (calibration_fit)"
/// formal_citation: "In-house Tyto mortar yield proxy calibration"
/// formal_envelope: "tests/calibration_tyto_mortar.rs"
#[must_use]
pub fn calibrated_tau0_pa(predicted_pa: f32, block: Option<&RheologyCalibrationBlock>) -> f32 {
    let theta = effective_theta_tau0(predicted_pa, block);
    (predicted_pa * theta).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theta_brackets_measured_band() {
        let theta = fit_theta_tau0_single_mix(2000.0, 800.0, 1200.0);
        let calibrated = 2000.0 * theta;
        assert!(calibrated >= 800.0 && calibrated <= 1200.0);
    }

    #[test]
    fn theta_scales_down_overshot_yodel() {
        let theta = fit_theta_tau0_single_mix(1_862_763.0, 180.0, 360.0);
        let calibrated = 1_862_763.0 * theta;
        assert!(calibrated >= 180.0 && calibrated <= 360.0);
    }
}
