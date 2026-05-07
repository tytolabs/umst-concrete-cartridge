// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: LicenseRef-Proprietary
//
// MaOS — Material Agnostic Operating System
// ShrinkageEngine: Jensen-Hansen Autogenous Shrinkage Model
//
// This file is part of MaOS, developed by Studio TYTO (Est. 2018), Chennai, India.
// For licensing terms, see the LICENSE file in the project root.

//! # Shrinkage Engine
//!
//! Implements autogenous shrinkage based on Jensen & Hansen (2001) self-desiccation model.
//!
//! ## Physical Basis
//!
//! Autogenous shrinkage occurs in sealed concrete due to:
//! 1. Chemical shrinkage: Volume reduction during hydration (~6.4 mL/100g cement)
//! 2. Self-desiccation: Internal RH drop when capillary water is consumed
//! 3. Meniscus formation in pores: Capillary tension causes bulk shrinkage
//!
//! ## Key Equations
//!
//! For w/c < 0.42 (self-desiccation regime):
//! - Internal RH: RH(α) = 1 - 0.20 × (α - α_sd) / (w/c × 3.15)
//! - Autogenous strain: ε_as = -β × (1 - RH)^n
//!
//! Where:
//! - α_sd: Hydration degree at self-desiccation onset (~w/c × 3.15 / 0.42)
//! - β: Amplitude factor (~1200 με for typical concrete)
//! - n: Exponent (typically 3)

use wasm_bindgen::prelude::*;

/// Result from shrinkage calculation
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct ShrinkageResult {
    /// Autogenous shrinkage strain (negative = shrinkage, με)
    pub autogenous_strain: f32,
    /// Chemical shrinkage (mL/100g cement hydrated)
    pub chemical_shrinkage: f32,
    /// Internal relative humidity (0-1)
    pub internal_rh: f32,
    /// Self-desiccation onset hydration degree
    pub alpha_sd: f32,
}

#[wasm_bindgen]
pub struct ShrinkageEngine;

#[wasm_bindgen]
impl ShrinkageEngine {
    /// Compute autogenous shrinkage using Jensen-Hansen model
    ///
    /// # Arguments
    /// * `w_c` - Water/cement ratio
    /// * `alpha` - Current hydration degree (0-1)
    /// * `cement_content_kg_m3` - Cement content in kg/m³
    ///
    /// # Returns
    /// ShrinkageResult with autogenous strain (negative = shrinkage)
    #[wasm_bindgen]
    pub fn compute_autogenous(w_c: f32, alpha: f32, cement_content_kg_m3: f32) -> ShrinkageResult {
        Self::compute_autogenous_full(w_c, alpha, cement_content_kg_m3, 0.0)
    }

    /// Compute with SCM effect
    #[wasm_bindgen]
    pub fn compute_autogenous_with_scm(
        w_c: f32,
        alpha: f32,
        cement_content_kg_m3: f32,
        scm_ratio: f32,
    ) -> ShrinkageResult {
        Self::compute_autogenous_full(w_c, alpha, cement_content_kg_m3, scm_ratio)
    }
}

impl ShrinkageEngine {
    /// Full autogenous shrinkage calculation with all parameters
    pub fn compute_autogenous_full(
        w_c: f32,
        alpha: f32,
        cement_content_kg_m3: f32,
        scm_ratio: f32,
    ) -> ShrinkageResult {
        // Edge case: invalid inputs
        if w_c <= 0.0 || alpha <= 0.0 {
            return ShrinkageResult {
                autogenous_strain: 0.0,
                chemical_shrinkage: 0.0,
                internal_rh: 1.0,
                alpha_sd: 0.0,
            };
        }

        // 1. Chemical shrinkage (Le Chatelier contraction)
        // ~6.4 mL per 100g cement hydrated (average for OPC)
        // SCM can modify this (silica fume increases it, fly ash decreases)
        let chemical_shrinkage_rate = 6.4 * (1.0 + 0.3 * scm_ratio); // mL/100g
        let hydrated_cement = cement_content_kg_m3 * alpha * 10.0; // 100g units
        let chemical_shrinkage = chemical_shrinkage_rate * hydrated_cement / 1000.0; // mL/L of concrete

        // 2. Self-desiccation threshold
        // At w/c < 0.42, not enough water for complete hydration
        // Self-desiccation becomes significant at lower w/c ratios
        let critical_wc = 0.42;
        let alpha_sd = if w_c < critical_wc {
            (w_c / critical_wc).min(1.0)
        } else {
            1.0 // No self-desiccation for high w/c
        };

        // 3. Internal relative humidity
        // Based on Powers model: RH drops as gel pores consume water
        // For low w/c, RH can drop to 0.75-0.85 at high hydration degrees
        //
        // Simplified model: RH = 1 - k × α × (1 - w/c/0.42)^0.5
        // where k depends on paste density
        let self_desiccation_potential = (1.0 - w_c / critical_wc).max(0.0).sqrt();
        let rh_drop = 0.25 * alpha * self_desiccation_potential;
        let internal_rh = (1.0 - rh_drop).max(0.75);

        // 4. Autogenous shrinkage strain (Jensen-Hansen type model)
        // ε_as = ε_as_ult × β(α)
        // where ε_as_ult depends on w/c and β(α) is development function
        //
        // Ultimate autogenous shrinkage (με) - literature values:
        // w/c = 0.25: -800 to -1200 με
        // w/c = 0.35: -400 to -600 με
        // w/c = 0.45: -150 to -250 με
        // w/c = 0.55: -50 to -100 με

        // Ultimate shrinkage as function of w/c (empirical fit)
        let eps_as_ult = if w_c < 0.30 {
            -1000.0 - 500.0 * (0.30 - w_c) / 0.05 // Up to -1500 for very low w/c
        } else if w_c < 0.42 {
            -600.0 - 400.0 * (0.42 - w_c) / 0.12 // -600 to -1000
        } else if w_c < 0.50 {
            -200.0 - 400.0 * (0.50 - w_c) / 0.08 // -200 to -600
        } else {
            -100.0 * (0.60 - w_c).max(0.0) / 0.10 // Very small for high w/c
        };

        // Development function: shrinkage develops with hydration
        // At α = 0: no shrinkage
        // At α = α_ult: full shrinkage
        // Use exponential form: β(α) = 1 - exp(-3α/α_ult)
        let alpha_ult = (w_c / 0.42).min(1.0);
        let development = if alpha_ult > 0.01 {
            (1.0 - (-3.0 * alpha / alpha_ult).exp()).min(1.0)
        } else {
            alpha.min(1.0)
        };

        // Paste volume effect: more paste = more shrinkage
        let paste_factor = (cement_content_kg_m3 / 350.0).powf(0.5);

        // SCM effect: silica fume increases autogenous shrinkage
        let scm_factor = 1.0 + 0.3 * scm_ratio;

        // Final autogenous shrinkage
        let autogenous_strain = eps_as_ult * development * paste_factor * scm_factor;

        // 5. Time-dependent development (not modeled here - use with age)
        // Full autogenous shrinkage develops over ~90-180 days

        ShrinkageResult {
            autogenous_strain, // με (microstrain)
            chemical_shrinkage,
            internal_rh,
            alpha_sd,
        }
    }

    /// Estimate total shrinkage (autogenous + drying)
    ///
    /// Drying shrinkage is additional when concrete is exposed to ambient RH
    pub fn compute_total_shrinkage(
        w_c: f32,
        alpha: f32,
        cement_content_kg_m3: f32,
        ambient_rh: f32,
        age_days: f32,
    ) -> f32 {
        let autogenous = Self::compute_autogenous(w_c, alpha, cement_content_kg_m3);

        // Drying shrinkage (simplified)
        // ε_dry = ε_dry_ult × (1 - RH_amb)^1.5 × time_factor
        let eps_dry_ult = 600.0; // με ultimate drying shrinkage
        let time_factor = (age_days / (age_days + 35.0)).min(1.0); // Half-time ~35 days
        let drying_strain = -eps_dry_ult * (1.0 - ambient_rh).powf(1.5) * time_factor;

        autogenous.autogenous_strain + drying_strain
    }

    /// Compute ultimate autogenous shrinkage for a mix
    /// Useful for design predictions
    pub fn ultimate_autogenous_shrinkage(w_c: f32, cement_content_kg_m3: f32) -> f32 {
        // At ultimate hydration degree
        let alpha_ult = (w_c / 0.42).min(1.0);
        let result = Self::compute_autogenous(w_c, alpha_ult, cement_content_kg_m3);
        result.autogenous_strain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_shrinkage_at_zero_hydration() {
        let result = ShrinkageEngine::compute_autogenous(0.45, 0.0, 350.0);
        assert_eq!(result.autogenous_strain, 0.0);
        assert_eq!(result.internal_rh, 1.0);
    }

    #[test]
    fn test_high_wc_minimal_autogenous() {
        // High w/c = minimal autogenous shrinkage (internal RH stays high)
        let result = ShrinkageEngine::compute_autogenous(0.55, 0.8, 350.0);
        assert!(
            result.autogenous_strain.abs() < 50.0,
            "High w/c should have minimal autogenous shrinkage, got {}",
            result.autogenous_strain
        );
        assert!(
            result.internal_rh > 0.95,
            "High w/c should maintain high internal RH, got {}",
            result.internal_rh
        );
    }

    #[test]
    fn test_low_wc_significant_autogenous() {
        // Low w/c = significant autogenous shrinkage
        let result = ShrinkageEngine::compute_autogenous(0.35, 0.85, 400.0);
        assert!(
            result.autogenous_strain < -100.0,
            "Low w/c should have significant shrinkage, got {}",
            result.autogenous_strain
        );
        assert!(
            result.internal_rh < 0.95,
            "Low w/c should have RH drop, got {}",
            result.internal_rh
        );
    }

    #[test]
    fn test_shrinkage_increases_with_hydration() {
        let low_alpha = ShrinkageEngine::compute_autogenous(0.35, 0.5, 350.0);
        let high_alpha = ShrinkageEngine::compute_autogenous(0.35, 0.9, 350.0);

        assert!(
            high_alpha.autogenous_strain < low_alpha.autogenous_strain,
            "Shrinkage should increase (more negative) with hydration"
        );
    }

    #[test]
    fn test_chemical_shrinkage_proportional_to_hydration() {
        let low_alpha = ShrinkageEngine::compute_autogenous(0.45, 0.3, 350.0);
        let high_alpha = ShrinkageEngine::compute_autogenous(0.45, 0.9, 350.0);

        assert!(
            high_alpha.chemical_shrinkage > low_alpha.chemical_shrinkage * 2.5,
            "Chemical shrinkage should scale with hydration"
        );
    }

    #[test]
    fn test_self_desiccation_threshold() {
        // w/c = 0.42 is the critical threshold
        let at_threshold = ShrinkageEngine::compute_autogenous(0.42, 0.9, 350.0);
        let below_threshold = ShrinkageEngine::compute_autogenous(0.35, 0.9, 350.0);

        assert!(
            below_threshold.alpha_sd < 1.0,
            "Below 0.42 should have α_sd < 1.0, got {}",
            below_threshold.alpha_sd
        );
        assert!(
            (at_threshold.alpha_sd - 1.0).abs() < 0.01,
            "At 0.42 should have α_sd = 1.0, got {}",
            at_threshold.alpha_sd
        );
    }

    #[test]
    fn test_scm_effect() {
        let no_scm = ShrinkageEngine::compute_autogenous_with_scm(0.35, 0.8, 350.0, 0.0);
        let with_scm = ShrinkageEngine::compute_autogenous_with_scm(0.35, 0.8, 350.0, 0.3);

        // SCM typically increases chemical shrinkage
        assert!(
            with_scm.chemical_shrinkage > no_scm.chemical_shrinkage,
            "SCM should increase chemical shrinkage"
        );
    }

    #[test]
    fn test_ultimate_shrinkage_realistic_range() {
        // Typical HPC (w/c = 0.30, 450 kg/m³ cement)
        let ultimate = ShrinkageEngine::ultimate_autogenous_shrinkage(0.30, 450.0);

        // Expected range: -400 to -1000 με for HPC
        assert!(
            ultimate < -200.0 && ultimate > -1500.0,
            "Ultimate shrinkage should be in realistic range, got {} με",
            ultimate
        );
    }

    #[test]
    fn test_total_shrinkage_includes_drying() {
        let autogenous = ShrinkageEngine::compute_autogenous(0.45, 0.8, 350.0);
        let total = ShrinkageEngine::compute_total_shrinkage(0.45, 0.8, 350.0, 0.50, 90.0);

        // Total should be more negative than autogenous alone (drying adds)
        assert!(
            total < autogenous.autogenous_strain,
            "Total shrinkage should exceed autogenous, got total={}, autogenous={}",
            total,
            autogenous.autogenous_strain
        );
    }
}
