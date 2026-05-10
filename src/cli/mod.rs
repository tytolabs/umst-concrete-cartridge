// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Pure CLI core: JSON validation (`MixSpec`), regime checks, homogeneous prediction through [`crate::homogeneous`],
//! and serialization into versioned wire JSON (`result.v1` / `result.v2`).

use crate::calibration::{self as calib, Profile};
use crate::homogeneous::{
    self as homog, constituent_masses_kg_m3, embodied_co2_kg_per_m3, mix_row_from_scalar_spec,
};
use burn::tensor::{Data, Shape, Tensor};
use burn_ndarray::NdArray;
use serde::Serialize;
use serde_json::Value;
use std::convert::TryFrom;
use thiserror::Error;
use umst_manifold::core::traits::PhysicalResult;

/// formal_anchor: lean://umst-formal/Lean/MeasurementCost.lean#zero_info_zero_energy
/// formal_status: Structural
/// formal_axioms: NONE
pub const RESULT_SCHEMA_VERSION_V1: &str = "result.v1";

/// formal_anchor: lean://umst-formal/Lean/MeasurementCost.lean#zero_info_zero_energy
/// formal_status: Structural
/// formal_axioms: NONE
pub const RESULT_SCHEMA_VERSION_V2: &str = "result.v2";

/// formal_anchor: lean://umst-formal/Lean/Naturality.lean#naturalitySquare
/// formal_status: Structural
/// formal_axioms: NONE
pub type CliBackend = NdArray;

/// formal_anchor: lean://umst-formal/Lean/Naturality.lean#gateMaterialAgnostic
/// formal_status: Structural
/// formal_axioms: NONE
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictionWireVersion {
    V1,
    V2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// formal_anchor: NONE
/// formal_status: Library
/// formal_axioms: NONE
/// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
pub struct WaterCementRatio(f32);

impl TryFrom<f64> for WaterCementRatio {
    type Error = MixSpecError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let x = value as f32;
        if !(0.20..=0.80).contains(&x) {
            return Err(MixSpecError::WaterCementRatioOutOfRange(x));
        }
        Ok(Self(x))
    }
}

impl WaterCementRatio {
    #[must_use]
    /// formal_anchor: NONE
    /// formal_status: Library
    /// formal_axioms: NONE
    /// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
    pub fn value(self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// formal_anchor: NONE
/// formal_status: Library
/// formal_axioms: NONE
/// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
pub struct TemperatureK(f32);

impl TryFrom<f64> for TemperatureK {
    type Error = MixSpecError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let x = value as f32;
        if !(273.0..=353.0).contains(&x) {
            return Err(MixSpecError::TemperatureOutOfRange(x));
        }
        Ok(Self(x))
    }
}

impl TemperatureK {
    #[must_use]
    /// formal_anchor: NONE
    /// formal_status: Library
    /// formal_axioms: NONE
    /// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
    pub fn value(self) -> f32 {
        self.0
    }
}

/// formal_anchor: lean://umst-formal/Lean/Gate.lean#Admissible
/// formal_status: Structural
/// formal_axioms: NONE
#[derive(Debug, Clone)]
pub struct MixSpec {
    pub w_c: WaterCementRatio,
    pub temperature_k: TemperatureK,
    pub superplasticiser_pct: f32,
    pub silica_fume_pct: f32,
    pub fly_ash_pct: f32,
    pub aggregate_volume_fraction: f32,
    pub target_age_hours: f32,
    pub profile_name: String,
}

#[derive(Debug, serde::Deserialize)]
struct MixSpecWire {
    w_c: f64,
    temperature_k: f64,
    #[serde(default)]
    superplasticiser_pct: Option<f64>,
    #[serde(default)]
    silica_fume_pct: Option<f64>,
    #[serde(default)]
    fly_ash_pct: Option<f64>,
    #[serde(default)]
    aggregate_volume_fraction: Option<f64>,
    #[serde(default)]
    target_age_hours: Option<f64>,
}

impl TryFrom<Value> for MixSpec {
    type Error = MixSpecError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let wire: MixSpecWire = serde_json::from_value(value).map_err(MixSpecError::Json)?;

        let sp = wire.superplasticiser_pct.unwrap_or(0.0);
        let sf = wire.silica_fume_pct.unwrap_or(0.0);
        let fa = wire.fly_ash_pct.unwrap_or(0.0);
        let agg = wire.aggregate_volume_fraction.unwrap_or(0.65);
        let age_h = wire.target_age_hours.unwrap_or(672.0);

        if !(0.0..=5.0).contains(&(sp as f32)) {
            return Err(MixSpecError::FieldOutOfRange {
                field: "superplasticiser_pct",
            });
        }
        if !(0.0..=15.0).contains(&(sf as f32)) {
            return Err(MixSpecError::FieldOutOfRange {
                field: "silica_fume_pct",
            });
        }
        if !(0.0..=60.0).contains(&(fa as f32)) {
            return Err(MixSpecError::FieldOutOfRange {
                field: "fly_ash_pct",
            });
        }
        if !(0.0..=0.85).contains(&(agg as f32)) {
            return Err(MixSpecError::FieldOutOfRange {
                field: "aggregate_volume_fraction",
            });
        }
        if !(1.0..=87_600.0).contains(&(age_h)) {
            return Err(MixSpecError::FieldOutOfRange {
                field: "target_age_hours",
            });
        }

        Ok(MixSpec {
            w_c: WaterCementRatio::try_from(wire.w_c)?,
            temperature_k: TemperatureK::try_from(wire.temperature_k)?,
            superplasticiser_pct: sp as f32,
            silica_fume_pct: sf as f32,
            fly_ash_pct: fa as f32,
            aggregate_volume_fraction: agg as f32,
            target_age_hours: age_h as f32,
            profile_name: "default".to_string(),
        })
    }
}

#[derive(Debug, Error)]
/// formal_anchor: NONE
/// formal_status: Library
/// formal_axioms: NONE
/// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
pub enum MixSpecError {
    #[error("invalid JSON mix specification: {0}")]
    Json(#[from] serde_json::Error),
    #[error("water-cement ratio {0} outside allowed range [0.20, 0.80]")]
    WaterCementRatioOutOfRange(f32),
    #[error("temperature_k {0} outside allowed range [273, 353] K")]
    TemperatureOutOfRange(f32),
    #[error("field `{field}` outside allowed physical range")]
    FieldOutOfRange { field: &'static str },
}

/// formal_anchor: lean://umst-formal/Lean/Naturality.lean#naturalitySquare
/// formal_status: Structural
/// formal_axioms: NONE
#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    MixSpec(#[from] MixSpecError),
    #[error("tensor pipeline: {0}")]
    Tensor(&'static str),
    #[error("optimization target `{0}` is not supported")]
    UnsupportedOptimizeTarget(String),
    #[error("could not parse optimization target (expected FIELD=VALUE)")]
    InvalidOptimizeTarget,
    #[error(transparent)]
    Homogeneous(#[from] homog::HomogeneousError),
    #[error(transparent)]
    Calibration(#[from] calib::CalibrationError),
    #[error("mix design lies outside all bundled calibration regimes")]
    OutsideAllRegimes,
}

#[derive(Serialize)]
struct PredictionWireV1 {
    compressive_strength_mpa: f64,
    yield_stress_pa: f64,
    degree_of_hydration: f64,
    gwp_kg_co2_eq_per_m3: f64,
    safety_margin: f64,
    schema_version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    deprecation: Option<&'static str>,
}

#[derive(Serialize)]
struct PredictionWireV2 {
    compressive_strength_mpa: f64,
    yield_stress_pa: f64,
    degree_of_hydration: f64,
    gwp_kg_co2_eq_per_m3: f64,
    safety_margin: f64,
    calibration_profile: String,
    calibration_model: String,
    formal_anchor: String,
    warnings: Vec<String>,
    schema_version: &'static str,
}

/// formal_anchor: lean://umst-formal/Lean/Naturality.lean#gateMaterialAgnostic
/// formal_status: Structural
/// formal_axioms: NONE
pub struct PredictBundle {
    pub physical: PhysicalResult<CliBackend>,
    pub warnings: Vec<String>,
    pub calibration_profile: String,
    pub calibration_model: String,
    pub formal_anchor: String,
}

fn model_kind_wire(profile: &Profile) -> String {
    match profile.model_section.kind {
        calib::ModelKind::PowersGelSpace => "powers_gel_space".to_string(),
        calib::ModelKind::JenningsGelSpace => "jennings_gel_space".to_string(),
    }
}

fn profile_formal_anchor_uri(profile: &Profile) -> String {
    profile
        .provenance
        .formal
        .as_ref()
        .map(|f| f.anchor.clone())
        .unwrap_or_else(|| "lean://NONE".to_string())
}

/// formal_anchor: lean://umst-formal/Lean/Powers.lean#powers_monotone
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
pub fn predict(profile: &Profile, spec: &MixSpec) -> Result<PredictBundle, CliError> {
    if !calib::any_bundled_profile_covers_scalars(
        spec.w_c.value(),
        spec.temperature_k.value(),
        spec.target_age_hours,
        spec.fly_ash_pct,
        spec.silica_fume_pct,
    ) {
        return Err(CliError::OutsideAllRegimes);
    }

    let violations = profile.regime_check_scalars(
        spec.w_c.value(),
        spec.temperature_k.value(),
        spec.target_age_hours,
        spec.fly_ash_pct,
        spec.silica_fume_pct,
    );
    let warnings: Vec<String> = violations.iter().map(|v| v.to_string()).collect();

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

    let alpha = homog::degree_of_hydration_alpha(profile, &row)?;
    let fc = homog::compressive_strength_mpa(profile, &row)?;
    let tau = homog::yield_stress_pa(
        profile,
        spec.w_c.value(),
        spec.superplasticiser_pct,
        spec.aggregate_volume_fraction,
    );
    let (cement, scm, agg, water) = constituent_masses_kg_m3(
        profile,
        spec.w_c.value(),
        spec.fly_ash_pct,
        spec.silica_fume_pct,
        spec.aggregate_volume_fraction,
    );
    let gwp = embodied_co2_kg_per_m3(profile, cement, scm, agg, water);
    let margin = homog::safety_margin(profile, spec.w_c.value(), alpha);

    let device = burn_ndarray::NdArrayDevice::default();
    let free_energy =
        Tensor::<CliBackend, 2>::from_data(Data::new(vec![fc, tau], Shape::new([1, 2])), &device);
    let dissipation =
        Tensor::<CliBackend, 2>::from_data(Data::new(vec![alpha], Shape::new([1, 1])), &device);
    let safety_margin_t =
        Tensor::<CliBackend, 2>::from_data(Data::new(vec![margin], Shape::new([1, 1])), &device);
    let cost =
        Tensor::<CliBackend, 2>::from_data(Data::new(vec![gwp], Shape::new([1, 1])), &device);

    let physical = PhysicalResult {
        free_energy,
        dissipation,
        safety_margin: safety_margin_t,
        cost,
    };

    Ok(PredictBundle {
        physical,
        warnings,
        calibration_profile: profile.bundle_id.clone(),
        calibration_model: model_kind_wire(profile),
        formal_anchor: profile_formal_anchor_uri(profile),
    })
}

/// formal_anchor: lean://umst-formal/Lean/MeasurementCost.lean#zero_info_zero_energy
/// formal_status: Structural
/// formal_axioms: NONE
pub fn serialize_prediction(
    bundle: &PredictBundle,
    version: PredictionWireVersion,
) -> Result<Value, CliError> {
    let pr = &bundle.physical;
    let fc = tensor_element_at(pr.free_energy.clone(), 0, 0)?;
    let tau = tensor_element_at(pr.free_energy.clone(), 0, 1)?;
    let alpha = tensor_element_at(pr.dissipation.clone(), 0, 0)?;
    let gwp = tensor_element_at(pr.cost.clone(), 0, 0)?;
    let safety = tensor_element_at(pr.safety_margin.clone(), 0, 0)?;

    match version {
        PredictionWireVersion::V1 => {
            let wire = PredictionWireV1 {
                compressive_strength_mpa: f64::from(fc),
                yield_stress_pa: f64::from(tau),
                degree_of_hydration: f64::from(alpha),
                gwp_kg_co2_eq_per_m3: f64::from(gwp),
                safety_margin: f64::from(safety),
                schema_version: RESULT_SCHEMA_VERSION_V1,
                deprecation: Some("use result.v2; v1 will be removed next minor release"),
            };
            serde_json::to_value(wire).map_err(|e| CliError::MixSpec(MixSpecError::Json(e)))
        }
        PredictionWireVersion::V2 => {
            let wire = PredictionWireV2 {
                compressive_strength_mpa: f64::from(fc),
                yield_stress_pa: f64::from(tau),
                degree_of_hydration: f64::from(alpha),
                gwp_kg_co2_eq_per_m3: f64::from(gwp),
                safety_margin: f64::from(safety),
                calibration_profile: bundle.calibration_profile.clone(),
                calibration_model: bundle.calibration_model.clone(),
                formal_anchor: bundle.formal_anchor.clone(),
                warnings: bundle.warnings.clone(),
                schema_version: RESULT_SCHEMA_VERSION_V2,
            };
            serde_json::to_value(wire).map_err(|e| CliError::MixSpec(MixSpecError::Json(e)))
        }
    }
}

/// formal_anchor: lean://umst-formal/Lean/Constitutional.lean#kleisliCompose
/// formal_status: Structural
/// formal_axioms: NONE
#[derive(Serialize)]
pub struct CertifyChain {
    pub profile: String,
    pub model_kind: String,
    pub model_anchor: String,
    pub acceptance_anchor: String,
    pub axioms: Vec<String>,
    pub provenance_sha256: String,
}

/// formal_anchor: lean://umst-formal/Lean/Constitutional.lean#kleisliComposeWellTypedN
/// formal_status: Structural
/// formal_axioms: NONE
#[must_use]
pub fn certify_profile_json(profile: &Profile) -> Value {
    let model_anchor = profile_formal_anchor_uri(profile);
    let acceptance_anchor = profile
        .acceptance
        .formal_anchor
        .clone()
        .unwrap_or_else(|| "lean://NONE".to_string());
    let mut axioms: Vec<String> = profile
        .provenance
        .formal
        .as_ref()
        .map(|f| f.axioms.clone())
        .unwrap_or_default();
    axioms.sort();
    let chain = CertifyChain {
        profile: profile.bundle_id.clone(),
        model_kind: model_kind_wire(profile),
        model_anchor,
        acceptance_anchor,
        axioms,
        provenance_sha256: profile.provenance.prototype_3_sha256.clone(),
    };
    serde_json::to_value(&chain).unwrap_or(Value::Null)
}

#[derive(Serialize)]
struct MixSpecWireOut {
    w_c: f64,
    temperature_k: f64,
    superplasticiser_pct: f64,
    silica_fume_pct: f64,
    fly_ash_pct: f64,
    aggregate_volume_fraction: f64,
    target_age_hours: f64,
}

/// formal_anchor: NONE
/// formal_anchor_rationale: JSON round-trip helper for optimize output; no mechanised wire claim.
pub fn serialize_mix_spec(spec: &MixSpec) -> Result<Value, CliError> {
    let wire = MixSpecWireOut {
        w_c: f64::from(spec.w_c.value()),
        temperature_k: f64::from(spec.temperature_k.value()),
        superplasticiser_pct: f64::from(spec.superplasticiser_pct),
        silica_fume_pct: f64::from(spec.silica_fume_pct),
        fly_ash_pct: f64::from(spec.fly_ash_pct),
        aggregate_volume_fraction: f64::from(spec.aggregate_volume_fraction),
        target_age_hours: f64::from(spec.target_age_hours),
    };
    serde_json::to_value(wire).map_err(|e| CliError::MixSpec(MixSpecError::Json(e)))
}

fn tensor_element_at(t: Tensor<CliBackend, 2>, row: usize, col: usize) -> Result<f32, CliError> {
    let dims = t.dims();
    if row >= dims[0] || col >= dims[1] {
        return Err(CliError::Tensor("physical result index out of bounds"));
    }
    let slice = t.slice([row..row + 1, col..col + 1]);
    Ok(slice.into_scalar())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// formal_anchor: NONE
/// formal_status: Library
/// formal_axioms: NONE
/// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
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

/// formal_anchor: lean://umst-formal/Lean/OrderStatisticsBand.lean#order_statistic_concentration
/// formal_status: Mechanised
/// formal_axioms: NONE
pub fn parse_optimize_target(raw: &str) -> Result<(OptimizeField, f64), CliError> {
    let (k, v) = raw.split_once('=').ok_or(CliError::InvalidOptimizeTarget)?;
    let field = OptimizeField::try_from(k.trim())?;
    let val: f64 = v
        .trim()
        .parse()
        .map_err(|_| CliError::InvalidOptimizeTarget)?;
    Ok((field, val))
}

/// formal_anchor: lean://umst-formal/Lean/Powers.lean#powers_monotone
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
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
    let mut lo = 0.20_f32;
    let mut hi = 0.80_f32;
    let mut best = base.clone();
    let mut best_err = f32::MAX;

    for _ in 0..steps.max(1) {
        let mid = (lo + hi) * 0.5_f32;
        let cand = mix_with_w_c(
            base,
            WaterCementRatio::try_from(f64::from(mid)).map_err(CliError::MixSpec)?,
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

fn mix_with_w_c(base: &MixSpec, w_c: WaterCementRatio) -> Result<MixSpec, MixSpecError> {
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

#[cfg(all(test, feature = "cli"))]
mod wire_roundtrip_tests {
    use super::*;

    #[test]
    fn prediction_json_v2_has_calibration_fields() -> Result<(), CliError> {
        let v = serde_json::json!({"w_c": 0.40, "temperature_k": 293.15});
        let mut spec = MixSpec::try_from(v)?;
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
