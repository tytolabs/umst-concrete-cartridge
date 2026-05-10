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
use umst_manifold::core::traits::PhysicalResult;

use crate::calibration::Profile;
use crate::pipeline::report::PhysicsPipelineReport;

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

    PhysicalResult {
        free_energy,
        dissipation,
        safety_margin: safety_margin_tensor,
        cost,
    }
}
