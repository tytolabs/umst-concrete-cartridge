// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! ═══════════════════════════════════════════════════════════════════════════
//! PRINTABILITY ENGINE - 3D Concrete Printing Assessment
//! ═══════════════════════════════════════════════════════════════════════════
//!
//! Computes printability metrics for 3D concrete printing applications.
//!
//! # Key Metrics
//! - **Extrudability**: Ability to flow through the nozzle without clogging
//! - **Buildability**: Ability to support subsequent layers without deformation
//! - **Open Time**: Working window before material becomes unprintable
//!
//! # Physics Models
//! - Roussel's buildability model (critical height)
//! - Thixotropy-based stiffening rate
//! - Nozzle flow assessment using Bingham parameters

use wasm_bindgen::prelude::*;

/// Printability assessment result
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct PrintabilityResult {
    /// Extrudability score (0-1): 1 = easily extrudable, 0 = will clog
    pub extrudability: f32,
    /// Buildability score (0-1): 1 = excellent shape retention, 0 = will collapse
    pub buildability: f32,
    /// Open time in minutes before material becomes unprintable
    pub open_time_min: f32,
    /// Critical height (mm) - max height before bottom layer collapse
    pub critical_height_mm: f32,
    /// Overall printability score (0-1)
    pub overall_score: f32,
}

/// Printing parameters for assessment
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct PrintingParams {
    /// Layer height in mm
    pub layer_height_mm: f32,
    /// Nozzle diameter in mm
    pub nozzle_diameter_mm: f32,
    /// Print speed in mm/s
    pub print_speed_mm_s: f32,
    /// Target structure height in mm
    pub target_height_mm: f32,
}

#[wasm_bindgen]
impl PrintingParams {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        PrintingParams {
            layer_height_mm: 10.0,    // Typical layer height
            nozzle_diameter_mm: 25.0, // Typical nozzle
            print_speed_mm_s: 100.0,  // Moderate speed
            target_height_mm: 500.0,  // 50cm target height
        }
    }

    pub fn with_layer_height(mut self, height: f32) -> Self {
        self.layer_height_mm = height;
        self
    }

    pub fn with_nozzle(mut self, diameter: f32) -> Self {
        self.nozzle_diameter_mm = diameter;
        self
    }
}

impl Default for PrintingParams {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
pub struct PrintabilityEngine;

#[wasm_bindgen]
impl PrintabilityEngine {
    /// Assess printability of concrete with given rheological properties
    ///
    /// # Arguments
    /// * `yield_stress_pa` - Static yield stress (Pa)
    /// * `plastic_viscosity_pa_s` - Plastic viscosity (Pa.s)
    /// * `thixotropy_index_pa_s` - Structural buildup rate (Pa/s), aka A_thix
    /// * `params` - Printing parameters
    ///
    /// # Physics
    /// - Extrudability: Based on Bingham number (τ₀ / (η × γ̇))
    /// - Buildability: Based on Roussel's critical height model
    /// - Open time: Based on thixotropy and yield stress evolution
    pub fn assess(
        yield_stress_pa: f32,
        plastic_viscosity_pa_s: f32,
        thixotropy_index_pa_s: f32,
        params: &PrintingParams,
    ) -> PrintabilityResult {
        // ═══════════════════════════════════════════════════════════════════
        // 1. EXTRUDABILITY ASSESSMENT
        // ═══════════════════════════════════════════════════════════════════
        // Based on flow through nozzle. High yield stress = difficult extrusion.
        // Optimal range: τ₀ = 100-500 Pa for most systems

        let extrudability = Self::compute_extrudability(
            yield_stress_pa,
            plastic_viscosity_pa_s,
            params.nozzle_diameter_mm,
            params.print_speed_mm_s,
        );

        // ═══════════════════════════════════════════════════════════════════
        // 2. BUILDABILITY ASSESSMENT
        // ═══════════════════════════════════════════════════════════════════
        // Based on Roussel's model for critical number of layers
        // N_crit = τ₀ / (ρ × g × h_layer)

        let (buildability, critical_height_mm) = Self::compute_buildability(
            yield_stress_pa,
            thixotropy_index_pa_s,
            params.layer_height_mm,
            params.target_height_mm,
        );

        // ═══════════════════════════════════════════════════════════════════
        // 3. OPEN TIME ASSESSMENT
        // ═══════════════════════════════════════════════════════════════════
        // Time until yield stress exceeds pumpable threshold (~2000 Pa)
        // τ(t) = τ₀ + A_thix × t

        let open_time_min = Self::compute_open_time(yield_stress_pa, thixotropy_index_pa_s);

        // ═══════════════════════════════════════════════════════════════════
        // 4. OVERALL SCORE
        // ═══════════════════════════════════════════════════════════════════
        // Geometric mean of individual scores (all must be good)
        let overall_score = (extrudability * buildability).sqrt();

        // Penalize if open time is too short
        let time_penalty = if open_time_min < 30.0 {
            open_time_min / 30.0
        } else {
            1.0
        };

        let overall_score = overall_score * time_penalty;

        PrintabilityResult {
            extrudability,
            buildability,
            open_time_min,
            critical_height_mm,
            overall_score,
        }
    }

    /// Compute extrudability score (0-1)
    ///
    /// Based on Bingham number and practical nozzle flow considerations.
    /// - Optimal yield stress range: 100-800 Pa for extrusion
    /// - Above 1000 Pa: significant pumping pressure required
    /// - Above 1500 Pa: risk of nozzle clogging or pump stall
    fn compute_extrudability(
        yield_stress_pa: f32,
        viscosity_pa_s: f32,
        nozzle_mm: f32,
        speed_mm_s: f32,
    ) -> f32 {
        // Shear rate in nozzle: γ̇ ≈ 8v/d (wall shear rate approximation)
        let nozzle_m = nozzle_mm / 1000.0;
        let speed_m_s = speed_mm_s / 1000.0;
        let shear_rate = 8.0 * speed_m_s / nozzle_m;

        // Bingham number: Bn = τ₀ / (η × γ̇)
        // High Bn = plug flow dominates = harder to extrude
        let bingham_number = if shear_rate > 0.0 && viscosity_pa_s > 0.0 {
            yield_stress_pa / (viscosity_pa_s * shear_rate)
        } else {
            10.0 // Default high value = poor extrudability
        };

        // Score based on Bingham number ranges:
        // Bn < 1: Easy flow (too fluid for 3D printing usually)
        // Bn 1-5: Good printability window
        // Bn > 10: Difficult extrusion (plug flow)
        let bn_score = if bingham_number < 0.5 {
            0.6 // Too fluid, will spread
        } else if bingham_number < 1.0 {
            0.8 + 0.2 * (bingham_number - 0.5) / 0.5
        } else if bingham_number < 5.0 {
            1.0 - 0.2 * (bingham_number - 1.0) / 4.0 // Peak at Bn~1
        } else if bingham_number < 10.0 {
            0.8 - 0.3 * (bingham_number - 5.0) / 5.0
        } else {
            0.5 - 0.3 * ((bingham_number - 10.0) / 10.0).min(1.0)
        };

        // Direct yield stress penalty for very high values
        // Above 1000 Pa: pumpability degrades significantly
        let yield_penalty = if yield_stress_pa < 500.0 {
            1.0
        } else if yield_stress_pa < 1000.0 {
            1.0 - 0.3 * (yield_stress_pa - 500.0) / 500.0
        } else if yield_stress_pa < 2000.0 {
            0.7 - 0.5 * (yield_stress_pa - 1000.0) / 1000.0
        } else {
            0.2 // Very difficult to pump
        };

        // Viscosity score
        let viscosity_score = if viscosity_pa_s < 5.0 {
            viscosity_pa_s / 5.0 // Too thin
        } else if viscosity_pa_s < 100.0 {
            1.0 // Good range
        } else {
            (100.0 / viscosity_pa_s).max(0.3) // Too thick
        };

        (bn_score * yield_penalty * viscosity_score)
            .max(0.0)
            .min(1.0)
    }

    /// Compute buildability score and critical height
    ///
    /// Based on Roussel's model for layer-by-layer deposition:
    /// - Initial h_crit = τ₀ / (ρ × g)
    /// - With thixotropy, each layer stiffens before next is added
    /// - Critical height grows as: h_total ∝ Σ τ(t_n) / (ρ × g)
    fn compute_buildability(
        yield_stress_pa: f32,
        thixotropy_pa_s: f32,
        _layer_height_mm: f32,
        target_height_mm: f32,
    ) -> (f32, f32) {
        // Concrete density (typical)
        let rho = 2400.0; // kg/m³
        let g = 9.81; // m/s²

        // Time per layer (typical for a layer around perimeter)
        let time_per_layer_s = 30.0;

        // Roussel's model with thixotropic stiffening:
        // For layer n at bottom, it has had (n-1) × t_layer to stiffen
        // τ_bottom = τ₀ + A_thix × (n-1) × t_layer
        // Critical layers before collapse:
        // n × ρ × g × h_layer = τ_bottom
        // Solving: we can iterate to find max stable height

        // Simplified: compute effective structural buildup
        // After total time T, average yield stress is higher
        // Use characteristic time for significant buildup
        let t_char = 60.0; // 60 seconds characteristic time
        let effective_tau = yield_stress_pa + thixotropy_pa_s * t_char;

        // Critical height with effective yield stress
        let h_crit_m = effective_tau / (rho * g);
        let h_crit_mm = h_crit_m * 1000.0;

        // For continuous printing with good thixotropy, multiply by factor
        // representing cumulative stiffening of all layers
        // Roussel's analysis shows factor of 3-10x for good thixotropy
        let thixo_factor = if thixotropy_pa_s > 0.1 {
            let stiffening_ratio = thixotropy_pa_s * time_per_layer_s / yield_stress_pa.max(1.0);
            // Factor scales logarithmically with stiffening ratio
            1.0 + 3.0 * (1.0 + stiffening_ratio).ln().min(2.0)
        } else {
            1.0
        };

        let effective_h_crit_mm = h_crit_mm * thixo_factor;

        // Buildability score: ratio of achievable to target height
        let buildability = if target_height_mm <= 0.0 {
            1.0
        } else {
            (effective_h_crit_mm / target_height_mm).min(1.0)
        };

        // Layer stability factor (shape retention at nozzle exit)
        let layer_stability = if yield_stress_pa < 50.0 {
            (yield_stress_pa / 50.0).max(0.1)
        } else {
            1.0
        };

        (
            (buildability * layer_stability).max(0.0).min(1.0),
            effective_h_crit_mm,
        )
    }

    /// Compute open time (minutes)
    ///
    /// Time until yield stress exceeds pumpable/extrudable threshold
    /// Using τ(t) = τ₀ + A_thix × t
    fn compute_open_time(yield_stress_pa: f32, thixotropy_pa_s: f32) -> f32 {
        // Upper threshold for extrusion (beyond this, nozzle clogs)
        let max_yield_for_pumping = 2000.0; // Pa

        if yield_stress_pa >= max_yield_for_pumping {
            return 0.0; // Already unprintable
        }

        if thixotropy_pa_s <= 0.001 {
            return 120.0; // No stiffening, 2 hour default window
        }

        // Time to reach max: t = (τ_max - τ₀) / A_thix
        let time_s = (max_yield_for_pumping - yield_stress_pa) / thixotropy_pa_s;
        let time_min = time_s / 60.0;

        // Cap at reasonable maximum (2 hours)
        time_min.min(120.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimal_printable_mix() {
        // Ideal 3D printing mix: high yield + high thixotropy
        // Real 3D printing mixes need 500-1000 Pa initial yield
        let mut params = PrintingParams::new();
        params.target_height_mm = 50.0; // Conservative target

        let result = PrintabilityEngine::assess(
            500.0, // 500 Pa yield stress (3D printing typical)
            40.0,  // 40 Pa.s viscosity
            2.0,   // 2.0 Pa/s thixotropy (strong buildup)
            &params,
        );

        println!("Optimal 3D printing mix results:");
        println!("  Extrudability: {:.2}", result.extrudability);
        println!("  Buildability: {:.2}", result.buildability);
        println!("  Open time: {:.1} min", result.open_time_min);
        println!("  Critical height: {:.1} mm", result.critical_height_mm);
        println!("  Overall: {:.2}", result.overall_score);

        // For real 3D printing concrete:
        // - Extrudability degrades above 500 Pa but still usable
        // - Buildability depends on critical height vs target
        // - Open time should be reasonable (>10 min)
        assert!(
            result.extrudability > 0.4,
            "Extrudability should be usable at 500Pa"
        );
        assert!(
            result.critical_height_mm > 30.0,
            "Critical height should exceed 30mm"
        );
        assert!(result.open_time_min > 10.0, "Open time should be >10 min");
    }

    #[test]
    fn test_too_fluid_mix() {
        // Too fluid: low yield stress, low thixotropy
        let params = PrintingParams::new();
        let result = PrintabilityEngine::assess(
            30.0, // Very low yield stress
            5.0,  // Low viscosity
            0.05, // Low thixotropy
            &params,
        );

        // Buildability should be poor (will collapse)
        assert!(
            result.buildability < 0.5,
            "Fluid mix should have poor buildability"
        );
        println!(
            "Fluid mix: buildability={:.2}, h_crit={:.1}mm",
            result.buildability, result.critical_height_mm
        );
    }

    #[test]
    fn test_too_stiff_mix() {
        // Too stiff for extrusion: very high yield stress, high viscosity
        let mut params = PrintingParams::new();
        params.target_height_mm = 100.0; // Reasonable target for comparison

        let result = PrintabilityEngine::assess(
            2000.0, // Very high yield stress - near pump limit
            150.0,  // High viscosity
            3.0,    // High thixotropy
            &params,
        );

        println!("Stiff mix: extrudability={:.2}, buildability={:.2}, open_time={:.1}min, h_crit={:.1}mm", 
                 result.extrudability, result.buildability, result.open_time_min, result.critical_height_mm);

        // Key insight: 2000 Pa is at the pumpability limit
        // Extrudability should be poor (very hard to pump)
        assert!(
            result.extrudability < 0.3,
            "2000Pa mix should be hard to extrude"
        );

        // Critical height should be high (good shape retention)
        assert!(
            result.critical_height_mm > 80.0,
            "High yield should give good critical height"
        );

        // Open time = 0 because already at threshold
        assert!(
            result.open_time_min <= 1.0,
            "Already at pump limit = no open time"
        );
    }

    #[test]
    fn test_critical_height_physics() {
        // Verify critical height matches Roussel's model
        let params = PrintingParams::new();

        // At τ₀ = 200 Pa, ρ = 2400 kg/m³, g = 9.81 m/s²
        // h_crit = 200 / (2400 * 9.81) ≈ 0.0085 m = 8.5 mm (per layer)
        // But with thixotropy bonus, should be higher
        let result = PrintabilityEngine::assess(200.0, 30.0, 0.5, &params);

        // Should be above simple theoretical value due to thixotropy
        assert!(
            result.critical_height_mm > 8.0,
            "Critical height should be at least 8mm at 200Pa yield"
        );
        println!(
            "Critical height at τ₀=200Pa: {:.1} mm",
            result.critical_height_mm
        );
    }
}
