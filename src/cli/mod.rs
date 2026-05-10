// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Pure CLI core: JSON validation (`MixSpec`), orchestration around the cartridge functor `F`,
//! and serialization (`ψ`) of [`PhysicalResult`] into the versioned wire JSON contract.

use crate::homogeneous;
use burn::tensor::{Data, Shape, Tensor};
use burn_ndarray::NdArray;
use serde::Serialize;
use serde_json::Value;
use std::convert::TryFrom;
use thiserror::Error;
use umst_manifold::core::traits::PhysicalResult;

/// Wire schema tag emitted with every prediction object.
pub const RESULT_SCHEMA_VERSION: &str = "result.v1";

/// NdArray backend used by the synchronous CLI path.
pub type CliBackend = NdArray;

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub fn value(self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub fn value(self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct MixSpec {
    pub w_c: WaterCementRatio,
    pub temperature_k: TemperatureK,
    pub superplasticiser_pct: f32,
    pub silica_fume_pct: f32,
    pub fly_ash_pct: f32,
    pub aggregate_volume_fraction: f32,
    pub target_age_hours: f32,
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
        })
    }
}

#[derive(Debug, Error)]
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
}

#[derive(Serialize)]
struct PredictionWireV1 {
    compressive_strength_mpa: f64,
    yield_stress_pa: f64,
    degree_of_hydration: f64,
    gwp_kg_co2_eq_per_m3: f64,
    safety_margin: f64,
    schema_version: &'static str,
}

/// Ordinal embedding convention for [`PhysicalResult`] rows produced by [`predict`]:
/// - `free_energy[:, 0]` — compressive strength (MPa)
/// - `free_energy[:, 1]` — yield stress (Pa)
/// - `dissipation` — uniform degree of hydration α ∈ [0, 1]
/// - `cost` — GWP indicator (kg CO₂-eq / m³)
/// - `safety_margin` — admissibility margin in [0, 1]
///
/// Dispatches through `crate::homogeneous`, which holds the calibrated 0-D
/// closed-forms used for single-shot prediction. The full burn-tensor
/// multi-physics pathway in `crate::physics::*` is reserved for
/// differentiable training; mixing the two paths in one entry point caused
/// physically implausible outputs in earlier revisions.
pub fn predict(spec: &MixSpec) -> Result<PhysicalResult<CliBackend>, CliError> {
    let device = burn_ndarray::NdArrayDevice::default();

    let alpha = homogeneous::degree_of_hydration(
        spec.w_c.value(),
        spec.target_age_hours,
        spec.temperature_k.value(),
    );
    let fc = homogeneous::compressive_strength_mpa(spec.w_c.value(), alpha);
    let tau = homogeneous::yield_stress_pa(
        spec.w_c.value(),
        spec.superplasticiser_pct,
        spec.aggregate_volume_fraction,
    );
    let (cement, scm, agg, water) = homogeneous::constituent_masses_kg_m3(
        spec.w_c.value(),
        spec.fly_ash_pct,
        spec.silica_fume_pct,
        spec.aggregate_volume_fraction,
    );
    let gwp = homogeneous::embodied_co2_kg_per_m3(cement, scm, agg, water);
    let margin = homogeneous::safety_margin(spec.w_c.value(), alpha);

    let free_energy =
        Tensor::<CliBackend, 2>::from_data(Data::new(vec![fc, tau], Shape::new([1, 2])), &device);
    let dissipation =
        Tensor::<CliBackend, 2>::from_data(Data::new(vec![alpha], Shape::new([1, 1])), &device);
    let safety_margin =
        Tensor::<CliBackend, 2>::from_data(Data::new(vec![margin], Shape::new([1, 1])), &device);
    let cost =
        Tensor::<CliBackend, 2>::from_data(Data::new(vec![gwp], Shape::new([1, 1])), &device);

    Ok(PhysicalResult {
        free_energy,
        dissipation,
        safety_margin,
        cost,
    })
}

/// Serialize [`PhysicalResult`] from [`predict`] using the v1 tensor embedding convention.
pub fn serialize_prediction(pr: &PhysicalResult<CliBackend>) -> Result<Value, CliError> {
    let fc = tensor_element_at(pr.free_energy.clone(), 0, 0)?;
    let tau = tensor_element_at(pr.free_energy.clone(), 0, 1)?;
    let alpha = tensor_element_at(pr.dissipation.clone(), 0, 0)?;
    let gwp = tensor_element_at(pr.cost.clone(), 0, 0)?;
    let safety = tensor_element_at(pr.safety_margin.clone(), 0, 0)?;

    let wire = PredictionWireV1 {
        compressive_strength_mpa: f64::from(fc),
        yield_stress_pa: f64::from(tau),
        degree_of_hydration: f64::from(alpha),
        gwp_kg_co2_eq_per_m3: f64::from(gwp),
        safety_margin: f64::from(safety),
        schema_version: RESULT_SCHEMA_VERSION,
    };

    serde_json::to_value(wire).map_err(|e| CliError::MixSpec(MixSpecError::Json(e)))
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

/// Serialize a validated [`MixSpec`] back into JSON matching the mix.v1 wire shape accepted by [`MixSpec::try_from`].
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

pub fn parse_optimize_target(raw: &str) -> Result<(OptimizeField, f64), CliError> {
    let (k, v) = raw.split_once('=').ok_or(CliError::InvalidOptimizeTarget)?;
    let field = OptimizeField::try_from(k.trim())?;
    let val: f64 = v
        .trim()
        .parse()
        .map_err(|_| CliError::InvalidOptimizeTarget)?;
    Ok((field, val))
}

pub fn optimize_mix(
    base: &MixSpec,
    field: OptimizeField,
    target: f64,
    steps: usize,
) -> Result<MixSpec, CliError> {
    match field {
        OptimizeField::CompressiveStrengthMpa => {
            optimize_w_c_for_strength(base, target as f32, steps)
        }
    }
}

fn optimize_w_c_for_strength(
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
        let pr = predict(&cand)?;
        let fc = tensor_element_at(pr.free_energy, 0, 0)?;
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
    })
}

#[cfg(all(test, feature = "cli"))]
mod wire_roundtrip_tests {
    use super::*;

    #[test]
    fn prediction_json_has_required_keys() -> Result<(), CliError> {
        let v = serde_json::json!({"w_c": 0.40, "temperature_k": 293.15});
        let spec = MixSpec::try_from(v)?;
        let pr = predict(&spec)?;
        let out = serialize_prediction(&pr)?;
        assert_eq!(out["schema_version"], RESULT_SCHEMA_VERSION);
        let alpha = out["degree_of_hydration"]
            .as_f64()
            .ok_or(CliError::Tensor("missing degree_of_hydration in wire JSON"))?;
        assert!(alpha >= 0.0);
        let fc = out["compressive_strength_mpa"]
            .as_f64()
            .ok_or(CliError::Tensor(
                "missing compressive_strength_mpa in wire JSON",
            ))?;
        assert!(fc > 0.0);
        Ok(())
    }
}
