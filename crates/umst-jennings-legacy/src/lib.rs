//! Pure-f64 Jennings strength — legacy algorithm from `umst-concrete-cartridge`
//! `physics/strength.rs` (R10-A0). No Burn. No golden-constant return.
//!
//! SPDX-License-Identifier: MIT

#![forbid(unsafe_code)]

/// Cement solid volume basis coefficient — inventory E-07.
pub const CEMENT_VOLUME_PER_WC: f64 = 0.317;
/// C-S-H gel volume multiplier — inventory E-06.
pub const CSH_VOLUME_FACTOR: f64 = 1.52;
/// Jennings LD slope — inventory E-05.
pub const CSH_LD_FRAC_SLOPE: f64 = 3.017;
/// Jennings LD intercept subtrahend — inventory E-05 (`−CSH_LD_FRAC_INTERCEPT`).
pub const CSH_LD_FRAC_INTERCEPT_SUBTRAHEND: f64 = 0.347;
/// Powers non-evaporable water coefficient — inventory B-02.
pub const POWERS_NON_EVAP_WATER_COEFF: f64 = 0.36;
/// E→fc stiffness bridge — inventory E-09 (cartridge calibration).
pub const E_TO_FC_STIFFNESS_BRIDGE: f64 = 0.05;
/// Ulm–Constantinides LD gel Young's modulus [GPa] (Vinet-anchored pin).
pub const E_LD_GPA: f64 = 21.7;
/// Ulm–Constantinides HD gel Young's modulus [GPa] (Vinet-anchored pin).
pub const E_HD_GPA: f64 = 29.4;

/// Pure f64 image of the Burn [`StrengthEngine::compute_strength_jennings`] algorithm.
///
/// Returns `(fc_mpa, v_hd, v_ld)`. Does **not** return a pinned golden constant.
#[must_use]
pub fn compute_strength_jennings_f64(
    wc_ratio: f64,
    degree_hydration: f64,
    air_content: f64,
    intrinsic_strength: f64,
) -> (f64, f64, f64) {
    let safe_wc = wc_ratio.clamp(0.20, 0.80);
    let v_cement = safe_wc.powf(-1.0) * CEMENT_VOLUME_PER_WC;
    let v_csh_total = degree_hydration * v_cement * CSH_VOLUME_FACTOR;
    let ld_fraction =
        (safe_wc * CSH_LD_FRAC_SLOPE - CSH_LD_FRAC_INTERCEPT_SUBTRAHEND).clamp(0.0, 1.0);
    let hd_fraction = 1.0 - ld_fraction;
    let v_ld = v_csh_total * ld_fraction;
    let v_hd = v_csh_total * hd_fraction;
    let e_matrix = v_ld * E_LD_GPA + v_hd * E_HD_GPA;
    let porosity_capillary = (safe_wc - degree_hydration * POWERS_NON_EVAP_WATER_COEFF).max(0.0);
    let total_porosity = porosity_capillary + air_content;
    let solid_fraction = (1.0 - total_porosity).max(0.01);
    let e_eff = e_matrix * solid_fraction.powi(3);
    let compressive_strength = e_eff * intrinsic_strength * E_TO_FC_STIFFNESS_BRIDGE;
    (compressive_strength, v_hd, v_ld)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orchestrator_pin_near_measured_golden() {
        let (fc, _, _) = compute_strength_jennings_f64(0.45, 0.75, 0.02, 80.0);
        assert!(
            (fc - 35.689_57).abs() < 0.05,
            "legacy pin fc={fc} drifted from STRENGTH_GOLDEN_FC_MPA"
        );
    }
}
