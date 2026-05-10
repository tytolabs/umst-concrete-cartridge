// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Closed-form hydration scalars used by [`crate::homogeneous`] routing.
//!
//! The asymptotic ultimate degree of hydration follows the Powers–Brownyard-style closure
//! documented in [`docs/Constitutive-Equations.md`](../../docs/Constitutive-Equations.md).
//! Calibrated rate parameters are not duplicated here; callers pass multipliers from
//! [`crate::calibration::Profile`].

#![allow(clippy::excessive_precision)]

/// formal_anchor: literature://Mills-1966-gel-stiffness-closure
/// formal_status: Literature
/// formal_axioms: NONE
/// formal_citation: "Mills (1966); OPC gel stiffness / ultimate hydration cap closure used in routing"
/// formal_form: "α_inf(w/c) = 1.031·w/c / (0.194 + w/c)"
///
/// Asymptotic ultimate degree of hydration α∞(w/c) for OPC-dominated pastes (Mills 1966 closure used in routing).
#[must_use]
pub fn ultimate_doh_wc(w_c: f32) -> f32 {
    1.031 * w_c / (0.194 + w_c)
}

/// formal_anchor: lean://umst-formal/Lean/Powers.lean#powers_monotone
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
///
/// Calibrated hydration degree α(t) with Arrhenius temperature factor and SCM slowdown.
/// `k_ref_multiplier` folds dataset-specific `k_ref` scaling from the active profile.
#[must_use]
pub fn hydration_degree_calibrated(
    age_days: f32,
    temp_c: f32,
    scm_ratio: f32,
    k_ref_multiplier: f32,
) -> f32 {
    let alpha_max = 0.95 - scm_ratio * 0.15;
    let k_ref = 0.55 * k_ref_multiplier;
    let t_ref_k = 293.15_f32;
    let t_k = temp_c + 273.15;
    let e_over_r = 5000.0;
    let temp_factor = (e_over_r * (1.0 / t_ref_k - 1.0 / t_k)).exp();
    let scm_factor = 1.0 - scm_ratio * 0.4;
    let k = k_ref * temp_factor * scm_factor;
    let alpha = alpha_max * (1.0 - (-k * age_days.sqrt()).exp());
    alpha.clamp(0.0, 1.0)
}
