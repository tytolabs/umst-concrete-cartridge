// SPDX-License-Identifier: MIT
// WS-CAL: θ fit on tyto_mortar profile; predict-in-band for S1 composition.

use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::calibration_fit::calibrated_tau0_pa;
use umst_concrete_cartridge::facade::{predict, MixSpec, TemperatureK, WaterCementRatio};
use umst_concrete_cartridge::pipeline::{PRINTABLE_TAU_HI, PRINTABLE_TAU_LO};

fn s1_mix() -> MixSpec {
    MixSpec {
        w_c: WaterCementRatio::try_from(0.45).expect("w_c"),
        temperature_k: TemperatureK::try_from(298.15).expect("T"),
        superplasticiser_pct: 1.0,
        silica_fume_pct: 10.0,
        fly_ash_pct: 0.0,
        aggregate_volume_fraction: 0.35,
        target_age_hours: 1.0,
        profile_name: "tyto_mortar".into(),
    }
}

#[test]
fn tyto_mortar_profile_loads_with_rheology_calibration() {
    let p = Profile::load_bundled("tyto_mortar").expect("bundled");
    assert_eq!(p.bundle_id, "tyto_mortar");
    let block = p.rheology_calibration.as_ref().expect("rheology block");
    assert!(block.measured_tau0_lo_pa.is_some());
    assert!(block.measured_tau0_hi_pa.is_some());
}

#[test]
fn predict_s1_calibrated_tau_in_measured_band() {
    let profile = Profile::load_bundled("tyto_mortar").expect("tyto_mortar");
    let spec = s1_mix();
    let bundle = predict(&profile, &spec).expect("predict");
    let raw_tau = bundle.physics_pipeline.summary.rheology_yield_stress_pa;
    let calibrated = calibrated_tau0_pa(raw_tau, profile.rheology_calibration.as_ref());
    let lo = profile
        .rheology_calibration
        .as_ref()
        .and_then(|b| b.measured_tau0_lo_pa)
        .unwrap_or(PRINTABLE_TAU_LO);
    let hi = profile
        .rheology_calibration
        .as_ref()
        .and_then(|b| b.measured_tau0_hi_pa)
        .unwrap_or(PRINTABLE_TAU_HI);
    assert!(
        calibrated >= lo && calibrated <= hi,
        "calibrated τ₀={calibrated} outside [{lo}, {hi}] (raw={raw_tau})"
    );
    let wire_tau = bundle.physical.free_energy.clone().into_data().value[1];
    assert!(
        (wire_tau - calibrated).abs() < 1e-3,
        "physical_result τ should match calibrated value"
    );
}
