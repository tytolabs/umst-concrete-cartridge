// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
// Virtual extrusion proxy: τ₀ vs pump window (literature band).

/// Literature extrudable τ₀ window [Pa] (Roussel/Coussot — see experiment AGENT_TENSOR_MAPPING).
/// formal_anchor: literature://roussel-2018-buildability-window
/// formal_status: Literature
/// formal_citation: "Roussel (2018) Cem. Concr. Res. 112, 76 — lower τ₀ bound"
/// formal_form: "τ₀ ≥ 180 Pa"
pub const EXTRUDABLE_TAU_LO_PA: f32 = 180.0;
/// formal_anchor: literature://roussel-2018-buildability-window
/// formal_status: Literature
/// formal_citation: "Roussel (2018) Cem. Concr. Res. 112, 76 — upper τ₀ bound"
/// formal_form: "τ₀ ≤ 360 Pa"
pub const EXTRUDABLE_TAU_HI_PA: f32 = 360.0;

/// Unit score when τ₀ lies inside the literature extrudable band.
/// formal_anchor: literature://roussel-2018-buildability-window
/// formal_status: Literature
/// formal_citation: "Roussel (2018) Cem. Concr. Res. 112, 76 — printable τ₀ band"
/// formal_form: "0.5 when τ₀ ∈ band else 0"
#[must_use]
pub fn extrusion_band_score(tau_y_pa: f32) -> f32 {
    if tau_y_pa >= EXTRUDABLE_TAU_LO_PA && tau_y_pa <= EXTRUDABLE_TAU_HI_PA {
        0.5
    } else {
        0.0
    }
}

/// Tensor extrudability contribution (half weight, clamped).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Headline scalar from pipeline printability stage.
#[must_use]
pub fn extrusion_tensor_score(extrudability_tensor: f32) -> f32 {
    if extrudability_tensor.is_finite() {
        0.5 * extrudability_tensor.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Score in [0, 1] peaked inside the printable window.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Combines τ₀ band with tensor extrudability headline.
#[must_use]
pub fn virtual_extrusion_score(tau_y_pa: f32, extrudability_tensor: f32) -> f32 {
    (extrusion_band_score(tau_y_pa) + extrusion_tensor_score(extrudability_tensor)).clamp(0.0, 1.0)
}
