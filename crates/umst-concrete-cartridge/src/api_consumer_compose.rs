// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! S2 B1 composed delegate bridge — production wire for conjunct **Q**.
//!
//! Routes `ConcreteApiCartridge` constitutive + gate evaluation through
//! `umst-cartridge-concrete::gate_route_composed` instead of monolith homogeneous closures.

use crate::concrete_bridge::{
    gate_route_composed, g0_consumer_history_binding, g0_probe_atom_state,
    try_consumer_gate_route_with_history, ComposedGateOutcome, ContinuumAtomRates,
    ContinuumAtomStateWithHistory, ContinuumPhysicsError, MixScalars, D_CLOSURE_ABS_TOL,
    PSI_CLOSURE_ABS_TOL,
};

use crate::calibration::Profile;
use crate::homogeneous::MixRow;
use umst_cartridge_concrete::{ComposeProfilePins, DEFAULT_TOTAL_BINDER_KG_M3};

/// Lift monolith [`Profile`] into consumer compose profile pins (regime + Powers SSOT).
#[must_use]
fn compose_profile_pins_from(profile: &Profile) -> ComposeProfilePins {
    ComposeProfilePins {
        s_intrinsic_mpa: profile.powers.s_intrinsic,
        w_c_max: profile.regime.w_c_max,
        total_binder_kg_m3: DEFAULT_TOTAL_BINDER_KG_M3,
    }
}

#[cfg(feature = "agent-layer")]
use serde_json::Value;
#[cfg(feature = "agent-layer")]
use crate::research::contribution::adapter::mix_wire_from_spec_value;

/// S6 — consumer [`MixScalars`] SSOT for `(w_c_eff, α, T°C)`.
///
/// Replaces monolith profile-aware `homogeneous::mix_hydration_state` body @
/// `g_spawn_i_s6_mh_2054`; callers keep the homogeneous signature.
#[must_use]
pub fn mix_hydration_scalars_from_row(row: &MixRow) -> (f32, f32, f32) {
    let mix = mix_row_to_mix_scalars(row);
    (
        mix.effective_w_c() as f32,
        mix.hydration_alpha() as f32,
        row.temperature_c,
    )
}

/// Lift MCP [`MixRow`] into B1 compose [`MixScalars`] with profile `s_intrinsic`.
#[must_use]
pub fn mix_row_to_mix_scalars_for_profile(profile: &Profile, row: &MixRow) -> MixScalars {
    mix_row_to_mix_scalars(row).with_compose_profile_pins(compose_profile_pins_from(profile))
}

/// Lift MCP [`MixRow`] into B1 compose [`MixScalars`].
#[must_use]
pub fn mix_row_to_mix_scalars(row: &MixRow) -> MixScalars {
    let binder = f64::from(row.cement_kg_m3 + row.slag_kg_m3 + row.fly_ash_kg_m3);
    let water = f64::from(row.water_kg_m3);
    let w_c = if binder > 0.0 { water / binder } else { 0.0 };
    let fly_ash_pct = if binder > 0.0 {
        f64::from(row.fly_ash_kg_m3) / binder * 100.0
    } else {
        0.0
    };
    let silica_fume_pct = if binder > 0.0 {
        f64::from(row.slag_kg_m3) / binder * 100.0
    } else {
        0.0
    };
    let superplasticiser_pct = if binder > 0.0 {
        f64::from(row.superplasticizer_kg_m3) / binder * 100.0
    } else {
        0.0
    };
    MixScalars {
        key: None,
        w_c,
        temperature_k: f64::from(row.temperature_c) + 273.15,
        fly_ash_pct,
        silica_fume_pct,
        superplasticiser_pct,
        target_age_hours: f64::from(row.age_days) * 24.0,
        s_intrinsic_mpa: None,
        w_c_max: None,
        total_binder_kg_m3: None,
        aggregate_volume_fraction: None,
    }
}

/// Lift rational mix JSON wire into B1 compose [`MixScalars`].
#[cfg(feature = "agent-layer")]
#[must_use]
pub fn mix_json_to_mix_scalars(mix_json: &Value, mix_key: Option<&str>) -> Option<MixScalars> {
    let wire = mix_wire_from_spec_value(mix_json)?;
    Some(MixScalars {
        key: mix_key.map(str::to_string),
        w_c: wire.w_c,
        temperature_k: wire.temperature_k,
        fly_ash_pct: wire.fly_ash_pct.unwrap_or(0.0),
        silica_fume_pct: wire.silica_fume_pct.unwrap_or(0.0),
        superplasticiser_pct: wire.superplasticiser_pct.unwrap_or(0.0),
        target_age_hours: wire.target_age_hours.unwrap_or(28.0 * 24.0),
        s_intrinsic_mpa: None,
        w_c_max: None,
        total_binder_kg_m3: None,
        aggregate_volume_fraction: wire.aggregate_volume_fraction,
    })
}

/// Lift rational mix JSON wire into B1 compose [`MixScalars`] with profile `s_intrinsic`.
#[cfg(feature = "agent-layer")]
#[must_use]
pub fn mix_json_to_mix_scalars_for_profile(
    profile: &Profile,
    mix_json: &Value,
    mix_key: Option<&str>,
) -> Option<MixScalars> {
    mix_json_to_mix_scalars(mix_json, mix_key)
        .map(|mix| mix.with_compose_profile_pins(compose_profile_pins_from(profile)))
}

/// B1 kinematic rates from reaction scalar α̇ — reaction-only routes keep ε̇=ḋ=0.
#[must_use]
pub fn continuum_rates_from_alpha_dot(alpha_dot: f64) -> ContinuumAtomRates {
    if alpha_dot == 0.0 {
        ContinuumAtomRates::PASSIVE
    } else {
        ContinuumAtomRates::PASSIVE
    }
}

/// Production delegate — composed gate route at G0 probe pin.
#[must_use]
pub fn gate_route_via_compose(
    profile: &Profile,
    row: &MixRow,
    alpha_dot: f64,
) -> ComposedGateOutcome {
    let mix = mix_row_to_mix_scalars_for_profile(profile, row);
    let rates = continuum_rates_from_alpha_dot(alpha_dot);
    gate_route_composed(
        &mix,
        g0_probe_atom_state(),
        rates,
        alpha_dot,
        PSI_CLOSURE_ABS_TOL,
        D_CLOSURE_ABS_TOL,
    )
}

/// Production delegate with irreversibility sidecar — routes via `api_consumer_history_prep`.
#[must_use]
pub fn gate_route_via_compose_with_history(
    _profile: &Profile,
    row: &MixRow,
    alpha_dot: f64,
    dt: f64,
) -> (ComposedGateOutcome, ContinuumAtomStateWithHistory) {
    try_gate_route_via_compose_with_history(_profile, row, alpha_dot, dt)
        .expect("G0 passive pin history gate")
}

/// Strict production delegate with history sidecar — `history_prep` + delegate `with_history`.
pub fn try_gate_route_via_compose_with_history(
    profile: &Profile,
    row: &MixRow,
    alpha_dot: f64,
    dt: f64,
) -> Result<(ComposedGateOutcome, ContinuumAtomStateWithHistory), ContinuumPhysicsError> {
    let mix = mix_row_to_mix_scalars_for_profile(profile, row);
    let rates = continuum_rates_from_alpha_dot(alpha_dot);
    let binding = g0_consumer_history_binding();
    try_consumer_gate_route_with_history(
        &mix,
        binding,
        rates,
        alpha_dot,
        dt,
        PSI_CLOSURE_ABS_TOL,
        D_CLOSURE_ABS_TOL,
    )
}

/// Gate admissibility via composed delegate (MCP wire path).
#[cfg(feature = "agent-layer")]
#[must_use]
pub fn gate_admissible_via_compose(profile: &Profile, mix_json: &Value) -> bool {
    let Some(mix) = mix_json_to_mix_scalars_for_profile(profile, mix_json, None) else {
        return false;
    };
    let outcome = gate_route_composed(
        &mix,
        g0_probe_atom_state(),
        ContinuumAtomRates::PASSIVE,
        0.0,
        PSI_CLOSURE_ABS_TOL,
        D_CLOSURE_ABS_TOL,
    );
    outcome.route.admissible
}

/// M1 ψ oracle [J/m³] — SSOT via consumer [`MixScalars::psi_m1`].
///
/// T2-S6 batch B2 (`g_spawn_i_s6_psi_2054`): monolith `psi_m1_oracle` cfg arm retired;
/// production scalar ψ slot reads consumer compose oracle only.
#[must_use]
pub fn psi_m1_oracle_from_row(row: &MixRow) -> f64 {
    mix_row_to_mix_scalars(row).psi_m1()
}

/// Powers compressive strength [MPa] — SSOT via consumer [`MixScalars::fc_mpa`].
///
/// T2-S6 batch B2 (`g_spawn_i_s6_psi_2054`): default-profile homogeneous closure
/// delegates here; dataset-specific modifiers remain in `homogeneous.rs`.
#[must_use]
pub fn compressive_strength_mpa_from_row(row: &MixRow) -> f64 {
    mix_row_to_mix_scalars(row).fc_mpa()
}

/// Powers f_c oracle [MPa] — alias for [`compressive_strength_mpa_from_row`].
///
/// T2-S6 FULL dup purge (`g_spawn_i_s6_pfc_2054`): `powers_compressive_strength_mpa`
/// default-profile hot path reads consumer compose SSOT only.
#[must_use]
pub fn powers_compressive_strength_mpa_from_row(row: &MixRow) -> f64 {
    compressive_strength_mpa_from_row(row)
}

/// M1 dissipation modulus η [J·s/m³] — consumer SSOT for `dissipation_modulus_from_profile`.
///
/// T2-S6 PARTIAL tail (`g_spawn_i_s6_eta_2054`): profile `s_intrinsic` threaded from
/// calibration [`Profile`]; enthalpy routes `umst-chem` SSOT via consumer compose (`g_spawn_i_chemC_2054`).
#[must_use]
pub fn dissipation_modulus_eta_from_profile(profile: &Profile) -> f64 {
    umst_cartridge_concrete::dissipation_modulus_eta_from_profile(f64::from(
        profile.powers.s_intrinsic,
    ))
}

/// Scalar state fields from composed constitutive ledger.
#[must_use]
pub fn scalar_fields_from_composed(
    profile: &Profile,
    row: &MixRow,
    _alpha_dot: f64,
) -> (f64, f64, f64) {
    let mix = mix_row_to_mix_scalars_for_profile(profile, row);
    let psi_j_per_m3 = mix.psi_m1();
    let density = concrete_bulk_density_kg_m3(row);
    let eta = mix.dissipation_modulus_eta();
    (psi_j_per_m3, density, eta)
}

#[must_use]
fn concrete_bulk_density_kg_m3(row: &MixRow) -> f64 {
    let binder = f64::from(row.cement_kg_m3 + row.slag_kg_m3 + row.fly_ash_kg_m3);
    let water = f64::from(row.water_kg_m3);
    let sp = f64::from(row.superplasticizer_kg_m3);
    binder + water + sp
}
