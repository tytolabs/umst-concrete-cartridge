// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! **Manifold photonics tie-in (Track H):** optical metrics for photocatalytic, cool-roof, and
//! radiative-cooling concrete — scalar spectral integrals used by audits / JSON capsules.
//!
//! ## Upstream solver
//!
//! The v0.4 brief targets a graph **FDFD Helmholtz** solve in
//! `umst_manifold::physics::solvers::photonics` (scalar TE/TM phasors on the 1-skeleton + PML).
//! Until [`ManifoldPhotonicsSolver`] wires that kernel, this module uses **closed-form anchors**:
//!
//! - **Solar / LWIR:** normal-incidence Fresnel power reflectance at the air–dielectric interface,
//!   averaged over discrete spectral samples (Simpson weights), plus a **small diffuse offset**
//!   so homogeneous Portland-cement **solar reflectance** matches published 0.30 ± 0.05 bands
//!   (ASTM E903-style reporting is still a frequency integral over the same samples).
//! - **UV absorption:** Beer–Lambert with extinction `k` from [`default_extinction_k_uv`], lower-bounded
//!   by **0.05** at 365 nm to match the plain-paste anchor in Track H3 (TiO₂ / cement UV absorption tests).
//!
//! When Helmholtz lands, replace the interior field with `solve_helmholtz` energy flux; keep these
//! functions as fast analytic regression guards.

use crate::physics::clinker_eos::{voigt_bulk_modulus_gpa, ClinkerPhase};

/// Re-export: manifold **phasor** photonics driver (`[B,N,3]` **E**-field contract). With
/// `--features solver-experimental`, the `photonics` feature enables the experimental path (still
/// passthrough until FDFD Helmholtz ships in `umst-manifold` Track H1).
/// formal_anchor: literature://electromagnetics/photonics-phasor-driver
/// formal_status: Literature
/// formal_citation: "Rumpf M. 2022 Computational Electromagnetics in MATLAB; Taflove & Hagness 2005 FDTD handbook"
/// formal_form: "Alias to PhotonicsSolver { frequency_hz } — Maxwell phasor placeholder pending FDFD Helmholtz"
pub type ManifoldPhotonicsSolver = umst_manifold::physics::solvers::PhotonicsSolver;

// --- Fresnel (normal incidence, power reflectance) ---------------------------------------------

/// Refractive index from lossless relative permittivity (real `ε_r > 0`).
/// formal_anchor: literature://optics/refractive-index-dielectric
/// formal_status: Literature
/// formal_citation: "Born & Wolf 1999 Principles of Optics"
/// formal_form: "n = sqrt(ε_r) for non-magnetic dielectric"
#[inline]
pub fn refractive_index_real(er: f32) -> f32 {
    er.max(1e-6).sqrt()
}

/// Fresnel **power** reflectance at normal incidence, air (\\(n_1=1\\)) to medium \\(n_2\\).
/// formal_anchor: literature://optics/fresnel-normal-incidence
/// formal_status: Literature
/// formal_citation: "Born & Wolf 1999 Principles of Optics"
/// formal_form: "R = ((n1-n2)/(n1+n2))² with n1=1"
#[must_use]
pub fn fresnel_power_reflectance_air_to_medium(n_medium: f32) -> f32 {
    let n2 = n_medium.max(1e-6);
    let r = (1.0 - n2) / (1.0 + n2);
    r * r
}

/// Diffuse hemispherical uplift (rough cement paste vs specular Fresnel slab).
///
/// Calibrated so [`solar_reflectance`] on [`plain_portland_visible_profile`] lands near the **0.30**
/// cool-roof / paste anchor (ASTM E903 bands) with the brief’s ε_r ≈ 5.6 knots — the underlying
/// Fresnel slab reflectance is ~0.16–0.17, so the diffuse tail must be modest (~0.16) not “white paint” high.
const SOLAR_DIFFUSE_FRACTION: f32 = 0.162_f32;

/// Linearly interpolate `y` from piecewise-linear `(x,y)` knots (x ascending).
fn interpolate_xy(xs_ys: &[(f32, f32)], xq: f32) -> f32 {
    if xs_ys.is_empty() {
        return 1.0;
    }
    if xq <= xs_ys[0].0 {
        return xs_ys[0].1;
    }
    for w in xs_ys.windows(2) {
        let (x0, y0) = w[0];
        let (x1, y1) = w[1];
        if xq <= x1 {
            let t = ((xq - x0) / (x1 - x0).max(1e-9)).clamp(0.0, 1.0);
            return y0 + t * (y1 - y0);
        }
    }
    xs_ys.last().unwrap().1
}

/// Simpson's rule weights for an odd-length uniform grid (here: 5 samples → Simpson on 4 segments; we use 5-point composite Simpson over 4 equal intervals requires odd count — 5 points is valid).
fn simpson_average_5<F: Fn(f32) -> f32>(lambda_nm: [f32; 5], integrand: F) -> f32 {
    // λ nodes: a, m1, mid, m2, b  — not necessarily uniform; for solar band we use fixed nm nodes from brief.
    let y: [f32; 5] = [
        integrand(lambda_nm[0]),
        integrand(lambda_nm[1]),
        integrand(lambda_nm[2]),
        integrand(lambda_nm[3]),
        integrand(lambda_nm[4]),
    ];
    // Composite Simpson for *uniform* spacing in λ — our nodes are approximately uniform 75 nm step from 400 to 700.
    let h = (lambda_nm[4] - lambda_nm[0]) / 4.0;
    let s = h / 3.0 * (y[0] + y[4] + 4.0 * (y[1] + y[3]) + 2.0 * y[2]);
    s / (lambda_nm[4] - lambda_nm[0]).max(1e-6)
}

// --- Public API (brief H3) ----------------------------------------------------------------------

/// Broadband **solar** (400–700 nm) hemispherical reflectance proxy.
///
/// `eps_profile`: `(wavelength_nm, ε_r)` samples; interpolated at five internal nodes then Simpson-averaged.
/// `thickness_m` reserved for future thin-film interference / volume scattering (Helmholtz); must be ≥ 0.
/// formal_anchor: literature://concrete/solar-reflectance-cool-roof
/// formal_status: Literature
/// formal_citation: "ASTM E903 standard practice for solar absorptance; Track H3 UMST v0.4 brief"
/// formal_form: "Simpson average of Fresnel R(λ) + diffuse fraction (rough paste); Helmholtz interior open"
#[must_use]
pub fn solar_reflectance(eps_profile: &[(f32, f32)], thickness_m: f32) -> f32 {
    let _ = thickness_m.max(0.0);
    let lambda_nm = [400.0_f32, 475.0, 550.0, 625.0, 700.0];
    let spec = simpson_average_5(lambda_nm, |l| {
        let er = interpolate_xy(eps_profile, l);
        fresnel_power_reflectance_air_to_medium(refractive_index_real(er))
    });
    (spec + (1.0 - spec) * SOLAR_DIFFUSE_FRACTION).clamp(0.0, 1.0)
}

/// UV absorption (0–1) at **365 nm** (TiO₂ photocatalysis line) through a slab of thickness `thickness_m`.
///
/// Uses Beer–Lambert \\(T = \\exp(-4\\pi k z / \\lambda)\\), \\(A = 1 - T - R\\) with first-surface \\(R\\)
/// from Fresnel. If the profile has no point near 365 nm, falls back to `eps_profile[0].1`.
/// formal_anchor: literature://photocatalysis/uv-absorption-cement
/// formal_status: Literature
/// formal_citation: "Beer 1852; Lambert 1760 absorption law; ISO photocatalytic concrete test lines (~365 nm)"
/// formal_form: "A = 1 - exp(-4π k z / λ) - R(n(ε_r)), k lower-bounded per paste UV anchor"
#[must_use]
pub fn photocatalytic_uv_absorption(eps_profile: &[(f32, f32)], thickness_m: f32) -> f32 {
    let lambda_m = 365e-9_f32;
    let lambda_nm = 365.0_f32;
    let er = interpolate_xy(eps_profile, lambda_nm).max(1.0);
    let n = refractive_index_real(er);
    // Plain-paste UV anchor (brief Track H3 integration narrative: k ≈ 0.05 at 365 nm).
    let k = default_extinction_k_uv(er).max(0.05);
    let r = fresnel_power_reflectance_air_to_medium(n);
    let alpha = 4.0 * std::f32::consts::PI * k / lambda_m.max(1e-15);
    let t = (-alpha * thickness_m.max(0.0)).exp();
    (1.0 - t - r).clamp(0.0, 1.0)
}

/// **LWIR** (8–13 µm atmospheric window) emissivity proxy: Kirchhoff at normal incidence,
/// \\( \\varepsilon \\approx 1 - R(\\lambda)\\) at **10.5 µm** (mid-window), interpolated `ε_r`.
/// formal_anchor: literature://radiative-cooling/lwir-emissivity
/// formal_status: Literature
/// formal_citation: "Zhai et al. 2017 Joule 1:359 (radiative cooling); Kirchhoff 1860"
/// formal_form: "ε ≈ 1 - R at λ = 10.5 μm, ε_r(λ) from piecewise-linear profile"
#[must_use]
pub fn radiative_cooling_emissivity(eps_profile: &[(f32, f32)], thickness_m: f32) -> f32 {
    let _ = thickness_m.max(0.0);
    let lambda_nm = 10.5e3_f32; // 10.5 µm
    let er = interpolate_xy(eps_profile, lambda_nm).max(1.0);
    let n = refractive_index_real(er);
    let r = fresnel_power_reflectance_air_to_medium(n);
    (1.0 - r).clamp(0.0, 1.0)
}

/// Default UV extinction coefficient \\(k\\) (imaginary part of complex index) from loss tangent
/// anchor `tan δ ≈ 0.018` for Portland cement in the visible (Sihvola 1999 order-of-magnitude).
/// formal_anchor: literature://dielectrics/sihvola-cement-complex-permittivity
/// formal_status: Literature
/// formal_citation: "Sihvola A. 1999 Electromagnetic Mixing Formulas and Applications"
/// formal_form: "k ≈ (n tan δ)/2 with tan δ = 0.018 anchor"
#[inline]
pub fn default_extinction_k_uv(er: f32) -> f32 {
    let n = refractive_index_real(er);
    let tan_delta = 0.018_f32;
    // Small-absorption: k ≈ n tan(δ) / 2 in the weak-loss limit.
    n * tan_delta * 0.5
}

/// Plain Portland cement visible / UV / LWIR anchor profile (piecewise linear in λ_nm).
/// formal_anchor: literature://dielectrics/portland-cement-permittivity-band
/// formal_status: Literature
/// formal_citation: "Track H3 UMST v0.4 brief (ε_r = 5.6 plain paste); LWIR ε_r order-of-magnitude"
/// formal_form: "Piecewise-linear (λ_nm, ε_r) knots for solar / UV / atmospheric-window interpolation"
#[must_use]
pub fn plain_portland_visible_profile() -> Vec<(f32, f32)> {
    vec![(300.0, 5.8), (365.0, 5.6), (550.0, 5.6), (10500.0, 4.2)]
}

/// **Strength module bridge:** Voigt bulk modulus (GPa) mixing HD / LD C-S-H literature moduli from
/// [`ClinkerPhase::Csh14nmTobermorite`] (nanopaste analogue) with Jennings-style LD fraction at `wc`.
///
/// This does **not** replace tensor [`crate::physics::strength::StrengthEngine`]; it exposes EOS-grade
/// \\(K_0\\) scalars for calibration JSON / future wiring to manifold `VinetEos`.
/// formal_anchor: literature://micromechanics/voigt-csh-bulk-from-wc
/// formal_status: Literature
/// formal_citation: "Jennings H.M. 2000 Cem. Concr. Res.; Pellenq et al. 2009 PNAS (C-S-H modulus)"
/// formal_form: "K = f_ld K_ld + (1-f_ld) K_hd with f_ld(w/c) from Jennings linear fit"
#[must_use]
pub fn paste_bulk_modulus_voigt_from_wc_gpa(wc_ratio: f32) -> f32 {
    let wc = wc_ratio.clamp(0.2, 0.8);
    let ld_fraction = (3.017 * wc - 0.347).clamp(0.0, 1.0);
    let k_csh = ClinkerPhase::Csh14nmTobermorite.bulk_modulus_ambient_gpa();
    // Map LD / HD to two literature anchors: LD softer, HD stiffer (Ulm & Constantinides nanoindent).
    let k_ld = k_csh * 0.31_f32;
    let k_hd = k_csh * 0.42_f32;
    voigt_bulk_modulus_gpa(ld_fraction, k_ld, k_hd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresnel_air_glass_matches_analytic_one_ninth() {
        // n = 2, ε_r = 4
        let r = fresnel_power_reflectance_air_to_medium(2.0);
        assert!((r - 1.0_f32 / 9.0).abs() < 1e-4);
    }

    #[test]
    fn plain_portland_solar_reflectance_within_ten_percent_of_reference_band() {
        // Published plain Portland / paste solar reflectance ~0.30 ± 0.05 (ASTM E903); ±10% gate on 0.30 anchor.
        let prof = plain_portland_visible_profile();
        let rs = solar_reflectance(&prof, 0.05);
        let r0 = 0.30_f32;
        let tol = 0.10_f32 * r0;
        assert!(
            (rs - r0).abs() <= tol,
            "solar_reflectance={rs} expected within ±10% of reference {r0} (tol={tol})"
        );
    }

    #[test]
    fn plain_portland_solar_uv_lwir_in_literature_ballpark() {
        let prof = plain_portland_visible_profile();
        let rs = solar_reflectance(&prof, 0.05);
        let a_uv = photocatalytic_uv_absorption(&prof, 0.05);
        let epslw = radiative_cooling_emissivity(&prof, 0.05);
        assert!(
            (0.22..=0.38).contains(&rs),
            "solar_reflectance={} expected ~0.25–0.35",
            rs
        );
        assert!(a_uv > 0.65, "uv absorption={}", a_uv);
        assert!(epslw > 0.85, "lwir emissivity={}", epslw);
    }

    #[test]
    fn paste_bulk_modulus_finite() {
        let k = paste_bulk_modulus_voigt_from_wc_gpa(0.4);
        assert!(k > 5.0 && k < 40.0, "k={}", k);
    }
}
