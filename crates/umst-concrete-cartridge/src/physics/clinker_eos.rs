// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Calibrated **Vinet equation of state** scalars for cement-relevant crystalline phases.
//!
//! Full tensor EOS + inverse Newton live in `umst-manifold` Track H2
//! (`physics::solvers::statistical_mechanics`). This cartridge module holds **peer-reviewed
//! bulk moduli and reference volumes** used to homogenise paste-scale models (see
//! [`crate::physics::strength::StrengthEngine`]) without pulling Burn into scalar audits.
//!
//! **M3 T3 A4 Cluster D:** tabulated `(V₀, K₀, K₀′)` triples route through [`crate::chem_adapter`]
//! into `umst-chem` SSOT — no duplicate literals in this file.
//!
//! ## Vinet pressure (scalar)
//!
//! \\[
//!   P(V) = 3 K_0 \\frac{1 - x}{x^2} \\exp\\bigl(\\eta(1 - x)\\bigr),\\quad
//!   x = \\Bigl(\\frac{V}{V_0}\\Bigr)^{1/3},\\quad
//!   \\eta = \\frac{3}{2}(K_0' - 1)
//! \\]
//!
//! **formal_citation:** Vinet et al., *J. Phys. C* **19** (1986) L467.  
//! Phase parameters: Manzano et al. 2009 (*JACS*), Speziale et al. 2008 (*PCM*), Clark et al. 2008 (*CCR*), Pellenq et al. 2009 (*PNAS*) — see table on each [`ClinkerPhase`] variant.

use crate::chem_adapter::{
    clinker_bulk_modulus_ambient_gpa_f32, clinker_vinet_params_f32, vinet_pressure_gpa_f32,
    voigt_bulk_modulus_gpa_f32, ClinkerPhaseTag,
};

/// Identifier for tabulated clinker / hydrate phases (DFT-backed moduli in the brief).
/// formal_anchor: literature://stat-mech/vinet-clinker-phase-enum
/// formal_status: Literature
/// formal_citation: "Manzano et al. 2009 J. Am. Chem. Soc. 131:7416; Speziale et al. 2008 Phys. Chem. Miner. 35:573; Clark et al. 2008 Cem. Concr. Res. 38:19; Pellenq et al. 2009 PNAS 106:16102"
/// formal_form: "Discrete phase tags carrying (V0, K0, K0') for Vinet P(V) calibration"
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClinkerPhase {
    AliteM3,
    BeliteBetaC2s,
    Portlandite,
    Ettringite,
    Csh14nmTobermorite,
}

impl ClinkerPhase {
    const fn tag(self) -> ClinkerPhaseTag {
        match self {
            Self::AliteM3 => ClinkerPhaseTag::AliteM3,
            Self::BeliteBetaC2s => ClinkerPhaseTag::BeliteBetaC2s,
            Self::Portlandite => ClinkerPhaseTag::Portlandite,
            Self::Ettringite => ClinkerPhaseTag::Ettringite,
            Self::Csh14nmTobermorite => ClinkerPhaseTag::Csh14nmTobermorite,
        }
    }
}

/// Reference formula-unit volume \\(V_0\\) (Å³/f.u.), bulk modulus \\(K_0\\) (GPa), and pressure derivative \\(K_0'\\).
/// formal_anchor: literature://stat-mech/vinet-phase-params
/// formal_status: Literature
/// formal_citation: "Vinet et al. 1986 J. Phys. C 19:L467"
/// formal_form: "(V0 [Å³/f.u.], K0 [GPa], K0' [1]) parameter triple"
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VinetPhaseParams {
    pub v0_per_fu_ang3: f32,
    pub bulk_modulus_gpa: f32,
    pub k0_prime: f32,
}

impl ClinkerPhase {
    /// Literature parameters via `chem_adapter` → `umst-chem` SSOT (A4 inventory cluster D).
    /// formal_anchor: literature://stat-mech/vinet-clinker-table
    /// formal_status: Literature
    /// formal_citation: "Manzano et al. 2009; Speziale et al. 2008; Clark et al. 2008; Pellenq et al. 2009"
    /// formal_form: "VinetPhaseParams { v0_per_fu_ang3, bulk_modulus_gpa, k0_prime }"
    pub fn params(self) -> VinetPhaseParams {
        let p = clinker_vinet_params_f32(self.tag());
        VinetPhaseParams {
            v0_per_fu_ang3: p.v0_per_fu_ang3,
            bulk_modulus_gpa: p.bulk_modulus_gpa,
            k0_prime: p.k0_prime,
        }
    }

    /// Isotropic bulk modulus at ambient pressure (GPa), i.e. \\(K_0\\) from the Vinet fit.
    /// formal_anchor: literature://stat-mech/vinet-k0-ambient
    /// formal_status: Literature
    /// formal_citation: "Vinet et al. 1986 J. Phys. C 19:L467"
    /// formal_form: "K0 from tabulated EOS fit at P \\approx 0"
    pub fn bulk_modulus_ambient_gpa(self) -> f32 {
        clinker_bulk_modulus_ambient_gpa_f32(self.tag())
    }
}

/// Vinet isothermal pressure \\(P\\) in GPa for a scalar volume ratio \\(V / V_0\\).
/// formal_anchor: literature://stat-mech/vinet-pressure-closed-form
/// formal_status: Literature
/// formal_citation: "Vinet et al. 1986 J. Phys. C 19:L467"
/// formal_form: "P(V) = 3 K0 ((1-x)/x²) exp(η(1-x)), x=(V/V0)^(1/3), η=(3/2)(K0'-1)"
#[must_use]
pub fn vinet_pressure_gpa(v0: f32, k0_gpa: f32, k0_prime: f32, v_per_fu_ang3: f32) -> f32 {
    vinet_pressure_gpa_f32(v0, k0_gpa, k0_prime, v_per_fu_ang3)
}

/// Voigt upper bound on bulk modulus (GPa) for a binary mixture of two phases by **volume fraction** `fv_phase_a`.
/// formal_anchor: literature://micromechanics/voigt-upper-bound
/// formal_status: Literature
/// formal_citation: "Voigt W. 1887 Ann. Phys. 274:573 (rule of mixtures)"
/// formal_form: "K_Voigt = f K_a + (1-f) K_b"
#[must_use]
pub fn voigt_bulk_modulus_gpa(fv_phase_a: f32, k_a: f32, k_b: f32) -> f32 {
    voigt_bulk_modulus_gpa_f32(fv_phase_a, k_a, k_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chem_adapter::{clinker_vinet_params_f32, ClinkerPhaseTag};

    #[test]
    fn vinet_pressure_near_reference_volume_is_small_all_phases() {
        for phase in [
            ClinkerPhase::AliteM3,
            ClinkerPhase::BeliteBetaC2s,
            ClinkerPhase::Portlandite,
            ClinkerPhase::Ettringite,
            ClinkerPhase::Csh14nmTobermorite,
        ] {
            let p = phase.params();
            let p0 = vinet_pressure_gpa(
                p.v0_per_fu_ang3,
                p.bulk_modulus_gpa,
                p.k0_prime,
                p.v0_per_fu_ang3,
            );
            assert!(
                p0.abs() < 0.02,
                "phase {phase:?}: expected P≈0 at V0, got {p0}",
            );
        }
    }

    #[test]
    fn vinet_derivative_sign_matches_compression() {
        let p = ClinkerPhase::AliteM3.params();
        let p0 = vinet_pressure_gpa(
            p.v0_per_fu_ang3,
            p.bulk_modulus_gpa,
            p.k0_prime,
            p.v0_per_fu_ang3,
        );
        let p_comp = vinet_pressure_gpa(
            p.v0_per_fu_ang3,
            p.bulk_modulus_gpa,
            p.k0_prime,
            p.v0_per_fu_ang3 * 0.97,
        );
        assert!(p_comp > p0, "compression should raise pressure");
    }

    /// Matches `chem_adapter_parity::VINET_F32_ABS_TOL` — f64 SSOT → f32 cartridge slack.
    const VINET_F32_ABS_TOL: f32 = 2e-5;

    #[test]
    fn cluster_d_params_match_chem_adapter_ssot() {
        for (phase, tag) in [
            (ClinkerPhase::AliteM3, ClinkerPhaseTag::AliteM3),
            (ClinkerPhase::BeliteBetaC2s, ClinkerPhaseTag::BeliteBetaC2s),
            (ClinkerPhase::Portlandite, ClinkerPhaseTag::Portlandite),
            (ClinkerPhase::Ettringite, ClinkerPhaseTag::Ettringite),
            (ClinkerPhase::Csh14nmTobermorite, ClinkerPhaseTag::Csh14nmTobermorite),
        ] {
            let local = phase.params();
            let ssot = clinker_vinet_params_f32(tag);
            assert!(
                (local.v0_per_fu_ang3 - ssot.v0_per_fu_ang3).abs() < VINET_F32_ABS_TOL,
                "phase {phase:?} v0 mismatch: {} vs {}",
                local.v0_per_fu_ang3,
                ssot.v0_per_fu_ang3,
            );
            assert_eq!(local.bulk_modulus_gpa, ssot.bulk_modulus_gpa);
            assert_eq!(local.k0_prime, ssot.k0_prime);
        }
    }

    /// A-01 AliteM3 `364.2` Å³/f.u. is the binding f64→f32 round-trip (~1.22e-5).
    #[test]
    fn cluster_d_alite_v0_f32_boundary_within_vinet_abs_tol() {
        let p = ClinkerPhase::AliteM3.params();
        let ssot_v0 = 364.2_f64;
        let delta = f64::from(p.v0_per_fu_ang3) - ssot_v0;
        assert!(
            delta.abs() < f64::from(VINET_F32_ABS_TOL),
            "AliteM3 V₀ f32 boundary: delta={delta} vs VINET_F32_ABS_TOL"
        );
        assert!(
            delta.abs() >= 1e-5,
            "regression guard: AliteM3 must still fail EPS_F32 (1e-5) — do not revert to generic tol"
        );
    }
}
