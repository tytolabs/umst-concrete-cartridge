// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

use burn::tensor::{backend::Backend, Tensor};

use crate::chem_adapter::{
    cement_volume_per_wc_f32, clinker_bulk_modulus_ambient_gpa_f32, csh_hd_scale_of_bulk_f32,
    csh_ld_frac_intercept_subtrahend_f32, csh_ld_frac_slope_f32, csh_ld_scale_of_bulk_f32,
    csh_volume_factor_f32, csh_youngs_moduli_from_k0_f32, e_to_fc_stiffness_bridge_f32,
    powers_non_evap_water_coeff_f32, ClinkerPhaseTag,
};

// Track H2 (v0.4): DFT-backed bulk moduli for clinker / C-S-H phases live in [`super::clinker_eos`].
// The two C-S-H Young's moduli used below (E_LD = 21.7 GPa, E_HD = 29.4 GPa) are no longer hardcoded —
// they are derived from Pellenq et al. 2009 (PNAS) DFT bulk modulus K_csh = 70 GPa via empirical
// LD/HD scaling factors (Jennings 2000 / Ulm & Constantinides 2004 nano-indent anchors). This makes
// the strength prediction a hierarchical homogenisation grounded in DFT phase moduli, not an
// orphaned empirical constant. See `paste_csh_youngs_moduli_gpa()` below.
//
// formal_anchor: literature://micromechanics/csh-modulus-dft-anchored
// formal_citation: Pellenq et al. 2009 PNAS 106:16102 (DFT K_csh); Jennings 2000 CCR 30:101 (LD/HD partitioning); Ulm & Constantinides 2004 (gel-scale moduli)
//
// LD/HD scaling factors delegate to `umst-chem` via `chem_adapter` (cluster E). If you change
// `clinker_bulk_modulus_ambient_gpa_f32(ClinkerPhaseTag::Csh14nmTobermorite)`, the gel moduli scale linearly.
// Regression tests pin the Ulm–Constantinides anchors (21.7 / 29.4 GPa).

/// Returns Vinet-anchored Young's moduli `(E_LD, E_HD)` for C-S-H gel in GPa, derived from
/// Pellenq et al. 2009 DFT bulk modulus of 1.4-nm tobermorite. The scaling factors reproduce
/// the Ulm & Constantinides 2004 nano-indentation anchors (21.7, 29.4 GPa) within rounding.
///
/// formal_anchor: literature://micromechanics/csh-vinet-anchored-gel-moduli
/// formal_status: Literature
/// formal_citation: "Pellenq et al. 2009 PNAS 106:16102; Ulm & Constantinides 2004; Jennings 2000"
/// formal_form: "(E_LD, E_HD) = (csh_ld_scale, csh_hd_scale) * K_csh_vinet via chem_adapter"
#[must_use]
pub fn paste_csh_youngs_moduli_gpa() -> (f32, f32) {
    // Vinet EoS `K₀` for 1.4-nm tobermorite (ambient) — cluster D via chem_adapter.
    let k0_csh = clinker_bulk_modulus_ambient_gpa_f32(ClinkerPhaseTag::Csh14nmTobermorite);
    csh_youngs_moduli_from_k0_f32(k0_csh)
}

/// Pure tensor implementation of the Strength & Micromechanics Engine.
/// Upgraded to the absolute SOTA: Jennings CM-II (Colloidal Model of C-S-H)
/// coupled with Ulm & Constantinides (2004) nano-indentation continuum micromechanics.
/// formal_anchor: lean://umst-formal/Lean/Concrete/Powers.lean#powers_monotone
/// catalog_id: thermodynamic_mix
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
pub struct StrengthEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend> StrengthEngine<B> {
    /// Computes Compressive Strength using cutting-edge Continuum Micromechanics.
    /// Maps w/c and hydration into High-Density (HD) and Low-Density (LD) C-S-H fractions.
    ///
    /// # Arguments
    /// * `wc_ratio` - Water/Cement ratio tensor [Batch, Depth, Height, Width]
    /// * `degree_hydration` - Hydration degree tensor α (0.0 to 1.0)
    /// * `air_content` - Entrapped/entrained air volume fraction
    /// * `intrinsic_strength` - Intrinsic scaling factor for the specific cement chemistry (MPa)
    /// formal_anchor: lean://umst-formal/Lean/Concrete/Powers.lean#powers_monotone
    /// catalog_id: thermodynamic_mix
    /// formal_status: Mechanised
    /// formal_axioms: physicalSecondLaw
    pub fn compute_strength_jennings(
        wc_ratio: Tensor<B, 4>,
        degree_hydration: Tensor<B, 4>,
        air_content: Tensor<B, 4>,
        intrinsic_strength: Tensor<B, 4>,
    ) -> (Tensor<B, 4>, Tensor<B, 4>, Tensor<B, 4>) {
        let safe_wc = wc_ratio.clone().clamp(0.20_f32, 0.80_f32);

        // 1. Volumes of Phases (Tennis & Jennings, 2000)
        // Normalized volume scaling
        let v_cement = safe_wc
            .clone()
            .powf_scalar(-1.0_f32)
            .mul_scalar(cement_volume_per_wc_f32());
        let v_csh_total = degree_hydration
            .clone()
            .mul(v_cement)
            .mul_scalar(csh_volume_factor_f32());

        // 2. High-Density (HD) vs Low-Density (LD) C-S-H partitioning (Jennings via chem_adapter)
        let ld_fraction = safe_wc
            .clone()
            .mul_scalar(csh_ld_frac_slope_f32())
            .sub_scalar(csh_ld_frac_intercept_subtrahend_f32())
            .clamp(0.0_f32, 1.0_f32);
        let hd_fraction = ld_fraction.clone().mul_scalar(-1.0_f32).add_scalar(1.0_f32);

        let v_ld = v_csh_total.clone().mul(ld_fraction);
        let v_hd = v_csh_total.clone().mul(hd_fraction);

        // 3. Continuum Micromechanics (Ulm & Constantinides 2004, DFT-anchored via Pellenq 2009)
        // C-S-H gel Young's moduli derived from cluster D Vinet bulk modulus (70 GPa) × cluster E LD/HD scales.
        let (e_ld, e_hd) = paste_csh_youngs_moduli_gpa();

        // Effective Paste Modulus via rule of mixtures for C-S-H matrix (Voigt approx)
        let e_matrix = v_ld
            .clone()
            .mul_scalar(e_ld)
            .add(v_hd.clone().mul_scalar(e_hd));

        // 4. Porosity penalization (Capillary + Air)
        let porosity_capillary = safe_wc
            .clone()
            .sub(
                degree_hydration
                    .clone()
                    .mul_scalar(powers_non_evap_water_coeff_f32()),
            )
            .clamp_min(0.0_f32);
        let total_porosity = porosity_capillary.add(air_content);

        // Modulus reduction due to porosity (Balshin model: E = E0 * (1-p)^3)
        let solid_fraction = total_porosity
            .clone()
            .mul_scalar(-1.0_f32)
            .add_scalar(1.0_f32)
            .clamp_min(0.01_f32);
        let e_eff = e_matrix.mul(solid_fraction.clone().powf_scalar(3.0_f32));

        // 5. Strength Scaling
        // Strength is proportional to the effective stiffness of the C-S-H gel network.
        // E→fc bridge (inventory E-09) is cartridge calibration — not chem SSOT (prep §2.5).
        let compressive_strength = e_eff
            .mul(intrinsic_strength)
            .mul_scalar(e_to_fc_stiffness_bridge_f32());

        (compressive_strength, v_hd, v_ld)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pins the Vinet-anchored gel moduli to the historical Ulm–Constantinides nano-indentation
    /// anchors (21.7 / 29.4 GPa). If cluster D `K₀` or cluster E LD/HD scales ever drift, this
    /// test catches the regression so we don't silently change a published strength curve.
    #[test]
    fn strength_module_uses_vinet_anchored_csh_moduli() {
        let (e_ld, e_hd) = paste_csh_youngs_moduli_gpa();
        assert!(
            (e_ld - 21.7_f32).abs() < 0.5_f32,
            "E_LD drift: got {e_ld}, expected ≈21.7 GPa (Ulm & Constantinides 2004 nano-indent anchor)"
        );
        assert!(
            (e_hd - 29.4_f32).abs() < 0.5_f32,
            "E_HD drift: got {e_hd}, expected ≈29.4 GPa (Ulm & Constantinides 2004 nano-indent anchor)"
        );
        assert!(
            e_hd > e_ld,
            "HD must be stiffer than LD; got E_LD={e_ld}, E_HD={e_hd}"
        );
    }

    #[test]
    fn csh_youngs_moduli_scale_linearly_with_vinet_bulk_modulus() {
        let (e_ld, e_hd) = paste_csh_youngs_moduli_gpa();
        let k_csh = clinker_bulk_modulus_ambient_gpa_f32(ClinkerPhaseTag::Csh14nmTobermorite);
        assert!(
            (e_ld / k_csh - csh_ld_scale_of_bulk_f32()).abs() < 1e-6_f32,
            "LD scaling factor regressed"
        );
        assert!(
            (e_hd / k_csh - csh_hd_scale_of_bulk_f32()).abs() < 1e-6_f32,
            "HD scaling factor regressed"
        );
    }
}
