// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Phase 0d — canonical gate routing for concrete admissibility (blueprint §7 0d).
//!
//! Routes regime envelope + manifold `core_gate` ∧ `material_gate` composed path via
//! [`umst_manifold::gate::canonical_thermo_transition_admissible`] — **not** the
//! `predict_with_options` physics composite.

use crate::calibration::{self as calib, Profile};
use crate::facade::MixSpec;
use crate::homogeneous::{self as homog, mix_row_from_scalar_spec};

use umst_manifold::gate::verdict::GateRejectReason;
#[cfg(feature = "manifest-bridge")]
use umst_manifold::gate::{
    transition_outcome, ThermodynamicStateSnapshot, TRANSITION_TOLERANCE,
};
#[cfg(feature = "manifest-bridge")]
use umst_manifold::gate::verdict::ConjunctVerdict;

/// Thin newtype over manifold P2 [`GateRejectReason`] at manifest-bridge boundary.
/// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
/// formal_anchor_rationale: Cartridge thermo reject carrier wrapping P2 `GateRejectReason`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThermoReject(pub GateRejectReason);

impl ThermoReject {
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Accessor for underlying P2 reject reason.
    #[must_use]
    pub fn reason(self) -> GateRejectReason {
        self.0
    }
}

/// Canonical thermodynamic admissibility for a calibrated [`MixSpec`].
///
/// Checks bundled-profile regime coverage, then delegates to manifold composed gate.
/// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
#[must_use]
pub fn thermodynamic_admissible(profile: &Profile, spec: &MixSpec) -> bool {
    thermodynamic_verdict(profile, spec).is_ok()
}

/// Thermodynamic leg verdict — internal enum carrier; bool wire unchanged via [`thermodynamic_admissible`].
/// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
/// formal_anchor_rationale: Enum path for thermodynamic leg; MCP wire frozen on bool shim.
#[must_use]
pub fn thermodynamic_verdict(profile: &Profile, spec: &MixSpec) -> Result<(), ThermoReject> {
    #[cfg(feature = "manifest-bridge")]
    {
        thermodynamic_verdict_manifest_bridge(profile, spec)
    }
    #[cfg(not(feature = "manifest-bridge"))]
    {
        let _ = (profile, spec);
        Err(ThermoReject(GateRejectReason::MalformedInput))
    }
}

#[cfg(feature = "manifest-bridge")]
fn thermodynamic_verdict_manifest_bridge(
    profile: &Profile,
    spec: &MixSpec,
) -> Result<(), ThermoReject> {
    if !calib::any_bundled_profile_covers_scalars(
        spec.w_c.value(),
        spec.temperature_k.value(),
        spec.target_age_hours,
        spec.fly_ash_pct,
        spec.silica_fume_pct,
    ) {
        return Err(ThermoReject(GateRejectReason::RegimeEnvelope));
    }

    let row = mix_row_from_scalar_spec(
        profile,
        spec.w_c.value(),
        spec.superplasticiser_pct,
        spec.fly_ash_pct,
        spec.silica_fume_pct,
        spec.aggregate_volume_fraction,
        spec.target_age_hours,
        spec.temperature_k.value(),
    );

    transition_verdict_for_row(profile, &row)
}

/// Manifold composed gate verdict for an already-built [`homog::MixRow`].
/// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
#[cfg(feature = "manifest-bridge")]
#[must_use]
pub fn transition_verdict_for_row(
    profile: &Profile,
    row: &homog::MixRow,
) -> Result<(), ThermoReject> {
    let Ok((w_c_eff, alpha, temp_c)) = homog::mix_hydration_state(profile, row) else {
        return Err(ThermoReject(GateRejectReason::MalformedInput));
    };
    let temp_k = f64::from(temp_c) + 273.15;
    let w_c = f64::from(w_c_eff);
    let s_intrinsic = f64::from(profile.powers.s_intrinsic);
    let old = ThermodynamicStateSnapshot::from_mix_calibrated(w_c, 0.0, temp_k, s_intrinsic);
    let new =
        ThermodynamicStateSnapshot::from_mix_calibrated(w_c, f64::from(alpha), temp_k, s_intrinsic);
    let dt_s = f64::from((row.age_days * 24.0 * 3600.0).max(1.0));
    let outcome = transition_outcome(&old, &new, dt_s, TRANSITION_TOLERANCE);
    transition_verdict_from_outcome(outcome)
}

/// Manifold composed gate for an already-built [`homog::MixRow`].
/// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
#[cfg(feature = "manifest-bridge")]
#[must_use]
pub fn transition_admissible_for_row(profile: &Profile, row: &homog::MixRow) -> bool {
    transition_verdict_for_row(profile, row).is_ok()
}

#[cfg(feature = "manifest-bridge")]
fn transition_verdict_from_outcome(
    outcome: umst_manifold::gate::ThermodynamicTransitionOutcome,
) -> Result<(), ThermoReject> {
    if outcome.accepted {
        Ok(())
    } else {
        Err(match outcome.verdict {
            ConjunctVerdict::Rejected(reason) => ThermoReject(reason),
            ConjunctVerdict::Accepted => ThermoReject(GateRejectReason::MalformedInput),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facade::MixSpec;

    fn default_profile() -> Profile {
        Profile::load_bundled("default").expect("default profile")
    }

    #[test]
    #[cfg(feature = "manifest-bridge")]
    fn canonical_gate_accepts_default_rational_mix() {
        let profile = default_profile();
        let spec = MixSpec::try_from(crate::facade::MixSpecWire {
            w_c: 0.45,
            temperature_k: 293.15,
            superplasticiser_pct: None,
            silica_fume_pct: None,
            fly_ash_pct: None,
            aggregate_volume_fraction: Some(0.7),
            target_age_hours: None,
        })
        .expect("wire");
        assert!(thermodynamic_admissible(&profile, &spec));
        assert!(thermodynamic_verdict(&profile, &spec).is_ok());
    }

    #[test]
    #[cfg(feature = "manifest-bridge")]
    fn transition_admissible_matches_verdict_path() {
        let profile = default_profile();
        let spec = MixSpec::try_from(crate::facade::MixSpecWire {
            w_c: 0.45,
            temperature_k: 293.15,
            superplasticiser_pct: None,
            silica_fume_pct: None,
            fly_ash_pct: None,
            aggregate_volume_fraction: Some(0.7),
            target_age_hours: None,
        })
        .expect("wire");
        let row = mix_row_from_scalar_spec(
            &profile,
            spec.w_c.value(),
            spec.superplasticiser_pct,
            spec.fly_ash_pct,
            spec.silica_fume_pct,
            spec.aggregate_volume_fraction,
            spec.target_age_hours,
            spec.temperature_k.value(),
        );
        assert_eq!(
            transition_admissible_for_row(&profile, &row),
            transition_verdict_for_row(&profile, &row).is_ok()
        );
        assert_eq!(
            thermodynamic_admissible(&profile, &spec),
            thermodynamic_verdict(&profile, &spec).is_ok()
        );
    }
}
