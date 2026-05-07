// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: Apache-2.0
//
// MaOS — Material Agnostic Operating System
// CreepEngine: Extended Microprestress Solidification (XMPS) Model
//
// This file is part of MaOS, developed by Studio TYTO (Est. 2018), Chennai, India.
// For licensing terms, see the LICENSE file in the project root.

//! # Creep Engine
//!
//! Implements concrete creep prediction using a simplified XMPS-inspired model.
//!
//! ## Physical Basis
//!
//! Concrete creep arises from:
//! 1. Basic creep (sealed conditions): viscous flow in C-S-H gel
//! 2. Drying creep (Pickett effect): additional creep when drying under load
//! 3. Aging: reduced creep compliance as hydration proceeds
//!
//! ## Model Components
//!
//! The creep compliance J(t,t') consists of:
//! - Instantaneous elastic: 1/E(t')
//! - Basic creep: C0(t') × ln(1 + (t-t')/λ0)
//! - Drying creep: Cd(t',h) × f(t-t')
//!
//! Where:
//! - t: current age (days)
//! - t': age at loading (days)
//! - λ0: characteristic time (~10 days)
//! - h: relative humidity
//!
//! ## References
//! - Bazant & Prasannan (1989): Solidification theory
//! - Model Code 2010: Engineering creep prediction
//! - ACI 209: Creep and shrinkage

use wasm_bindgen::prelude::*;

/// Creep calculation result
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct CreepResult {
    /// Total creep compliance (1/GPa)
    pub compliance: f32,
    /// Creep coefficient φ = J × E28 (dimensionless)
    pub creep_coefficient: f32,
    /// Basic creep component (1/GPa)
    pub basic_creep: f32,
    /// Drying creep component (1/GPa)  
    pub drying_creep: f32,
    /// Aging factor (0-1, 1 = young concrete, more creep)
    pub aging_factor: f32,
}

#[wasm_bindgen]
pub struct CreepEngine;

#[wasm_bindgen]
impl CreepEngine {
    /// Compute creep coefficient using simplified XMPS model
    ///
    /// # Arguments
    /// * `fc_28` - 28-day compressive strength (MPa)
    /// * `w_c` - Water/cement ratio
    /// * `t_load_days` - Age at loading (days)
    /// * `t_current_days` - Current age (days)
    /// * `rh` - Relative humidity (0-1)
    ///
    /// # Returns
    /// Creep coefficient (dimensionless, typically 1.5-4.0)
    #[wasm_bindgen]
    pub fn compute_coefficient(
        fc_28: f32,
        w_c: f32,
        t_load_days: f32,
        t_current_days: f32,
        rh: f32,
    ) -> f32 {
        let result = Self::compute_full(fc_28, w_c, t_load_days, t_current_days, rh);
        result.creep_coefficient
    }

    /// Compute creep coefficient for 28-day loaded concrete at standard conditions
    /// Simplified version for quick estimates
    #[wasm_bindgen]
    pub fn estimate_coefficient_simple(fc_28: f32) -> f32 {
        // Standard conditions: loaded at 28 days, 70% RH, 10000 days duration
        Self::compute_coefficient(fc_28, 0.45, 28.0, 10000.0, 0.70)
    }
}

impl CreepEngine {
    /// Full creep calculation with all components
    pub fn compute_full(
        fc_28: f32,
        w_c: f32,
        t_load_days: f32,
        t_current_days: f32,
        rh: f32,
    ) -> CreepResult {
        // Edge cases
        if fc_28 <= 0.0 || t_load_days <= 0.0 || t_current_days <= t_load_days {
            return CreepResult {
                compliance: 0.0,
                creep_coefficient: 0.0,
                basic_creep: 0.0,
                drying_creep: 0.0,
                aging_factor: 1.0,
            };
        }

        let duration = t_current_days - t_load_days;

        // 1. Elastic modulus at 28 days and at loading
        // E = 22 × (fc/10)^0.3 GPa (EC2 formula)
        let e_28 = 22.0 * (fc_28 / 10.0).powf(0.3);

        // E(t') = E_28 × β_cc(t')^0.5
        // β_cc = exp(s × (1 - sqrt(28/t')))
        let s = if fc_28 > 50.0 {
            0.20
        } else if fc_28 > 35.0 {
            0.25
        } else {
            0.38
        };
        let beta_cc_load = (s * (1.0 - (28.0 / t_load_days).sqrt())).exp();
        let e_load = e_28 * beta_cc_load.sqrt();

        // 2. Aging factor
        // Young concrete creeps more than old concrete
        // Aging function: 1 / (0.1 + t'^0.2)
        let aging_factor = 1.0 / (0.1 + t_load_days.powf(0.2));

        // 3. Basic creep compliance (sealed conditions)
        // J_basic = C0 × ln(1 + (t-t')/λ0) / E_28
        // C0 depends on w/c and strength
        let lambda_0 = 10.0; // Characteristic time (days)

        // Higher w/c = more basic creep (more gel, more viscous flow)
        // Lower strength = more basic creep
        let c0_base = 0.30 + 0.40 * w_c; // Coefficient
        let strength_factor = (40.0 / fc_28.max(20.0)).powf(0.5); // Reference 40 MPa
        let c0 = c0_base * strength_factor * aging_factor;

        let basic_creep = c0 * (1.0 + duration / lambda_0).ln() / e_28;

        // 4. Drying creep (Pickett effect)
        // Additional creep when drying under load
        // Proportional to (1 - RH) and affected by specimen size (not modeled here)
        let rh_effect = (1.0 - rh).max(0.0).powf(1.5);

        // Drying creep develops faster initially, then slows
        let drying_time_factor = (duration / (duration + 100.0)).min(1.0);

        // Drying creep coefficient (dimensionless)
        let cd = 0.15 * (w_c / 0.45).powf(1.5) * strength_factor;
        let drying_creep = cd * rh_effect * drying_time_factor / e_28;

        // 5. Total compliance
        let elastic_compliance = 1.0 / e_load;
        let total_creep = basic_creep + drying_creep;
        let total_compliance = elastic_compliance + total_creep;

        // 6. Creep coefficient φ = (J - 1/E) × E_28 = creep strain / elastic strain
        let creep_coefficient = total_creep * e_28;

        CreepResult {
            compliance: total_compliance,
            creep_coefficient,
            basic_creep,
            drying_creep,
            aging_factor,
        }
    }

    /// Compute creep strain for given stress
    pub fn compute_creep_strain(
        stress_mpa: f32,
        fc_28: f32,
        w_c: f32,
        t_load_days: f32,
        t_current_days: f32,
        rh: f32,
    ) -> f32 {
        let result = Self::compute_full(fc_28, w_c, t_load_days, t_current_days, rh);
        // Strain = stress × compliance (with units conversion)
        // Compliance is 1/GPa, stress is MPa, so multiply by 1000 for microstrain
        stress_mpa * result.compliance * 1000.0 / 1000.0 // ε = σ × J
    }

    /// Ultimate creep coefficient (t → ∞)
    /// Used for design and limit state verification
    pub fn ultimate_creep_coefficient(fc_28: f32, w_c: f32, t_load_days: f32, rh: f32) -> f32 {
        // Use 10000 days as approximation of infinity
        Self::compute_coefficient(fc_28, w_c, t_load_days, 10000.0, rh)
    }

    /// Creep recovery factor
    /// When load is removed, only part of creep is recovered
    pub fn recovery_factor(duration_under_load_days: f32) -> f32 {
        // Short-term loading: more recovery
        // Long-term loading: less recovery (irreversible)
        let irreversible_fraction =
            (duration_under_load_days / (duration_under_load_days + 30.0)).min(0.7);
        1.0 - irreversible_fraction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_creep_before_loading() {
        let result = CreepEngine::compute_full(40.0, 0.45, 28.0, 28.0, 0.70);
        assert_eq!(result.creep_coefficient, 0.0);
    }

    #[test]
    fn test_creep_increases_with_time() {
        let short = CreepEngine::compute_full(40.0, 0.45, 28.0, 56.0, 0.70);
        let long = CreepEngine::compute_full(40.0, 0.45, 28.0, 365.0, 0.70);

        assert!(
            long.creep_coefficient > short.creep_coefficient,
            "Creep should increase with time: short={:.2}, long={:.2}",
            short.creep_coefficient,
            long.creep_coefficient
        );
    }

    #[test]
    fn test_early_loading_more_creep() {
        // Loading at 7 days vs 28 days
        let early = CreepEngine::compute_full(40.0, 0.45, 7.0, 365.0, 0.70);
        let late = CreepEngine::compute_full(40.0, 0.45, 28.0, 365.0, 0.70);

        assert!(
            early.creep_coefficient > late.creep_coefficient,
            "Early loading should cause more creep: early={:.2}, late={:.2}",
            early.creep_coefficient,
            late.creep_coefficient
        );
    }

    #[test]
    fn test_high_strength_less_creep() {
        let normal = CreepEngine::compute_full(30.0, 0.50, 28.0, 365.0, 0.70);
        let high = CreepEngine::compute_full(60.0, 0.35, 28.0, 365.0, 0.70);

        assert!(
            high.creep_coefficient < normal.creep_coefficient,
            "High strength should creep less: normal={:.2}, high={:.2}",
            normal.creep_coefficient,
            high.creep_coefficient
        );
    }

    #[test]
    fn test_low_rh_more_creep() {
        // Dry conditions cause more creep (drying creep)
        let dry = CreepEngine::compute_full(40.0, 0.45, 28.0, 365.0, 0.50);
        let wet = CreepEngine::compute_full(40.0, 0.45, 28.0, 365.0, 0.90);

        assert!(
            dry.creep_coefficient > wet.creep_coefficient,
            "Low RH should cause more creep: dry={:.2}, wet={:.2}",
            dry.creep_coefficient,
            wet.creep_coefficient
        );

        assert!(
            dry.drying_creep > wet.drying_creep,
            "Drying creep should be higher at low RH"
        );
    }

    #[test]
    fn test_high_wc_more_creep() {
        let low_wc = CreepEngine::compute_full(40.0, 0.35, 28.0, 365.0, 0.70);
        let high_wc = CreepEngine::compute_full(40.0, 0.55, 28.0, 365.0, 0.70);

        assert!(
            high_wc.basic_creep > low_wc.basic_creep,
            "High w/c should have more basic creep"
        );
    }

    #[test]
    fn test_creep_coefficient_realistic_range() {
        // Typical conditions: C30, w/c=0.50, loaded at 28 days, 70% RH
        let result = CreepEngine::compute_full(30.0, 0.50, 28.0, 10000.0, 0.70);

        // Expected range from literature: 1.5-3.5
        assert!(
            result.creep_coefficient > 1.0 && result.creep_coefficient < 5.0,
            "Creep coefficient should be in realistic range, got {:.2}",
            result.creep_coefficient
        );
    }

    #[test]
    fn test_simple_estimate() {
        let phi = CreepEngine::estimate_coefficient_simple(40.0);

        // Should give reasonable value for standard conditions
        assert!(
            phi > 1.0 && phi < 4.0,
            "Simple estimate should be in typical range, got {:.2}",
            phi
        );
    }

    #[test]
    fn test_ultimate_creep() {
        let ultimate = CreepEngine::ultimate_creep_coefficient(40.0, 0.45, 28.0, 0.70);
        let intermediate = CreepEngine::compute_coefficient(40.0, 0.45, 28.0, 365.0, 0.70);

        assert!(
            ultimate > intermediate,
            "Ultimate should exceed intermediate: ult={:.2}, int={:.2}",
            ultimate,
            intermediate
        );
    }

    #[test]
    fn test_recovery_factor() {
        let short_load = CreepEngine::recovery_factor(1.0);
        let long_load = CreepEngine::recovery_factor(365.0);

        assert!(
            short_load > long_load,
            "Short loading should have more recovery"
        );
        assert!(long_load >= 0.3, "Some recovery should always be possible");
    }
}
