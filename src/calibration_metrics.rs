// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Shared regression metrics for headline CSV calibration — used by [`tests/calibration/dataset_metrics`]
//! and the `calibration_report` binary so MAE / RMSE / R² definitions cannot drift.

/// formal_anchor: lean://umst-formal/Lean/OrderStatisticsBand.lean#order_statistic_concentration
/// formal_status: Mechanised
/// formal_axioms: NONE
///
/// Ordinary least-squares aggregates over paired model predictions `ŷᵢ` and observations `yᵢ`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegressionMetrics {
    pub n: usize,
    pub mae: f64,
    pub rmse: f64,
    pub r2: f64,
    pub max_abs_error: f64,
}

/// formal_anchor: lean://umst-formal/Lean/OrderStatisticsBand.lean#order_statistic_concentration
/// formal_status: Mechanised
/// formal_axioms: NONE
///
/// # Panics
/// Panics if `predicted.len() != observed.len()` or either slice is empty.
#[must_use]
pub fn regression_metrics(predicted: &[f64], observed: &[f64]) -> RegressionMetrics {
    assert_eq!(
        predicted.len(),
        observed.len(),
        "predicted and observed must have the same length"
    );
    let n = predicted.len();
    assert!(n > 0, "empty slices");

    let mut sum_abs = 0.0_f64;
    let mut sum_sq = 0.0_f64;
    let mut sum_y = 0.0_f64;
    let mut max_err = 0.0_f64;
    for (pi, yi) in predicted.iter().zip(observed) {
        let ae = (pi - yi).abs();
        sum_abs += ae;
        sum_sq += (pi - yi).powi(2);
        max_err = max_err.max(ae);
        sum_y += yi;
    }
    let nf = n as f64;
    let mae = sum_abs / nf;
    let rmse = (sum_sq / nf).sqrt();
    let mean_y = sum_y / nf;
    let ss_tot: f64 = observed.iter().map(|yi| (yi - mean_y).powi(2)).sum();
    let ss_res: f64 = predicted
        .iter()
        .zip(observed)
        .map(|(pi, yi)| (yi - pi).powi(2))
        .sum();
    let r2 = if ss_tot <= 1e-12 {
        0.0
    } else {
        1.0 - ss_res / ss_tot
    };

    RegressionMetrics {
        n,
        mae,
        rmse,
        r2,
        max_abs_error: max_err,
    }
}
