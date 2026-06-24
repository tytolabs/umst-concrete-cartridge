// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Smoothstep soft-gate templates for differentiable constraint penalties.
//!
//! Hard gates (`umst_gate_check`) commit with `f64` witnesses at the Cold boundary.
//! Training and exploration optimizers **multiply** these `f32` ramps into policy loss
//! so rejected proposals receive analytical slack gradients before the hard witness.
//!
//! See [`docs/AGENT_MCP.md`](../../../../docs/AGENT_MCP.md#soft-gates).
//!
//! ## Cold vs hot boundary (post-Wave 10)
//!
//! | Layer | Role |
//! |-------|------|
//! | **Cold (this module)** | Differentiable `smoothstep` ramps for MCP / training templates |
//! | **Warm (`umst-manifold` `ManifoldGateway`)** | `constraint_loss_penalty` + `RejectionTelemetry` on CBF witness |
//! | **Hot (Burn graph)** | `clausius_duhem_violation` only when `kleisli-ppo-hot-bind` / `epistemic-ppo` enabled |
//!
//! Cartridge `soft_gate` outputs are **not** wired directly into `ThmcSolver::step`; manifold penalize
//! consumes host `TransitionEvidence` / tensor CD slack instead. Bridge future work: map smoothstep
//! slack → `lambda_cd` episode weight at cold orchestration only.

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Scalar clamp utility; no thermodynamic gate claim.
/// Saturate `t` to `[0, 1]`.
#[must_use]
pub fn saturate01(t: f32) -> f32 {
    t.clamp(0.0, 1.0)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Hermite smoothstep kernel; training slack only.
/// Classic smoothstep on normalized `t ∈ [0, 1]`: `3t² − 2t³`.
#[must_use]
pub fn smoothstep01(t: f32) -> f32 {
    let t = saturate01(t);
    t * t * (3.0 - 2.0 * t)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Bounded Hermite ramp; not a hard admissibility witness.
/// Hermite smoothstep between `edge0` and `edge1`.
///
/// Returns `0` at or below `edge0`, `1` at or above `edge1`, with C¹ ramps in between.
#[must_use]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if edge0 == edge1 {
        return if x >= edge1 { 1.0 } else { 0.0 };
    }
    let t = saturate01((x - edge0) / (edge1 - edge0));
    smoothstep01(t)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Soft lower-bound multiplier for policy loss; hard gate is separate.
/// Soft lower bound: multiplier `1` when `x ≥ bound`, smooth ramp to `0` below.
#[must_use]
pub fn soft_lower_gate(x: f32, bound: f32, width: f32) -> f32 {
    if width <= 0.0 {
        return if x >= bound { 1.0 } else { 0.0 };
    }
    smoothstep(bound - width, bound, x)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Soft upper-bound multiplier for policy loss; hard gate is separate.
/// Soft upper bound: multiplier `1` when `x ≤ bound`, smooth ramp to `0` above.
#[must_use]
pub fn soft_upper_gate(x: f32, bound: f32, width: f32) -> f32 {
    if width <= 0.0 {
        return if x <= bound { 1.0 } else { 0.0 };
    }
    1.0 - smoothstep(bound, bound + width, x)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Product of soft lower/upper ramps; exploration slack only.
/// Band multiplier: `soft_lower_gate × soft_upper_gate` (engine soft-gate product pattern).
#[must_use]
pub fn soft_band_gate(x: f32, lo: f32, hi: f32, width: f32) -> f32 {
    soft_lower_gate(x, lo, width) * soft_upper_gate(x, hi, width)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Supercap percolation soft ramp; not connected-fraction witness.
/// Percolation / connected-fraction ramp near threshold `phi_c` (supercap template).
#[must_use]
pub fn connected_fraction_gate(phi: f32, phi_c: f32, width: f32) -> f32 {
    soft_lower_gate(phi, phi_c, width)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Alias for supercap conductivity nomenclature; soft ramp only.
/// Alias for supercap `network_conductivity_factor` nomenclature.
#[must_use]
pub fn network_conductivity_factor(connected_fraction: f32, phi_c: f32, width: f32) -> f32 {
    connected_fraction_gate(connected_fraction, phi_c, width)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Upper slack penalty for policy loss; not CD reject.
/// Penalty `∈ [0, 1]` when `x` exceeds `limit` (upper slack surrogate).
#[must_use]
pub fn soft_violation_penalty(x: f32, limit: f32, width: f32) -> f32 {
    1.0 - soft_upper_gate(x, limit, width)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Lower slack penalty for policy loss; not CD reject.
/// Penalty `∈ [0, 1]` when `x` falls below `limit` (lower slack surrogate).
#[must_use]
pub fn soft_deficit_penalty(x: f32, limit: f32, width: f32) -> f32 {
    1.0 - soft_lower_gate(x, limit, width)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Combined band margin penalty; hard band gate remains on cartridge.
/// Combined band margin penalty: deficit below `lo` plus violation above `hi`.
#[must_use]
pub fn band_margin_penalty(x: f32, lo: f32, hi: f32, width: f32) -> f32 {
    soft_deficit_penalty(x, lo, width) + soft_violation_penalty(x, hi, width)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Roussel τ₀ window soft multiplier; printability gate is hard path.
/// Concrete printability τ₀ band (Roussel window `180–360` Pa) as a soft multiplier.
#[must_use]
pub fn printability_tau_gate(tau_y_pa: f32, width: f32) -> f32 {
    soft_band_gate(tau_y_pa, 180.0, 360.0, width)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Extrudability floor soft ramp; hard extrudability gate is separate.
/// Concrete extrudability floor (`0.35`) as a soft lower gate.
#[must_use]
pub fn extrudability_gate(extrudability: f32, width: f32) -> f32 {
    soft_lower_gate(extrudability, 0.35, width)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Product of printability soft gates; dual hard gate on cartridge path.
/// Product of printability τ₀ and extrudability soft gates (dual-gate surrogate).
#[must_use]
pub fn printability_dual_gate(tau_y_pa: f32, extrudability: f32, width: f32) -> f32 {
    printability_tau_gate(tau_y_pa, width) * extrudability_gate(extrudability, width)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-5;

    #[test]
    fn smoothstep01_endpoints() {
        assert!((smoothstep01(0.0) - 0.0).abs() < EPS);
        assert!((smoothstep01(1.0) - 1.0).abs() < EPS);
        assert!((smoothstep01(0.5) - 0.5).abs() < EPS);
    }

    #[test]
    fn smoothstep_edges() {
        assert!((smoothstep(0.0, 1.0, -1.0) - 0.0).abs() < EPS);
        assert!((smoothstep(0.0, 1.0, 2.0) - 1.0).abs() < EPS);
        assert!((smoothstep(0.0, 1.0, 0.5) - 0.5).abs() < EPS);
    }

    #[test]
    fn soft_lower_gate_ramp() {
        assert!((soft_lower_gate(10.0, 5.0, 2.0) - 1.0).abs() < EPS);
        assert!((soft_lower_gate(3.0, 5.0, 2.0) - 0.0).abs() < EPS);
        let mid = soft_lower_gate(4.0, 5.0, 2.0);
        assert!(mid > 0.0 && mid < 1.0);
    }

    #[test]
    fn soft_upper_gate_ramp() {
        assert!((soft_upper_gate(1.0, 5.0, 2.0) - 1.0).abs() < EPS);
        assert!((soft_upper_gate(7.0, 5.0, 2.0) - 0.0).abs() < EPS);
    }

    #[test]
    fn soft_band_gate_interior() {
        let g = soft_band_gate(270.0, 180.0, 360.0, 20.0);
        assert!((g - 1.0).abs() < EPS);
    }

    #[test]
    fn connected_fraction_gate_percolation() {
        assert!((connected_fraction_gate(0.2, 0.18, 0.02) - 1.0).abs() < EPS);
        assert!((connected_fraction_gate(0.1, 0.18, 0.02) - 0.0).abs() < EPS);
        assert_eq!(
            network_conductivity_factor(0.2, 0.18, 0.02),
            connected_fraction_gate(0.2, 0.18, 0.02)
        );
    }

    #[test]
    fn band_margin_penalty_zero_inside() {
        assert!((band_margin_penalty(270.0, 180.0, 360.0, 20.0) - 0.0).abs() < EPS);
    }

    #[test]
    fn printability_dual_gate_passes_band() {
        let g = printability_dual_gate(250.0, 0.5, 15.0);
        assert!((g - 1.0).abs() < EPS);
    }

    #[test]
    fn soft_gate_penalties_increase_outside() {
        let inside = soft_violation_penalty(4.0, 5.0, 1.0);
        let outside = soft_violation_penalty(6.5, 5.0, 1.0);
        assert!(inside < outside);
        assert!((inside - 0.0).abs() < EPS);
    }
}
