// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Mix-space geometry index — UMST Hilbert **discipline** on constitutive coordinates (not SDF voxels).
//!
//! Maps `(w_c, temperature_k, aggregate_volume_fraction)` to a 2D quantized grid, then a Morton
//! (Z-order) curve index for locality-preserving retrieval. Pure morphisms only.

use super::contribution::mix_wire_from_spec_value;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Geometry key stored on durable memory rows (regime bucket + curve index).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Design-coordinate locality index; no thermodynamic claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MixGeometryKey {
    pub hilbert_index: u32,
    pub regime_bucket: String,
    pub grid_x: u32,
    pub grid_y: u32,
}

const GRID_BITS: u32 = 8;
const GRID_SIDE: u32 = 1 << GRID_BITS;

/// Quantize `w_c` ∈ [0.25, 0.75] to grid axis.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Morton grid quantization on w_c; retrieval heuristic only.
#[must_use]
pub fn quantize_w_c(w_c: f64) -> u32 {
    let t = ((w_c - 0.25) / 0.50).clamp(0.0, 1.0);
    (t * (GRID_SIDE - 1) as f64).round() as u32
}

/// Quantize temperature ∈ [273, 333] K to grid axis.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Morton grid quantization on temperature; no constitutive law.
#[must_use]
pub fn quantize_temperature_k(temp_k: f64) -> u32 {
    let t = ((temp_k - 273.0) / 60.0).clamp(0.0, 1.0);
    (t * (GRID_SIDE - 1) as f64).round() as u32
}

/// Morton (Z-order) interleave — locality heuristic matching cockpit `hilbert_index` role.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Z-order curve index for memory locality queries.
#[must_use]
pub fn morton_index(x: u32, y: u32) -> u32 {
    let x = x.min(GRID_SIDE - 1);
    let y = y.min(GRID_SIDE - 1);
    let mut idx = 0u32;
    for bit in 0..GRID_BITS {
        idx |= ((x >> bit) & 1) << (2 * bit);
        idx |= ((y >> bit) & 1) << (2 * bit + 1);
    }
    idx
}

/// Pure: derive geometry key from `mix_spec` + optional curing regime label.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Mix-space Morton key for `memory_query`; not admissibility gate.
#[must_use]
pub fn mix_geometry_key(mix_spec: &Value, curing_regime: Option<&str>) -> Option<MixGeometryKey> {
    let wire = mix_wire_from_spec_value(mix_spec)?;
    let w_c = wire.w_c;
    let temp = wire.temperature_k;
    let grid_x = quantize_w_c(w_c);
    let grid_y = quantize_temperature_k(temp);
    let regime = curing_regime.unwrap_or("unspecified").to_string();
    Some(MixGeometryKey {
        hilbert_index: morton_index(grid_x, grid_y),
        regime_bucket: regime,
        grid_x,
        grid_y,
    })
}

/// L1 distance in normalized mix space (w_c, temperature_k, aggregate).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Nearest-neighbor mix distance for query sorting; advisory geometry.
#[must_use]
pub fn mix_l1_distance(a: &Value, b: &Value) -> Option<f64> {
    let wa = mix_wire_from_spec_value(a)?;
    let wb = mix_wire_from_spec_value(b)?;
    let agg_a = wa.aggregate_volume_fraction.unwrap_or(0.65);
    let agg_b = wb.aggregate_volume_fraction.unwrap_or(0.65);
    Some(
        (wa.w_c - wb.w_c).abs()
            + ((wa.temperature_k - wb.temperature_k) / 60.0).abs()
            + (agg_a - agg_b).abs(),
    )
}

/// Morton index distance (locality proxy along curve).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Hilbert-index distance for locality filter; no physics claim.
#[must_use]
pub fn morton_index_distance(a: u32, b: u32) -> u32 {
    a.abs_diff(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn morton_locality_neighbor_closer_than_far() {
        let near = morton_index(10, 10);
        let neighbor = morton_index(11, 10);
        let far = morton_index(200, 200);
        assert!(morton_index_distance(near, neighbor) < morton_index_distance(near, far));
    }

    #[test]
    fn mix_geometry_key_from_rationals() {
        let mix = json!({
            "w_c": "9/20",
            "temperature_k": "29315/100",
            "aggregate_volume_fraction": "7/10"
        });
        let key = mix_geometry_key(&mix, Some("standard_20C_water")).unwrap();
        assert!(!key.regime_bucket.is_empty());
        assert!(key.hilbert_index < (1 << (2 * GRID_BITS)));
    }

    #[test]
    fn l1_distance_zero_for_same_mix() {
        let mix = json!({ "w_c": "1/2", "temperature_k": "29315/100" });
        let d = mix_l1_distance(&mix, &mix).unwrap();
        assert!(d < 1e-9);
    }
}
