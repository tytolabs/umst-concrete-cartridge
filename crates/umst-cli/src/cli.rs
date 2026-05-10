// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! CLI transport: JSON (`serde_json`), error aggregation, and optimisation driver.
//! Physical wire types and [`predict`](umst_concrete_cartridge::facade::predict) live in **`umst_concrete_cartridge::facade`**.

use serde_json::Value;
use std::convert::TryFrom;
use std::fmt;

use umst_concrete_cartridge::calibration::Profile;
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Thin transport re-export of `umst_concrete_cartridge::facade`; authoritative formal blocks live on facade definitions.
pub use umst_concrete_cartridge::facade::{
    certify_profile_chain, mix_spec_wire_out, predict, predict_with_options, prediction_wire_v1,
    prediction_wire_v2, tensor_element_at, CertifyChain, FacadeBackend, FacadeError,
    HomogeneousCompareWire, MixSpec, MixSpecError, MixSpecWire, MixSpecWireOut, PredictBundle,
    PredictOptions, PredictionWireV1, PredictionWireV2, PredictionWireVersion,
};
use umst_concrete_cartridge::facade::{
    RESULT_SCHEMA_VERSION_V1 as FACADE_RESULT_V1, RESULT_SCHEMA_VERSION_V2 as FACADE_RESULT_V2,
};

/// formal_anchor: literature://wire-schema-result-v1
/// formal_status: Literature
/// formal_citation: "UMST concrete cartridge JSON wire schema tag (`result.v1`)"
/// formal_form: "`result.v1` — version tag for deprecated prediction JSON envelope"
pub const RESULT_SCHEMA_VERSION_V1: &str = FACADE_RESULT_V1;

/// formal_anchor: literature://wire-schema-result-v2
/// formal_status: Literature
/// formal_citation: "UMST concrete cartridge JSON wire schema tag (`result.v2`)"
/// formal_form: "`result.v2` — version tag for current prediction JSON envelope"
pub const RESULT_SCHEMA_VERSION_V2: &str = FACADE_RESULT_V2;

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: CLI re-export; ndarray backend alias for historical `CliBackend` name.
pub type CliBackend = FacadeBackend;

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Binary-boundary error aggregation; extends [`FacadeError`] with JSON/optimise glue.
#[derive(Debug)]
pub enum CliError {
    Facade(FacadeError),
    Json(serde_json::Error),
    UnsupportedOptimizeTarget(String),
    InvalidOptimizeTarget,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Facade(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::UnsupportedOptimizeTarget(t) => {
                write!(f, "optimization target `{t}` is not supported")
            }
            Self::InvalidOptimizeTarget => {
                write!(
                    f,
                    "could not parse optimization target (expected FIELD=VALUE)"
                )
            }
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Facade(e) => Some(e),
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<FacadeError> for CliError {
    fn from(e: FacadeError) -> Self {
        Self::Facade(e)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<umst_concrete_cartridge::calibration::CalibrationError> for CliError {
    fn from(e: umst_concrete_cartridge::calibration::CalibrationError) -> Self {
        Self::Facade(FacadeError::Calibration(e))
    }
}

/// Parse mix design from `serde_json::Value` (CLI / MCP transport).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: JSON boundary helper; validation uses [`MixSpecWire`] + [`MixSpec::try_from`].
pub fn mix_spec_from_json_value(value: Value) -> Result<MixSpec, CliError> {
    let wire: MixSpecWire = serde_json::from_value(value)?;
    Ok(MixSpec::try_from(wire)?)
}

impl From<MixSpecError> for CliError {
    fn from(e: MixSpecError) -> Self {
        CliError::Facade(FacadeError::MixSpec(e))
    }
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: JSON-serialise glue; no physical claim.
pub fn serialize_prediction(
    bundle: &PredictBundle,
    version: PredictionWireVersion,
) -> Result<Value, CliError> {
    match version {
        PredictionWireVersion::V1 => {
            let wire = prediction_wire_v1(bundle)?;
            Ok(serde_json::to_value(&wire)?)
        }
        PredictionWireVersion::V2 => {
            let wire = prediction_wire_v2(bundle)?;
            let mut out = serde_json::to_value(&wire)?;
            let pipeline_val = serde_json::to_value(&bundle.physics_pipeline)?;
            if let Value::Object(ref mut map) = out {
                map.insert("physics_pipeline".into(), pipeline_val);
                if let Some(h) = bundle.homogeneous_compare.clone() {
                    map.insert("homogeneous_compare".into(), serde_json::to_value(&h)?);
                }
            }
            Ok(out)
        }
    }
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: JSON Value view of certify chain; structural wrapper over [`certify_profile_chain`].
#[must_use]
pub fn certify_profile_json(profile: &Profile) -> Value {
    let chain = certify_profile_chain(profile);
    serde_json::to_value(&chain).unwrap_or(Value::Null)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: JSON-serialise glue.
pub fn serialize_mix_spec(spec: &MixSpec) -> Result<Value, CliError> {
    serde_json::to_value(mix_spec_wire_out(spec)).map_err(Into::into)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Exhaustive enum of optimisation targets for the CLI bisection driver.
pub enum OptimizeField {
    CompressiveStrengthMpa,
}

impl TryFrom<&str> for OptimizeField {
    type Error = CliError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "compressive_strength_mpa" => Ok(Self::CompressiveStrengthMpa),
            _ => Err(CliError::UnsupportedOptimizeTarget(value.to_string())),
        }
    }
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: String-parse glue for `FIELD=VALUE` optimise CLI syntax.
pub fn parse_optimize_target(raw: &str) -> Result<(OptimizeField, f64), CliError> {
    let (k, v) = raw.split_once('=').ok_or(CliError::InvalidOptimizeTarget)?;
    let field = OptimizeField::try_from(k.trim())?;
    let val: f64 = v
        .trim()
        .parse()
        .map_err(|_| CliError::InvalidOptimizeTarget)?;
    Ok((field, val))
}

/// formal_anchor: empirical://datasets/cli-optimize-wc-bisection.v1.csv
/// formal_status: Empirical
/// formal_dataset: "cli optimize_mix bisection grid"
/// formal_citation: "Driver-only inverse search on w/c holding other mix fields fixed"
/// formal_envelope: "tests/cli/optimize.rs"
pub fn optimize_mix(
    profile: &Profile,
    base: &MixSpec,
    field: OptimizeField,
    target: f64,
    steps: usize,
) -> Result<MixSpec, CliError> {
    match field {
        OptimizeField::CompressiveStrengthMpa => {
            optimize_w_c_for_strength(profile, base, target as f32, steps)
        }
    }
}

fn optimize_w_c_for_strength(
    profile: &Profile,
    base: &MixSpec,
    target_fc: f32,
    steps: usize,
) -> Result<MixSpec, CliError> {
    use umst_concrete_cartridge::facade::WaterCementRatio;

    let mut lo = 0.20_f32;
    let mut hi = 0.80_f32;
    let mut best = base.clone();
    let mut best_err = f32::MAX;

    for _ in 0..steps.max(1) {
        let mid = (lo + hi) * 0.5_f32;
        let cand = mix_with_w_c(
            base,
            WaterCementRatio::try_from(f64::from(mid)).map_err(FacadeError::MixSpec)?,
        )?;
        let bundle = predict(profile, &cand)?;
        let fc = tensor_element_at(bundle.physical.free_energy.clone(), 0, 0)?;
        let err = (fc - target_fc).abs();
        if err < best_err {
            best_err = err;
            best = cand;
        }
        if fc > target_fc {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    debug_assert!(best_err.is_finite());
    Ok(best)
}

fn mix_with_w_c(
    base: &MixSpec,
    w_c: umst_concrete_cartridge::facade::WaterCementRatio,
) -> Result<MixSpec, CliError> {
    Ok(MixSpec {
        w_c,
        temperature_k: base.temperature_k,
        superplasticiser_pct: base.superplasticiser_pct,
        silica_fume_pct: base.silica_fume_pct,
        fly_ash_pct: base.fly_ash_pct,
        aggregate_volume_fraction: base.aggregate_volume_fraction,
        target_age_hours: base.target_age_hours,
        profile_name: base.profile_name.clone(),
    })
}

#[cfg(test)]
mod wire_roundtrip_tests {
    use super::*;

    #[test]
    fn prediction_json_v2_has_calibration_fields() -> Result<(), CliError> {
        let v = serde_json::json!({"w_c": 0.40, "temperature_k": 293.15});
        let mut spec = mix_spec_from_json_value(v)?;
        spec.profile_name = "default".to_string();
        let profile = Profile::load_bundled("default")?;
        let bundle = predict(&profile, &spec)?;
        let out = serialize_prediction(&bundle, PredictionWireVersion::V2)?;
        assert_eq!(out["schema_version"], RESULT_SCHEMA_VERSION_V2);
        assert!(out["warnings"].is_array());
        assert_eq!(out["calibration_profile"], "default");
        Ok(())
    }
}
