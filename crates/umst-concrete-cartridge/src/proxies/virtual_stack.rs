// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
// Virtual Roussel buildability proxy (stack height vs yield stress).

/// Target wet column height [m] for virtual stack test.
/// formal_anchor: literature://roussel-2016-buildability
/// formal_status: Literature
/// formal_citation: "Roussel et al. (2016) Cem. Concr. Res. 85 — buildability height"
/// formal_form: "H = 0.012 m fresh layer (mortar printable proxy; full 0.30 m column uses tests/printability.rs)"
pub const VIRTUAL_STACK_HEIGHT_M: f32 = 0.012;

/// Fresh bulk density [kg/m³] when not measured (S1 gap documented).
/// formal_anchor: literature://aci-211-density-nominal
/// formal_status: Literature
/// formal_citation: "ACI 211.1 nominal fresh density for mortar/concrete"
/// formal_form: "ρ = 2300 kg/m³ surrogate"
pub const VIRTUAL_STACK_RHO_KG_M3: f32 = 2300.0;

const G: f32 = 9.81;

/// Roussel minimum yield stress for buildability at height H.
/// formal_anchor: literature://roussel-2016-buildability
/// formal_status: Literature
/// formal_citation: "Roussel et al. (2016) Cem. Concr. Res. 85 — τ_min = ρ g H / √3"
/// formal_form: "τ_min(H, ρ)"
#[must_use]
pub fn roussel_min_yield_pa(rho_kg_m3: f32, height_m: f32) -> f32 {
    rho_kg_m3 * G * height_m / 3.0_f32.sqrt()
}

/// Score in [0, 1]: 1 = buildable, 0 = collapses. Higher τ₀ → higher score.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Normalized score from [`roussel_min_yield_pa`]; not a standalone Lean witness.
#[must_use]
pub fn virtual_stack_score(tau_y_pa: f32) -> f32 {
    let tau_min = roussel_min_yield_pa(VIRTUAL_STACK_RHO_KG_M3, VIRTUAL_STACK_HEIGHT_M);
    if !tau_y_pa.is_finite() || tau_y_pa <= 0.0 {
        return 0.0;
    }
    ((tau_y_pa - tau_min) / tau_min.max(1.0)).clamp(0.0, 1.0)
}

/// Score when τ₀ is inside the literature printable band (mortar proxy path).
/// formal_anchor: literature://roussel-2018-buildability-window
/// formal_status: Literature
/// formal_citation: "Roussel (2018) Cem. Concr. Res. 112, 76 — printable τ₀ band"
/// formal_form: "τ₀ ∈ [180, 360] Pa → score 1.0 else Roussel stack proxy"
/// formal_envelope: "tests/virtual_proxies.rs"
#[must_use]
pub fn virtual_stack_score_in_band(tau_y_pa: f32) -> f32 {
    if tau_y_pa >= crate::pipeline::PRINTABLE_TAU_LO
        && tau_y_pa <= crate::pipeline::PRINTABLE_TAU_HI
    {
        return 1.0;
    }
    virtual_stack_score(tau_y_pa)
}
