// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Canonical `[Batch, Features]` layout for [`StatePoint`](umst_manifold::StatePoint) used by tensor physics.
//!
//! Hydration kernels (`physics::hydration`) read cement at column **1**, slag **5**, fly ash **6** — unchanged.

use burn::tensor::{backend::Backend, Data, Shape, Tensor};
use umst_manifold::core::tensors::StatePoint;

use crate::homogeneous::MixRow;

/// Number of scalar features stored per batch row (`mix.fractions` second dimension).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Structural convention for CLI ↔ tensor engines; documented here as SSOT for column indices.
pub const MIX_FEATURE_COUNT: usize = 16;

/// Water (kg/m³).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Structural column tag for wire ↔ tensor bridging.
pub const IDX_WATER_KG_M3: usize = 0;
/// Portland cement (kg/m³). Hydration extracts this index.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Must stay aligned with `physics::hydration` slicers.
pub const IDX_CEMENT_KG_M3: usize = 1;
/// Coarse aggregate mass density placeholder column (kg/m³).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Reserved packing-engine input when recipe supplies split gradation.
pub const IDX_AGG_COARSE_KG_M3: usize = 2;
/// Fine aggregate mass density placeholder column (kg/m³).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Reserved packing-engine input when recipe supplies split gradation.
pub const IDX_AGG_FINE_KG_M3: usize = 3;
/// Reserved layout slot for future calibrated channel.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Keeps hydration column indices historically stable (`hydration.rs` assumptions).
pub const IDX_RESERVED_4: usize = 4;
/// Ground granulated blast-furnace slag (kg/m³).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Structural column aligned with SCM tensor hydrates.
pub const IDX_SLAG_KG_M3: usize = 5;
/// Fly ash (kg/m³).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Structural column for pozzolan mass routing.
pub const IDX_FLY_ASH_KG_M3: usize = 6;
/// Superplasticizer (kg/m³).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Admixture mass channel for homogeneous + rheology stubs.
pub const IDX_SUPERPLASTICIZER_KG_M3: usize = 7;
/// Age (days) for kinetic engines.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Scalar age carried on-layout for `compute_all` without extra manifold state.
pub const IDX_AGE_DAYS: usize = 8;
/// Curing temperature (°C).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Arrhenius-style tensor engines consume this lane.
pub const IDX_TEMPERATURE_C: usize = 9;
/// Silica fume (kg/m³) when differentiated from SP.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: SCM extension field for supplemental micro-silica mass.
pub const IDX_SILICA_FUME_KG_M3: usize = 10;
/// Target aggregate solids volume fraction (0–1) from mix design / recipe.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Drives packing + paste fraction surrogates on collapsed paths.
pub const IDX_AGGREGATE_VOLUME_FRACTION: usize = 11;
// Indices 12..16 reserved for future calibrated scalars encoded on the wire.

fn zero_row() -> [f32; MIX_FEATURE_COUNT] {
    [0.0; MIX_FEATURE_COUNT]
}

/// Packs [`MixRow`] mass fractions plus recipe metadata used by staged physics.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Deterministic CSV-style encoding; aligns with homogeneous `MixRow` used by CLI.
#[must_use]
pub fn fractions_from_mix_row(
    row: &MixRow,
    aggregate_volume_fraction: f32,
) -> [f32; MIX_FEATURE_COUNT] {
    let mut z = zero_row();
    z[IDX_WATER_KG_M3] = row.water_kg_m3;
    z[IDX_CEMENT_KG_M3] = row.cement_kg_m3;
    z[IDX_SLAG_KG_M3] = row.slag_kg_m3;
    z[IDX_FLY_ASH_KG_M3] = row.fly_ash_kg_m3;
    z[IDX_SUPERPLASTICIZER_KG_M3] = row.superplasticizer_kg_m3;
    z[IDX_AGE_DAYS] = row.age_days;
    z[IDX_TEMPERATURE_C] = row.temperature_c;
    z[IDX_AGGREGATE_VOLUME_FRACTION] = aggregate_volume_fraction.clamp(0.0, 0.90);
    z
}

/// Builds a single-batch [`StatePoint`] from a packed layout row.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Constructor replacing non-existent `MixTensor::from_proportions`; see README Quick start.
#[must_use]
pub fn mix_tensor_from_layout<B: Backend<FloatElem = f32>>(
    layout: &[f32; MIX_FEATURE_COUNT],
    device: &B::Device,
) -> StatePoint<B> {
    StatePoint {
        fractions: Tensor::from_data(
            Data::new(layout.to_vec(), Shape::new([1, MIX_FEATURE_COUNT])),
            device,
        ),
    }
}

/// Collapsed `[1,1,1,1]` tensor broadcasting a single scalar from `[1,1]` layout slice.
/// **Convention:** degenerate spatial axes — results are batch-only surrogates, not FEM-grade fields.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Engine APIs require rank-4 tensors; singleton spatial dims document batch-collapsed mode.
#[must_use]
pub fn collapsed_rank4_from_rank2_scalar<B: Backend<FloatElem = f32>>(
    t01: Tensor<B, 2>,
    device: &B::Device,
) -> Tensor<B, 4> {
    let v = t01.slice([0..1, 0..1]).into_scalar();
    Tensor::from_data(Data::new(vec![v], Shape::new([1, 1, 1, 1])), device)
}
