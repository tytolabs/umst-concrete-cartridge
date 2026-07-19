// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Pure `f32`/`f64` cement physics — no tensors, no heap handles.
//!
//! All chemistry scalars route through [`umst_concrete_cartridge::chem_adapter`] → `umst-chem` SSOT.

use umst_concrete_cartridge::chem_adapter;
use umst_concrete_cartridge::material_transition::CementMaterialParams;
use umst_manifold::gate::ThermodynamicStateSnapshot;

/// Avrami–Parrott hydration degree α ∈ [0, 1].
#[must_use]
pub fn hydration_degree(age_days: f32, temp_c: f32, scm_ratio: f32) -> f32 {
    chem_adapter::hydration_degree_calibrated(age_days, temp_c, scm_ratio, 1.0)
}

/// Powers gel-space compressive strength (MPa).
#[must_use]
pub fn strength_powers(
    wc_ratio: f32,
    degree_hydration: f32,
    air_content: f32,
    intrinsic_strength: f32,
) -> f32 {
    chem_adapter::powers_compressive_strength_f32(
        wc_ratio,
        degree_hydration,
        air_content,
        intrinsic_strength,
    )
}

/// Thermodynamic snapshot from mix scalars (Powers closure).
#[must_use]
pub fn thermo_snapshot_from_mix(w_c: f64, alpha: f64, temp: f64) -> ThermodynamicStateSnapshot {
    ThermodynamicStateSnapshot::from_mix_calibrated_with_params(
        w_c,
        alpha,
        temp,
        chem_adapter::cartridge_default_intrinsic_strength_mpa(),
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
        max_strength: chem_adapter::cartridge_default_intrinsic_strength_mpa(),
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
