// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Maps [`PhysicsPipelineReport`](super::PhysicsPipelineReport) into manifold [`PhysicalResult`](umst_manifold::core::traits::PhysicalResult).
//!
//! This is **control-policy**, not a thermodynamic identity:
//! - `free_energy[..,0]` = Jennings tensor strength headline (MPa).
//! - `free_energy[..,1]` = YODEL yield stress (Pa).
//! - `dissipation[..,0]` = tensor-route hydration degree α (`physics::hydration`).
//! - `safety_margin[..,0]` = homogeneous admissibility margin using the **same** α and effective w/c.
//! - `cost[..,0]` = sustainability embodied carbon (kg CO₂-eq / m³).
//!
//! formal_anchor: NONE
//! formal_status: NONE
//! formal_anchor_rationale: Orchestrator summary only; differentiable potentials live in manifold adapters, not duplicated here.

use burn::tensor::{backend::Backend, Data, Shape, Tensor};
use umst_manifold::core::tensors::MixTensor;
use umst_manifold::core::traits::PhysicalResult;

use crate::calibration::Profile;
use crate::homogeneous::mix_row_from_scalar_spec;
use crate::mix_layout::{fractions_from_mix_row, mix_tensor_from_layout};
use crate::pipeline::report::PhysicsPipelineReport;

/// Single-run nominal-mix [`PhysicsPipelineReport`] for topology (avoids duplicate `run_full_physics_pipeline` calls).
/// formal_anchor / formal_status omitted (`pub(crate)` — not part of public façade ledger).
#[must_use]
pub(crate) fn topology_pipeline_report<B: Backend<FloatElem = f32>>(
    profile: &Profile,
    device: &B::Device,
) -> PhysicsPipelineReport {
    let mix: MixTensor<B> = nominal_mix_tensor_for_topology::<B>(profile, device);
    super::run_full_physics_pipeline::<B>(profile, &mix)
}

#[must_use]
pub(crate) fn topology_pipeline_headlines_from_report(
    report: &PhysicsPipelineReport,
) -> (f32, f32, f32, f32, f32, f32) {
    let s = &report.summary;
    (
        s.strength_jennings_mpa,
        s.rheology_yield_stress_pa,
        s.sustainability_gwp_kg_co2_m3,
        s.effective_water_cement_ratio,
        s.hydration_alpha,
        s.fracture_toughness_k_ic_mpa_sqrt_m,
    )
}

/// Single source of truth for [`PhysicalResult`] assembly from a pipeline report + profile cues.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Encodes CLI tensor summary contract; see module-level policy mapping.
#[must_use]
pub fn physical_result_from_report<B: Backend<FloatElem = f32>>(
    profile: &Profile,
    report: &PhysicsPipelineReport,
    device: &B::Device,
) -> PhysicalResult<B> {
    let fc = report.summary.strength_jennings_mpa;
    let tau = report.summary.rheology_yield_stress_pa;
    let alpha = report.summary.hydration_alpha;
    let gwp = report.summary.sustainability_gwp_kg_co2_m3;
    let w_c_eff = report.summary.effective_water_cement_ratio;

    let margin = crate::homogeneous::safety_margin(profile, w_c_eff, alpha);

    let free_energy =
        Tensor::<B, 2>::from_data(Data::new(vec![fc, tau], Shape::new([1, 2])), device);
    let dissipation = Tensor::<B, 2>::from_data(Data::new(vec![alpha], Shape::new([1, 1])), device);
    let safety_margin_tensor =
        Tensor::<B, 2>::from_data(Data::new(vec![margin], Shape::new([1, 1])), device);
    let cost = Tensor::<B, 2>::from_data(Data::new(vec![gwp], Shape::new([1, 1])), device);

    let damage = Tensor::<B, 2>::zeros([1, 1], device);

    PhysicalResult {
        free_energy,
        dissipation,
        safety_margin: safety_margin_tensor,
        cost,
        damage,
        temperature_delta: None,
    }
}

/// Regime-centered nominal [`MixTensor`] when topology receives only a [`UnifiedMaterialStateTensor`](umst_manifold::core::tensors::UnifiedMaterialStateTensor)
/// (no explicit recipe). Uses profile `[regime]` midpoints and conservative SCM splits from optional caps.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Deterministic surrogate mix so staged tensor engines (`StrengthEngine`, fracture headline, sustainability) match `compute_all` semantics.
#[must_use]
pub fn nominal_mix_tensor_for_topology<B: Backend<FloatElem = f32>>(
    profile: &Profile,
    device: &B::Device,
) -> MixTensor<B> {
    let w_c = (((profile.regime.w_c_min + profile.regime.w_c_max) * 0.5) as f32).clamp(0.10, 1.0);
    let age_h =
        (((profile.regime.age_hours_min + profile.regime.age_hours_max) * 0.5) as f32).max(0.0);
    let temp_k = (((profile.regime.temperature_k_min + profile.regime.temperature_k_max) * 0.5)
        as f32)
        .max(1.0);
    let fly_pct = profile
        .regime
        .fly_ash_pct_max
        .map(|x| ((x * 0.5) as f32).clamp(0.0, 75.0))
        .unwrap_or(0.0);
    let silica_pct = profile
        .regime
        .silica_fume_pct_max
        .map(|x| ((x * 0.5) as f32).clamp(0.0, 75.0))
        .unwrap_or(0.0);
    let phi = 0.65_f32;
    let sp_pct = 0.5_f32;
    let row = mix_row_from_scalar_spec(
        profile, w_c, sp_pct, fly_pct, silica_pct, phi, age_h, temp_k,
    );
    let layout = fractions_from_mix_row(&row, phi);
    mix_tensor_from_layout(&layout, device)
}

/// Headline scalars from [`super::run_full_physics_pipeline`] on [`nominal_mix_tensor_for_topology`], aligned with
/// [`physical_result_from_report`] (`free_energy` / `cost` / admissibility wires).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Single SSOT with Jennings/YODEL/GWP tensor pipeline used by bulk predict.
#[must_use]
pub fn topology_pipeline_headlines<B: Backend<FloatElem = f32>>(
    profile: &Profile,
    device: &B::Device,
) -> (f32, f32, f32, f32, f32, f32) {
    topology_pipeline_headlines_from_report(&topology_pipeline_report::<B>(profile, device))
}
