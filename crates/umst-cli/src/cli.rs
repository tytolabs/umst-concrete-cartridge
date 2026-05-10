// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Pure CLI core: JSON validation (`MixSpec`), regime checks, tensor pipeline prediction (see [`PredictBundle::physics_pipeline`]),
//! homogeneous compare flag, and versioned wire JSON (`result.v1` / `result.v2`).

use burn::tensor::Tensor;
use burn_ndarray::NdArray;
use serde::Serialize;
use serde_json::Value;
use std::convert::TryFrom;
use std::fmt;
use umst_concrete_cartridge::calibration::{self as calib, Profile};
use umst_concrete_cartridge::homogeneous::{
    self as homog, constituent_masses_kg_m3, embodied_co2_kg_per_m3, mix_row_from_scalar_spec,
};
use umst_concrete_cartridge::mix_layout::{fractions_from_mix_row, mix_tensor_from_layout};
use umst_concrete_cartridge::pipeline::{
    physical_result_from_report, run_full_physics_pipeline, PhysicsPipelineReport,
};
use umst_manifold::core::traits::PhysicalResult;

/// formal_anchor: literature://wire-schema-result-v1
/// formal_status: Literature
/// formal_citation: "UMST concrete cartridge JSON wire schema tag (`result.v1`)"
/// formal_form: "`result.v1` — version tag for deprecated prediction JSON envelope"
pub const RESULT_SCHEMA_VERSION_V1: &str = "result.v1";

/// formal_anchor: literature://wire-schema-result-v2
/// formal_status: Literature
/// formal_citation: "UMST concrete cartridge JSON wire schema tag (`result.v2`)"
/// formal_form: "`result.v2` — version tag for current prediction JSON envelope"
pub const RESULT_SCHEMA_VERSION_V2: &str = "result.v2";

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Burn backend selection; structural type alias to the ndarray tensor runtime.
pub type CliBackend = NdArray;

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Exhaustive enum over wire-schema variants; pattern matching guarantees both tags handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictionWireVersion {
    V1,
    V2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// formal_anchor: lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime
/// formal_status: Mechanised
/// formal_axioms: NONE
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
    /// formal_status: NONE
    /// formal_anchor_rationale: Trivial accessor; getter for the wrapped `f32`.
    pub fn value(self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// formal_anchor: lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime
/// formal_status: Mechanised
/// formal_axioms: NONE
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
    /// formal_status: NONE
    /// formal_anchor_rationale: Trivial accessor.
    pub fn value(self) -> f32 {
        self.0
    }
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Field invariants enforced by `WaterCementRatio` / `TemperatureK` newtypes and range-checked fractions.
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

#[derive(Debug)]
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: IO / parsing error variants; classification of mix-spec rejection causes.
pub enum MixSpecError {
    Json(serde_json::Error),
    WaterCementRatioOutOfRange(f32),
    TemperatureOutOfRange(f32),
    FieldOutOfRange { field: &'static str },
}

impl fmt::Display for MixSpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "invalid JSON mix specification: {e}"),
            Self::WaterCementRatioOutOfRange(x) => write!(
                f,
                "water-cement ratio {x} outside allowed range [0.20, 0.80]"
            ),
            Self::TemperatureOutOfRange(x) => {
                write!(f, "temperature_k {x} outside allowed range [273, 353] K")
            }
            Self::FieldOutOfRange { field } => {
                write!(f, "field `{field}` outside allowed physical range")
            }
        }
    }
}

impl std::error::Error for MixSpecError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for MixSpecError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Binary-boundary error aggregation; sum-type over `MixSpecError`, calibration, tensor IO, and routing failures.
#[derive(Debug)]
pub enum CliError {
    MixSpec(MixSpecError),
    Tensor(&'static str),
    UnsupportedOptimizeTarget(String),
    InvalidOptimizeTarget,
    Homogeneous(homog::HomogeneousError),
    Calibration(calib::CalibrationError),
    OutsideAllRegimes,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MixSpec(e) => write!(f, "{e}"),
            Self::Tensor(msg) => write!(f, "tensor pipeline: {msg}"),
            Self::UnsupportedOptimizeTarget(t) => {
                write!(f, "optimization target `{t}` is not supported")
            }
            Self::InvalidOptimizeTarget => write!(
                f,
                "could not parse optimization target (expected FIELD=VALUE)"
            ),
            Self::Homogeneous(e) => write!(f, "{e}"),
            Self::Calibration(e) => write!(f, "{e}"),
            Self::OutsideAllRegimes => {
                write!(f, "mix design lies outside all bundled calibration regimes")
            }
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MixSpec(e) => Some(e),
            Self::Homogeneous(e) => Some(e),
            Self::Calibration(e) => Some(e),
            _ => None,
        }
    }
}

impl From<MixSpecError> for CliError {
    fn from(e: MixSpecError) -> Self {
        Self::MixSpec(e)
    }
}

impl From<homog::HomogeneousError> for CliError {
    fn from(e: homog::HomogeneousError) -> Self {
        Self::Homogeneous(e)
    }
}

impl From<calib::CalibrationError> for CliError {
    fn from(e: calib::CalibrationError) -> Self {
        Self::Calibration(e)
    }
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

/// Predict output toggles surfaced by MCP / `umst predict`.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Behavioral flags only — no Lean witness.
#[derive(Debug, Clone, Copy, Default)]
pub struct PredictOptions {
    /// When true, attaches a legacy homogeneous scalar envelope under `homogeneous_compare`.
    pub compare_homogeneous: bool,
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Bundle of physical tensors plus calibration metadata returned by [`predict`] / [`predict_with_options`].
pub struct PredictBundle {
    pub physical: PhysicalResult<CliBackend>,
    pub warnings: Vec<String>,
    pub calibration_profile: String,
    pub calibration_model: String,
    pub formal_anchor: String,
    /// Staged tensor-physics capsule serialized as `physics_pipeline` on `result.v2`.
    pub physics_pipeline: PhysicsPipelineReport,
    /// Present only when [`PredictOptions::compare_homogeneous`] is enabled.
    pub homogeneous_compare: Option<serde_json::Value>,
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

fn wire_formal_status(profile: &Profile) -> String {
    let raw = profile
        .provenance
        .formal
        .as_ref()
        .map(|f| f.status.as_str())
        .or(profile.acceptance.acceptance_bucket.as_deref());
    match raw.map(str::trim) {
        Some("Mechanised") => "Mechanised".to_string(),
        Some("Structural") => "Structural".to_string(),
        Some("Empirical") => "Empirical".to_string(),
        Some("Literature") => "Literature".to_string(),
        Some("NONE") => "NONE".to_string(),
        Some("Boundary") | None => "NONE".to_string(),
        Some(_) => "NONE".to_string(),
    }
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Natural transformation φ ∘ F ∘ ψ over the cartridge functor (CLI orchestration entry).
pub fn predict(profile: &Profile, spec: &MixSpec) -> Result<PredictBundle, CliError> {
    predict_with_options(profile, spec, PredictOptions::default())
}

/// Same as [`predict`] with optional homogeneous sidecar for regression diffs.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Feature flag glue for MCP/CLI; no standalone formal claim.
pub fn predict_with_options(
    profile: &Profile,
    spec: &MixSpec,
    options: PredictOptions,
) -> Result<PredictBundle, CliError> {
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

    let layout = fractions_from_mix_row(&row, spec.aggregate_volume_fraction);
    let device = burn_ndarray::NdArrayDevice::default();
    let mix_tensor = mix_tensor_from_layout::<CliBackend>(&layout, &device);
    let physics_pipeline = run_full_physics_pipeline::<CliBackend>(profile, &mix_tensor);
    let physical = physical_result_from_report::<CliBackend>(profile, &physics_pipeline, &device);

    let homogeneous_compare = if options.compare_homogeneous {
        let alpha_h = homog::degree_of_hydration_alpha(profile, &row)?;
        let fc_h = homog::compressive_strength_mpa(profile, &row)?;
        let tau_h = homog::yield_stress_pa(
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
        let gwp_h = embodied_co2_kg_per_m3(profile, cement, scm, agg, water);
        let margin_h = homog::safety_margin(profile, spec.w_c.value(), alpha_h);
        Some(serde_json::json!({
            "compressive_strength_mpa": f64::from(fc_h),
            "yield_stress_pa": f64::from(tau_h),
            "degree_of_hydration": f64::from(alpha_h),
            "gwp_kg_co2_eq_per_m3": f64::from(gwp_h),
            "safety_margin": f64::from(margin_h),
        }))
    } else {
        None
    };

    Ok(PredictBundle {
        physical,
        warnings,
        calibration_profile: profile.bundle_id.clone(),
        calibration_model: model_kind_wire(profile),
        formal_anchor: profile_formal_anchor_uri(profile),
        physics_pipeline,
        homogeneous_compare,
    })
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: JSON-serialise glue; no physical claim.
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
            let mut out = serde_json::to_value(&wire)
                .map_err(|e| CliError::MixSpec(MixSpecError::Json(e)))?;
            let pipeline_val = serde_json::to_value(&bundle.physics_pipeline)
                .map_err(|e| CliError::MixSpec(MixSpecError::Json(e)))?;
            if let Value::Object(ref mut map) = out {
                map.insert("physics_pipeline".into(), pipeline_val);
                if let Some(h) = bundle.homogeneous_compare.clone() {
                    map.insert("homogeneous_compare".into(), h);
                }
            }
            Ok(out)
        }
    }
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: JSON payload schema for `umst certify` output (profile, anchors, mapped formal bucket).
#[derive(Serialize)]
pub struct CertifyChain {
    pub profile: String,
    pub model_kind: String,
    pub model_anchor: String,
    pub acceptance_anchor: String,
    pub formal_status: String,
    pub axioms: Vec<String>,
    pub provenance_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zenodo_record: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zenodo_doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zenodo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subset: Option<String>,
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Builds the certify JSON view including wire `formal_status` mapped from profile metadata.
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
    let formal_status = wire_formal_status(profile);
    let chain = CertifyChain {
        profile: profile.bundle_id.clone(),
        model_kind: model_kind_wire(profile),
        model_anchor,
        acceptance_anchor,
        formal_status,
        axioms,
        provenance_sha256: profile.provenance.prototype_3_sha256.clone(),
        zenodo_record: profile.provenance.zenodo_record.clone(),
        zenodo_doi: profile.provenance.zenodo_doi.clone(),
        zenodo_url: profile.provenance.zenodo_url.clone(),
        license: profile.provenance.license.clone(),
        subset: profile.provenance.subset.clone(),
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
/// formal_status: NONE
/// formal_anchor_rationale: JSON-serialise glue.
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

#[cfg(test)]
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
