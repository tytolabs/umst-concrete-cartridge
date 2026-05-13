// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Adversarial-input sanity bounds on tensor engines; dataset claims are grounded in
//! [`datasets/PROVENANCE.md`](../../../datasets/PROVENANCE.md); colloidal, rheological, thermal,
//! transport, printability, and ITZ behaviours reference the citations collected in
//! [`docs/Constitutive-Equations.md`](../../docs/Constitutive-Equations.md).

use burn::tensor::{Data, Shape, Tensor};
use burn_ndarray::{NdArray, NdArrayDevice};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::homogeneous::{
    compressive_strength_mpa, mix_hydration_state, safety_margin, MixRow,
};
use umst_concrete_cartridge::physics::colloidal::ColloidalEngine;
use umst_concrete_cartridge::physics::itz::{
    compute_itz_percolation_factor, compute_itz_thickness_microns,
};
use umst_concrete_cartridge::physics::printability::PrintabilityEngine;
use umst_concrete_cartridge::physics::rheology::RheologyEngine;
use umst_concrete_cartridge::physics::thermo::ThermoEngine;
use umst_concrete_cartridge::physics::transport::TransportEngine;

type B = NdArray<f32>;

fn device() -> NdArrayDevice {
    NdArrayDevice::default()
}

fn t4(value: f32) -> Tensor<B, 4> {
    Tensor::<B, 4>::from_data(Data::new(vec![value], Shape::new([1, 1, 1, 1])), &device())
}

fn t2(value: f32) -> Tensor<B, 2> {
    Tensor::<B, 2>::from_data(Data::new(vec![value], Shape::new([1, 1])), &device())
}

fn assert_all_finite_2(t: Tensor<B, 2>, ctx: &str) {
    let data = t.into_data();
    let v = data.value.as_slice();
    assert!(
        v.iter().all(|x| x.is_finite()),
        "{ctx}: non-finite values in {v:?}",
    );
}

fn assert_all_finite_4(t: Tensor<B, 4>, ctx: &str) {
    let data = t.into_data();
    let v = data.value.as_slice();
    assert!(
        v.iter().all(|x| x.is_finite()),
        "{ctx}: non-finite values in {v:?}",
    );
}

fn min_f32_4(t: Tensor<B, 4>) -> f32 {
    t.into_data()
        .value
        .iter()
        .copied()
        .fold(f32::INFINITY, f32::min)
}

#[test]
fn tensor_engines_physical_sanity_under_adversarial_inputs() {
    // --- DLVO colloidal: inputs carry zeta_potential_mv and separation_nm ---
    let zeta_nom = t4(-25.0);
    let ionic = t4(0.03);
    let dlvo =
        ColloidalEngine::<B>::compute_dlvo_potential(t4(50.0), zeta_nom.clone(), ionic.clone());
    assert_all_finite_4(dlvo, "DLVO nominal");

    let dlvo_collapse =
        ColloidalEngine::<B>::compute_dlvo_potential(t4(0.105), zeta_nom, ionic.clone());
    let v_coll = dlvo_collapse.into_data().value[0];
    assert!(
        v_coll <= -900.0,
        "expected collapse-mask deep attractive kT bucket, got {v_coll}"
    );

    // Extreme zeta (mV) should still yield finite potentials (stress-test numeric blow-ups).
    let dlvo_adv = ColloidalEngine::<B>::compute_dlvo_potential(t4(5.0), t4(-1e6), ionic);
    assert_all_finite_4(dlvo_adv, "DLVO adversarial zeta");

    // --- Rheology yield (YODEL) ---
    let phi = t4(0.35);
    let phi_m = t4(0.74);
    let d50 = t4(50e-6);
    let f_sigma = t4(50.0);
    let tau_y = RheologyEngine::<B>::compute_yield_stress_yodel(phi, phi_m, d50, f_sigma);
    assert!(
        min_f32_4(tau_y.clone()) > 10.0,
        "expected yield stress well above rouleau floor at phi=0.35"
    );

    let tau_adv =
        RheologyEngine::<B>::compute_yield_stress_yodel(t4(0.72), t4(0.74), t4(1e-3), t4(1e6));
    assert_all_finite_4(tau_adv, "YODEL adversarial near-jam");

    // --- Thermo hydration heat ---
    let temp_c = t4(20.0);
    let alpha = t4(0.5);
    let ea = t4(40e3_f32);
    let (_q, dt_adiabatic) = ThermoEngine::<B>::compute_heat_rate(temp_c, alpha, ea);
    let rise = dt_adiabatic.into_data().value[0];
    assert!(
        rise > 20.0 && rise <= 55.0,
        "adiabatic proxy α·50 °C inconsistent with nominal α=0.5 (rise={rise})"
    );

    // --- Chloride transport: porosity-fed diffusivity (orders of magnitude guard) ---
    let wc_ratio = t4(0.42);
    let alpha28 = t4(0.75);
    let phi_c = TransportEngine::<B>::compute_capillary_porosity(wc_ratio.clone(), alpha28.clone());
    let ref_d = Tensor::<B, 4>::from_data(
        Data::new(vec![1e-12_f32], Shape::new([1, 1, 1, 1])),
        &device(),
    );
    let d_cl = TransportEngine::<B>::compute_chloride_diffusivity(phi_c.clone(), ref_d);
    assert_all_finite_4(phi_c, "capillary porosity");
    assert_all_finite_4(d_cl.clone(), "chloride diffusivity");
    let d_max = d_cl.into_data().value.iter().copied().fold(0_f32, f32::max);
    assert!(
        d_max < 1e-8 && d_max > 1e-20,
        "expected sub-nanoscale D_Cl with ref=1e-12 m²/s, got {d_max}",
    );

    // --- Printability ---
    let build = PrintabilityEngine::<B>::compute_buildability(t4(120.0), t4(0.05), 80.0);
    assert_all_finite_4(build.clone(), "buildability");
    let b0 = min_f32_4(build.clone());
    assert!(
        (0.05..=1.0).contains(&b0),
        "expected finite buildability score in plausible band, got {b0}",
    );

    // --- ITZ micron-scale thickness ---
    let thick = compute_itz_thickness_microns::<B>(t2(24.0));
    let microns = thick.into_data().value[0];
    assert!(
        microns > 10.0 && microns < 5000.0,
        "ITZ thickness {microns} μm implausible"
    );

    let perc = compute_itz_percolation_factor::<B>(t2(0.25));
    assert_all_finite_2(perc, "ITZ percolation factor");

    let extr = PrintabilityEngine::<B>::compute_extrudability(t4(350.0), t4(45.0), 16.0, 120.0);
    assert_all_finite_4(extr, "extrudability");
}

#[test]
fn zero_portland_cement_with_scm_still_reports_non_negative_strength() {
    let profile = Profile::load_bundled("highscm").expect("bundled HIGHSCM");
    let row = MixRow {
        cement_kg_m3: 0.0,
        slag_kg_m3: 120.0,
        fly_ash_kg_m3: 120.0,
        water_kg_m3: 110.0,
        superplasticizer_kg_m3: 3.0,
        age_days: 28.0,
        temperature_c: 23.0,
    };
    let fc = compressive_strength_mpa(&profile, &row).expect("SCM-only binder should evaluate");
    assert!(
        fc.is_finite() && fc >= 0.0,
        "expected finite non-negative strength, got {fc}"
    );
}

#[test]
fn extreme_w_c_clamps_hydration_and_strength() {
    let profile = Profile::load_bundled("uci_d1").expect("bundled uci_d1");
    let base = MixRow {
        cement_kg_m3: 350.0,
        slag_kg_m3: 0.0,
        fly_ash_kg_m3: 0.0,
        water_kg_m3: 35.0,
        superplasticizer_kg_m3: 0.0,
        age_days: 28.0,
        temperature_c: 20.0,
    };
    let mut high_w = base.clone();
    high_w.water_kg_m3 = 600.0;

    let (_, alpha_low, _) = mix_hydration_state(&profile, &base).unwrap();
    let (_, alpha_high, _) = mix_hydration_state(&profile, &high_w).unwrap();
    assert!(
        (0.0..=1.0).contains(&alpha_low) && (0.0..=1.0).contains(&alpha_high),
        "DoH must stay in [0,1] under extreme w/c routing"
    );

    let fc_low = compressive_strength_mpa(&profile, &base).unwrap();
    let fc_high = compressive_strength_mpa(&profile, &high_w).unwrap();
    assert!(fc_low.is_finite() && fc_high.is_finite());
}

#[test]
fn powers_intrinsic_prefactor_monotone_on_same_mix() {
    let mut low = Profile::load_bundled("uci_d1").expect("uci_d1");
    let mut high = low.clone();
    low.powers.s_intrinsic = 30.0;
    high.powers.s_intrinsic = 120.0;
    let row = MixRow {
        cement_kg_m3: 320.0,
        slag_kg_m3: 0.0,
        fly_ash_kg_m3: 0.0,
        water_kg_m3: 128.0,
        superplasticizer_kg_m3: 0.0,
        age_days: 28.0,
        temperature_c: 20.0,
    };
    let fc_low = compressive_strength_mpa(&low, &row).unwrap();
    let fc_high = compressive_strength_mpa(&high, &row).unwrap();
    assert!(
        fc_high > fc_low,
        "expected higher intrinsic gel strength to raise fc (low={fc_low}, high={fc_high})"
    );
}

#[test]
fn orchestration_constitution_layer_not_in_cartridge_minimal_margin_probe() {
    // Full constitutional admissibility gates live in higher-level orchestration (see `umst-manifold`);
    // this cartridge only exposes scalar `safety_margin` for agents to interpret.
    let profile = Profile::load_bundled("uci_d1").expect("uci_d1");
    let margin = safety_margin(&profile, 0.40, 0.35);
    assert!(margin.is_finite());
    assert!((0.0..=1.0).contains(&margin));
}
