// SPDX-License-Identifier: MIT
// WS-RHEO: τ₀ monotone with w/c and superplasticiser on mix-faithful YODEL path.

use burn_ndarray::{NdArray, NdArrayDevice};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::facade::{MixSpec, TemperatureK, WaterCementRatio};
use umst_concrete_cartridge::pipeline::physical_summary::nominal_mix_tensor_for_mix_spec;
use umst_concrete_cartridge::run_full_physics_pipeline;

type B = NdArray<f32>;

fn spec(w_c: f32, sp: f32) -> MixSpec {
    MixSpec {
        w_c: WaterCementRatio::try_from(f64::from(w_c)).expect("w_c"),
        temperature_k: TemperatureK::try_from(298.15).expect("T"),
        superplasticiser_pct: sp,
        silica_fume_pct: 0.0,
        fly_ash_pct: 0.0,
        aggregate_volume_fraction: 0.35,
        target_age_hours: 1.0,
        profile_name: "tyto_mortar".into(),
    }
}

fn tau_for(profile: &Profile, mix: &MixSpec) -> f32 {
    let dev = NdArrayDevice::default();
    let tensor = nominal_mix_tensor_for_mix_spec::<B>(profile, mix, &dev);
    let report = run_full_physics_pipeline::<B>(profile, &tensor);
    report.summary.rheology_yield_stress_pa
}

#[test]
fn tau0_decreases_with_increasing_w_c() {
    let profile = Profile::load_bundled("tyto_mortar").expect("tyto_mortar");
    let wc_grid = [0.36_f32, 0.42, 0.48, 0.54];
    let mut last = f32::MAX;
    for w_c in wc_grid {
        let tau = tau_for(&profile, &spec(w_c, 0.5));
        assert!(tau.is_finite() && tau > 0.0, "tau at w_c={w_c}: {tau}");
        assert!(
            tau <= last + 1e-3,
            "τ₀ should decrease (non-increasing) as w/c increases: w_c={w_c} tau={tau} prev={last}"
        );
        last = tau;
    }
}

#[test]
fn tau0_decreases_with_increasing_sp() {
    let profile = Profile::load_bundled("tyto_mortar").expect("tyto_mortar");
    let sp_grid = [0.0_f32, 0.5, 1.0, 1.5];
    let mut last = f32::MAX;
    for sp in sp_grid {
        let tau = tau_for(&profile, &spec(0.45, sp));
        assert!(tau.is_finite() && tau > 0.0, "tau at sp={sp}: {tau}");
        assert!(
            tau <= last + 1e-3,
            "τ₀ should decrease (non-increasing) as SP increases: sp={sp} tau={tau} prev={last}"
        );
        last = tau;
    }
}

#[test]
fn printability_extrudability_finite_on_sweep() {
    let profile = Profile::load_bundled("tyto_mortar").expect("tyto_mortar");
    let dev = NdArrayDevice::default();
    let mix = nominal_mix_tensor_for_mix_spec::<B>(&profile, &spec(0.45, 1.0), &dev);
    let report = run_full_physics_pipeline::<B>(&profile, &mix);
    let extr = report.summary.printability_extrudability;
    assert!(
        extr.is_finite() && (0.0..=1.5).contains(&extr),
        "extrudability out of band: {extr}"
    );
}
