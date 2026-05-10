// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Library integration smoke tests for `ConcreteCartridge::compute_all` and the tensor pipeline.

use burn_ndarray::{NdArray, NdArrayDevice};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::core::ConcreteCartridge;
use umst_concrete_cartridge::homogeneous::MixRow;
use umst_concrete_cartridge::mix_layout::{fractions_from_mix_row, mix_tensor_from_layout};
use umst_concrete_cartridge::{run_full_physics_pipeline, IScienceCartridge};

type B = NdArray<f32>;

fn dev() -> NdArrayDevice {
    NdArrayDevice::default()
}

fn assert_finite_2(slice: &[f32], ctx: &str) {
    assert!(
        slice.iter().copied().all(|x| x.is_finite()),
        "{ctx}: non-finite {slice:?}"
    );
}

#[test]
fn compute_all_returns_expected_ranks_and_finite_values() {
    let profile = Profile::load_bundled("uci_d1").expect("uci_d1");
    let row = MixRow {
        cement_kg_m3: 350.0,
        slag_kg_m3: 0.0,
        fly_ash_kg_m3: 0.0,
        water_kg_m3: 140.0,
        superplasticizer_kg_m3: 0.0,
        age_days: 28.0,
        temperature_c: 20.0,
    };
    let layout = fractions_from_mix_row(&row, 0.65);
    let mix = mix_tensor_from_layout::<B>(&layout, &dev());

    let cart = ConcreteCartridge::with_profile(profile.clone());
    let pr = cart.compute_all(&mix);

    assert_eq!(pr.free_energy.dims(), [1, 2]);
    assert_eq!(pr.dissipation.dims(), [1, 1]);
    assert_eq!(pr.safety_margin.dims(), [1, 1]);
    assert_eq!(pr.cost.dims(), [1, 1]);

    assert_finite_2(&pr.free_energy.clone().into_data().value, "free_energy");
    assert_finite_2(&pr.dissipation.clone().into_data().value, "dissipation");
    assert_finite_2(&pr.safety_margin.clone().into_data().value, "safety_margin");
    assert_finite_2(&pr.cost.clone().into_data().value, "cost");
}

#[test]
fn orchestrator_reports_many_stages() {
    let profile = Profile::load_bundled("uci_d1").expect("uci_d1");
    let row = MixRow {
        cement_kg_m3: 350.0,
        slag_kg_m3: 0.0,
        fly_ash_kg_m3: 0.0,
        water_kg_m3: 140.0,
        superplasticizer_kg_m3: 0.0,
        age_days: 28.0,
        temperature_c: 20.0,
    };
    let mix = mix_tensor_from_layout::<B>(&fractions_from_mix_row(&row, 0.65), &dev());
    let report = run_full_physics_pipeline::<B>(&profile, &mix);

    assert_eq!(
        report.schema_version,
        umst_concrete_cartridge::PHYSICS_PIPELINE_SCHEMA_VERSION
    );
    assert!(
        report.stages.len() >= 18,
        "expected broad manifest coverage"
    );
    let executed = report
        .stages
        .iter()
        .filter(|s| {
            matches!(
                s.status,
                umst_concrete_cartridge::PipelineStageStatus::Executed
            )
        })
        .count();
    assert!(
        executed >= 14,
        "most stages should execute on the bulk path"
    );

    assert!(report.summary.hydration_alpha.is_finite());
    assert!(report.summary.strength_jennings_mpa >= 0.0);
}
