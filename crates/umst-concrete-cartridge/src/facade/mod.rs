// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Transport-facing wire types and pure prediction helpers (**serde** only, no **`serde_json`**).
//! Parsing JSON blobs into [`MixSpecWire`] stays in **`umst-cli`** / **`umst-py`**.

use burn::tensor::Tensor;
use burn_ndarray::NdArray;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;

use crate::calibration::{self as calib, Profile};
use crate::homogeneous::{
    self as homog, constituent_masses_kg_m3, embodied_co2_kg_per_m3, mix_row_from_scalar_spec,
};
use crate::mix_layout::{fractions_from_mix_row, mix_tensor_from_layout};
use crate::pipeline::{
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
pub type FacadeBackend = NdArray;

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

/// Deserializable wire record for mix JSON (parsed with **`serde_json`** only outside this crate).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Serde shape for mix.v1 JSON; field validation on conversion to [`MixSpec`].
#[derive(Debug, serde::Deserialize)]
pub struct MixSpecWire {
    pub w_c: f64,
    pub temperature_k: f64,
    #[serde(default)]
    pub superplasticiser_pct: Option<f64>,
    #[serde(default)]
    pub silica_fume_pct: Option<f64>,
    #[serde(default)]
    pub fly_ash_pct: Option<f64>,
    #[serde(default)]
    pub aggregate_volume_fraction: Option<f64>,
    #[serde(default)]
    pub target_age_hours: Option<f64>,
}

impl TryFrom<MixSpecWire> for MixSpec {
    type Error = MixSpecError;

    fn try_from(wire: MixSpecWire) -> Result<Self, Self::Error> {
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
/// formal_anchor_rationale: Mix-spec rejection causes without JSON parse errors (handled at transport boundary).
pub enum MixSpecError {
    WaterCementRatioOutOfRange(f32),
    TemperatureOutOfRange(f32),
    FieldOutOfRange { field: &'static str },
}

impl fmt::Display for MixSpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

impl std::error::Error for MixSpecError {}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Binary-boundary error aggregation for facade calls (no vendor IO).
#[derive(Debug)]
pub enum FacadeError {
    MixSpec(MixSpecError),
    Tensor(&'static str),
    Homogeneous(homog::HomogeneousError),
    Calibration(calib::CalibrationError),
    OutsideAllRegimes,
}

impl fmt::Display for FacadeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MixSpec(e) => write!(f, "{e}"),
            Self::Tensor(msg) => write!(f, "tensor pipeline: {msg}"),
            Self::Homogeneous(e) => write!(f, "{e}"),
            Self::Calibration(e) => write!(f, "{e}"),
            Self::OutsideAllRegimes => {
                write!(f, "mix design lies outside all bundled calibration regimes")
            }
        }
    }
}

impl std::error::Error for FacadeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MixSpec(e) => Some(e),
            Self::Homogeneous(e) => Some(e),
            Self::Calibration(e) => Some(e),
            _ => None,
        }
    }
}

impl From<MixSpecError> for FacadeError {
    fn from(e: MixSpecError) -> Self {
        Self::MixSpec(e)
    }
}

impl From<homog::HomogeneousError> for FacadeError {
    fn from(e: homog::HomogeneousError) -> Self {
        Self::Homogeneous(e)
    }
}

impl From<calib::CalibrationError> for FacadeError {
    fn from(e: calib::CalibrationError) -> Self {
        Self::Calibration(e)
    }
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Homogeneous sidecar scalars for optional regression diff (serde-friendly).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomogeneousCompareWire {
    pub compressive_strength_mpa: f64,
    pub yield_stress_pa: f64,
    pub degree_of_hydration: f64,
    pub gwp_kg_co2_eq_per_m3: f64,
    pub safety_margin: f64,
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
    pub physical: PhysicalResult<FacadeBackend>,
    pub warnings: Vec<String>,
    pub calibration_profile: String,
    pub calibration_model: String,
    pub formal_anchor: String,
    /// Sorted axiom identifiers from `[provenance.formal]` (same lineage as certify / audit envelopes).
    pub axioms: Vec<String>,
    /// Staged tensor-physics capsule serialized as `physics_pipeline` on `result.v2`.
    pub physics_pipeline: PhysicsPipelineReport,
    /// Present only when [`PredictOptions::compare_homogeneous`] is enabled.
    pub homogeneous_compare: Option<HomogeneousCompareWire>,
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

fn profile_axioms_for_wire(profile: &Profile) -> Vec<String> {
    let mut axioms: Vec<String> = profile
        .provenance
        .formal
        .as_ref()
        .map(|f| f.axioms.clone())
        .unwrap_or_default();
    axioms.sort();
    axioms
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Natural transformation φ ∘ F ∘ ψ over the cartridge functor (facade orchestration entry).
pub fn predict(profile: &Profile, spec: &MixSpec) -> Result<PredictBundle, FacadeError> {
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
) -> Result<PredictBundle, FacadeError> {
    if !calib::any_bundled_profile_covers_scalars(
        spec.w_c.value(),
        spec.temperature_k.value(),
        spec.target_age_hours,
        spec.fly_ash_pct,
        spec.silica_fume_pct,
    ) {
        return Err(FacadeError::OutsideAllRegimes);
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
    let mix_tensor = mix_tensor_from_layout::<FacadeBackend>(&layout, &device);
    let physics_pipeline = run_full_physics_pipeline::<FacadeBackend>(profile, &mix_tensor);
    let physical =
        physical_result_from_report::<FacadeBackend>(profile, &physics_pipeline, &device);

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
        Some(HomogeneousCompareWire {
            compressive_strength_mpa: f64::from(fc_h),
            yield_stress_pa: f64::from(tau_h),
            degree_of_hydration: f64::from(alpha_h),
            gwp_kg_co2_eq_per_m3: f64::from(gwp_h),
            safety_margin: f64::from(margin_h),
        })
    } else {
        None
    };

    Ok(PredictBundle {
        physical,
        warnings,
        calibration_profile: profile.bundle_id.clone(),
        calibration_model: model_kind_wire(profile),
        formal_anchor: profile_formal_anchor_uri(profile),
        axioms: profile_axioms_for_wire(profile),
        physics_pipeline,
        homogeneous_compare,
    })
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Tensor prediction from dataset-style [`homog::MixRow`] masses; regime gates use binder-normalised SCM splits (slag routed through the silica regime channel).
pub fn predict_from_mix_row(
    profile: &Profile,
    row: &homog::MixRow,
    aggregate_volume_fraction: f32,
    options: PredictOptions,
) -> Result<PredictBundle, FacadeError> {
    const BINDER_KG_EPS: f32 = 1.0;
    let binder = row.cement_kg_m3 + row.slag_kg_m3 + row.fly_ash_kg_m3;
    if binder < BINDER_KG_EPS {
        return Err(FacadeError::Tensor(
            "binder mass (cement+slag+fly_ash) below numerical floor",
        ));
    }
    let w_cm = row.water_kg_m3 / binder;
    let fly_pct = 100.0 * row.fly_ash_kg_m3 / binder;
    let scm_silica_slot_pct = 100.0 * row.slag_kg_m3 / binder;
    let age_hours = row.age_days * 24.0;
    let temperature_k = row.temperature_c + 273.15;
    let superplasticiser_pct = (row.superplasticizer_kg_m3 / binder) * 100.0;

    if !calib::any_bundled_profile_covers_scalars(
        w_cm,
        temperature_k,
        age_hours,
        fly_pct,
        scm_silica_slot_pct,
    ) {
        return Err(FacadeError::OutsideAllRegimes);
    }

    let violations =
        profile.regime_check_scalars(w_cm, temperature_k, age_hours, fly_pct, scm_silica_slot_pct);
    let warnings: Vec<String> = violations.iter().map(|v| v.to_string()).collect();

    let layout = fractions_from_mix_row(row, aggregate_volume_fraction);
    let device = burn_ndarray::NdArrayDevice::default();
    let mix_tensor = mix_tensor_from_layout::<FacadeBackend>(&layout, &device);
    let physics_pipeline = run_full_physics_pipeline::<FacadeBackend>(profile, &mix_tensor);
    let physical =
        physical_result_from_report::<FacadeBackend>(profile, &physics_pipeline, &device);

    let homogeneous_compare = if options.compare_homogeneous {
        let alpha_h = homog::degree_of_hydration_alpha(profile, row)?;
        let fc_h = homog::compressive_strength_mpa(profile, row)?;
        let tau_h = homog::yield_stress_pa(
            profile,
            w_cm,
            superplasticiser_pct,
            aggregate_volume_fraction,
        );
        let (cement, scm, agg, water) = constituent_masses_kg_m3(
            profile,
            w_cm,
            fly_pct,
            scm_silica_slot_pct,
            aggregate_volume_fraction,
        );
        let gwp_h = embodied_co2_kg_per_m3(profile, cement, scm, agg, water);
        let margin_h = homog::safety_margin(profile, w_cm, alpha_h);
        Some(HomogeneousCompareWire {
            compressive_strength_mpa: f64::from(fc_h),
            yield_stress_pa: f64::from(tau_h),
            degree_of_hydration: f64::from(alpha_h),
            gwp_kg_co2_eq_per_m3: f64::from(gwp_h),
            safety_margin: f64::from(margin_h),
        })
    } else {
        None
    };

    Ok(PredictBundle {
        physical,
        warnings,
        calibration_profile: profile.bundle_id.clone(),
        calibration_model: model_kind_wire(profile),
        formal_anchor: profile_formal_anchor_uri(profile),
        axioms: profile_axioms_for_wire(profile),
        physics_pipeline,
        homogeneous_compare,
    })
}

/// formal_anchor: literature://wire-schema-audit-v1
/// formal_status: Literature
/// formal_citation: "UMST concrete cartridge JSON wire schema tag (`audit.v1`)"
/// formal_form: "`audit.v1` — batch CSV audit envelope with tensor predictions vs optional CSV strength"
/// formal_anchor_rationale: Version discriminator for UMST CLI `umst audit`.
pub const AUDIT_SCHEMA_VERSION: &str = "audit.v1";

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: One CSV row wired for corpus audit alongside aggregate packing fraction derived in CLI from coarse/fine masses (ρ=2600 kg/m³, same surrogate as homogeneous layout).
#[derive(Debug, Clone)]
pub struct PreparedAuditRow<'a> {
    pub row_index: usize,
    pub mix_row: &'a homog::MixRow,
    pub aggregate_volume_fraction: f32,
    pub observed_strength_mpa: Option<f32>,
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: JSON summary stats for auditors; aggregates row-level residuals only.
#[derive(Debug, Serialize)]
pub struct AuditSummaryV1 {
    pub row_count: u32,
    pub rows_with_observations: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_absolute_error_mpa: Option<f64>,
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Top-level serde envelope for CLI/MCP corpus audit tooling.
#[derive(Debug, Serialize)]
pub struct AuditReportV1 {
    pub schema_version: String,
    pub calibration_profile: String,
    pub formal_anchor: String,
    pub calibration_model: String,
    pub axioms: Vec<String>,
    pub summary: AuditSummaryV1,
    pub rows: Vec<AuditRowWireV1>,
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Normalised mix scalars carried from the CSV row for audit consumers (`audit.v1` row `input`).
#[derive(Debug, Serialize)]
pub struct AuditRowInputV1 {
    pub cement_kg_m3: f64,
    pub slag_kg_m3: f64,
    pub fly_ash_kg_m3: f64,
    pub water_kg_m3: f64,
    pub superplasticizer_kg_m3: f64,
    pub aggregate_volume_fraction: f64,
    pub age_days: f64,
    pub temperature_c: f64,
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: One audited CSV row projection with tensor headline strength.
#[derive(Debug, Serialize)]
pub struct AuditRowWireV1 {
    pub row_index: u32,
    pub input: AuditRowInputV1,
    pub profile_used: String,
    pub formal_anchor: String,
    pub predicted_strength_mpa: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_strength_mpa: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abs_error_mpa: Option<f64>,
    pub regime_warnings: Vec<String>,
    pub safety_margin: f64,
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Deterministic corpus audit projection over prepared rows (tensor strength channel).
pub fn audit_build_report_v1(
    profile: &Profile,
    entries: &[PreparedAuditRow<'_>],
) -> Result<AuditReportV1, FacadeError> {
    let chain = certify_profile_chain(profile);
    let mut rows_out = Vec::with_capacity(entries.len());
    let mut abs_err_acc = 0.0_f64;
    let mut obs_count = 0_u32;

    for e in entries {
        let bundle = predict_from_mix_row(
            profile,
            e.mix_row,
            e.aggregate_volume_fraction,
            PredictOptions::default(),
        )?;
        let fc = tensor_element_at(bundle.physical.free_energy.clone(), 0, 0)?;
        let margin = tensor_element_at(bundle.physical.safety_margin.clone(), 0, 0)?;
        let pred = f64::from(fc);
        let safety_margin = f64::from(margin);
        let obs_opt = e.observed_strength_mpa.map(f64::from);
        let res_abs = obs_opt.map(|obs| (obs - pred).abs());
        if let Some(ra) = res_abs {
            abs_err_acc += ra;
            obs_count += 1;
        }
        rows_out.push(AuditRowWireV1 {
            row_index: e.row_index as u32,
            input: AuditRowInputV1 {
                cement_kg_m3: f64::from(e.mix_row.cement_kg_m3),
                slag_kg_m3: f64::from(e.mix_row.slag_kg_m3),
                fly_ash_kg_m3: f64::from(e.mix_row.fly_ash_kg_m3),
                water_kg_m3: f64::from(e.mix_row.water_kg_m3),
                superplasticizer_kg_m3: f64::from(e.mix_row.superplasticizer_kg_m3),
                aggregate_volume_fraction: f64::from(e.aggregate_volume_fraction),
                age_days: f64::from(e.mix_row.age_days),
                temperature_c: f64::from(e.mix_row.temperature_c),
            },
            profile_used: chain.profile.clone(),
            formal_anchor: chain.model_anchor.clone(),
            predicted_strength_mpa: pred,
            observed_strength_mpa: obs_opt,
            abs_error_mpa: res_abs,
            regime_warnings: bundle.warnings.clone(),
            safety_margin,
        });
    }

    let mae = if obs_count > 0 {
        Some(abs_err_acc / f64::from(obs_count))
    } else {
        None
    };

    Ok(AuditReportV1 {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        calibration_profile: chain.profile.clone(),
        formal_anchor: chain.model_anchor.clone(),
        calibration_model: chain.model_kind.clone(),
        axioms: chain.axioms.clone(),
        summary: AuditSummaryV1 {
            row_count: entries.len() as u32,
            rows_with_observations: obs_count,
            mean_absolute_error_mpa: mae,
        },
        rows: rows_out,
    })
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: JSON payload schema for `umst certify` output (profile, anchors, mapped formal bucket).
#[derive(Debug, Serialize)]
pub struct CertifyChain {
    pub profile: String,
    pub model_kind: String,
    pub model_anchor: String,
    pub acceptance_anchor: String,
    #[serde(rename = "formal_status")]
    pub profile_formal_bucket: String,
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
/// formal_anchor_rationale: Builds the certify view including wire `formal_status` mapped from profile metadata.
#[must_use]
pub fn certify_profile_chain(profile: &Profile) -> CertifyChain {
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
    let profile_formal_bucket = wire_formal_status(profile);
    CertifyChain {
        profile: profile.bundle_id.clone(),
        model_kind: model_kind_wire(profile),
        model_anchor,
        acceptance_anchor,
        profile_formal_bucket,
        axioms,
        provenance_sha256: profile.provenance.prototype_3_sha256.clone(),
        zenodo_record: profile.provenance.zenodo_record.clone(),
        zenodo_doi: profile.provenance.zenodo_doi.clone(),
        zenodo_url: profile.provenance.zenodo_url.clone(),
        license: profile.provenance.license.clone(),
        subset: profile.provenance.subset.clone(),
    }
}

/// Bundled JSON Schema draft for `mix.v1` (static bytes).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: SSOT `include_str!` of repo-root schema for CLI/MCP/Python.
#[must_use]
pub fn schema_mix_v1_json() -> &'static str {
    include_str!("../../../../schema/mix.v1.json")
}

/// Bundled JSON Schema draft for `result.v1`.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: SSOT `include_str!` of repo-root schema for CLI/MCP/Python.
#[must_use]
pub fn schema_result_v1_json() -> &'static str {
    include_str!("../../../../schema/result.v1.json")
}

/// Bundled JSON Schema draft for `result.v2`.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: SSOT `include_str!` of repo-root schema for CLI/MCP/Python.
#[must_use]
pub fn schema_result_v2_json() -> &'static str {
    include_str!("../../../../schema/result.v2.json")
}

/// Bundled JSON Schema draft for `audit.v1`.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: SSOT `include_str!` of repo-root schema for CLI `umst audit`.
#[must_use]
pub fn schema_audit_v1_json() -> &'static str {
    include_str!("../../../../schema/audit.v1.json")
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Internal tensor scalar read for wire projection; index contract from pipeline layout.
pub fn tensor_element_at(
    t: Tensor<FacadeBackend, 2>,
    row: usize,
    col: usize,
) -> Result<f32, FacadeError> {
    let dims = t.dims();
    if row >= dims[0] || col >= dims[1] {
        return Err(FacadeError::Tensor("physical result index out of bounds"));
    }
    let slice = t.slice([row..row + 1, col..col + 1]);
    Ok(slice.into_scalar())
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Serde wire projection for `result.v1` scalars; versioning tagged in `schema_version`.
#[derive(Serialize)]
pub struct PredictionWireV1 {
    pub compressive_strength_mpa: f64,
    pub yield_stress_pa: f64,
    pub degree_of_hydration: f64,
    pub gwp_kg_co2_eq_per_m3: f64,
    pub safety_margin: f64,
    pub schema_version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation: Option<&'static str>,
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Serde wire projection for `result.v2` scalars; `physics_pipeline` merged at JSON boundary.
#[derive(Serialize)]
pub struct PredictionWireV2 {
    pub compressive_strength_mpa: f64,
    pub yield_stress_pa: f64,
    pub degree_of_hydration: f64,
    pub gwp_kg_co2_eq_per_m3: f64,
    pub safety_margin: f64,
    pub calibration_profile: String,
    pub calibration_model: String,
    pub formal_anchor: String,
    pub axioms: Vec<String>,
    pub warnings: Vec<String>,
    pub schema_version: &'static str,
}

/// Build serializable v1 wire scalars (CLI/MCP add JSON encoding).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Pure wire projection; transport encoding is caller-owned.
pub fn prediction_wire_v1(bundle: &PredictBundle) -> Result<PredictionWireV1, FacadeError> {
    let pr = &bundle.physical;
    let fc = tensor_element_at(pr.free_energy.clone(), 0, 0)?;
    let tau = tensor_element_at(pr.free_energy.clone(), 0, 1)?;
    let alpha = tensor_element_at(pr.dissipation.clone(), 0, 0)?;
    let gwp = tensor_element_at(pr.cost.clone(), 0, 0)?;
    let safety = tensor_element_at(pr.safety_margin.clone(), 0, 0)?;
    Ok(PredictionWireV1 {
        compressive_strength_mpa: f64::from(fc),
        yield_stress_pa: f64::from(tau),
        degree_of_hydration: f64::from(alpha),
        gwp_kg_co2_eq_per_m3: f64::from(gwp),
        safety_margin: f64::from(safety),
        schema_version: RESULT_SCHEMA_VERSION_V1,
        deprecation: Some("use result.v2; v1 will be removed next minor release"),
    })
}

/// Build serializable v2 wire scalars (embed `physics_pipeline` / `homogeneous_compare` at JSON layer).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Pure wire projection; nested objects merged by CLI/MCP `serde_json`.
pub fn prediction_wire_v2(bundle: &PredictBundle) -> Result<PredictionWireV2, FacadeError> {
    let pr = &bundle.physical;
    let fc = tensor_element_at(pr.free_energy.clone(), 0, 0)?;
    let tau = tensor_element_at(pr.free_energy.clone(), 0, 1)?;
    let alpha = tensor_element_at(pr.dissipation.clone(), 0, 0)?;
    let gwp = tensor_element_at(pr.cost.clone(), 0, 0)?;
    let safety = tensor_element_at(pr.safety_margin.clone(), 0, 0)?;
    Ok(PredictionWireV2 {
        compressive_strength_mpa: f64::from(fc),
        yield_stress_pa: f64::from(tau),
        degree_of_hydration: f64::from(alpha),
        gwp_kg_co2_eq_per_m3: f64::from(gwp),
        safety_margin: f64::from(safety),
        calibration_profile: bundle.calibration_profile.clone(),
        calibration_model: bundle.calibration_model.clone(),
        formal_anchor: bundle.formal_anchor.clone(),
        axioms: bundle.axioms.clone(),
        warnings: bundle.warnings.clone(),
        schema_version: RESULT_SCHEMA_VERSION_V2,
    })
}

#[derive(Serialize)]
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Round-trip mix spec view for CLI `mix print` / MCP.
pub struct MixSpecWireOut {
    pub w_c: f64,
    pub temperature_k: f64,
    pub superplasticiser_pct: f64,
    pub silica_fume_pct: f64,
    pub fly_ash_pct: f64,
    pub aggregate_volume_fraction: f64,
    pub target_age_hours: f64,
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Serialize-friendly mix view without JSON crate in core.
#[must_use]
pub fn mix_spec_wire_out(spec: &MixSpec) -> MixSpecWireOut {
    MixSpecWireOut {
        w_c: f64::from(spec.w_c.value()),
        temperature_k: f64::from(spec.temperature_k.value()),
        superplasticiser_pct: f64::from(spec.superplasticiser_pct),
        silica_fume_pct: f64::from(spec.silica_fume_pct),
        fly_ash_pct: f64::from(spec.fly_ash_pct),
        aggregate_volume_fraction: f64::from(spec.aggregate_volume_fraction),
        target_age_hours: f64::from(spec.target_age_hours),
    }
}
