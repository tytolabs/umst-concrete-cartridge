// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
//
//! MP3.5 — CastPhase orchestrator router + CastGateVerdict integration golden pins.
//! Schedule: `outputs/.tmp/fp_concrete_dual_gate_adt_plan.md` MP3.5.

use burn_ndarray::{NdArray, NdArrayDevice};
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::facade::{MixSpec, TemperatureK, WaterCementRatio};
use umst_concrete_cartridge::homogeneous::MixRow;
use umst_concrete_cartridge::mix_layout::{fractions_from_mix_row, mix_tensor_from_layout};
use umst_concrete_cartridge::pipeline::cast_phase::{
    classify_cast_phase, stage_eligible, CastLifecycleThresholds, CastPhase, CastPhaseInputs,
};
use umst_concrete_cartridge::pipeline::{
    evaluate_dual_gate, run_full_physics_pipeline, CastGateVerdict, PhysicsPipelineReport,
    PipelineStageStatus, PRINTABLE_TAU_HI, PRINTABLE_TAU_LO,
};
use umst_concrete_cartridge::pipeline::physical_summary::nominal_mix_tensor_for_mix_spec;

type B = NdArray<f32>;

const THRESH: CastLifecycleThresholds = CastLifecycleThresholds {
    alpha_set: 0.15,
    alpha_hard: 0.85,
};

/// Locked stage × phase eligibility matrix (`fp_concrete_dual_gate_adt_plan.md` MP3.2).
const ROUTER_GOLDEN: &[(&str, [bool; 3])] = &[
    // stage_id, [Fluid, Setting, Solid]
    ("hydration_degree", [true, true, true]),
    ("packing_density", [true, true, true]),
    ("porosity_capillary_bulk", [true, true, true]),
    ("strength_jennings", [false, true, true]),
    ("colloidal_dlvo", [true, true, true]),
    ("rheology_yodel", [true, true, false]),
    ("rheology_chateau_ovarlez", [true, true, false]),
    ("thermo_heat_rate_proxy", [false, true, true]),
    ("transport_chloride", [false, true, true]),
    ("printability", [true, false, false]),
    ("itz", [false, true, true]),
    ("chemo_water", [false, true, true]),
    ("fracture", [false, false, true]),
    ("nano_enhancement_baseline", [false, true, true]),
    ("sustainability", [true, true, true]),
    ("cost_linear_dot", [true, true, true]),
    ("creep", [false, false, true]),
    ("set_time", [true, true, false]),
    ("shrinkage", [false, true, true]),
    ("freeze_thaw", [false, false, true]),
    ("self_heal", [false, false, true]),
];

fn tyto_s1_spec() -> MixSpec {
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

fn uci_d1_bulk_row() -> MixRow {
    MixRow {
        cement_kg_m3: 350.0,
        slag_kg_m3: 0.0,
        fly_ash_kg_m3: 0.0,
        water_kg_m3: 140.0,
        superplasticizer_kg_m3: 0.0,
        age_days: 28.0,
        temperature_c: 20.0,
    }
}

fn run_tyto_report() -> PhysicsPipelineReport {
    let profile = Profile::load_bundled("tyto_mortar").expect("tyto_mortar");
    let dev = NdArrayDevice::default();
    let mix = nominal_mix_tensor_for_mix_spec::<B>(&profile, &tyto_s1_spec(), &dev);
    run_full_physics_pipeline::<B>(&profile, &mix)
}

fn run_uci_d1_report() -> PhysicsPipelineReport {
    let profile = Profile::load_bundled("uci_d1").expect("uci_d1");
    let row = uci_d1_bulk_row();
    let mix = mix_tensor_from_layout::<B>(&fractions_from_mix_row(&row, 0.65), &NdArrayDevice::default());
    run_full_physics_pipeline::<B>(&profile, &mix)
}

fn stage_status(report: &PhysicsPipelineReport, id: &str) -> Option<PipelineStageStatus> {
    report
        .stages
        .iter()
        .find(|s| s.id == id)
        .map(|s| s.status)
}

#[test]
fn cast_phase_router_golden_table_matches_stage_eligible() {
    let phases = [CastPhase::Fluid, CastPhase::Setting, CastPhase::Solid];
    for &(stage_id, expected) in ROUTER_GOLDEN {
        for (phase, &pin) in phases.iter().zip(expected.iter()) {
            assert_eq!(
                stage_eligible(stage_id, *phase),
                pin,
                "stage={stage_id} phase={phase:?}"
            );
        }
    }
}

#[test]
fn cast_phase_classifier_golden_threshold_pins() {
    let cases: &[(f32, CastPhase)] = &[
        (0.0, CastPhase::Fluid),
        (0.149_999, CastPhase::Fluid),
        (0.15, CastPhase::Setting),
        (0.5, CastPhase::Setting),
        (0.849_999, CastPhase::Setting),
        (0.85, CastPhase::Solid),
        (1.0, CastPhase::Solid),
    ];
    for &(alpha, expected) in cases {
        let inputs = CastPhaseInputs {
            hydration_alpha: alpha,
            yield_stress_pa: 250.0,
            age_days: 1.0,
        };
        assert_eq!(
            classify_cast_phase(&inputs, &THRESH),
            expected,
            "α={alpha}"
        );
    }
}

#[test]
fn tyto_s1_pipeline_resolves_fluid_phase_and_skips_solid_stages() {
    let report = run_tyto_report();
    assert_eq!(report.material_phase, CastPhase::Fluid);
    assert!(
        report.summary.hydration_alpha < THRESH.alpha_set,
        "tyto S1 young mix should classify Fluid: α={}",
        report.summary.hydration_alpha
    );
    assert!(report.phase_skip_sentinels);

    assert_eq!(
        stage_status(&report, "printability"),
        Some(PipelineStageStatus::Executed)
    );
    assert_eq!(
        stage_status(&report, "strength_jennings"),
        Some(PipelineStageStatus::SkippedIncompatiblePhase)
    );
    assert_eq!(
        stage_status(&report, "fracture"),
        Some(PipelineStageStatus::SkippedIncompatiblePhase)
    );
    assert_eq!(
        stage_status(&report, "creep"),
        Some(PipelineStageStatus::SkippedIncompatiblePhase)
    );

    // Parity sentinels — skipped solid stages stay at documented placeholders.
    assert_eq!(report.summary.strength_jennings_mpa, 0.0);
    assert_eq!(report.summary.fracture_toughness_k_ic_mpa_sqrt_m, 0.0);
    assert_eq!(report.summary.creep_compliance_1_over_gpa, 0.0);
    assert!(report.summary.rheology_yield_stress_pa.is_finite());
    assert!(report.summary.rheology_yield_stress_pa > 0.0);
    assert!(report.summary.printability_extrudability.is_finite());
}

#[test]
fn uci_d1_mature_pipeline_resolves_solid_phase_and_skips_fluid_stages() {
    let report = run_uci_d1_report();
    assert!(
        report.summary.hydration_alpha >= THRESH.alpha_hard,
        "uci_d1 28d should be Solid: α={}",
        report.summary.hydration_alpha
    );
    assert_eq!(report.material_phase, CastPhase::Solid);

    assert_eq!(
        stage_status(&report, "printability"),
        Some(PipelineStageStatus::SkippedIncompatiblePhase)
    );
    assert_eq!(
        stage_status(&report, "fracture"),
        Some(PipelineStageStatus::Executed)
    );
    assert_eq!(
        stage_status(&report, "set_time"),
        Some(PipelineStageStatus::SkippedIncompatiblePhase)
    );

    assert_eq!(report.summary.printability_extrudability, 0.0);
    assert!(report.summary.strength_jennings_mpa >= 0.0);
}

#[test]
fn pipeline_stage_manifest_consistent_with_cast_phase_router() {
    for report in [run_tyto_report(), run_uci_d1_report()] {
        let phase = report.material_phase;
        for &(stage_id, _) in ROUTER_GOLDEN {
            let Some(status) = stage_status(&report, stage_id) else {
                continue;
            };
            let eligible = stage_eligible(stage_id, phase);
            match status {
                PipelineStageStatus::Executed => {
                    assert!(
                        eligible,
                        "stage `{stage_id}` executed but ineligible for {phase:?}"
                    );
                }
                PipelineStageStatus::SkippedIncompatiblePhase => {
                    assert!(
                        !eligible,
                        "stage `{stage_id}` phase-skipped but eligible for {phase:?}"
                    );
                }
                _ => {}
            }
        }
    }
}

#[test]
fn cast_gate_verdict_leg_pass_matches_admissibility() {
    let profile = Profile::load_bundled("tyto_mortar").expect("tyto_mortar");
    let spec = tyto_s1_spec();
    let report = run_tyto_report();
    let verdict = evaluate_dual_gate(&profile, &spec, &report.summary);

    assert_eq!(verdict.is_admissible(), matches!(verdict, CastGateVerdict::Admissible));

    let leg_pass = verdict.printability_leg_pass() && verdict.thermodynamic_leg_pass();
    assert_eq!(leg_pass, verdict.is_admissible());

    // Printability leg pins — τ₀ band from pipeline summary.
    let tau = report.summary.rheology_yield_stress_pa;
    let extr = report.summary.printability_extrudability;
    let print_ok = (PRINTABLE_TAU_LO..=PRINTABLE_TAU_HI).contains(&tau)
        && extr.is_finite()
        && extr >= 0.35;
    assert_eq!(verdict.printability_leg_pass(), print_ok);
}

#[cfg(feature = "manifest-bridge")]
#[test]
fn cast_gate_verdict_tyto_s1_thermodynamic_leg_integration_pin() {
    let profile = Profile::load_bundled("tyto_mortar").expect("tyto_mortar");
    let spec = tyto_s1_spec();
    let report = run_tyto_report();
    let verdict = evaluate_dual_gate(&profile, &spec, &report.summary);

    use umst_concrete_cartridge::pipeline::thermodynamic_admissible;
    let thermo_bool = thermodynamic_admissible(&profile, &spec);
    assert_eq!(verdict.thermodynamic_leg_pass(), thermo_bool);
    if thermo_bool {
        assert!(!matches!(
            verdict,
            CastGateVerdict::RejectThermodynamic(_) | CastGateVerdict::RejectBoth { .. }
        ));
    }
}
