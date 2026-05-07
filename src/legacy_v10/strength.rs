// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: LicenseRef-Proprietary
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct StrengthResult {
    pub compressive_strength: f32, // MPa
    pub gel_space_ratio: f32,      // 0.0 - 1.0 (ξ)
    #[wasm_bindgen(getter_with_clone)]
    pub predicted_class: String, // e.g., "C30/37"
}

/// ═══════════════════════════════════════════════════════════════════════════
/// [V8.1] CEMENT TYPE CLASSIFICATION
/// ═══════════════════════════════════════════════════════════════════════════
///
/// Different cement types have different intrinsic gel strengths based on
/// their clinker composition (C3S, C2S, C3A content).
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_camel_case_types)] // CEM II/A, CEM III/B etc. follow EN 197-1 cement naming
pub enum CementType {
    /// OPC 42.5N - Standard Portland cement
    OPC42_5N = 0,
    /// OPC 52.5R - Rapid hardening, high early strength
    OPC52_5R = 1,
    /// CEM II/A - Portland-composite with 6-20% additives
    CEM_II_A = 2,
    /// CEM II/B - Portland-composite with 21-35% additives
    CEM_II_B = 3,
    /// CEM III/A - Blast furnace cement with 36-65% slag
    CEM_III_A = 4,
    /// CEM III/B - Blast furnace cement with 66-80% slag
    CEM_III_B = 5,
    /// White Cement - High C3S, low iron
    WhiteCement = 6,
    /// Unknown/Generic - Use default intrinsic strength
    Generic = 7,
}

impl CementType {
    /// Get the intrinsic gel strength (MPa) for this cement type
    ///
    /// Based on empirical correlations with clinker composition:
    /// - Higher C3S content → higher early/intrinsic strength
    /// - SCM additions → lower intrinsic (but improved durability)
    pub fn intrinsic_strength(&self) -> f32 {
        match self {
            CementType::OPC42_5N => 80.0,    // Standard Portland
            CementType::OPC52_5R => 100.0,   // Rapid hardening (~65% C3S)
            CementType::CEM_II_A => 75.0,    // Slight reduction from additives
            CementType::CEM_II_B => 70.0,    // More additives
            CementType::CEM_III_A => 68.0,   // Moderate slag
            CementType::CEM_III_B => 60.0,   // High slag content
            CementType::WhiteCement => 90.0, // High C3S, pure clinker
            CementType::Generic => 80.0,     // Default fallback
        }
    }

    /// Get cement type from Blaine fineness (cm²/g) - heuristic classification
    ///
    /// Higher fineness often indicates rapid-hardening cements
    pub fn from_blaine(blaine: f32) -> Self {
        if blaine > 5000.0 {
            CementType::OPC52_5R // Very fine = rapid hardening
        } else if blaine > 3500.0 {
            CementType::OPC42_5N // Normal fineness
        } else {
            CementType::CEM_III_B // Coarse = likely slag cement
        }
    }
}

impl Default for CementType {
    fn default() -> Self {
        CementType::Generic
    }
}

#[wasm_bindgen]
pub struct StrengthEngine;

#[wasm_bindgen]
impl StrengthEngine {
    /// Computes compressive strength using Powers' Gel-Space Ratio model.
    ///
    /// # Arguments
    /// * `wc_ratio`: Water-to-cement ratio (by mass)
    /// * `degree_hydration`: Alpha (0.0 - 1.0), typical 28-day is ~0.85
    /// * `air_content`: Entrapped/entrained air fraction (0.0 - 0.1)
    /// * `intrinsic_strength`: Strength of the gel itself (approx 240 MPa)
    pub fn compute_powers(
        wc_ratio: f32,
        degree_hydration: f32,
        air_content: f32,
        intrinsic_strength: f32,
    ) -> StrengthResult {
        // Volume of cement (approx density 3.15) vs water (1.0)
        // Vc = 1/3.15, Vw = wc_ratio
        // Gel-Space Ratio (x) = Volume of Gel / (Volume of Gel + Capillary Pores)

        // Simplified Powers model:
        // x = (0.68 * alpha) / (0.32 * alpha + wc_ratio)
        // But let's use the explicit volume approach for accuracy.

        // [GUARDRAIL] Infinite W/C (Zero Cement) Protection
        if wc_ratio > 100.0 {
            return StrengthResult {
                compressive_strength: 0.0,
                gel_space_ratio: 0.0,
                predicted_class: "N/A".to_string(),
            };
        }

        let vg_volume_gel = 0.68 * degree_hydration;
        let vc_volume_capillary = wc_ratio - 0.36 * degree_hydration;

        // Total space available for gel = Volume of Gel + Capillary Pores + Air
        // Note: Powers usually ignores air in the base equation, but for concrete we strictly include it.
        // x = Vgel / (Vgel + Vcap + Vair)

        let space = vg_volume_gel + vc_volume_capillary + air_content;

        // Handle physical impossibility (wc < 0.36 alpha)
        if space <= 0.001 {
            return StrengthResult {
                compressive_strength: 0.0,
                gel_space_ratio: 0.0,
                predicted_class: "INVALID".to_string(),
            };
        }

        let x = vg_volume_gel / space; // Gel-Space Ratio

        // Strength = S * x^3
        let fc = intrinsic_strength * x.powi(3);

        StrengthResult {
            compressive_strength: fc,
            gel_space_ratio: x,
            predicted_class: Self::classify_strength(fc),
        }
    }

    /// Computes strength using Bolomey's empirical equation (Standard Industrial).
    /// fc = K * (1/WC - 0.5)
    pub fn compute_bolomey(wc_ratio: f32, k_factor: f32) -> f32 {
        if wc_ratio <= 0.01 {
            return 0.0;
        }
        let fc = k_factor * ((1.0 / wc_ratio) - 0.5);
        if fc < 0.0 {
            0.0
        } else {
            fc
        }
    }

    /// [V8.1] Compute strength using cement type for intrinsic strength
    ///
    /// This is the recommended API when cement properties are available.
    pub fn compute_powers_with_cement_type(
        wc_ratio: f32,
        degree_hydration: f32,
        air_content: f32,
        cement_type: CementType,
    ) -> StrengthResult {
        let intrinsic_strength = cement_type.intrinsic_strength();
        Self::compute_powers(wc_ratio, degree_hydration, air_content, intrinsic_strength)
    }

    /// [V8.1] Compute weighted intrinsic strength for blended cements
    ///
    /// When multiple cementitious materials are present (cement + SCM),
    /// compute a weighted average of intrinsic strengths.
    ///
    /// # Arguments
    /// * `cement_mass`: Mass of primary cement (kg)
    /// * `cement_type`: Type of primary cement
    /// * `scm_mass`: Mass of supplementary cementitious material (kg)
    /// * `scm_efficiency`: SCM strength efficiency factor (0.3-0.8, default 0.5)
    pub fn compute_blended_intrinsic_strength(
        cement_mass: f32,
        cement_type: CementType,
        scm_mass: f32,
        scm_efficiency: f32,
    ) -> f32 {
        let total_binder = cement_mass + scm_mass;
        if total_binder < 0.001 {
            return cement_type.intrinsic_strength();
        }

        let cement_contribution = cement_mass * cement_type.intrinsic_strength();
        // SCM contributes reduced strength (pozzolanic reaction is slower/weaker)
        let scm_contribution = scm_mass * cement_type.intrinsic_strength() * scm_efficiency;

        (cement_contribution + scm_contribution) / total_binder
    }

    fn classify_strength(fc: f32) -> String {
        if fc < 12.0 {
            return "C8/10".to_string();
        }
        if fc < 16.0 {
            return "C12/15".to_string();
        }
        if fc < 20.0 {
            return "C16/20".to_string();
        }
        if fc < 25.0 {
            return "C20/25".to_string();
        }
        if fc < 30.0 {
            return "C25/30".to_string();
        }
        if fc < 37.0 {
            return "C30/37".to_string();
        }
        if fc < 45.0 {
            return "C35/45".to_string();
        }
        if fc < 50.0 {
            return "C40/50".to_string();
        }
        if fc < 60.0 {
            return "C50/60".to_string();
        }
        "C60+".to_string()
    }
}

// ============================================================================
// [V8.2] MORI-TANAKA HOMOGENIZATION FOR COMPOSITE STRENGTH
// ============================================================================
//
// Three-phase composite model: Paste (matrix) + ITZ + Aggregate (inclusion)
//
// The Interfacial Transition Zone (ITZ) is the ~20-50 μm layer around
// aggregates with higher porosity and lower strength than bulk paste.
//
// Physics:
//   - E_itz ≈ 0.5-0.7 × E_paste (due to higher w/c and porosity)
//   - E_agg depends on aggregate type (limestone ~50 GPa, granite ~70 GPa)
//   - Effective modulus is computed via Mori-Tanaka scheme
//   - Strength reduction factor = (E_eff / E_paste)^n where n ≈ 0.8-1.0
//
// References:
//   - Lutz et al. (1997) "Inhomogeneous ITZ model"
//   - Nilsen & Monteiro (1993) "Concrete: A three phase material"
// ============================================================================

/// Aggregate type classification for modulus estimation
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AggregateType {
    /// Limestone/Dolomite - lower modulus
    Limestone = 0,
    /// Granite/Basalt - higher modulus
    Granite = 1,
    /// Quartzite/Sandstone - medium modulus
    Quartzite = 2,
    /// Lightweight aggregate - very low modulus
    Lightweight = 3,
    /// Recycled aggregate - lower than natural
    Recycled = 4,
    /// Unknown/Generic
    Generic = 5,
}

impl AggregateType {
    /// Get typical elastic modulus for aggregate type (GPa)
    pub fn elastic_modulus(&self) -> f32 {
        match self {
            AggregateType::Limestone => 50.0,
            AggregateType::Granite => 70.0,
            AggregateType::Quartzite => 60.0,
            AggregateType::Lightweight => 15.0,
            AggregateType::Recycled => 35.0,
            AggregateType::Generic => 50.0,
        }
    }

    /// Get tensile/compressive strength ratio modifier
    /// Lightweight and recycled have weaker ITZ
    pub fn tensile_factor(&self) -> f32 {
        match self {
            AggregateType::Limestone => 1.0,
            AggregateType::Granite => 1.05,
            AggregateType::Quartzite => 1.0,
            AggregateType::Lightweight => 0.85,
            AggregateType::Recycled => 0.80,
            AggregateType::Generic => 1.0,
        }
    }
}

impl Default for AggregateType {
    fn default() -> Self {
        AggregateType::Generic
    }
}

/// Result of composite strength calculation
#[derive(Clone, Debug)]
pub struct CompositeStrengthResult {
    /// Paste (matrix) strength without aggregate effect (MPa)
    pub paste_strength: f32,
    /// Effective composite strength including ITZ weakening (MPa)
    pub composite_strength: f32,
    /// Effective elastic modulus (GPa)
    pub effective_modulus: f32,
    /// ITZ volume fraction
    pub itz_volume_fraction: f32,
    /// Strength reduction factor due to ITZ
    pub itz_strength_factor: f32,
}

/// Mori-Tanaka homogenization engine for three-phase composite
#[wasm_bindgen]
pub struct MoriTanakaEngine;

#[wasm_bindgen]
impl MoriTanakaEngine {
    /// Compute effective properties of three-phase composite (Paste + ITZ + Aggregate)
    ///
    /// # Arguments
    /// * `fc_paste` - Compressive strength of paste matrix (MPa)
    /// * `agg_volume_fraction` - Volume fraction of aggregates (0-0.8)
    /// * `agg_type` - Type of aggregate (as integer: 0=Limestone, 1=Granite, etc.)
    /// * `mean_agg_size_mm` - Mean aggregate size in mm (affects ITZ volume)
    /// * `wc_ratio` - Water-cement ratio (affects ITZ quality)
    ///
    /// # Returns
    /// Effective composite strength (MPa)
    #[wasm_bindgen]
    pub fn compute_composite_strength_simple(
        fc_paste: f32,
        agg_volume_fraction: f32,
        agg_type_id: u8,
        mean_agg_size_mm: f32,
        wc_ratio: f32,
    ) -> f32 {
        let agg_type = match agg_type_id {
            0 => AggregateType::Limestone,
            1 => AggregateType::Granite,
            2 => AggregateType::Quartzite,
            3 => AggregateType::Lightweight,
            4 => AggregateType::Recycled,
            _ => AggregateType::Generic,
        };

        let result = Self::compute_composite_strength(
            fc_paste,
            agg_volume_fraction,
            agg_type,
            mean_agg_size_mm,
            wc_ratio,
        );
        result.composite_strength
    }
}

impl MoriTanakaEngine {
    /// Full composite strength calculation with detailed result
    pub fn compute_composite_strength(
        fc_paste: f32,
        agg_volume_fraction: f32,
        agg_type: AggregateType,
        mean_agg_size_mm: f32,
        wc_ratio: f32,
    ) -> CompositeStrengthResult {
        // 1. Estimate paste modulus from strength (empirical correlation)
        // E_paste ≈ 22 * (fc/10)^0.3 for mature paste (EC2-like)
        let e_paste = 22.0 * (fc_paste / 10.0).powf(0.3);

        // 2. ITZ properties
        // ITZ modulus is typically 50-70% of paste modulus
        // Lower w/c → better ITZ (less porosity difference)
        let itz_quality = if wc_ratio < 0.40 {
            0.70 // High quality ITZ for HPC
        } else if wc_ratio < 0.50 {
            0.625 // Standard ITZ
        } else {
            0.55 // Poor ITZ for high w/c
        };
        let e_itz = e_paste * itz_quality;

        // 3. Aggregate modulus
        let e_agg = agg_type.elastic_modulus();

        // 4. ITZ volume fraction
        // v_itz ≈ 3 * v_agg * t_itz / r_agg for spherical aggregates
        // where t_itz ≈ 30-50 μm and r_agg is mean aggregate radius
        let t_itz_mm = 0.040; // 40 μm typical ITZ thickness
        let r_agg_mm = (mean_agg_size_mm / 2.0).max(0.5);
        let v_itz = (3.0 * agg_volume_fraction * t_itz_mm / r_agg_mm).min(0.20);

        // 5. Volume fractions (must sum to 1)
        let v_agg = agg_volume_fraction;
        let v_paste = 1.0 - v_agg - v_itz;

        if v_paste < 0.1 {
            // Edge case: too much aggregate, no room for paste
            return CompositeStrengthResult {
                paste_strength: fc_paste,
                composite_strength: fc_paste * 0.5,
                effective_modulus: e_paste * 0.5,
                itz_volume_fraction: v_itz,
                itz_strength_factor: 0.5,
            };
        }

        // 6. Simplified Mori-Tanaka for effective modulus
        // Voigt upper bound (parallel)
        let e_voigt = v_paste * e_paste + v_itz * e_itz + v_agg * e_agg;

        // Reuss lower bound (series)
        let e_reuss = if e_paste > 0.0 && e_itz > 0.0 && e_agg > 0.0 {
            1.0 / (v_paste / e_paste + v_itz / e_itz + v_agg / e_agg)
        } else {
            e_paste
        };

        // Hashin-Shtrikman-like estimate (geometric mean)
        let e_eff = (e_voigt * e_reuss).sqrt();

        // 7. Strength reduction factor
        // ITZ is ALWAYS the weak link - composite strength <= paste strength
        // Even with stiff aggregate, the weak ITZ shell limits load transfer

        // ITZ weakness factor: how much ITZ weakens the composite
        // v_itz/v_paste is the "damage" ratio - more ITZ volume = more weakness
        let itz_weakness = 1.0 - 0.3 * (v_itz / v_paste.max(0.1)).min(1.0);

        // w/c effect: higher w/c makes ITZ even weaker (more porous)
        let wc_penalty = if wc_ratio > 0.50 {
            0.95 - 0.1 * (wc_ratio - 0.50).min(0.2) // Up to 2% extra penalty
        } else {
            1.0
        };

        // Aggregate stiffness effect:
        // Stiffer aggregate → stress concentration at ITZ → slight additional penalty
        // Softer aggregate → more uniform stress → less ITZ stress concentration
        let stiffness_ratio = e_agg / e_paste;
        let stiffness_penalty = if stiffness_ratio > 1.5 {
            0.98 // Very stiff aggregate: slight penalty from stress concentration
        } else if stiffness_ratio < 0.5 {
            0.90 // Very soft aggregate: aggregate failure dominates
        } else {
            0.97 // Moderate mismatch
        };

        // Combined factor: ITZ weakness × w/c effect × stiffness mismatch
        let itz_strength_factor = itz_weakness * wc_penalty * stiffness_penalty;

        // Ensure composite never exceeds paste (ITZ always weakens)
        let fc_composite = fc_paste * itz_strength_factor.clamp(0.5, 1.0);

        CompositeStrengthResult {
            paste_strength: fc_paste,
            composite_strength: fc_composite,
            effective_modulus: e_eff,
            itz_volume_fraction: v_itz,
            itz_strength_factor,
        }
    }

    /// Compute tensile strength with aggregate type correction
    pub fn compute_tensile_strength(fc_compressive: f32, agg_type: AggregateType) -> f32 {
        // Base tensile from EC2: f_t = 0.3 * f_c^(2/3)
        let base_ft = 0.3 * fc_compressive.powf(0.67);
        base_ft * agg_type.tensile_factor()
    }
}

// ============================================================================
// [V8.2] Hashin-Shtrikman Homogenization for Elastic Modulus
// ============================================================================
//
// The Hashin-Shtrikman bounds provide the tightest possible bounds for
// effective elastic properties of two-phase composites without assuming
// specific microstructure.
//
// For concrete: E_HS = f(E_paste, E_aggregate, v_agg)
//
// These bounds are used for validation and improved E-modulus prediction
// when aggregate properties are known.

/// Result from Hashin-Shtrikman homogenization
#[derive(Clone, Debug)]
pub struct HSModulusResult {
    /// Lower bound on effective bulk modulus (GPa)
    pub k_lower: f32,
    /// Upper bound on effective bulk modulus (GPa)
    pub k_upper: f32,
    /// Lower bound on effective shear modulus (GPa)
    pub g_lower: f32,
    /// Upper bound on effective shear modulus (GPa)
    pub g_upper: f32,
    /// Lower bound on effective Young's modulus (GPa)
    pub e_lower: f32,
    /// Upper bound on effective Young's modulus (GPa)
    pub e_upper: f32,
    /// Best estimate (average of bounds) (GPa)
    pub e_effective: f32,
}

#[wasm_bindgen]
pub struct HashinShtrikmanEngine;

#[wasm_bindgen]
impl HashinShtrikmanEngine {
    /// Compute effective modulus using Hashin-Shtrikman bounds
    /// Returns the best estimate (average of bounds)
    #[wasm_bindgen]
    pub fn compute_modulus(
        e_matrix: f32,    // Paste modulus (GPa)
        e_inclusion: f32, // Aggregate modulus (GPa)
        v_inclusion: f32, // Aggregate volume fraction
    ) -> f32 {
        let result = Self::compute_bounds(e_matrix, e_inclusion, v_inclusion);
        result.e_effective
    }
}

impl HashinShtrikmanEngine {
    /// Compute full Hashin-Shtrikman bounds
    ///
    /// Uses the direct HS bounds formula for Young's modulus.
    /// Reference: Hashin & Shtrikman (1963), J. Mech. Phys. Solids
    pub fn compute_bounds(
        e_matrix: f32,    // Paste modulus (GPa)
        e_inclusion: f32, // Aggregate modulus (GPa)
        v_inclusion: f32, // Aggregate volume fraction (0-1)
    ) -> HSModulusResult {
        // Edge cases
        if e_matrix <= 0.0 || e_inclusion <= 0.0 {
            return HSModulusResult {
                k_lower: 0.0,
                k_upper: 0.0,
                g_lower: 0.0,
                g_upper: 0.0,
                e_lower: 0.0,
                e_upper: 0.0,
                e_effective: 0.0,
            };
        }

        let f = v_inclusion.max(0.0).min(0.99); // Inclusion volume fraction

        if f < 0.001 {
            // No inclusions: return matrix properties
            return HSModulusResult {
                k_lower: e_matrix / 3.0,
                k_upper: e_matrix / 3.0,
                g_lower: e_matrix / 2.6,
                g_upper: e_matrix / 2.6,
                e_lower: e_matrix,
                e_upper: e_matrix,
                e_effective: e_matrix,
            };
        }

        // Assume Poisson's ratio ν = 0.20 for both phases (typical for concrete)
        let nu_m = 0.20_f32; // Matrix
        let nu_i = 0.20_f32; // Inclusion

        // Convert E to bulk (K) and shear (G) moduli
        // K = E / (3(1-2ν)), G = E / (2(1+ν))
        let k_m = e_matrix / (3.0 * (1.0 - 2.0 * nu_m));
        let g_m = e_matrix / (2.0 * (1.0 + nu_m));
        let k_i = e_inclusion / (3.0 * (1.0 - 2.0 * nu_i));
        let g_i = e_inclusion / (2.0 * (1.0 + nu_i));

        // Hashin-Shtrikman bounds
        // Lower bound uses the softer phase as the reference (matrix)
        // Upper bound uses the stiffer phase as the reference

        let (k_soft, g_soft, k_stiff, g_stiff, v_soft, v_stiff) = if k_m <= k_i {
            (k_m, g_m, k_i, g_i, 1.0 - f, f)
        } else {
            (k_i, g_i, k_m, g_m, f, 1.0 - f)
        };

        // HS lower bound for bulk modulus (soft phase as matrix)
        let k_lower = k_soft
            + v_stiff * (k_stiff - k_soft)
                / (1.0 + v_soft * (k_stiff - k_soft) / (k_soft + 4.0 * g_soft / 3.0));

        // HS upper bound for bulk modulus (stiff phase as matrix)
        let k_upper = k_stiff
            + v_soft * (k_soft - k_stiff)
                / (1.0 + v_stiff * (k_soft - k_stiff) / (k_stiff + 4.0 * g_stiff / 3.0));

        // HS lower bound for shear modulus
        let beta_l = 6.0 * (k_soft + 2.0 * g_soft) / (5.0 * (3.0 * k_soft + 4.0 * g_soft));
        let g_lower = g_soft
            + v_stiff * (g_stiff - g_soft) / (1.0 + v_soft * (g_stiff - g_soft) * beta_l / g_soft);

        // HS upper bound for shear modulus
        let beta_u = 6.0 * (k_stiff + 2.0 * g_stiff) / (5.0 * (3.0 * k_stiff + 4.0 * g_stiff));
        let g_upper = g_stiff
            + v_soft * (g_soft - g_stiff) / (1.0 + v_stiff * (g_soft - g_stiff) * beta_u / g_stiff);

        // Convert K, G to E: E = 9KG / (3K + G)
        let e_from_kg = |k: f32, g: f32| -> f32 {
            if k <= 0.0 || g <= 0.0 {
                return 0.0;
            }
            9.0 * k * g / (3.0 * k + g)
        };

        let e_l = e_from_kg(k_lower, g_lower);
        let e_u = e_from_kg(k_upper, g_upper);

        let e_lower = e_l.min(e_u);
        let e_upper = e_l.max(e_u);

        // Best estimate: Hill average (arithmetic mean of bounds)
        let e_effective = (e_lower + e_upper) / 2.0;

        HSModulusResult {
            k_lower: k_lower.min(k_upper),
            k_upper: k_lower.max(k_upper),
            g_lower: g_lower.min(g_upper),
            g_upper: g_lower.max(g_upper),
            e_lower,
            e_upper,
            e_effective,
        }
    }

    /// Compute effective modulus for concrete given component properties
    pub fn compute_concrete_modulus(
        fc_paste: f32, // Paste compressive strength (MPa)
        agg_type: AggregateType,
        agg_volume_fraction: f32,
    ) -> f32 {
        // Paste modulus from strength: E = 22 × (fc/10)^0.3 GPa
        let e_paste = 22.0 * (fc_paste / 10.0).powf(0.3);
        let e_agg = agg_type.elastic_modulus();

        Self::compute_modulus(e_paste, e_agg, agg_volume_fraction)
    }

    /// Get bounds spread (useful for uncertainty quantification)
    pub fn bounds_spread(e_matrix: f32, e_inclusion: f32, v_inclusion: f32) -> f32 {
        let result = Self::compute_bounds(e_matrix, e_inclusion, v_inclusion);
        result.e_upper - result.e_lower
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // [V8.2] Mori-Tanaka Composite Tests
    // =========================================================================

    #[test]
    fn test_mori_tanaka_no_aggregate() {
        let result = MoriTanakaEngine::compute_composite_strength(
            40.0,
            0.0,
            AggregateType::Generic,
            10.0,
            0.45,
        );
        assert!(
            (result.composite_strength - 40.0).abs() < 2.0,
            "Zero aggregate should give ~paste strength, got {:.1}",
            result.composite_strength
        );
    }

    #[test]
    fn test_mori_tanaka_typical_concrete() {
        let result = MoriTanakaEngine::compute_composite_strength(
            45.0,
            0.40,
            AggregateType::Limestone,
            10.0,
            0.45,
        );

        println!(
            "Paste: {:.1}, Composite: {:.1}, E_eff: {:.1}",
            result.paste_strength, result.composite_strength, result.effective_modulus
        );

        assert!(
            result.composite_strength < result.paste_strength,
            "ITZ should reduce strength"
        );
        assert!(
            result.itz_strength_factor > 0.70,
            "ITZ factor too low: {:.3}",
            result.itz_strength_factor
        );
    }

    #[test]
    fn test_mori_tanaka_wc_effect() {
        let low_wc = MoriTanakaEngine::compute_composite_strength(
            50.0,
            0.40,
            AggregateType::Generic,
            10.0,
            0.35,
        );
        let high_wc = MoriTanakaEngine::compute_composite_strength(
            50.0,
            0.40,
            AggregateType::Generic,
            10.0,
            0.55,
        );

        assert!(
            low_wc.itz_strength_factor > high_wc.itz_strength_factor,
            "Lower w/c should give better ITZ quality"
        );
    }

    #[test]
    fn test_tensile_aggregate_type() {
        let ft_normal = MoriTanakaEngine::compute_tensile_strength(40.0, AggregateType::Limestone);
        let ft_lwa = MoriTanakaEngine::compute_tensile_strength(40.0, AggregateType::Lightweight);

        assert!(
            ft_normal > ft_lwa,
            "Normal should have higher tensile than LWA"
        );
    }

    // =========================================================================
    // [V8.2] Hashin-Shtrikman Homogenization Tests
    // =========================================================================

    #[test]
    fn test_hs_no_aggregate() {
        // No aggregate: effective modulus = matrix modulus
        let result = HashinShtrikmanEngine::compute_bounds(30.0, 50.0, 0.0);
        assert!(
            (result.e_effective - 30.0).abs() < 1.0,
            "No aggregate should give matrix modulus, got {:.1}",
            result.e_effective
        );
    }

    #[test]
    fn test_hs_stiffer_aggregate() {
        // Stiffer aggregate should increase effective modulus
        let e_paste = 25.0; // GPa
        let e_agg = 50.0; // GPa (typical limestone)
        let result = HashinShtrikmanEngine::compute_bounds(e_paste, e_agg, 0.40);

        assert!(
            result.e_effective > e_paste,
            "Stiffer aggregate should increase modulus: E_eff={:.1}, E_paste={:.1}",
            result.e_effective,
            e_paste
        );
        assert!(
            result.e_effective < e_agg,
            "Effective should be less than aggregate: E_eff={:.1}, E_agg={:.1}",
            result.e_effective,
            e_agg
        );
    }

    #[test]
    fn test_hs_softer_aggregate() {
        // Softer aggregate (LWA) should decrease effective modulus
        let e_paste = 30.0; // GPa
        let e_lwa = 15.0; // GPa (lightweight aggregate)
        let result = HashinShtrikmanEngine::compute_bounds(e_paste, e_lwa, 0.40);

        assert!(
            result.e_effective < e_paste,
            "Softer aggregate should decrease modulus: E_eff={:.1}, E_paste={:.1}",
            result.e_effective,
            e_paste
        );
    }

    #[test]
    fn test_hs_bounds_valid() {
        // Bounds should satisfy: E_lower <= E_effective <= E_upper
        let result = HashinShtrikmanEngine::compute_bounds(25.0, 50.0, 0.45);

        assert!(
            result.e_lower <= result.e_effective,
            "E_lower ({:.1}) should be <= E_effective ({:.1})",
            result.e_lower,
            result.e_effective
        );
        assert!(
            result.e_effective <= result.e_upper,
            "E_effective ({:.1}) should be <= E_upper ({:.1})",
            result.e_effective,
            result.e_upper
        );
    }

    #[test]
    fn test_hs_concrete_modulus() {
        // Typical concrete: fc=40 MPa paste, limestone aggregate, 40% volume
        let e_eff =
            HashinShtrikmanEngine::compute_concrete_modulus(40.0, AggregateType::Limestone, 0.40);

        // E_paste ≈ 22 × (40/10)^0.3 ≈ 32 GPa
        // With limestone (50 GPa) at 40%, expect E_eff > 32 GPa
        assert!(
            e_eff > 30.0 && e_eff < 50.0,
            "Concrete modulus should be in realistic range, got {:.1} GPa",
            e_eff
        );
    }

    #[test]
    fn test_hs_increasing_volume_fraction() {
        // More aggregate should increase effective modulus (for stiff aggregate)
        let low_agg = HashinShtrikmanEngine::compute_modulus(25.0, 50.0, 0.20);
        let high_agg = HashinShtrikmanEngine::compute_modulus(25.0, 50.0, 0.50);

        println!("Low agg (v=0.20): E_eff = {:.2} GPa", low_agg);
        println!("High agg (v=0.50): E_eff = {:.2} GPa", high_agg);

        assert!(
            high_agg > low_agg,
            "Higher aggregate fraction should increase modulus: low={:.2}, high={:.2}",
            low_agg,
            high_agg
        );
    }

    // =========================================================================
    // Original Powers Tests
    // =========================================================================

    #[test]
    fn test_zero_cement_safety() {
        // CASE: Zero Cement means W/C is Infinite.
        // The engine must handle f32::INFINITY gracefully and return 0.0 strength.
        let result = StrengthEngine::compute_powers(f32::INFINITY, 0.85, 0.02, 150.0);
        assert_eq!(result.compressive_strength, 0.0);
        assert_eq!(result.predicted_class, "N/A");
    }

    #[test]
    fn test_wc_trend_abrams_law() {
        // CASE: Lower W/C should yield Higher Strength
        // W/C = 0.3 (Strong)
        let strong = StrengthEngine::compute_powers(0.3, 0.85, 0.02, 150.0);

        // W/C = 0.6 (Weak)
        let weak = StrengthEngine::compute_powers(0.6, 0.85, 0.02, 150.0);

        println!("Strong (0.3): {} MPa", strong.compressive_strength);
        println!("Weak (0.6): {} MPa", weak.compressive_strength);

        assert!(strong.compressive_strength > weak.compressive_strength);
        assert!(strong.compressive_strength > 50.0); // Expect high strength for 0.3
        assert!(weak.compressive_strength < 50.0); // Expect lower strength for 0.6
    }

    #[test]
    fn test_cement_type_intrinsic_strength() {
        // [V8.1] Verify intrinsic strength values for different cement types
        assert_eq!(CementType::OPC42_5N.intrinsic_strength(), 80.0);
        assert_eq!(CementType::OPC52_5R.intrinsic_strength(), 100.0);
        assert_eq!(CementType::CEM_III_B.intrinsic_strength(), 60.0);
        assert_eq!(CementType::WhiteCement.intrinsic_strength(), 90.0);
    }

    #[test]
    fn test_cement_type_affects_strength() {
        // [V8.1] Same w/c but different cement types should give different strengths
        let wc = 0.45;
        let alpha = 0.85;
        let air = 0.02;

        let opc =
            StrengthEngine::compute_powers_with_cement_type(wc, alpha, air, CementType::OPC42_5N);
        let rapid =
            StrengthEngine::compute_powers_with_cement_type(wc, alpha, air, CementType::OPC52_5R);
        let slag =
            StrengthEngine::compute_powers_with_cement_type(wc, alpha, air, CementType::CEM_III_B);

        println!("OPC 42.5N: {:.1} MPa", opc.compressive_strength);
        println!("OPC 52.5R: {:.1} MPa", rapid.compressive_strength);
        println!("CEM III/B: {:.1} MPa", slag.compressive_strength);

        // Rapid hardening > Standard > Slag cement
        assert!(rapid.compressive_strength > opc.compressive_strength);
        assert!(opc.compressive_strength > slag.compressive_strength);

        // Ratio should match intrinsic strength ratio
        let ratio = rapid.compressive_strength / opc.compressive_strength;
        let expected_ratio = 100.0 / 80.0;
        assert!(
            (ratio - expected_ratio).abs() < 0.01,
            "Strength ratio should match intrinsic ratio"
        );
    }

    #[test]
    fn test_blended_cement_strength() {
        // [V8.1] Test blended cement calculation
        let cement_mass = 300.0; // kg
        let scm_mass = 100.0; // kg fly ash
        let scm_efficiency = 0.5; // Typical fly ash efficiency

        let blended_intrinsic = StrengthEngine::compute_blended_intrinsic_strength(
            cement_mass,
            CementType::OPC42_5N,
            scm_mass,
            scm_efficiency,
        );

        // Expected: (300*80 + 100*80*0.5) / 400 = (24000 + 4000) / 400 = 70 MPa
        assert!(
            (blended_intrinsic - 70.0).abs() < 0.1,
            "Blended intrinsic should be ~70 MPa, got {}",
            blended_intrinsic
        );

        // Pure cement should have higher intrinsic than blended
        let pure_intrinsic = CementType::OPC42_5N.intrinsic_strength();
        assert!(pure_intrinsic > blended_intrinsic);
    }
}
