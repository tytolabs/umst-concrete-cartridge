// SPDX-License-Identifier: MIT
// WS-PROXY: topology path uses explicit MixSpec when pinned on cartridge.

use burn_ndarray::{NdArray, NdArrayDevice};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::core::ConcreteCartridge;
use umst_concrete_cartridge::facade::{MixSpec, TemperatureK, WaterCementRatio};
use umst_concrete_cartridge::pipeline::physical_summary::{
    nominal_mix_tensor_for_mix_spec, nominal_mix_tensor_for_topology,
};
use umst_concrete_cartridge::pipeline::{topology_pipeline_report, TopologyNominalMix};

type B = NdArray<f32>;

#[test]
fn mix_spec_tensor_differs_from_regime_midpoint() {
    let profile = Profile::load_bundled("tyto_mortar").expect("tyto_mortar");
    let dev = NdArrayDevice::default();
    let spec = MixSpec {
        w_c: WaterCementRatio::try_from(0.45).expect("w_c"),
        temperature_k: TemperatureK::try_from(298.15).expect("T"),
        superplasticiser_pct: 1.0,
        silica_fume_pct: 10.0,
        fly_ash_pct: 0.0,
        aggregate_volume_fraction: 0.35,
        target_age_hours: 1.0,
        profile_name: "tyto_mortar".into(),
    };
    let a = nominal_mix_tensor_for_mix_spec::<B>(&profile, &spec, &dev);
    let b = nominal_mix_tensor_for_topology::<B>(&profile, &dev);
    let va = a.fractions.clone().into_data().value;
    let vb = b.fractions.clone().into_data().value;
    assert_ne!(
        va, vb,
        "explicit mix spec should differ from regime midpoint layout"
    );
}

#[test]
fn topology_report_uses_nominal_mix_when_set() {
    let profile = Profile::load_bundled("tyto_mortar").expect("tyto_mortar");
    let dev = NdArrayDevice::default();
    let nominal = TopologyNominalMix {
        w_c: 0.45,
        superplasticiser_pct: 1.0,
        fly_ash_pct: 0.0,
        silica_fume_pct: 10.0,
        aggregate_volume_fraction: 0.35,
        target_age_hours: 1.0,
        temperature_k: 298.15,
    };
    let default_report = topology_pipeline_report::<B>(&profile, &dev, None);
    let spec_report = topology_pipeline_report::<B>(&profile, &dev, Some(nominal));
    assert_ne!(
        default_report.summary.rheology_yield_stress_pa,
        spec_report.summary.rheology_yield_stress_pa,
        "topology pipeline should reflect explicit nominal mix"
    );

    let cart = ConcreteCartridge::<B>::with_profile(profile).with_topology_nominal(nominal);
    assert!(cart.topology_nominal.is_some());
}
