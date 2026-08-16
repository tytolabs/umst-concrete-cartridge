// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Closed-form hydration scalars used by [`crate::homogeneous`] routing.
//!
//! The asymptotic ultimate degree of hydration follows the Powers–Brownyard-style closure
//! documented in [`docs/Constitutive-Equations.md`](../../docs/Constitutive-Equations.md).
//! Calibrated rate parameters are not duplicated here; callers pass multipliers from
//! [`crate::calibration::Profile`].
//!
//! T2-S6 dup-ψ inventory — card `g_spawn_i_s6_form_2054`. Under `b1-delegate`, thin shims
//! re-export [`crate::chem_adapter`] directly (cfg-gated bypass).
//!
//! R-H3 monolith cfg-gate honesty @ `g_spawn_i_rh2h6_impl_0721` — thin delegate retained
//! until T3 archive.
//!
//! R-H3 monolith cfg-gate honesty @ `g_spawn_i_rh2h6_impl_0721` — thin delegate retained
//! until T3 archive; consumer SSOT routes through `b1-delegate` builds.

#![allow(clippy::excessive_precision)]

/// formal_anchor: literature://Mills-1966-gel-stiffness-closure
/// formal_status: Literature
/// formal_citation: "Mills (1966); OPC gel stiffness / ultimate hydration cap closure used in routing"
/// formal_form: "α_inf(w/c) = 1.031·w/c / (0.194 + w/c)"
///
/// Asymptotic ultimate degree of hydration α∞(w/c) for OPC-dominated pastes (Mills 1966 closure used in routing).
/// T2-S6 dup-ψ allowlist [PARTIAL] `ultimate_doh_wc` — cfg-gated thin shim @ `g_spawn_i_s6_form_2054`
/// consumer SSOT: `umst-chem::ultimate_degree_of_hydration` via `chem_adapter`
#[cfg(not(feature = "b1-delegate"))]
#[must_use]
pub fn ultimate_doh_wc(w_c: f32) -> f32 {
    crate::chem_adapter::ultimate_degree_of_hydration_f32(w_c)
}

#[cfg(feature = "b1-delegate")]
pub use crate::chem_adapter::ultimate_degree_of_hydration_f32 as ultimate_doh_wc;

/// formal_anchor: empirical://datasets/hydration-kinetics-calibration-grid.v1.csv
/// formal_status: Empirical
/// formal_dataset: "profile-scaled k_ref hydration grid"
/// formal_citation: "Mills (1966) ultimate cap with stretched-exponential √t kinetics and Arrhenius temperature factor (calibrated multipliers from profile TOML)"
/// formal_envelope: "tests/hydration.rs::powers_doh_envelope"
///
/// Calibrated hydration degree α(t) with Arrhenius temperature factor and SCM slowdown.
/// T2-S6 dup-ψ allowlist [PARTIAL] `hydration_degree_calibrated` — R-H3 cfg-gate thin shim
/// @ `g_spawn_i_rh2h6_impl_0721` · consumer: `chem_inject_module::hydration_alpha_from_chem`
/// `k_ref_multiplier` folds dataset-specific `k_ref` scaling from the active profile.
#[cfg(not(feature = "b1-delegate"))]
#[must_use]
pub fn hydration_degree_calibrated(
    age_days: f32,
    temp_c: f32,
    scm_ratio: f32,
    k_ref_multiplier: f32,
) -> f32 {
    crate::chem_adapter::hydration_degree_calibrated(age_days, temp_c, scm_ratio, k_ref_multiplier)
}

#[cfg(feature = "b1-delegate")]
pub use crate::chem_adapter::hydration_degree_calibrated;
