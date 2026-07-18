// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! S2 B1 composed delegate bridge — production wire for conjunct **Q**.
//!
//! Routes `ConcreteApiCartridge` constitutive + gate evaluation through
//! `umst-cartridge-concrete::gate_route_composed` instead of monolith homogeneous closures.

use serde_json::Value;
use umst_cartridge_concrete::{
    gate_route_composed, g0_probe_atom_state, dissipation_modulus_eta, ComposedGateOutcome,
    MixScalars, D_CLOSURE_ABS_TOL, PSI_CLOSURE_ABS_TOL,
};
use umst_cartridge_continuum::ContinuumAtomRates;

use crate::calibration::Profile;
use crate::homogeneous::MixRow;
use crate::research::contribution::adapter::mix_wire_from_spec_value;

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
    MixScalars {
        key: None,
        w_c,
        temperature_k: f64::from(row.temperature_c) + 273.15,
        fly_ash_pct,
        silica_fume_pct,
        target_age_hours: f64::from(row.age_days) * 24.0,
    }
}

/// Lift rational mix JSON wire into B1 compose [`MixScalars`].
#[must_use]
pub fn mix_json_to_mix_scalars(mix_json: &Value, mix_key: Option<&str>) -> Option<MixScalars> {
    let wire = mix_wire_from_spec_value(mix_json)?;
    Some(MixScalars {
        key: mix_key.map(str::to_string),
        w_c: wire.w_c,
        temperature_k: wire.temperature_k,
        fly_ash_pct: wire.fly_ash_pct.unwrap_or(0.0),
        silica_fume_pct: wire.silica_fume_pct.unwrap_or(0.0),
        target_age_hours: wire.target_age_hours.unwrap_or(28.0 * 24.0),
    })
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
    _profile: &Profile,
    row: &MixRow,
    alpha_dot: f64,
) -> ComposedGateOutcome {
    let mix = mix_row_to_mix_scalars(row);
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

/// Gate admissibility via composed delegate (MCP wire path).
#[must_use]
pub fn gate_admissible_via_compose(_profile: &Profile, mix_json: &Value) -> bool {
    let Some(mix) = mix_json_to_mix_scalars(mix_json, None) else {
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

/// Scalar state fields from composed constitutive ledger.
#[must_use]
pub fn scalar_fields_from_composed(
    profile: &Profile,
    row: &MixRow,
    alpha_dot: f64,
) -> (f64, f64, f64) {
    let outcome = gate_route_via_compose(profile, row, alpha_dot);
    let psi_j_per_m3 = outcome.constitutive.psi_total();
    let density = concrete_bulk_density_kg_m3(row);
    let eta = dissipation_modulus_eta();
    (psi_j_per_m3, density, eta)
}

#[must_use]
fn concrete_bulk_density_kg_m3(row: &MixRow) -> f64 {
    let binder = f64::from(row.cement_kg_m3 + row.slag_kg_m3 + row.fly_ash_kg_m3);
    let water = f64::from(row.water_kg_m3);
    let sp = f64::from(row.superplasticizer_kg_m3);
    binder + water + sp
}
