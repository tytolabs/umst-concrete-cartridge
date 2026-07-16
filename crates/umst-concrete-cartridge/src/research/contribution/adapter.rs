// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Phase 0c — **Adapter** boundary: mix JSON wire → typed [`MixSpec`] lift.
//!
//! Parsing rationals and building facade wires lives here — **not** in the gate predicate
//! or explain/remediation infra (blueprint §7 0c · `NEW_REPOS_BUILD_SPEC` §E.4).
//!
//! Future A1/M1: this module lifts toward `umst-cartridge-api` `ConstitutiveResponse`
//! via [`crate::api_consumer::ConcreteApiCartridge`].

use crate::calibration::Profile;
use crate::facade::{MixSpec, MixSpecWire};
use serde_json::Value;

/// Rational wire `n/d` → `f64` for mix_spec parsing.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Wire decode only; physical units validated downstream on `MixSpec`.
#[must_use]
pub fn rational_to_f64(s: &str) -> Option<f64> {
    let (n, d) = s.split_once('/')?;
    let nf = n.parse::<f64>().ok()?;
    let df = d.parse::<f64>().ok()?;
    if df == 0.0 {
        return None;
    }
    Some(nf / df)
}

/// Pure: rational wire field `n/d` → `f64`.
#[must_use]
pub(crate) fn field_as_rational(obj: &Value, key: &str) -> Option<f64> {
    obj.get(key)
        .and_then(|v| v.as_str())
        .and_then(rational_to_f64)
}

/// Parse mix_spec JSON rationals into [`MixSpecWire`].
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Serde routing to facade wire; gate on `MixSpec::try_from`.
#[must_use]
pub fn mix_wire_from_spec_value(v: &Value) -> Option<MixSpecWire> {
    Some(MixSpecWire {
        w_c: field_as_rational(v, "w_c")?,
        temperature_k: field_as_rational(v, "temperature_k")?,
        superplasticiser_pct: v
            .get("superplasticiser_pct")
            .and_then(|x| x.as_str())
            .and_then(rational_to_f64),
        silica_fume_pct: v
            .get("silica_fume_pct")
            .and_then(|x| x.as_str())
            .and_then(rational_to_f64),
        fly_ash_pct: v
            .get("fly_ash_pct")
            .and_then(|x| x.as_str())
            .and_then(rational_to_f64),
        aggregate_volume_fraction: v
            .get("aggregate_volume_fraction")
            .and_then(|x| x.as_str())
            .and_then(rational_to_f64),
        target_age_hours: v
            .get("target_age_hours")
            .and_then(|x| x.as_str())
            .and_then(rational_to_f64),
    })
}

/// Adapter lift: mix JSON + profile → calibrated [`MixSpec`] (or parse failure).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Single adapter entry for gate + explain paths; no admissibility here.
#[must_use]
pub fn mix_spec_from_json(profile: &Profile, mix_json: &Value) -> Option<MixSpec> {
    let wire = mix_wire_from_spec_value(mix_json)?;
    let mut spec = MixSpec::try_from(wire).ok()?;
    spec.profile_name = profile.bundle_id.clone();
    Some(spec)
}
