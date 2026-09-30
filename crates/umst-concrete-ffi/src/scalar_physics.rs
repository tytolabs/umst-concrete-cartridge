// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Pure `f32`/`f64` cement physics — no tensors, no heap handles.

use umst_concrete_cartridge::material_transition::CementMaterialParams;
use umst_manifold::gate::ThermodynamicStateSnapshot;

use umst_concrete_cartridge::CEMENT_DEFAULT_S_INTRINSIC_MPA;

/// Avrami–Parrott hydration degree α ∈ [0, 1].
#[must_use]
pub fn hydration_degree(age_days: f32, temp_c: f32, scm_ratio: f32) -> f32 {
    let alpha_max = 0.95 - scm_ratio * 0.15;
    let k_ref = 0.55f32;
    let t_ref_k = 293.15f32;
    let t_k = temp_c + 273.15;
    let e_over_r = 5000.0f32;
    let temp_factor = (e_over_r * (1.0 / t_ref_k - 1.0 / t_k)).exp();
    let scm_factor = 1.0 - scm_ratio * 0.4;
    let k = k_ref * temp_factor * scm_factor;
    let alpha = alpha_max * (1.0 - (-k * age_days.sqrt()).exp());
    alpha.clamp(0.0, 1.0)
}

/// Powers gel-space compressive strength (MPa).
#[must_use]
pub fn strength_powers(
    wc_ratio: f32,
    degree_hydration: f32,
    air_content: f32,
    intrinsic_strength: f32,
) -> f32 {
    if wc_ratio > 100.0 {
        return 0.0;
    }
    let vg_volume_gel = 0.68 * degree_hydration;
    let vc_volume_capillary = wc_ratio - 0.36 * degree_hydration;
    let space = vg_volume_gel + vc_volume_capillary + air_content;
    if space <= 0.001 {
        return 0.0;
    }
    let x = vg_volume_gel / space;
    intrinsic_strength * x.powi(3)
}

/// Thermodynamic snapshot from mix scalars (Powers closure).
#[must_use]
pub fn thermo_snapshot_from_mix(w_c: f64, alpha: f64, temp: f64) -> ThermodynamicStateSnapshot {
    ThermodynamicStateSnapshot::from_mix_calibrated_with_params(
        w_c,
        alpha,
        temp,
        CEMENT_DEFAULT_S_INTRINSIC_MPA,
        &CementMaterialParams,
    )
}

/// C-ABI struct mirror for Haskell `Storable` consumers.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CThermodynamicState {
    pub density: f64,
    pub free_energy: f64,
    pub hydration_degree: f64,
    pub strength: f64,
    pub max_strength: f64,
}

#[must_use]
pub fn c_state_from_mix(w_c: f64, alpha: f64, temp: f64) -> CThermodynamicState {
    let snap = thermo_snapshot_from_mix(w_c, alpha, temp);
    CThermodynamicState {
        density: snap.density,
        free_energy: snap.free_energy,
        hydration_degree: snap.reaction_extent,
        strength: snap.strength,
        max_strength: CEMENT_DEFAULT_S_INTRINSIC_MPA,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hydration_monotone_with_age() {
        let a7 = hydration_degree(7.0, 20.0, 0.0);
        let a28 = hydration_degree(28.0, 20.0, 0.0);
        assert!(a28 >= a7);
    }

    #[test]
    fn strength_monotone_with_alpha() {
        let low = strength_powers(0.45, 0.40, 0.02, 234.0);
        let high = strength_powers(0.45, 0.70, 0.02, 234.0);
        assert!(high >= low);
    }

    #[test]
    fn from_mix_matches_snapshot_fields() {
        let snap = thermo_snapshot_from_mix(0.45, 0.5, 293.0);
        let c = c_state_from_mix(0.45, 0.5, 293.0);
        assert_eq!(c.density, snap.density);
        assert_eq!(c.free_energy, snap.free_energy);
        assert!(c.strength > 0.0);
    }
}
