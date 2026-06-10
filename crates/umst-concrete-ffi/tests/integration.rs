// Cement C-ABI integration tests (pure morphisms; no gate filter handles).

use umst_concrete_ffi::*;

#[test]
fn test_hydration_degree_monotone_with_age() {
    let alpha_7d = umst_hydration_degree(7.0, 20.0, 0.0);
    let alpha_28d = umst_hydration_degree(28.0, 20.0, 0.0);
    assert!(
        alpha_28d >= alpha_7d,
        "Hydration degree must be monotone with age: α(7d)={alpha_7d}, α(28d)={alpha_28d}"
    );
}

#[test]
fn test_strength_powers_monotone_with_hydration() {
    let fc_low = umst_strength_powers(0.45, 0.40, 0.02, 234.0);
    let fc_high = umst_strength_powers(0.45, 0.70, 0.02, 234.0);
    assert!(
        fc_high >= fc_low,
        "Powers strength must be monotone with α: fc(0.40)={fc_low}, fc(0.70)={fc_high}"
    );
}

#[test]
fn test_theorem1_from_mix_forward_fields() {
    let old_state = umst_thermo_state_from_mix(0.45, 0.40, 20.0);
    let new_state = umst_thermo_state_from_mix(0.45, 0.60, 20.0);

    assert!(new_state.hydration_degree > old_state.hydration_degree);
    assert!(new_state.free_energy <= old_state.free_energy);
    assert!(new_state.strength >= old_state.strength);
}

#[test]
fn test_thermo_state_from_mix_ptr() {
    let mut out = CThermodynamicState {
        density: 0.0,
        free_energy: 0.0,
        hydration_degree: 0.0,
        strength: 0.0,
        max_strength: 0.0,
    };
    unsafe {
        umst_thermo_state_from_mix_ptr(0.45, 0.40, 20.0, &mut out);
    }
    assert!(out.hydration_degree > 0.0);
    assert!(out.strength > 0.0);
}
