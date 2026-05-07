// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//
// MaOS — Material Agnostic Operating System
// TransportEngine: Sorptivity and Chloride Diffusivity
//
// This file is part of MaOS, developed by Studio TYTO (Est. 2018), Chennai, India.
// For licensing terms, see the LICENSE file in the project root.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct TransportResult {
    pub sorptivity: f32,      // mm/min^0.5
    pub diffusion_coeff: f32, // m2/s
}

#[wasm_bindgen]
pub struct TransportEngine;

#[wasm_bindgen]
impl TransportEngine {
    /// Computes Sorptivity (S) based on capillary scaling.
    /// S approx sqrt(C * sigma * r * cos(theta) / 2eta)
    pub fn compute_sorptivity(
        pore_radius_nm: f32,
        surface_tension: f32, // N/m typ 0.072
        viscosity: f32,       // Pa.s typ 0.001
    ) -> TransportResult {
        let r = pore_radius_nm * 1e-9;

        // Washburn relation for penetration depth L = S * sqrt(t)
        // S = sqrt( (r * sigma * cos(theta)) / (2 * eta) )
        // Assuming contact angle 0

        if r <= 0.0 {
            return TransportResult {
                sorptivity: 0.0,
                diffusion_coeff: 0.0,
            };
        }

        let s_squared = (r * surface_tension) / (2.0 * viscosity);
        let s_si = s_squared.sqrt(); // m / s^0.5

        // Convert to mm / min^0.5
        // S_mm = S_si * 1000
        // t_min = t_sec / 60 -> sqrt(t_sec) = sqrt(t_min) * sqrt(60)
        // x = S_si * sqrt(t_sec) = S_si * sqrt(60) * sqrt(t_min)
        // x_mm = 1000 * S_si * sqrt(60) * sqrt(t_min)
        // So S_metric = S_si * 1000 * 7.746

        let sorptivity_metric = s_si * 1000.0 * 7.746;

        TransportResult {
            sorptivity: sorptivity_metric, // mm/min^0.5
            diffusion_coeff: s_squared,    // Rough proxy
        }
    }
}

// ============================================================================
// [V8.2] Chloride Diffusivity Engine with SCM Effects
// ============================================================================
//
// SCMs (Supplementary Cementitious Materials) significantly reduce chloride
// diffusivity through:
// 1. Pore refinement (finer pore structure)
// 2. Chloride binding (C-S-H with lower Ca/Si binds more chlorides)
// 3. Reduced connectivity of capillary pores
//
// Literature values for reduction factors at 28 days:
// - Silica Fume (8-10%): 0.15-0.30
// - Slag (50-70%): 0.30-0.50
// - Fly Ash (25-35%): 0.40-0.60
// - Metakaolin (10-15%): 0.25-0.40

/// SCM type enumeration
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SCMType {
    None = 0,
    SilicaFume = 1, // SF - 8-12% replacement typical
    Slag = 2,       // GGBS - 50-70% replacement typical
    FlyAsh = 3,     // FA Class F - 20-35% replacement typical
    Metakaolin = 4, // MK - 10-15% replacement typical
    Limestone = 5,  // LS - 5-15% replacement (filler)
    Mixed = 6,      // Multiple SCMs (ternary/quaternary blends)
}

impl SCMType {
    /// Get the diffusivity reduction factor at optimal replacement level
    /// Values < 1.0 indicate reduction in diffusivity (better durability)
    pub fn diffusivity_factor(&self) -> f32 {
        match self {
            SCMType::None => 1.0,        // No reduction
            SCMType::SilicaFume => 0.22, // Very effective (fine particles, high pozzolanic activity)
            SCMType::Slag => 0.40,       // Good reduction (latent hydraulic + pozzolanic)
            SCMType::FlyAsh => 0.50,     // Moderate reduction (slower pozzolanic reaction)
            SCMType::Metakaolin => 0.30, // Good reduction (high pozzolanic activity)
            SCMType::Limestone => 0.85,  // Minimal reduction (mainly filler effect)
            SCMType::Mixed => 0.35,      // Synergistic effect of multiple SCMs
        }
    }

    /// Get the aging exponent (m) for time-dependent diffusivity
    /// D(t) = D_28 × (28/t)^m
    /// Higher m = more improvement with time
    pub fn aging_exponent(&self) -> f32 {
        match self {
            SCMType::None => 0.30,       // OPC only
            SCMType::SilicaFume => 0.35, // Slight improvement
            SCMType::Slag => 0.60,       // Significant time-dependent improvement
            SCMType::FlyAsh => 0.55,     // Good improvement over time
            SCMType::Metakaolin => 0.40, // Moderate improvement
            SCMType::Limestone => 0.30,  // Same as OPC
            SCMType::Mixed => 0.50,      // Average effect
        }
    }
}

/// Result from chloride diffusivity calculation
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct ChlorideDiffusivityResult {
    /// 28-day chloride diffusivity (×10⁻¹² m²/s)
    pub d_28: f32,
    /// Diffusivity at specified age (×10⁻¹² m²/s)
    pub d_t: f32,
    /// SCM reduction factor applied
    pub scm_factor: f32,
    /// Aging exponent used
    pub aging_exponent: f32,
}

#[wasm_bindgen]
pub struct ChlorideDiffusivityEngine;

#[wasm_bindgen]
impl ChlorideDiffusivityEngine {
    /// Compute chloride diffusivity at 28 days
    ///
    /// # Arguments
    /// * `w_c` - Water/cement ratio
    /// * `scm_type` - Type of SCM used
    /// * `scm_level` - Replacement level (0-1, e.g., 0.30 for 30%)
    ///
    /// # Returns
    /// Chloride diffusivity (×10⁻¹² m²/s)
    #[wasm_bindgen]
    pub fn compute_d28(w_c: f32, scm_type: SCMType, scm_level: f32) -> f32 {
        let result = Self::compute_full(w_c, scm_type, scm_level, 28.0);
        result.d_28
    }

    /// Compute chloride diffusivity with simple inputs (OPC only)
    #[wasm_bindgen]
    pub fn compute_simple(w_c: f32) -> f32 {
        let result = Self::compute_full(w_c, SCMType::None, 0.0, 28.0);
        result.d_28
    }
}

impl ChlorideDiffusivityEngine {
    /// Full chloride diffusivity calculation
    pub fn compute_full(
        w_c: f32,
        scm_type: SCMType,
        scm_level: f32,
        age_days: f32,
    ) -> ChlorideDiffusivityResult {
        // Edge cases
        if w_c <= 0.0 {
            return ChlorideDiffusivityResult {
                d_28: 0.0,
                d_t: 0.0,
                scm_factor: 1.0,
                aging_exponent: 0.3,
            };
        }

        // 1. Base diffusivity for OPC concrete (no SCM)
        // D_0 = A × (w/c)^n where A ≈ 10-15, n ≈ 3-4
        // Typical values: w/c=0.40→5, w/c=0.50→12, w/c=0.60→25 (×10⁻¹² m²/s)
        let d_opc_28 = 12.0 * (w_c / 0.50).powf(3.5);

        // 2. SCM reduction factor
        // The factor depends on SCM type and replacement level
        // Optimal levels: SF~10%, Slag~60%, FA~25%, MK~12%
        let optimal_level = match scm_type {
            SCMType::None => 0.0,
            SCMType::SilicaFume => 0.10,
            SCMType::Slag => 0.60,
            SCMType::FlyAsh => 0.25,
            SCMType::Metakaolin => 0.12,
            SCMType::Limestone => 0.10,
            SCMType::Mixed => 0.40,
        };

        // Effectiveness curve: efficiency peaks near optimal level
        // Below optimal: linear increase; Above optimal: diminishing returns
        let effectiveness = if optimal_level > 0.0 {
            let ratio = scm_level / optimal_level;
            if ratio <= 1.0 {
                ratio // Linear increase to optimal
            } else {
                1.0 - 0.3 * (ratio - 1.0).min(1.0) // Slight decrease past optimal
            }
        } else {
            0.0
        };

        // Apply SCM factor
        let base_factor = scm_type.diffusivity_factor();
        let scm_factor = 1.0 - (1.0 - base_factor) * effectiveness;

        // 3. Calculate 28-day diffusivity
        let d_28 = d_opc_28 * scm_factor;

        // 4. Time-dependent reduction (aging)
        // D(t) = D_28 × (28/t)^m
        let m = scm_type.aging_exponent();
        let aging_factor = if age_days > 28.0 {
            (28.0 / age_days).powf(m)
        } else {
            // Before 28 days, assume linear development
            (age_days / 28.0).max(0.1)
        };

        let d_t = d_28 * aging_factor;

        ChlorideDiffusivityResult {
            d_28,
            d_t,
            scm_factor,
            aging_exponent: m,
        }
    }

    /// Estimate service life for given chloride threshold
    /// Uses Fick's second law simplified solution
    pub fn estimate_service_life_years(
        cover_mm: f32,
        d_28: f32,        // ×10⁻¹² m²/s
        c_threshold: f32, // Threshold chloride % by weight of cement
        c_surface: f32,   // Surface chloride % by weight of cement
    ) -> f32 {
        // Simplified solution: t = cover² / (4 × D × erfc⁻¹(C_th/C_s)²)
        // For C_th/C_s ≈ 0.4, erfc⁻¹ ≈ 0.6
        // This gives approximate initiation time

        if d_28 <= 0.0 || c_surface <= c_threshold {
            return f32::MAX;
        }

        let cover_m = cover_mm / 1000.0;
        let d_si = d_28 * 1e-12; // Convert to m²/s

        // Chloride ratio
        let ratio = (c_threshold / c_surface).min(0.99);

        // Approximate inverse error function
        // For ratio 0.3-0.5: erfc_inv ≈ 0.5-0.8
        let erfc_inv = 0.5 + 0.5 * (1.0 - ratio);

        // Time in seconds
        let t_sec = cover_m * cover_m / (4.0 * d_si * erfc_inv * erfc_inv);

        // Convert to years
        t_sec / (365.25 * 24.0 * 3600.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opc_baseline() {
        // OPC at w/c = 0.50 should give ~12 ×10⁻¹² m²/s
        let result = ChlorideDiffusivityEngine::compute_full(0.50, SCMType::None, 0.0, 28.0);
        assert!(
            (result.d_28 - 12.0).abs() < 1.0,
            "OPC baseline at w/c=0.50 should be ~12, got {:.1}",
            result.d_28
        );
    }

    #[test]
    fn test_wc_effect() {
        // Higher w/c should give higher diffusivity
        let low_wc = ChlorideDiffusivityEngine::compute_full(0.35, SCMType::None, 0.0, 28.0);
        let high_wc = ChlorideDiffusivityEngine::compute_full(0.55, SCMType::None, 0.0, 28.0);

        assert!(
            high_wc.d_28 > low_wc.d_28 * 2.0,
            "High w/c should significantly increase D: low={:.1}, high={:.1}",
            low_wc.d_28,
            high_wc.d_28
        );
    }

    #[test]
    fn test_silica_fume_effect() {
        let opc = ChlorideDiffusivityEngine::compute_full(0.45, SCMType::None, 0.0, 28.0);
        let sf = ChlorideDiffusivityEngine::compute_full(0.45, SCMType::SilicaFume, 0.10, 28.0);

        assert!(
            sf.d_28 < opc.d_28 * 0.35,
            "SF should reduce D by >65%: OPC={:.1}, SF={:.1}",
            opc.d_28,
            sf.d_28
        );
    }

    #[test]
    fn test_slag_effect() {
        let opc = ChlorideDiffusivityEngine::compute_full(0.45, SCMType::None, 0.0, 28.0);
        let slag = ChlorideDiffusivityEngine::compute_full(0.45, SCMType::Slag, 0.60, 28.0);

        assert!(
            slag.d_28 < opc.d_28 * 0.55,
            "Slag should reduce D by >45%: OPC={:.1}, Slag={:.1}",
            opc.d_28,
            slag.d_28
        );
    }

    #[test]
    fn test_fly_ash_effect() {
        let opc = ChlorideDiffusivityEngine::compute_full(0.45, SCMType::None, 0.0, 28.0);
        let fa = ChlorideDiffusivityEngine::compute_full(0.45, SCMType::FlyAsh, 0.25, 28.0);

        assert!(
            fa.d_28 < opc.d_28 * 0.65,
            "FA should reduce D by >35%: OPC={:.1}, FA={:.1}",
            opc.d_28,
            fa.d_28
        );
    }

    #[test]
    fn test_aging_reduction() {
        // Diffusivity should decrease with age
        let d_28 = ChlorideDiffusivityEngine::compute_full(0.45, SCMType::Slag, 0.50, 28.0);
        let d_365 = ChlorideDiffusivityEngine::compute_full(0.45, SCMType::Slag, 0.50, 365.0);

        assert!(
            d_365.d_t < d_28.d_28 * 0.6,
            "D should decrease with age: 28d={:.1}, 365d={:.1}",
            d_28.d_28,
            d_365.d_t
        );
    }

    #[test]
    fn test_slag_ages_better() {
        // Slag should show more improvement with age than OPC
        let opc_28 = ChlorideDiffusivityEngine::compute_full(0.45, SCMType::None, 0.0, 28.0);
        let opc_365 = ChlorideDiffusivityEngine::compute_full(0.45, SCMType::None, 0.0, 365.0);
        let slag_28 = ChlorideDiffusivityEngine::compute_full(0.45, SCMType::Slag, 0.50, 28.0);
        let slag_365 = ChlorideDiffusivityEngine::compute_full(0.45, SCMType::Slag, 0.50, 365.0);

        let opc_reduction = opc_365.d_t / opc_28.d_28;
        let slag_reduction = slag_365.d_t / slag_28.d_28;

        assert!(
            slag_reduction < opc_reduction,
            "Slag should show more aging improvement: OPC ratio={:.2}, Slag ratio={:.2}",
            opc_reduction,
            slag_reduction
        );
    }

    #[test]
    fn test_service_life() {
        // Typical marine exposure: 75mm cover (marine requirement), moderate chloride
        let d_28 = 3.0; // Very good concrete (HPC)
        let life = ChlorideDiffusivityEngine::estimate_service_life_years(
            75.0, // 75mm cover (marine structural)
            d_28, 0.4, // Threshold: 0.4% by cement weight
            2.0, // Surface: 2% (marine splash zone)
        );

        // Should be in realistic range (> 10 years for good concrete with high cover)
        assert!(
            life > 5.0 && life < 500.0,
            "Service life should be realistic, got {:.1} years",
            life
        );
    }
}
