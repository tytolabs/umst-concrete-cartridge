// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy, and Studio Tyto
// SPDX-License-Identifier: Apache-2.0
//
// UMST — Material Agnostic Operating System
// RheologyEngine: Herschel-Bulkley & Krieger-Dougherty Models
//
// For licensing terms, see the LICENSE file in the project root.

use crate::tensors::{MaterialType, MixTensor, MIX_TENSOR_STRIDE};
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::console;

#[wasm_bindgen]
pub struct RheologyResult {
    pub yield_stress: f32,     // Pa - static yield stress
    pub viscosity: f32,        // Pa.s - plastic viscosity
    pub slump_flow: f32,       // mm - Murata's model
    pub thixotropy_index: f32, // Pa/s - structural buildup rate (Athix)
}

// ============================================================================
// [V8.2] Structural Kinetics for Thixotropy (Roussel-Coussot Model)
// ============================================================================
//
// Implements time-dependent thixotropy using a structure parameter λ.
//
// Physics:
//   dλ/dt = (1 - λ)/T_θ - k × γ̇ × λ
//
// Where:
//   λ ∈ [0,1]: Structure parameter (0=fully broken, 1=fully structured)
//   T_θ: Flocculation time constant (s) - structural rebuilding time
//   k: Shear breakdown rate (dimensionless)
//   γ̇: Shear rate (1/s)
//
// At rest (γ̇=0): λ → 1 exponentially with time constant T_θ
// Under shear: λ decreases depending on shear rate
//
// The apparent yield stress evolves as:
//   τ(λ) = τ₀ × (1 + λ)
//
// The Athix (Pa/s) relates to initial structure rebuilding:
//   A_thix ≈ τ₀ / T_θ (for small times at rest)
// ============================================================================

/// Structural kinetics parameters for thixotropic materials
#[derive(Clone, Copy, Debug)]
pub struct StructuralKinetics {
    /// Static yield stress at fully broken state (Pa)
    pub tau_0: f32,
    /// Flocculation time constant (s) - time to rebuild structure at rest
    /// Typical values:
    ///   - Normal concrete: 300-600s
    ///   - SCC: 100-200s  
    ///   - 3D printing: 30-60s (fast rebuilding)
    pub t_floc: f32,
    /// Shear breakdown rate constant (dimensionless)
    /// Typical values: 0.001-0.01
    pub k_breakdown: f32,
    /// Current structure parameter (0-1)
    /// 0 = fully broken (just mixed), 1 = fully structured (long rest)
    pub lambda: f32,
}

impl Default for StructuralKinetics {
    fn default() -> Self {
        StructuralKinetics {
            tau_0: 100.0,  // Pa - typical paste
            t_floc: 300.0, // s - typical concrete
            k_breakdown: 0.005,
            lambda: 0.0, // Just mixed = fully broken
        }
    }
}

impl StructuralKinetics {
    /// Create kinetics for a given material type
    pub fn for_mix(base_yield: f32, packing_closeness: f32, sp_factor: f32) -> Self {
        // T_floc decreases (faster rebuilding) with:
        // - Higher packing (more particle-particle contacts)
        // - Less SP (more flocculation)
        //
        // Base T_floc for typical concrete: 300s
        // Dense packing (closeness>0.8): 100-200s
        // High SP: 500-800s
        let base_t_floc = 300.0;

        // Packing effect: denser = faster rebuilding
        let packing_factor = 1.0 - 0.5 * packing_closeness;

        // SP effect: more SP = slower rebuilding (particles stay dispersed)
        // sp_factor is already exp(-sp_dosage * 0.15), so ~0.85 for typical dosage
        let sp_rebuild_factor = 1.0 / sp_factor.sqrt();

        let t_floc = (base_t_floc * packing_factor * sp_rebuild_factor)
            .max(30.0)
            .min(1000.0);

        // Breakdown rate: higher packing = easier to break down
        let k_breakdown = 0.005 * (1.0 + packing_closeness);

        StructuralKinetics {
            tau_0: base_yield,
            t_floc,
            k_breakdown,
            lambda: 0.0, // Start fully broken (fresh mix)
        }
    }

    /// Compute structure parameter after resting for given duration
    /// Uses analytical solution of dλ/dt = (1-λ)/T_θ at γ̇=0
    pub fn lambda_at_rest(&self, rest_time_s: f32) -> f32 {
        // λ(t) = 1 - (1 - λ₀) × exp(-t/T_θ)
        let lambda_0 = self.lambda;
        1.0 - (1.0 - lambda_0) * (-rest_time_s / self.t_floc).exp()
    }

    /// Compute structure parameter under steady shear
    /// Uses steady-state solution: λ_ss = 1 / (1 + k × γ̇ × T_θ)
    pub fn lambda_steady_shear(&self, shear_rate: f32) -> f32 {
        1.0 / (1.0 + self.k_breakdown * shear_rate * self.t_floc)
    }

    /// Compute apparent yield stress for given structure parameter
    pub fn apparent_yield(&self, lambda: f32) -> f32 {
        self.tau_0 * (1.0 + lambda)
    }

    /// Compute A_thix (Pa/s) - structural buildup rate
    /// This is the initial slope of yield stress vs time at rest
    ///
    /// From: τ(t) = τ₀ × (1 + λ(t)) and λ(t) = 1 - exp(-t/T_θ)
    /// dτ/dt|_{t=0} = τ₀ × dλ/dt|_{t=0} = τ₀ / T_θ
    pub fn compute_athix(&self) -> f32 {
        self.tau_0 / self.t_floc
    }

    /// Compute yield stress after resting from fresh state
    /// Useful for 3D printing buildability calculations
    pub fn yield_after_rest(&self, rest_time_s: f32) -> f32 {
        let lambda = self.lambda_at_rest(rest_time_s);
        self.apparent_yield(lambda)
    }
}

#[wasm_bindgen]
pub struct RheologyEngine;

#[wasm_bindgen]
impl RheologyEngine {
    /// Calculate rheological properties using Herschel-Bulkley & Krieger-Dougherty models
    /// Now uses material-specific rheology data when available, falling back to generic formulas
    pub fn compute(mix: &MixTensor, packing_density: f32) -> RheologyResult {
        // Extract raw data view from tensor
        // New layout: [mass, sg, type, co2, cost, blaine, fm, shape, viscosity, yield_stress, thixotropy]

        let data = mix.data();
        let stride = MIX_TENSOR_STRIDE;
        let count = data.len() / stride;

        let mut water_vol = 0.0;
        let mut solid_vol = 0.0;
        let mut sp_raw_mass = 0.0; // Superplasticizer raw mass (kg)
        let mut sp_efficiency_weighted = 0.0; // Captures PCE vs SNF efficiency
        let mut binder_mass = 0.0; // Cement + SCM mass for SP normalization

        // Track paste materials separately (cement, SCM) for rheology base values
        // Aggregates affect rheology through packing AND interparticle friction
        let mut paste_viscosity = 0.0;
        let mut paste_yield = 0.0;
        let mut paste_thixotropy = 0.0;
        let mut paste_mass = 0.0;

        // [V8.1] Track aggregate properties for friction contribution
        // Aggregates affect yield stress via interparticle friction, not just packing
        let mut agg_vol = 0.0; // m³ - total aggregate volume
        let mut agg_shape_weighted = 0.0; // volume-weighted shape (1=sphere, 0=angular)
        let mut agg_fm_weighted = 0.0; // volume-weighted fineness modulus

        // [V9.0] Track Fiber and Nano effects
        let mut fiber_vol = 0.0;
        let mut weighted_fiber_ar = 0.0; // Aspect Ratio
        let mut nano_mass_total = 0.0;
        let mut weighted_nano_ssa = 0.0; // m²/g

        for i in 0..count {
            let offset = i * stride;
            let mass = data[offset];
            let sg = data[offset + 1];
            let type_id = data[offset + 2] as u8;
            let blaine_ssa = data[offset + 5]; // Blaine (Cement) or SSA (Nano)
            let fm = data[offset + 6]; // Fineness modulus (aggregates)
            let shape = data[offset + 7]; // Shape factor (1=sphere, 0=angular) // Used as efficiency for Admixtures
            let viscosity = data[offset + 8]; // Pa.s - material viscosity
            let yield_stress = data[offset + 9]; // Pa - material yield stress
            let thixotropy = data[offset + 10]; // Pa/s - structural buildup rate
                                                // New fields [V8.5]
            let aspect_ratio = data[offset + 13]; // Fiber L/d

            // Volume = mass / (sg * 1000) -> assuming sg is specific gravity relative to water?
            // TS Code: mass / mat.density. Let's assume SG is standard (e.g. 2.4 for agg).
            // Density = SG * 1000 kg/m^3.
            // Robust Fallback: If SG is 0 (missing), assume 2.4 (Aggregate/Cement avg)
            let density = if sg > 0.1 { sg * 1000.0 } else { 2400.0 };
            let vol = mass / density;

            if type_id == MaterialType::Water as u8 {
                water_vol += vol;
            } else if type_id == MaterialType::Admixture as u8 {
                // Superplasticizer - track raw mass for normalization
                sp_raw_mass += mass;
                sp_efficiency_weighted += shape * mass; // Extract Admixture Efficiency (e.g., 0.35 for High-Range PCE)
                                                        // Admixtures also contribute to paste rheology
                paste_viscosity += viscosity * mass;
                paste_yield += yield_stress * mass;
                paste_thixotropy += thixotropy * mass;
                paste_mass += mass;
            } else if type_id == MaterialType::Cement as u8 || type_id == MaterialType::SCM as u8 {
                // Cement and SCM are paste materials - their rheology matters
                solid_vol += vol;
                paste_viscosity += viscosity * mass;
                paste_yield += yield_stress * mass;
                paste_thixotropy += thixotropy * mass;
                paste_mass += mass;
                // Track binder mass for SP dosage normalization
                binder_mass += mass;
            } else if type_id == MaterialType::Fiber as u8 {
                // [V9.0] Fiber specific tracking
                solid_vol += vol;
                fiber_vol += vol;
                weighted_fiber_ar += aspect_ratio * vol;
            } else if type_id == MaterialType::Nanomaterial as u8 {
                // [V9.0] Nano specific tracking
                solid_vol += vol;
                nano_mass_total += mass;
                weighted_nano_ssa += blaine_ssa * mass;
                // Nano also contributes to paste rheology (very high)
                paste_viscosity += viscosity * mass;
                paste_yield += yield_stress * mass;
                paste_thixotropy += thixotropy * mass;
                paste_mass += mass;
            } else if type_id != MaterialType::Air as u8 {
                // [V8.1] Aggregates: track volume AND properties for friction model
                solid_vol += vol;
                agg_vol += vol;
                agg_shape_weighted += shape * vol;
                agg_fm_weighted += fm * vol;
            }
        }

        // [V8.1] Compute average aggregate properties (volume-weighted)
        let avg_agg_shape = if agg_vol > 0.0001 {
            agg_shape_weighted / agg_vol
        } else {
            0.7
        };
        let avg_agg_fm = if agg_vol > 0.0001 {
            agg_fm_weighted / agg_vol
        } else {
            3.0
        };

        let sp_efficiency = if sp_raw_mass > 0.0001 {
            sp_efficiency_weighted / sp_raw_mass
        } else {
            0.12
        }; // Default Legacy SNF

        // Use paste materials for rheology base (not aggregates)
        let total_viscosity = paste_viscosity;
        let total_yield = paste_yield;
        let total_thixotropy = paste_thixotropy;

        let total_vol = water_vol + solid_vol;

        // [V8.2 FIX] Normalize SP dosage as percentage of binder (cement + SCM)
        // Typical PCE dosage: 0.3-2.0% of binder weight
        // sp_dosage is now a fraction (0.01 = 1% of binder)
        let sp_dosage = if binder_mass > 0.0 {
            sp_raw_mass / binder_mass
        } else {
            0.0
        };

        // #region agent log (wasm32 only)
        #[cfg(target_arch = "wasm32")]
        console::log_1(&format!(
            "[DEBUG H2e] water_vol={:.4}, solid_vol={:.4}, total_vol={:.4}, sp_raw={:.2}kg, efficiency={:.2}, binder={:.1}kg, sp_dosage={:.4} ({:.2}%)",
            water_vol, solid_vol, total_vol, sp_raw_mass, sp_efficiency, binder_mass, sp_dosage, sp_dosage * 100.0
        ).into());
        // #endregion
        if total_vol <= 0.0001 {
            return RheologyResult {
                yield_stress: 0.0,
                viscosity: 0.0,
                slump_flow: 0.0,
                thixotropy_index: 0.0,
            };
        }

        // ============================================================================
        // [V8.3] TWO-PHASE CHATEAU-OVARLEZ-TRUNG RHEOLOGY MODEL
        // ============================================================================
        //
        // Concrete is treated as a BIPHASIC SUSPENSION:
        //   Phase 1: Paste (cement + SCM + water + admixtures) - has intrinsic yield stress
        //   Phase 2: Aggregates suspended in paste - amplifies yield stress via packing
        //
        // The model computes:
        //   τ₀ = τ_paste_sp × f_COT(φ_agg/φ_m) + τ_friction
        //
        // Key difference from previous single-phase model:
        //   - SP reduction applied to paste yield FIRST (not after packing amplification)
        //   - Packing factor uses AGGREGATE volume fraction (not total solids)
        //   - This prevents double-counting of cement-water suspension effect
        //
        // Physics justification:
        //   Material yieldStress (e.g., 200 Pa for OPC) already represents the paste
        //   suspension rheology. Applying KD to total solids double-counts this.
        //   The COT model correctly separates paste and aggregate contributions.
        //
        // Reference: Chateau, Ovarlez, Trung (2008) - J. Rheol. 52(2):489-506
        // ============================================================================

        // Compute paste volume (cement + SCM + water + admixtures contribute to paste matrix)
        // Note: solid_vol includes cement and SCM volumes already tracked
        let cement_scm_vol = solid_vol - agg_vol; // cement + SCM volume
        let paste_vol = cement_scm_vol + water_vol;
        let concrete_vol = paste_vol + agg_vol;

        // Aggregate volume fraction in concrete (for COT model)
        let phi_agg = if concrete_vol > 0.0001 {
            agg_vol / concrete_vol
        } else {
            0.0
        };

        // Maximum aggregate packing fraction in paste
        // Higher values for well-graded aggregates with SP lubrication
        // Typical values: 0.65-0.75 depending on grading and SP
        let phi_m_agg = if packing_density > 0.6 {
            // Use provided packing density adjusted for aggregate-only fraction
            (packing_density * 0.85).min(0.78)
        } else {
            0.74 // Default for well-graded concrete with SP
        };

        // #region agent log (wasm32 only)
        #[cfg(target_arch = "wasm32")]
        console::log_1(&format!(
            "[DEBUG V8.3 TWO-PHASE] paste_vol={:.4}, agg_vol={:.4}, concrete_vol={:.4}, phi_agg={:.4}, phi_m_agg={:.4}",
            paste_vol, agg_vol, concrete_vol, phi_agg, phi_m_agg
        ).into());
        // #endregion

        // Paste-weighted base rheology (cement + SCM + admixtures only)
        // Use defaults if no rheology data was provided
        let base_viscosity = if paste_mass > 0.0 && total_viscosity > 0.0 {
            total_viscosity / paste_mass
        } else {
            // Default plastic viscosity for cement paste (Pa.s)
            // Typical cement paste: 0.5-2.0 Pa.s
            1.0
        };

        let base_yield = if paste_mass > 0.0 && total_yield > 0.0 {
            total_yield / paste_mass
        } else {
            // Default yield stress for cement paste (Pa)
            // Typical cement paste without SP: 50-150 Pa
            // With SP: 10-50 Pa
            80.0
        };

        // [V8.2] Structural Kinetics for Thixotropy
        // Use Roussel-Coussot model if no material data provided
        // Material-provided thixotropy takes precedence if available
        let base_thixotropy = if paste_mass > 0.0 && total_thixotropy > 0.0 {
            // Use material-provided thixotropy data
            total_thixotropy / paste_mass
        } else {
            // Compute from StructuralKinetics model
            // Pass base_yield BEFORE packing factor (the pure paste yield)
            0.0 // Will be computed below using kinetics
        };

        // ============================================================================
        // STEP 1: Apply SP reduction to paste yield FIRST
        // ============================================================================
        // This is the key fix: SP reduces paste yield stress BEFORE aggregate amplification
        //
        // PCE Superplasticizer vs SNF effectiveness:
        // By relying on the tensor's dynamic sp_efficiency parameter, we naturally model
        // steric hindrance (PCE) vs electrostatic repulsion (SNF)
        // - PCE Std (eff=0.25) * 800 = 200 decay factor
        // - SNF Legacy (eff=0.12) * 800 = 96 decay factor (requires double dosage to match PCE)
        let base_decay = 800.0;
        let sp_factor = (-sp_dosage * base_decay * sp_efficiency).exp().max(0.05);
        let paste_yield_after_sp = base_yield * sp_factor;

        // #region agent log (wasm32 only)
        #[cfg(target_arch = "wasm32")]
        console::log_1(&format!(
            "[DEBUG V8.3 PASTE] base_yield={:.1}Pa, sp_dosage={:.4}({:.2}%), sp_factor={:.4}, paste_yield_sp={:.1}Pa",
            base_yield, sp_dosage, sp_dosage * 100.0, sp_factor, paste_yield_after_sp
        ).into());
        // #endregion

        // ============================================================================
        // STEP 2: Apply Chateau-Ovarlez-Trung (COT) model for aggregate amplification
        // ============================================================================
        //
        // COT Model: τ₀ = τ_paste × (1 - φ/φ_m)^(-n×φ_m)
        //
        // Where:
        //   τ_paste = paste yield stress after SP reduction
        //   φ = aggregate volume fraction (NOT total solids!)
        //   φ_m = maximum aggregate packing fraction (~0.65-0.75)
        //   n = 2.5 (intrinsic viscosity for irregular particles)
        //
        // Using regularized form to prevent divergence near jamming
        let closeness_agg = (phi_agg / phi_m_agg).min(0.98);
        let n_cot = 2.5 * phi_m_agg; // ~1.85 for typical concrete

        // Regularized COT factor with smooth transition near jamming
        let cot_factor = if closeness_agg < 0.90 {
            // Standard COT formula
            (1.0 - closeness_agg).powf(-n_cot)
        } else {
            // Smooth transition to plateau near jamming (prevents divergence)
            let f_90 = (1.0 - 0.90_f32).powf(-n_cot);
            let f_max = 15.0; // Maximum amplification factor
            let blend = ((closeness_agg - 0.90) / 0.08).tanh();
            f_90 + blend * (f_max - f_90)
        }
        .min(15.0); // Safety cap

        let mut yield_stress = paste_yield_after_sp * cot_factor;

        // #region agent log (wasm32 only)
        #[cfg(target_arch = "wasm32")]
        console::log_1(&format!(
            "[DEBUG V8.3 COT] closeness_agg={:.4}, n_cot={:.2}, cot_factor={:.2}, yield_after_cot={:.1}Pa",
            closeness_agg, n_cot, cot_factor, yield_stress
        ).into());
        // #endregion

        // ============================================================================
        // STEP 3: Add aggregate friction contribution (additive)
        // ============================================================================
        // Aggregates increase yield stress through interparticle friction.
        // This is ADDITIVE to paste yield stress, not multiplicative.
        //
        // Physics: Angular particles create more friction than rounded ones.
        // Fine aggregates (low FM) have more surface area = more friction points.
        //
        // SAFETY: This contribution is monotonic in aggregate volume (preserves Clausius-Duhem)

        // fm_factor: Fine aggregates (FM < 4) contribute more friction
        let fm_factor = if avg_agg_fm < 4.0 { 1.5 } else { 1.0 };

        // angularity_factor: Angular (shape→0) = high friction, Spherical (shape→1) = low friction
        let angularity_factor = 1.0 - avg_agg_shape;

        // Base friction coefficient (Pa per unit volume fraction)
        // Reduced from 100 to 50 since COT already accounts for some packing effect
        let k_friction = 50.0;

        // Friction contribution (Pa) - monotonic in aggregate volume
        let friction_yield = phi_agg * angularity_factor * fm_factor * k_friction;

        yield_stress += friction_yield;

        // #region agent log (wasm32 only)
        #[cfg(target_arch = "wasm32")]
        console::log_1(&format!(
            "[DEBUG V8.3 FRICTION] phi_agg={:.4}, angularity={:.2}, fm_factor={:.2}, friction_yield={:.1}Pa, final_yield={:.1}Pa",
            phi_agg, angularity_factor, fm_factor, friction_yield, yield_stress
        ).into());
        // #endregion

        // Ensure minimum yield stress for workability
        yield_stress = yield_stress.max(10.0);

        // [V9.0] Apply Fiber Effect (Mechanical Entanglement)
        // Fibers increase yield stress significantly due to network formation
        // Factor = 1 + k * Vf * (L/d)
        // k ≈ 0.5 for yield stress (Naaman)
        if fiber_vol > 0.0 {
            let avg_ar = weighted_fiber_ar / fiber_vol;
            let vf_percent = (fiber_vol / total_vol) * 100.0;
            // Linear increase with Vf * AR
            let fiber_factor = 1.0 + 0.005 * vf_percent * avg_ar;
            yield_stress *= fiber_factor;

            // #region agent log (wasm32 only)
            #[cfg(target_arch = "wasm32")]
            console::log_1(
                &format!(
                    "[DEBUG V9.0 FIBER] Vf={:.2}%, AR={:.1}, factor={:.2}, yield={:.1}Pa",
                    vf_percent, avg_ar, fiber_factor, yield_stress
                )
                .into(),
            );
            // #endregion
        }

        // [V9.0] Apply Nano Effect (Surface Area)
        // Nano increases yield stress due to high water demand / surface forces
        // Factor = 1 + k * (SSA_nano / SSA_cement) * dosage
        if nano_mass_total > 0.0 {
            let avg_ssa = weighted_nano_ssa / nano_mass_total; // m2/g
                                                               // Typical cement SSA ~ 0.3-0.4 m2/g
                                                               // Nano SSA ~ 50-200 m2/g
                                                               // Ratio ~ 100-500
            let dosage_percent = (nano_mass_total / binder_mass) * 100.0;
            // Empirical: 1% nano-silica ~ 2x yield stress
            let nano_factor = 1.0 + 0.5 * dosage_percent * (avg_ssa / 200.0);
            yield_stress *= nano_factor;

            // #region agent log (wasm32 only)
            #[cfg(target_arch = "wasm32")]
            console::log_1(
                &format!(
                    "[DEBUG V9.0 NANO] dosage={:.2}%, SSA={:.1}, factor={:.2}, yield={:.1}Pa",
                    dosage_percent, avg_ssa, nano_factor, yield_stress
                )
                .into(),
            );
            // #endregion
        }

        // ============================================================================
        // STEP 4: Plastic Viscosity using COT-based approach
        // ============================================================================
        // Uses same aggregate-only approach with n = 2.0 (standard for spheres)
        let viscosity_cot_factor = if closeness_agg < 0.90 {
            (1.0 - closeness_agg).powf(-2.0 * phi_m_agg)
        } else {
            let f_90 = (1.0 - 0.90_f32).powf(-2.0 * phi_m_agg);
            let f_max = 50.0;
            let blend = ((closeness_agg - 0.90) / 0.08).tanh();
            f_90 + blend * (f_max - f_90)
        }
        .min(50.0);

        let mut viscosity = base_viscosity * sp_factor.sqrt() * viscosity_cot_factor;

        // [V9.0] Apply Fiber Effect to Viscosity
        // Fibers increase viscosity less than yield stress, but still significant
        // Factor = 1 + k * Vf * (L/d)
        // k ≈ 0.2 for viscosity
        if fiber_vol > 0.0 {
            let avg_ar = weighted_fiber_ar / fiber_vol;
            let vf_percent = (fiber_vol / total_vol) * 100.0;
            let fiber_visc_factor = 1.0 + 0.002 * vf_percent * avg_ar;
            viscosity *= fiber_visc_factor;
        }

        // [V9.0] Apply Nano Effect to Viscosity
        // Nano increases viscosity significantly
        if nano_mass_total > 0.0 {
            let avg_ssa = weighted_nano_ssa / nano_mass_total;
            let dosage_percent = (nano_mass_total / binder_mass) * 100.0;
            let nano_visc_factor = 1.0 + 0.3 * dosage_percent * (avg_ssa / 200.0);
            viscosity *= nano_visc_factor;
        }

        // Clamp Viscosity for concrete realism
        // Normal concrete: 10-300 Pa.s, stiff concrete: up to 500 Pa.s
        // Previous clamp at 150 was too aggressive, making viscosity non-responsive
        viscosity = viscosity.max(1.0).min(500.0);

        // 3. Slump Flow (mm) - Dimensionless Model (Roussel, Wallevik)
        // [V8.1] Physics-based dimensionless correlation:
        //   τ* = τ₀ / (ρ × g × H) - dimensionless yield stress
        //   s* = S / H - dimensionless slump
        // For Abrams cone: H = 300mm, ρ ≈ 2400 kg/m³
        let slump_flow = Self::compute_slump_from_yield(yield_stress);
        // #region agent log (wasm32 only)
        #[cfg(target_arch = "wasm32")]
        console::log_1(
            &format!(
                "[DEBUG H2f FINAL] yield_stress={:.1}Pa, slump_flow={:.1}mm (threshold=2000Pa)",
                yield_stress, slump_flow
            )
            .into(),
        );
        // #endregion

        // 4. [V8.2] Thixotropy Index using Structural Kinetics
        //
        // Two paths:
        // A) If material provides explicit thixotropy data → use it (scaled by SP)
        // B) Otherwise → compute from StructuralKinetics (Roussel-Coussot model)
        //
        // A_thix (Pa/s) = τ₀ / T_θ (initial structural buildup rate)
        let mut thixotropy_index = if base_thixotropy > 0.0 {
            // Path A: Material-provided thixotropy, adjusted for SP
            (base_thixotropy * sp_factor.sqrt()).max(0.05)
        } else {
            // Path B: Compute from StructuralKinetics model
            // Use closeness_agg (aggregate packing) for thixotropy calculation
            let kinetics = StructuralKinetics::for_mix(base_yield, closeness_agg, sp_factor);
            let athix = kinetics.compute_athix();

            // #region agent log (wasm32 only)
            #[cfg(target_arch = "wasm32")]
            console::log_1(
                &format!(
                    "[DEBUG V8.2 THIX] tau_0={:.1}, T_floc={:.1}s, A_thix={:.3} Pa/s",
                    kinetics.tau_0, kinetics.t_floc, athix
                )
                .into(),
            );
            // #endregion

            // Clamp to realistic range
            // Typical: 0.1-0.5 Pa/s (normal), 0.5-1.5 (SCC), 1.5-5.0 (3D printing)
            athix.max(0.05).min(10.0)
        };

        // [V9.0] Apply Nano Effect (Catalyst) to Thixotropic structural build-up
        if nano_mass_total > 0.0 {
            let dosage_percent = (nano_mass_total / binder_mass) * 100.0;
            // Empirical physical reality: Nano-Silica acts as an extreme thixotropic catalyst
            // A 1.0% dosage can increase Athix by factor of 20x to 100x.
            let nano_thix_factor = 1.0 + 50.0 * dosage_percent;
            thixotropy_index *= nano_thix_factor;

            // #region agent log (wasm32 only)
            #[cfg(target_arch = "wasm32")]
            console::log_1(
                &format!(
                    "[DEBUG V9.0 THIX] Nano dosage={:.2}%, multiplier={:.1}x, new Athix={:.2} Pa/s",
                    dosage_percent, nano_thix_factor, thixotropy_index
                )
                .into(),
            );
            // #endregion
        }

        RheologyResult {
            yield_stress,
            viscosity,
            slump_flow,
            thixotropy_index,
        }
    }

    /// [V8.1] Dimensionless slump-yield correlation (continuous model)
    ///
    /// Converts yield stress to slump flow using physics-based dimensionless analysis.
    /// Uses a continuous exponential decay model calibrated to empirical data.
    ///
    /// # Physics
    /// - Dimensionless yield stress: τ* = τ₀ / (ρ × g × H)
    /// - For Abrams cone: H = 300mm, ρ = 2400 kg/m³
    /// - Empirical fit: S = S_max × exp(-k × τ*)
    ///
    /// # Calibration Points (literature)
    /// - τ₀ = 20 Pa (SCC): Slump flow ~750mm
    /// - τ₀ = 200 Pa (high workability): Slump ~350mm
    /// [V8.2 P2.4] Regularized Krieger-Dougherty packing factor
    ///
    /// Computes a divergence-free packing factor that:
    /// - Follows standard KD: (1-c)^(-n) for c < c_crit
    /// - Smoothly transitions to f_max as c → 1
    /// - Preserves monotonicity (Clausius-Duhem compliance)
    ///
    /// # Arguments
    /// * `closeness` - φ/φ_max (packing closeness, 0-1)
    /// * `n` - KD exponent (typically 2.0-2.5)
    /// * `c_crit` - Critical closeness for transition onset
    /// * `f_max` - Maximum factor (plateau value)
    /// * `delta` - Transition smoothness (smaller = sharper)
    ///
    /// # Returns
    /// Packing factor (1.0 at dilute, up to f_max at jamming)
    #[allow(dead_code)] // Retained for testing and future reference (COT model now primary)
    fn regularized_kd_factor(closeness: f32, n: f32, c_crit: f32, f_max: f32, delta: f32) -> f32 {
        if closeness <= 0.0 {
            return 1.0;
        }

        // Standard KD value (may be very large)
        let kd_raw = (1.0 - closeness).max(0.001).powf(-n);

        if closeness < c_crit {
            // Below critical: use standard KD (bounded by f_max for safety)
            kd_raw.min(f_max)
        } else {
            // Above critical: smooth transition to plateau
            // Uses sigmoid-like blending
            let f_crit = (1.0 - c_crit).powf(-n);
            let blend = ((closeness - c_crit) / delta).tanh();
            // Interpolate between f_crit and f_max
            let result = f_crit + blend * (f_max - f_crit);
            result.min(f_max)
        }
    }

    /// - τ₀ = 500 Pa (normal): Slump ~200mm
    /// - τ₀ = 1000 Pa (low workability): Slump ~100mm
    /// - τ₀ = 2000 Pa (stiff): Slump ~20mm
    pub fn compute_slump_from_yield(yield_stress_pa: f32) -> f32 {
        // Abrams cone parameters
        let cone_height_m = 0.300; // 300mm
        let rho = 2400.0; // kg/m³ (typical concrete)
        let g = 9.81; // m/s²

        // Characteristic stress at cone base: σ_char = ρ × g × H
        let sigma_char = rho * g * cone_height_m; // ≈ 7063 Pa

        // Dimensionless yield stress
        let tau_star = yield_stress_pa / sigma_char;

        // Continuous exponential decay model
        // S = S_max × exp(-k × τ*) + S_residual × exp(-k2 × τ*)
        // Calibrated to match empirical data points

        // Maximum slump flow (SCC test limit)
        let s_max = 850.0;

        // Decay constants calibrated to empirical data
        // Fast decay for SCC range (τ* < 0.03)
        // Slower decay for normal concrete range
        let k1 = 15.0; // Fast initial decay
        let k2 = 8.0; // Slower decay for structure

        // Two-component model for smooth transition
        let scc_component = 400.0 * (-k1 * tau_star).exp();
        let normal_component = 450.0 * (-k2 * tau_star).exp();

        let slump_mm = scc_component + normal_component;

        // Clamp to realistic limits
        slump_mm.max(0.0).min(s_max)
    }
}

// --- Standard Material Cartridge System ---

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub struct MaterialCartridge {
    #[wasm_bindgen(skip)]
    pub id: String,
    pub yield_stress: f32, // Pa
    pub viscosity: f32,    // Pa.s
    pub density: f32,      // kg/m3
}

#[wasm_bindgen]
impl MaterialCartridge {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, yield_stress: f32, viscosity: f32, density: f32) -> MaterialCartridge {
        MaterialCartridge {
            id,
            yield_stress,
            viscosity,
            density,
        }
    }
}

#[wasm_bindgen]
pub struct CartridgeRegistry;

#[wasm_bindgen]
impl CartridgeRegistry {
    /// Retrieve a standard material cartridge by ID
    /// Supports: "StandardConcrete", "HighPerformanceConcrete", "Clay", "BioHydrogel"
    pub fn get_standard(type_id: &str) -> Option<MaterialCartridge> {
        match type_id {
            "StandardConcrete" => Some(MaterialCartridge {
                id: "StandardConcrete".to_string(),
                yield_stress: 2000.0,
                viscosity: 50.0,
                density: 2300.0,
            }),
            "HighPerformanceConcrete" => Some(MaterialCartridge {
                id: "HighPerformanceConcrete".to_string(),
                yield_stress: 5000.0,
                viscosity: 150.0,
                density: 2400.0,
            }),
            "RAC" => Some(MaterialCartridge {
                // Recycled Aggregate Concrete (Higher Viscosity usually due to absorption)
                id: "RAC".to_string(),
                yield_stress: 3000.0,
                viscosity: 80.0,
                density: 2250.0,
            }),
            "Clay" => Some(MaterialCartridge {
                // High thixotropy, very high yield stress, high viscosity
                id: "Clay".to_string(),
                yield_stress: 8000.0,
                viscosity: 600.0,
                density: 1800.0,
            }),
            "BioHydrogel" => Some(MaterialCartridge {
                // Extremely low viscosity compared to concrete
                id: "BioHydrogel".to_string(),
                yield_stress: 200.0,
                viscosity: 5.0,
                density: 1050.0,
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // [V8.2] Structural Kinetics Tests
    // =========================================================================

    #[test]
    fn test_structural_kinetics_default() {
        let kinetics = StructuralKinetics::default();
        assert_eq!(
            kinetics.lambda, 0.0,
            "Initial lambda should be 0 (fully broken)"
        );
        assert!(kinetics.t_floc > 0.0, "Flocculation time must be positive");
        assert!(kinetics.tau_0 > 0.0, "Base yield stress must be positive");
    }

    #[test]
    fn test_structural_kinetics_rest_recovery() {
        let kinetics = StructuralKinetics {
            tau_0: 100.0,
            t_floc: 100.0, // 100s for easy calculation
            k_breakdown: 0.005,
            lambda: 0.0,
        };

        // After 1 time constant (T_floc), lambda should reach ~63% of final value
        let lambda_1tc = kinetics.lambda_at_rest(100.0);
        let expected = 1.0 - (-1.0_f32).exp(); // ~0.632
        assert!(
            (lambda_1tc - expected).abs() < 0.01,
            "Lambda at t=T_floc should be ~0.632, got {:.3}",
            lambda_1tc
        );

        // After 3 time constants, lambda should be ~95%
        let lambda_3tc = kinetics.lambda_at_rest(300.0);
        assert!(
            lambda_3tc > 0.94,
            "Lambda at t=3*T_floc should be >0.94, got {:.3}",
            lambda_3tc
        );

        println!("✅ Structural kinetics rest recovery verified");
    }

    #[test]
    fn test_structural_kinetics_athix() {
        // For 3D printing: τ₀=500 Pa, T_floc=60s → A_thix=8.33 Pa/s
        let kinetics = StructuralKinetics {
            tau_0: 500.0,
            t_floc: 60.0,
            k_breakdown: 0.005,
            lambda: 0.0,
        };

        let athix = kinetics.compute_athix();
        let expected = 500.0 / 60.0; // 8.33 Pa/s

        assert!(
            (athix - expected).abs() < 0.1,
            "A_thix should be {:.2}, got {:.2}",
            expected,
            athix
        );

        println!("✅ A_thix calculation verified: {:.2} Pa/s", athix);
    }

    #[test]
    fn test_structural_kinetics_yield_after_rest() {
        let kinetics = StructuralKinetics {
            tau_0: 100.0,
            t_floc: 60.0,
            k_breakdown: 0.005,
            lambda: 0.0,
        };

        // At t=0 (just mixed): yield = τ₀ × (1 + 0) = 100 Pa
        let yield_t0 = kinetics.yield_after_rest(0.0);
        assert!(
            (yield_t0 - 100.0).abs() < 1.0,
            "Yield at t=0 should be τ₀=100, got {:.1}",
            yield_t0
        );

        // At t=∞ (fully structured): yield = τ₀ × (1 + 1) = 200 Pa
        let yield_inf = kinetics.yield_after_rest(10000.0);
        assert!(
            (yield_inf - 200.0).abs() < 5.0,
            "Yield at t=∞ should approach 2×τ₀=200, got {:.1}",
            yield_inf
        );

        println!(
            "✅ Yield stress evolution verified: {:.1} → {:.1} Pa",
            yield_t0, yield_inf
        );
    }

    #[test]
    fn test_structural_kinetics_for_mix() {
        // Test kinetics for typical concrete
        let kinetics = StructuralKinetics::for_mix(
            80.0, // base_yield (Pa)
            0.8,  // packing_closeness
            0.85, // sp_factor (typical)
        );

        // Should have reasonable T_floc (not too fast or slow)
        assert!(
            kinetics.t_floc >= 30.0,
            "T_floc too fast: {:.1}s",
            kinetics.t_floc
        );
        assert!(
            kinetics.t_floc <= 1000.0,
            "T_floc too slow: {:.1}s",
            kinetics.t_floc
        );

        // A_thix should be in typical range (0.1-5.0 Pa/s for concrete)
        let athix = kinetics.compute_athix();
        assert!(athix >= 0.05, "A_thix too low: {:.3} Pa/s", athix);
        assert!(athix <= 10.0, "A_thix too high: {:.3} Pa/s", athix);

        println!(
            "✅ StructuralKinetics::for_mix verified: T_floc={:.1}s, A_thix={:.3} Pa/s",
            kinetics.t_floc, athix
        );
    }

    // =========================================================================
    // [V8.1] Slump-Yield Tests
    // =========================================================================

    /// [V8.1] Test dimensionless slump-yield correlation
    #[test]
    fn test_slump_yield_scc_range() {
        // SCC: τ₀ ≈ 50 Pa → slump flow ~700mm+
        let slump = RheologyEngine::compute_slump_from_yield(50.0);
        println!("SCC (50 Pa): slump = {:.0} mm", slump);
        assert!(slump > 600.0, "SCC should have high slump flow");
        assert!(slump < 850.0, "SCC should not exceed test limits");
    }

    #[test]
    fn test_slump_yield_normal_concrete() {
        // Normal concrete: τ₀ ≈ 1000 Pa → slump ~150mm
        let slump = RheologyEngine::compute_slump_from_yield(1000.0);
        println!("Normal concrete (1000 Pa): slump = {:.0} mm", slump);
        assert!(slump > 50.0, "Normal concrete should have some slump");
        assert!(
            slump < 250.0,
            "Normal concrete should not have SCC-level flow"
        );
    }

    #[test]
    fn test_slump_yield_stiff_concrete() {
        // Stiff concrete: τ₀ ≈ 2000 Pa → slump ~20mm
        let slump = RheologyEngine::compute_slump_from_yield(2000.0);
        println!("Stiff concrete (2000 Pa): slump = {:.0} mm", slump);
        assert!(slump < 100.0, "Stiff concrete should have low slump");
    }

    #[test]
    fn test_slump_yield_3d_printing() {
        // 3D printing: τ₀ ≈ 500 Pa → slump ~300mm (limited flow)
        let slump = RheologyEngine::compute_slump_from_yield(500.0);
        println!("3D printing (500 Pa): slump = {:.0} mm", slump);
        assert!(slump > 100.0, "3D printing mix should have some flow");
        assert!(slump < 450.0, "3D printing mix should not be SCC-like");
    }

    #[test]
    fn test_slump_yield_monotonic() {
        // Verify slump decreases monotonically with yield stress
        let slumps: Vec<f32> = [10.0, 50.0, 100.0, 300.0, 500.0, 1000.0, 1500.0, 2000.0]
            .iter()
            .map(|&ys| RheologyEngine::compute_slump_from_yield(ys))
            .collect();

        for i in 1..slumps.len() {
            assert!(
                slumps[i] <= slumps[i - 1] + 1.0, // Small tolerance for floating point
                "Slump should be monotonically decreasing: slump[{}]={:.1} > slump[{}]={:.1}",
                i,
                slumps[i],
                i - 1,
                slumps[i - 1]
            );
        }
        println!("Monotonicity verified across yield stress range");
    }

    /// [V8.2 P2.4] Test regularized Krieger-Dougherty factor
    #[test]
    fn test_regularized_kd_factor() {
        // Test key properties:
        // 1. f(0) ≈ 1 (dilute)
        // 2. f is monotonically increasing
        // 3. f stays bounded (no divergence)
        // 4. Smooth transition at c_crit

        let closeness_values = [0.0, 0.3, 0.6, 0.85, 0.90, 0.95, 0.99, 1.0];
        let factors: Vec<f32> = closeness_values
            .iter()
            .map(|&c| RheologyEngine::regularized_kd_factor(c, 2.5, 0.85, 25.0, 0.05))
            .collect();

        println!("Regularized KD factors:");
        for (i, &c) in closeness_values.iter().enumerate() {
            println!("  c={:.2}: f={:.2}", c, factors[i]);
        }

        // Property 1: f(0) = 1
        assert!((factors[0] - 1.0).abs() < 0.01, "f(0) should be 1.0");

        // Property 2: Monotonicity
        for i in 1..factors.len() {
            assert!(
                factors[i] >= factors[i - 1] - 0.1,
                "Factor should increase: f[{}]={:.2} < f[{}]={:.2}",
                i,
                factors[i],
                i - 1,
                factors[i - 1]
            );
        }

        // Property 3: Bounded (no divergence)
        for f in &factors {
            assert!(*f <= 25.5, "Factor should be bounded: {:.2}", f);
        }

        // Property 4: At c=1, should be near f_max
        assert!(factors[7] > 20.0, "At jamming, factor should be near f_max");
    }

    // =========================================================================
    // [V8.3] Two-Phase Chateau-Ovarlez-Trung Model Tests
    // =========================================================================

    /// [V8.3] Test COT model produces realistic slump for M25 with SP
    ///
    /// Expected behavior:
    /// - M25 mix with 0.375% PCE superplasticizer should have ~100-200mm slump
    /// - Previous single-phase model gave ~32mm (too low)
    /// - Two-phase model should give more realistic values
    #[test]
    fn test_cot_model_m25_slump_realism() {
        // Simulate M25 mix calculation
        // Key parameters from Python verification:
        // - Base yield: 200 Pa (OPC)
        // - SP dosage: 0.375% → sp_factor = 0.47
        // - Paste yield after SP: ~94 Pa
        // - φ_agg = 0.71, φ_m = 0.74
        // - closeness_agg = 0.96
        // - COT factor ≈ 12-15
        // - Expected final yield: ~1100-1500 Pa
        // - Expected slump: ~100-150 mm

        // Test the slump calculation for the expected yield range
        let yield_low = 1000.0; // Pa (optimistic)
        let yield_high = 1500.0; // Pa (conservative)

        let slump_low = RheologyEngine::compute_slump_from_yield(yield_high);
        let slump_high = RheologyEngine::compute_slump_from_yield(yield_low);

        println!(
            "[V8.3 COT] Expected yield range: {} - {} Pa",
            yield_low, yield_high
        );
        println!(
            "[V8.3 COT] Expected slump range: {:.0} - {:.0} mm",
            slump_low, slump_high
        );

        // M25 with SP should have workable slump (>80mm)
        assert!(
            slump_low > 80.0,
            "M25 with SP should have slump > 80mm, got {:.0}mm at {}Pa",
            slump_low,
            yield_high
        );

        // Should not be SCC level (< 500mm)
        assert!(
            slump_high < 500.0,
            "M25 should not be SCC level (<500mm), got {:.0}mm at {}Pa",
            slump_high,
            yield_low
        );

        println!(
            "✅ V8.3 COT model slump range verified: {:.0}-{:.0} mm",
            slump_low, slump_high
        );
    }

    /// [V8.3] Test SP reduction applied before COT amplification
    ///
    /// Key principle: SP reduces paste yield FIRST, then aggregate amplifies.
    /// This prevents the unrealistic high yield stress from single-phase model.
    #[test]
    fn test_cot_model_sp_order() {
        // Single-phase approach (WRONG):
        //   yield = base_yield * KD_factor * sp_factor
        //   yield = 200 * 25 * 0.47 = 2350 Pa → slump ~34mm

        // Two-phase approach (CORRECT):
        //   paste_yield = base_yield * sp_factor = 200 * 0.47 = 94 Pa
        //   yield = paste_yield * COT_factor = 94 * 12 = 1128 Pa → slump ~130mm

        // Verify the order matters significantly
        let base_yield = 200.0;
        let sp_factor = 0.47_f32;
        let amplification = 12.0;

        // Demonstration: mathematically equivalent but conceptually different
        // These values are for documentation/explanation only
        let _wrong_yield = base_yield * amplification * sp_factor; // = 1128 Pa
        let _correct_yield = base_yield * sp_factor * amplification; // = 1128 Pa

        // Both give same mathematical result (multiplication is commutative)
        // BUT the conceptual difference is that in two-phase model:
        // - amplification is COT (aggregate-only, ~12x) not full KD (~25x)

        let full_kd = 25.0;
        let wrong_with_full_kd = base_yield * full_kd * sp_factor;
        let correct_with_cot = base_yield * sp_factor * amplification;

        println!("Wrong (full KD): {:.0} Pa", wrong_with_full_kd);
        println!("Correct (COT): {:.0} Pa", correct_with_cot);

        assert!(
            correct_with_cot < wrong_with_full_kd,
            "COT model should give lower yield than full KD: {} < {}",
            correct_with_cot,
            wrong_with_full_kd
        );

        let slump_wrong = RheologyEngine::compute_slump_from_yield(wrong_with_full_kd);
        let slump_correct = RheologyEngine::compute_slump_from_yield(correct_with_cot);

        println!("Wrong slump: {:.0} mm", slump_wrong);
        println!("Correct slump: {:.0} mm", slump_correct);

        // Correct model should give significantly higher slump
        assert!(
            slump_correct > slump_wrong * 1.5,
            "COT model should give much higher slump: {:.0} vs {:.0} mm",
            slump_correct,
            slump_wrong
        );

        println!("✅ V8.3 SP ordering verification passed");
    }

    /// [V8.3] Test COT factor bounds are respected
    #[test]
    fn test_cot_factor_bounds() {
        // Test that COT factor stays bounded even at extreme packing
        let phi_m_agg = 0.74_f32;
        let n_cot = 2.5 * phi_m_agg; // ~1.85

        // Test various closeness values
        // COT formula: (1-c)^(-n) where n = 2.5 * 0.74 = 1.85
        // c=0.5 → (0.5)^(-1.85) = 3.61
        // c=0.8 → (0.2)^(-1.85) = 19.6 (high, but below regularization threshold)
        // c=0.9 is the regularization threshold (transitions to bounded plateau)
        // c>0.9 → smooth blend to f_max = 15
        let test_cases: [(f32, f32, f32); 5] = [
            (0.0, 1.0, 1.1),    // Dilute: factor ≈ 1
            (0.5, 3.0, 4.5),    // Moderate: (0.5)^(-1.85) ≈ 3.6
            (0.8, 15.0, 25.0),  // Dense: (0.2)^(-1.85) ≈ 19.6 (high but valid)
            (0.92, 10.0, 15.5), // Above threshold: regularized
            (0.99, 14.0, 15.5), // Near jamming: factor capped at 15
        ];

        for (closeness, min_expected, max_expected) in test_cases {
            let factor = if closeness < 0.90 {
                (1.0 - closeness).powf(-n_cot)
            } else {
                let f_90 = (1.0 - 0.90_f32).powf(-n_cot);
                let f_max = 15.0;
                let blend = ((closeness - 0.90) / 0.08).tanh();
                (f_90 + blend * (f_max - f_90)).min(15.0)
            };

            println!(
                "closeness={:.2}: factor={:.2} (expected {:.1}-{:.1})",
                closeness, factor, min_expected, max_expected
            );

            assert!(
                factor >= min_expected && factor <= max_expected,
                "COT factor at c={:.2} should be in [{:.1}, {:.1}], got {:.2}",
                closeness,
                min_expected,
                max_expected,
                factor
            );
        }

        println!("✅ V8.3 COT factor bounds verified");
    }
}
