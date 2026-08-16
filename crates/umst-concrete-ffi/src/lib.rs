// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//
// umst-concrete-ffi — C-ABI cement chemistry (material-specific fiber).
//
// Every export is a pure morphism: scalars in → scalars out. No opaque filter handles.

mod scalar_physics;

pub use scalar_physics::{
    c_state_from_mix, hydration_degree, strength_powers, thermo_snapshot_from_mix,
    CThermodynamicState,
};

pub const UMST_CONCRETE_FFI_ABI_VERSION: u32 = 1;

#[must_use]
#[no_mangle]
pub extern "C" fn umst_concrete_ffi_abi_version() -> u32 {
    UMST_CONCRETE_FFI_ABI_VERSION
}

#[must_use]
#[no_mangle]
pub extern "C" fn umst_hydration_degree(age_days: f32, temp_c: f32, scm_ratio: f32) -> f32 {
    scalar_physics::hydration_degree(age_days, temp_c, scm_ratio)
}

#[must_use]
#[no_mangle]
pub extern "C" fn umst_strength_powers(
    wc_ratio: f32,
    degree_hydration: f32,
    air_content: f32,
    intrinsic_strength: f32,
) -> f32 {
    scalar_physics::strength_powers(wc_ratio, degree_hydration, air_content, intrinsic_strength)
}

#[must_use]
#[no_mangle]
pub extern "C" fn umst_thermo_state_from_mix(
    w_c: f64,
    alpha: f64,
    temp: f64,
) -> CThermodynamicState {
    scalar_physics::c_state_from_mix(w_c, alpha, temp)
}

/// # Safety
/// `out` must point to a valid `CThermodynamicState` (or be null → no-op).
#[no_mangle]
pub unsafe extern "C" fn umst_thermo_state_from_mix_ptr(
    w_c: f64,
    alpha: f64,
    temp: f64,
    out: *mut CThermodynamicState,
) {
    if out.is_null() {
        return;
    }
    *out = umst_thermo_state_from_mix(w_c, alpha, temp);
}
