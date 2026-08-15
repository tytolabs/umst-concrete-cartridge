// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

use burn::tensor::{backend::Backend, Tensor};

use crate::chem_adapter::{
    cement_volume_per_wc_f32, clinker_bulk_modulus_ambient_gpa_f32,
    csh_ld_frac_intercept_subtrahend_f32, csh_ld_frac_slope_f32, csh_volume_factor_f32,
    csh_youngs_moduli_from_k0_f32, e_to_fc_stiffness_bridge_f32, powers_non_evap_water_coeff_f32,
    ClinkerPhaseTag,
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

/// Orchestrator strength pin — matches `pipeline/orchestrator.rs` air default and
/// `calibration/profiles/default.v1.toml` `s_intrinsic`.
/// Class: **Primitive-fact** (routing contract, not fitted from f_c output).
/// Pub for `umst-diff` B3 old-side adapter (DIFF-HARNESS-B3).
pub const ORCHESTRATOR_PIN_WC: f32 = 0.45;
pub const ORCHESTRATOR_PIN_ALPHA: f32 = 0.75;
pub const ORCHESTRATOR_PIN_AIR: f32 = 0.02;
pub const ORCHESTRATOR_PIN_S_INTRINSIC: f32 = 80.0;

/// Measured golden compressive strength [MPa] at orchestrator paste pin — pinned by
/// `strength_engine_measured_golden_vector_paste_at_orchestrator_pin`.
/// Class: **Measured** (engine output under recorded pin, not invented).
/// Pub for `umst-diff` B3 old-side adapter (DIFF-HARNESS-B3).
pub const STRENGTH_GOLDEN_FC_MPA: f32 = 35.689_57_f32;

/// Pure f64 image of [`StrengthEngine::compute_strength_jennings`] for differential harnesses
/// (`umst-diff` R10-A0). Delegates to [`umst_jennings_legacy`] — the Burn-free extract of this
/// algorithm living under the concrete cartridge tree.
///
/// Returns `(fc_mpa, v_hd, v_ld)`. Does **not** return the pinned golden constant — callers
/// must compute from inputs. Not a claim of physics GREEN.
#[must_use]
pub fn compute_strength_jennings_f64(
    wc_ratio: f64,
    degree_hydration: f64,
    air_content: f64,
    intrinsic_strength: f64,
) -> (f64, f64, f64) {
    umst_jennings_legacy::compute_strength_jennings_f64(
        wc_ratio,
        degree_hydration,
        air_content,
        intrinsic_strength,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chem_adapter::{csh_hd_scale_of_bulk_f32, csh_ld_scale_of_bulk_f32};
    use burn::tensor::{Data, Shape, Tensor};
    use burn_ndarray::{NdArray, NdArrayDevice};

    type B = NdArray<f32>;

    fn scalar_rank4(v: f32) -> Tensor<B, 4> {
        let dev = NdArrayDevice::default();
        Tensor::from_data(Data::new(vec![v], Shape::new([1, 1, 1, 1])), &dev)
    }

    fn strength_at_pin(wc: f32, alpha: f32, air: f32, s_intrinsic: f32) -> (f32, f32, f32) {
        let (fc, v_hd, v_ld) = StrengthEngine::<B>::compute_strength_jennings(
            scalar_rank4(wc),
            scalar_rank4(alpha),
            scalar_rank4(air),
            scalar_rank4(s_intrinsic),
        );
        (
            fc.into_data().value[0],
            v_hd.into_data().value[0],
            v_ld.into_data().value[0],
        )
    }

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

    /// Monolith golden vector — pins Jennings paste strength at orchestrator mix contract.
    /// Constant class: **Measured** (first witness under pin; tolerance guards drift).
    #[test]
    fn strength_engine_measured_golden_vector_paste_at_orchestrator_pin() {
        let (fc_mpa, v_hd, v_ld) = strength_at_pin(
            ORCHESTRATOR_PIN_WC,
            ORCHESTRATOR_PIN_ALPHA,
            ORCHESTRATOR_PIN_AIR,
            ORCHESTRATOR_PIN_S_INTRINSIC,
        );
        assert!(
            fc_mpa.is_finite() && fc_mpa > 0.0,
            "orchestrator-pin f_c must be finite and positive; got {fc_mpa}"
        );
        assert!(
            v_hd.is_finite() && v_ld.is_finite() && v_hd >= 0.0 && v_ld >= 0.0,
            "C-S-H phase volumes must be finite and non-negative: v_hd={v_hd} v_ld={v_ld}"
        );
        // Witness value recorded 2026-07-21 AC11 — update only with new measured run + receipt.
        const GOLDEN_FC_MPA: f32 = STRENGTH_GOLDEN_FC_MPA;
        let rel_err = (fc_mpa - GOLDEN_FC_MPA).abs() / GOLDEN_FC_MPA;
        assert!(
            rel_err < 1e-5,
            "strength paste golden drift: measured={fc_mpa} golden={GOLDEN_FC_MPA} rel_err={rel_err}"
        );
    }

    /// Admissibility: higher hydration at fixed w/c ⇒ higher paste strength.
    #[test]
    fn strength_engine_paste_fc_increases_with_hydration() {
        let (fc_early, _, _) = strength_at_pin(
            ORCHESTRATOR_PIN_WC,
            0.40,
            ORCHESTRATOR_PIN_AIR,
            ORCHESTRATOR_PIN_S_INTRINSIC,
        );
        let (fc_late, _, _) = strength_at_pin(
            ORCHESTRATOR_PIN_WC,
            0.90,
            ORCHESTRATOR_PIN_AIR,
            ORCHESTRATOR_PIN_S_INTRINSIC,
        );
        assert!(
            fc_late > fc_early,
            "f_c must rise with α at fixed w/c: early={fc_early} late={fc_late}"
        );
    }
}
