// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Feed-forward orchestration across tensor physics engines (collapsed batch semantics).
//!
//! formal_anchor: NONE
//! formal_status: NONE
//! formal_anchor_rationale: Staged dispatch only; coupling iterations belong in manifold orchestrator follow-ons.

use burn::tensor::{backend::Backend, Data, Shape, Tensor};
use umst_manifold::core::tensors::MaterialCompositionTensor;

use crate::calibration::Profile;
use crate::homogeneous::{self, MixRow};
use crate::mix_layout::{
    self, IDX_AGGREGATE_VOLUME_FRACTION, IDX_CEMENT_KG_M3, IDX_FLY_ASH_KG_M3,
    IDX_SILICA_FUME_KG_M3, IDX_SLAG_KG_M3, IDX_SUPERPLASTICIZER_KG_M3, IDX_TEMPERATURE_C,
    IDX_WATER_KG_M3, MIX_FEATURE_COUNT,
};
use crate::physics::chemo_water::ChemoWaterEngine;
use crate::physics::colloidal::ColloidalEngine;
use crate::physics::cost::compute_cost;
use crate::physics::freeze_thaw::FreezeThawEngine;
use crate::physics::hydration::compute_hydration_degree;
use crate::physics::itz::{
    compute_itz_percolation_factor, compute_itz_porosity, compute_itz_thickness_microns,
};
use crate::physics::nano::NanoEngine;
use crate::physics::packing::compute_packing_density;
use crate::physics::porosity::compute_capillary_porosity;
use crate::physics::printability::PrintabilityEngine;
use crate::physics::rheology::RheologyEngine;
use crate::physics::self_heal::SelfHealEngine;
use crate::physics::set_time::SetTimeEngine;
use crate::physics::strength::StrengthEngine;
use crate::physics::sustainability::SustainabilityEngine;
use crate::physics::thermo::ThermoEngine;
use crate::physics::transport::TransportEngine;
use crate::pipeline::b1_orchestrator_delegate::compute_effective_modulus_mt_orchestrator;
use crate::pipeline::b2_orchestrator_delegate::{
    capillary_porosity_b3_audit, try_autogenous_shrinkage_orchestrator,
    try_creep_compliance_orchestrator, try_fracture_k_ic_orchestrator, OrchestratorMixScalars,
};
use crate::pipeline::cast_phase::{
    classify_cast_phase, stage_eligible, CastPhaseInputs,
};
use crate::pipeline::report::{
    PhysicsPipelineReport, PhysicsPipelineSummary, PipelineStageRecord,
    PHYSICS_PIPELINE_SCHEMA_VERSION,
};

fn layout_snapshot<B: Backend<FloatElem = f32>>(
    mix: &MaterialCompositionTensor<B>,
) -> [f32; MIX_FEATURE_COUNT] {
    let data = mix.fractions.clone().into_data().value;
    let mut arr = [0_f32; MIX_FEATURE_COUNT];
    for (i, v) in data.iter().copied().enumerate().take(MIX_FEATURE_COUNT) {
        arr[i] = v;
    }
    arr
}

#[must_use]
fn mix_row_from_layout(layout: &[f32; MIX_FEATURE_COUNT]) -> MixRow {
    MixRow {
        cement_kg_m3: layout[IDX_CEMENT_KG_M3],
        slag_kg_m3: layout[IDX_SLAG_KG_M3],
        fly_ash_kg_m3: layout[IDX_FLY_ASH_KG_M3],
        water_kg_m3: layout[IDX_WATER_KG_M3],
        superplasticizer_kg_m3: layout[IDX_SUPERPLASTICIZER_KG_M3],
        age_days: layout[crate::mix_layout::IDX_AGE_DAYS].max(0.0),
        temperature_c: layout[IDX_TEMPERATURE_C],
    }
}

#[must_use]
fn effective_w_c_homogeneous(profile: &Profile, row: &MixRow) -> Option<f32> {
    homogeneous::mix_hydration_state(profile, row)
        .ok()
        .map(|(w, _, _)| w)
}

#[must_use]
fn binder_kg(row: &MixRow) -> f32 {
    row.cement_kg_m3 + row.slag_kg_m3 + row.fly_ash_kg_m3
}

#[must_use]
fn scm_mass_fraction(row: &MixRow) -> f32 {
    let b = binder_kg(row);
    if b <= 0.0 {
        return 0.0;
    }
    (row.slag_kg_m3 + row.fly_ash_kg_m3) / b
}

/// Mix-faithful YODEL inputs from collapsed layout (φ from solids + w/c, f_σ knocked by SP).
#[must_use]
fn yodel_inputs_from_layout(
    layout: &[f32; MIX_FEATURE_COUNT],
    w_c_eff: f32,
) -> (f32, f32, f32, f32) {
    let agg_vf = layout[IDX_AGGREGATE_VOLUME_FRACTION].clamp(0.0, 0.90);
    // Higher w/c → wetter paste → lower effective solid fraction in YODEL packing term.
    let wc_knock = (w_c_eff / 0.45_f32).clamp(0.65, 1.35);
    let phi = ((1.0 - agg_vf) / wc_knock).clamp(0.15, 0.55);
    let phi_m = 0.74_f32;
    let d50 = 50e-6_f32;
    let cement = layout[IDX_CEMENT_KG_M3].max(1.0);
    let sp_frac = (layout[IDX_SUPERPLASTICIZER_KG_M3] / cement).clamp(0.0, 0.05);
    let f_sigma = 50.0_f32 * (1.0 - 0.35 * (sp_frac / 0.01).min(1.0));
    (phi, phi_m, d50, f_sigma)
}

fn t01<B: Backend<FloatElem = f32>>(v: f32, device: &B::Device) -> Tensor<B, 2> {
    Tensor::from_data(Data::new(vec![v], Shape::new([1, 1])), device)
}

fn t4_scalar<B: Backend<FloatElem = f32>>(v: f32, device: &B::Device) -> Tensor<B, 4> {
    mix_layout::collapsed_rank4_from_rank2_scalar(t01(v, device), device)
}

#[must_use]
fn min_f32_rank4<B: Backend<FloatElem = f32>>(t: Tensor<B, 4>) -> f32 {
    t.into_data()
        .value
        .into_iter()
        .fold(f32::INFINITY, f32::min)
}

/// Parity-preserving sentinel when a stage is skipped for cast-phase incompatibility (MP3.2).
const PHASE_SKIP_SENTINEL: f32 = 0.0;

/// Runs staged tensor physics modules and aggregates a deterministic JSON/report capsule.
///
/// **Representation:** `batch_collapsed` — rank‑4 kernels use singleton spatial axes (see [`crate::mix_layout::collapsed_rank4_from_rank2_scalar`]).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Cartridge functor composition root exercised by tooling + tests.
#[must_use]
pub fn run_full_physics_pipeline<B: Backend<FloatElem = f32>>(
    profile: &Profile,
    mix: &MaterialCompositionTensor<B>,
) -> PhysicsPipelineReport {
    let dev = mix.fractions.device();
    let layout = layout_snapshot(mix);
    let row = mix_row_from_layout(&layout);
    let agg_vf = layout[IDX_AGGREGATE_VOLUME_FRACTION].clamp(0.0, 0.90);

    let w_c_eff = effective_w_c_homogeneous(profile, &row)
        .unwrap_or_else(|| (row.water_kg_m3 / binder_kg(&row).max(1.0)).clamp(0.2, 0.85));

    let mut orchestrator_mix = OrchestratorMixScalars {
        w_c_eff: w_c_eff as f64,
        hydration_alpha: 0.0,
        fc_mpa: PHASE_SKIP_SENTINEL as f64,
        cement_kg_m3: row.cement_kg_m3.max(1.0) as f64,
        scm_mass_fraction: scm_mass_fraction(&row) as f64,
        age_days: row.age_days.max(0.1) as f64,
    };

    let mut stages = Vec::new();

    let age_t = t01(row.age_days, &dev);
    let temp_t = t01(row.temperature_c, &dev);
    let alpha_t = compute_hydration_degree(mix, age_t, temp_t);
    let alpha = alpha_t.slice([0..1, 0..1]).into_scalar().clamp(0.0, 1.0);
    orchestrator_mix.hydration_alpha = alpha as f64;
    stages.push(PipelineStageRecord::ok("hydration_degree"));

    // MP3.2: classify cast lifecycle and route stages by eligibility matrix.
    let material_phase = classify_cast_phase(
        &CastPhaseInputs {
            hydration_alpha: alpha,
            yield_stress_pa: PHASE_SKIP_SENTINEL,
            age_days: row.age_days,
        },
        &profile.cast_lifecycle,
    );

    // Summary scalars default to parity sentinels; overwritten only when a stage executes.
    let mut fc_scalar = PHASE_SKIP_SENTINEL;
    let mut tau_scalar = PHASE_SKIP_SENTINEL;
    let mut eta_scalar = PHASE_SKIP_SENTINEL;
    let mut thermo_rise = PHASE_SKIP_SENTINEL;
    let mut d_scalar = PHASE_SKIP_SENTINEL;
    let mut build_scalar = PHASE_SKIP_SENTINEL;
    let mut extr_scalar = PHASE_SKIP_SENTINEL;
    let mut itz_mic = PHASE_SKIP_SENTINEL;
    let mut k_ic_scalar = PHASE_SKIP_SENTINEL;
    let mut shrink_scalar = PHASE_SKIP_SENTINEL;
    let mut ft_scalar = PHASE_SKIP_SENTINEL;
    let mut creep_scalar = PHASE_SKIP_SENTINEL;
    let mut dlvo_min = PHASE_SKIP_SENTINEL;

    let coarse_vf = agg_vf * 0.60_f32;
    let fine_vf = (agg_vf * 0.40_f32).max(0.0);
    compute_packing_density::<B>(t01(coarse_vf, &dev), t01(fine_vf, &dev));
    stages.push(PipelineStageRecord::ok("packing_density"));

    let wc_rank2 = t01(w_c_eff, &dev);
    let alpha_rank2 = t01(alpha, &dev);
    let poro_b = compute_capillary_porosity(wc_rank2.clone(), alpha_rank2.clone());
    let porosity_scalar = poro_b.slice([0..1, 0..1]).into_scalar();
    // B3 shadow audit — chem capwrap φ_c; report SSOT remains tensor path until parity witness.
    let _phi_b3_audit = capillary_porosity_b3_audit(w_c_eff as f64, alpha as f64);
    debug_assert!(_phi_b3_audit.is_finite());
    debug_assert!((0.0..=1.0).contains(&_phi_b3_audit));
    stages.push(PipelineStageRecord::ok("porosity_capillary_bulk"));

    let air_content = mix_layout::collapsed_rank4_from_rank2_scalar(t01(0.02_f32, &dev), &dev);
    let wc4 = mix_layout::collapsed_rank4_from_rank2_scalar(wc_rank2.clone(), &dev);
    let a4 = mix_layout::collapsed_rank4_from_rank2_scalar(alpha_rank2.clone(), &dev);
    let intrinsic_scale = profile.powers.s_intrinsic as f32;
    let intrinsic = mix_layout::collapsed_rank4_from_rank2_scalar(t01(intrinsic_scale, &dev), &dev);

    if stage_eligible("strength_jennings", material_phase) {
        let (fc_tensor, _, _) = StrengthEngine::<B>::compute_strength_jennings(
            wc4.clone(),
            a4.clone(),
            air_content,
            intrinsic,
        );
        fc_scalar = fc_tensor.into_data().value[0];
        orchestrator_mix.fc_mpa = fc_scalar as f64;
        stages.push(PipelineStageRecord::ok("strength_jennings"));
    } else {
        stages.push(PipelineStageRecord::skip_incompatible_phase(
            "strength_jennings",
            material_phase,
        ));
    }

    if stage_eligible("colloidal_dlvo", material_phase) {
        let zeta_nom = Tensor::from_data(Data::new(vec![-25.0_f32], Shape::new([1, 1, 1, 1])), &dev);
        let ionic = Tensor::from_data(Data::new(vec![0.03_f32], Shape::new([1, 1, 1, 1])), &dev);
        let sep = Tensor::from_data(Data::new(vec![50.0_f32], Shape::new([1, 1, 1, 1])), &dev);
        let dlvo_col = ColloidalEngine::<B>::compute_dlvo_potential(sep, zeta_nom, ionic);
        dlvo_min = min_f32_rank4(dlvo_col);
        stages.push(PipelineStageRecord::ok("colloidal_dlvo"));
    } else {
        stages.push(PipelineStageRecord::skip_incompatible_phase(
            "colloidal_dlvo",
            material_phase,
        ));
    }

    if stage_eligible("rheology_yodel", material_phase) {
        let (phi_s, phi_m_s, d50_s, f_sigma_s) = yodel_inputs_from_layout(&layout, w_c_eff);
        let phi = Tensor::from_data(Data::new(vec![phi_s], Shape::new([1, 1, 1, 1])), &dev);
        let phi_m = Tensor::from_data(Data::new(vec![phi_m_s], Shape::new([1, 1, 1, 1])), &dev);
        let d50 = Tensor::from_data(Data::new(vec![d50_s], Shape::new([1, 1, 1, 1])), &dev);
        let f_sigma = Tensor::from_data(Data::new(vec![f_sigma_s], Shape::new([1, 1, 1, 1])), &dev);
        let tau_y = RheologyEngine::<B>::compute_yield_stress_yodel(
            phi.clone(),
            phi_m.clone(),
            d50.clone(),
            f_sigma.clone(),
        );
        tau_scalar = tau_y.into_data().value.into_iter().fold(0_f32, f32::max);
        stages.push(PipelineStageRecord::ok("rheology_yodel"));

        if stage_eligible("rheology_chateau_ovarlez", material_phase) {
            let eta_intrinsic =
                Tensor::from_data(Data::new(vec![2.5_f32], Shape::new([1, 1, 1, 1])), &dev);
            let eta_fluid =
                Tensor::from_data(Data::new(vec![0.001_f32], Shape::new([1, 1, 1, 1])), &dev);
            let eta_plastic = RheologyEngine::<B>::compute_chateau_ovarlez(
                phi.clone(),
                phi_m.clone(),
                eta_intrinsic,
                eta_fluid,
            );
            eta_scalar = min_f32_rank4(eta_plastic);
            stages.push(PipelineStageRecord::ok("rheology_chateau_ovarlez"));
        } else {
            stages.push(PipelineStageRecord::skip_incompatible_phase(
                "rheology_chateau_ovarlez",
                material_phase,
            ));
        }
    } else {
        stages.push(PipelineStageRecord::skip_incompatible_phase(
            "rheology_yodel",
            material_phase,
        ));
        stages.push(PipelineStageRecord::skip_incompatible_phase(
            "rheology_chateau_ovarlez",
            material_phase,
        ));
    }

    let temp_c_4 = Tensor::from_data(
        Data::new(vec![row.temperature_c], Shape::new([1, 1, 1, 1])),
        &dev,
    );
    let alpha_4 = Tensor::from_data(Data::new(vec![alpha], Shape::new([1, 1, 1, 1])), &dev);

    if stage_eligible("thermo_heat_rate_proxy", material_phase) {
        let ea = Tensor::from_data(Data::new(vec![40e3_f32], Shape::new([1, 1, 1, 1])), &dev);
        let (_, dt_adiabatic) =
            ThermoEngine::<B>::compute_heat_rate(temp_c_4.clone(), alpha_4, ea.clone());
        thermo_rise = dt_adiabatic.into_data().value[0];
        stages.push(PipelineStageRecord::ok("thermo_heat_rate_proxy"));
    } else {
        stages.push(PipelineStageRecord::skip_incompatible_phase(
            "thermo_heat_rate_proxy",
            material_phase,
        ));
    }

    let phi_cap_t = if stage_eligible("transport_chloride", material_phase)
        || stage_eligible("chemo_water", material_phase)
    {
        Some(TransportEngine::<B>::compute_capillary_porosity(wc4.clone(), a4.clone()))
    } else {
        None
    };

    if stage_eligible("transport_chloride", material_phase) {
        let phi_cap = phi_cap_t.as_ref().expect("phi_cap prepared for transport");
        let ref_d = Tensor::from_data(Data::new(vec![1e-12_f32], Shape::new([1, 1, 1, 1])), &dev);
        let d_cl = TransportEngine::<B>::compute_chloride_diffusivity(phi_cap.clone(), ref_d);
        d_scalar = min_f32_rank4(d_cl);
        stages.push(PipelineStageRecord::ok("transport_chloride"));
    } else {
        stages.push(PipelineStageRecord::skip_incompatible_phase(
            "transport_chloride",
            material_phase,
        ));
    }

    if stage_eligible("printability", material_phase) {
        let tau_for_print = tau_scalar.max(50.0_f32);
        let build = PrintabilityEngine::<B>::compute_buildability(
            t4_scalar(tau_for_print, &dev),
            t4_scalar(w_c_eff, &dev),
            80.0,
        );
        build_scalar = min_f32_rank4(build);
        let pump_pa = (tau_for_print * 0.85).max(45.0);
        let extr = PrintabilityEngine::<B>::compute_extrudability(
            t4_scalar(tau_for_print, &dev),
            t4_scalar(pump_pa, &dev),
            16.0,
            120.0,
        );
        extr_scalar = min_f32_rank4(extr.clone());
        if extr
            .into_data()
            .value
            .iter()
            .copied()
            .all(|x| x.is_finite())
        {
            stages.push(PipelineStageRecord::ok("printability"));
        } else {
            stages.push(PipelineStageRecord::fail(
                "printability",
                "non-finite extrudability",
            ));
        }
    } else {
        stages.push(PipelineStageRecord::skip_incompatible_phase(
            "printability",
            material_phase,
        ));
    }

    if stage_eligible("itz", material_phase) {
        let thick = compute_itz_thickness_microns::<B>(Tensor::from_data(
            Data::new(vec![w_c_eff * 48.0_f32.sqrt()], Shape::new([1, 1])),
            &dev,
        ));
        itz_mic = thick.slice([0..1, 0..1]).into_scalar();
        let itz_poro = compute_itz_porosity::<B>(wc_rank2.clone());
        let _itz_p = itz_poro.slice([0..1, 0..1]).into_scalar();
        let _perc = compute_itz_percolation_factor::<B>(Tensor::from_data(
            Data::new(vec![0.25_f32], Shape::new([1, 1])),
            &dev,
        ));
        stages.push(PipelineStageRecord::ok("itz"));
    } else {
        stages.push(PipelineStageRecord::skip_incompatible_phase("itz", material_phase));
    }

    if stage_eligible("chemo_water", material_phase) {
        let phi_cap = phi_cap_t
            .as_ref()
            .cloned()
            .unwrap_or_else(|| TransportEngine::<B>::compute_capillary_porosity(wc4.clone(), a4.clone()));
        let (_rh_internal, tension) = ChemoWaterEngine::<B>::compute_moisture_transport(
            wc4.clone(),
            a4.clone(),
            mix_layout::collapsed_rank4_from_rank2_scalar(
                Tensor::from_data(Data::new(vec![0.65_f32], Shape::new([1, 1])), &dev),
                &dev,
            ),
            phi_cap,
        );
        let _rh_peak = tension.into_data().value.into_iter().fold(0_f32, f32::max);
        stages.push(PipelineStageRecord::ok("chemo_water"));
    } else {
        stages.push(PipelineStageRecord::skip_incompatible_phase(
            "chemo_water",
            material_phase,
        ));
    }

    if stage_eligible("fracture", material_phase) {
        // B1 carve @ g_spawn_i_b16_mt_carve_0721 — continuum homogenization; monolith tensor retained until S7.
        let e_eff_scalar = compute_effective_modulus_mt_orchestrator(fc_scalar as f64);
        k_ic_scalar = try_fracture_k_ic_orchestrator(e_eff_scalar, profile.powers.s_intrinsic)
            .unwrap_or(PHASE_SKIP_SENTINEL);
        stages.push(PipelineStageRecord::ok("fracture"));
    } else {
        stages.push(PipelineStageRecord::skip_incompatible_phase("fracture", material_phase));
    }

    if stage_eligible("nano_enhancement_baseline", material_phase) {
        NanoEngine::<B>::compute_enhancements(
            Tensor::from_data(Data::new(vec![0.01_f32], Shape::new([1, 1, 1, 1])), &dev),
            Tensor::from_data(Data::new(vec![200.0_f32], Shape::new([1, 1, 1, 1])), &dev),
            Tensor::from_data(Data::new(vec![1.0_f32], Shape::new([1, 1, 1, 1])), &dev),
        );
        stages.push(PipelineStageRecord::ok("nano_enhancement_baseline"));
    } else {
        stages.push(PipelineStageRecord::skip_incompatible_phase(
            "nano_enhancement_baseline",
            material_phase,
        ));
    }

    let mass_ce = Tensor::from_data(
        Data::new(vec![row.cement_kg_m3.max(0.0)], Shape::new([1, 1, 1, 1])),
        &dev,
    );
    let mass_scm = Tensor::from_data(
        Data::new(
            vec![row.slag_kg_m3 + row.fly_ash_kg_m3 + layout[IDX_SILICA_FUME_KG_M3]],
            Shape::new([1, 1, 1, 1]),
        ),
        &dev,
    );
    let agg_mass_kg = agg_vf * 2400.0_f32;
    let mass_agg = Tensor::from_data(Data::new(vec![agg_mass_kg], Shape::new([1, 1, 1, 1])), &dev);
    let mass_water = Tensor::from_data(
        Data::new(vec![row.water_kg_m3.max(0.0)], Shape::new([1, 1, 1, 1])),
        &dev,
    );
    let co2 = (0.93_f32, 0.05_f32, 0.005_f32, 0.0003_f32);
    let gwp_tensor = SustainabilityEngine::<B>::compute_embodied_carbon(
        mass_ce.clone(),
        mass_scm.clone(),
        mass_agg.clone(),
        mass_water.clone(),
        co2,
    );
    let gwp_scalar = min_f32_rank4(gwp_tensor.clone());

    let usd = (0.13_f32, 0.05_f32, 0.02_f32, 0.002_f32);
    let cost_tensor = SustainabilityEngine::<B>::compute_financial_cost(
        mass_ce, mass_scm, mass_agg, mass_water, usd,
    );
    let cost_scalar = min_f32_rank4(cost_tensor);
    stages.push(PipelineStageRecord::ok("sustainability"));

    let mut unit = [0.01_f32; MIX_FEATURE_COUNT];
    unit[IDX_CEMENT_KG_M3] = 0.93;
    unit[IDX_SLAG_KG_M3] = 0.05;
    unit[IDX_FLY_ASH_KG_M3] = 0.05;
    unit[IDX_WATER_KG_M3] = 0.0003;
    let unit_t = Tensor::from_data(
        Data::new(unit.to_vec(), Shape::new([1, MIX_FEATURE_COUNT])),
        &dev,
    );
    let _dot_cost = compute_cost(mix, unit_t);
    stages.push(PipelineStageRecord::ok("cost_linear_dot"));

    // S6_RETIRE @ g_spawn_i_orch_2054 — B2 scalar delegate; Burn `physics/creep.rs` retained.
    if stage_eligible("creep", material_phase) {
        creep_scalar = try_creep_compliance_orchestrator(orchestrator_mix)
            .unwrap_or(PHASE_SKIP_SENTINEL);
        stages.push(PipelineStageRecord::ok("creep"));
    } else {
        stages.push(PipelineStageRecord::skip_incompatible_phase("creep", material_phase));
    }

    let scm_r_t = scm_mass_fraction(&row);
    if stage_eligible("set_time", material_phase) {
        let (_, _) = SetTimeEngine::<B>::compute_setting_time(
            wc4.clone(),
            temp_c_4.clone(),
            Tensor::from_data(Data::new(vec![0.75_f32], Shape::new([1, 1, 1, 1])), &dev),
            mix_layout::collapsed_rank4_from_rank2_scalar(
                Tensor::from_data(Data::new(vec![scm_r_t], Shape::new([1, 1])), &dev),
                &dev,
            ),
            Tensor::from_data(Data::new(vec![0.0_f32], Shape::new([1, 1, 1, 1])), &dev),
            Tensor::from_data(Data::new(vec![1.0_f32], Shape::new([1, 1, 1, 1])), &dev),
            Tensor::from_data(Data::new(vec![350.0_f32], Shape::new([1, 1, 1, 1])), &dev),
            Tensor::from_data(Data::new(vec![0.55_f32], Shape::new([1, 1, 1, 1])), &dev),
        );
        stages.push(PipelineStageRecord::ok("set_time"));
    } else {
        stages.push(PipelineStageRecord::skip_incompatible_phase("set_time", material_phase));
    }

    if stage_eligible("shrinkage", material_phase) {
        shrink_scalar = try_autogenous_shrinkage_orchestrator(orchestrator_mix)
            .map(|v| v.max(0.0_f32))
            .unwrap_or(PHASE_SKIP_SENTINEL);
        stages.push(PipelineStageRecord::ok("shrinkage"));
    } else {
        stages.push(PipelineStageRecord::skip_incompatible_phase("shrinkage", material_phase));
    }

    if stage_eligible("freeze_thaw", material_phase) {
        let paste_frac = Tensor::from_data(
            Data::new(
                vec![(1.0_f32 - agg_vf).clamp(0.15, 0.65)],
                Shape::new([1, 1, 1, 1]),
            ),
            &dev,
        );
        let (_, durability_factor) = FreezeThawEngine::<B>::compute_durability(
            Tensor::from_data(Data::new(vec![0.04_f32], Shape::new([1, 1, 1, 1])), &dev),
            paste_frac,
            Tensor::from_data(Data::new(vec![35.0_f32], Shape::new([1, 1, 1, 1])), &dev),
            6.0_f32,
        );
        ft_scalar = min_f32_rank4(durability_factor);
        stages.push(PipelineStageRecord::ok("freeze_thaw"));
    } else {
        stages.push(PipelineStageRecord::skip_incompatible_phase(
            "freeze_thaw",
            material_phase,
        ));
    }

    stages.push(PipelineStageRecord::skip_missing(
        "polymer",
        "zero polymer-cement ratio on bulk path",
    ));

    if stage_eligible("self_heal", material_phase) {
        let internal_rh = mix_layout::collapsed_rank4_from_rank2_scalar(
            Tensor::from_data(Data::new(vec![0.92_f32], Shape::new([1, 1])), &dev),
            &dev,
        );
        SelfHealEngine::<B>::compute_healing_potential(a4.clone(), internal_rh, t4_scalar(0.0, &dev));
        stages.push(PipelineStageRecord::ok("self_heal"));
    } else {
        stages.push(PipelineStageRecord::skip_incompatible_phase("self_heal", material_phase));
    }

    stages.push(PipelineStageRecord::skip_missing(
        "fiber",
        "zero fiber volume fraction on bulk path",
    ));

    let phase_skip_sentinels = stages.iter().any(|s| {
        matches!(
            s.status,
            crate::pipeline::report::PipelineStageStatus::SkippedIncompatiblePhase
        )
    });

    PhysicsPipelineReport {
        schema_version: PHYSICS_PIPELINE_SCHEMA_VERSION.to_string(),
        representation: "batch_collapsed",
        material_phase,
        phase_skip_sentinels,
        stages,
        summary: PhysicsPipelineSummary {
            effective_water_cement_ratio: w_c_eff,
            hydration_alpha: alpha,
            porosity_capillary: porosity_scalar,
            strength_jennings_mpa: fc_scalar,
            rheology_yield_stress_pa: tau_scalar,
            rheology_plastic_viscosity_pa_s: eta_scalar,
            thermo_adiabatic_rise_proxy_c: thermo_rise,
            chloride_diffusivity_m2_s: d_scalar,
            printability_buildability: build_scalar,
            printability_extrudability: extr_scalar,
            itz_thickness_microns: itz_mic,
            fracture_toughness_k_ic_mpa_sqrt_m: k_ic_scalar,
            sustainability_gwp_kg_co2_m3: gwp_scalar,
            sustainability_cost_usd_per_m3: cost_scalar,
            dlvo_potential_kt_minimum: dlvo_min,
            shrinkage_microstrain_proxy: shrink_scalar,
            freeze_thaw_durability_factor: ft_scalar,
            creep_compliance_1_over_gpa: creep_scalar,
        },
    }
}
