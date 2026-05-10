// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Pure CLI core: JSON validation (`MixSpec`), orchestration around the cartridge functor `F`,
//! and serialization (`ψ`) of [`PhysicalResult`] into the versioned wire JSON contract.

use crate::core::ConcreteCartridge;
use crate::physics::rheology::RheologyEngine;
use crate::physics::strength::StrengthEngine;
use crate::physics::sustainability::SustainabilityEngine;
use burn::tensor::{backend::Backend, Data, Int, Shape, Tensor};
use burn_ndarray::NdArray;
use serde::Serialize;
use serde_json::Value;
use std::convert::TryFrom;
use thiserror::Error;
use umst_manifold::core::tensors::{MixTensor, UnifiedMaterialStateTensor};
use umst_manifold::core::traits::{IScienceCartridge, PhysicalResult};

/// Wire schema tag emitted with every prediction object.
pub const RESULT_SCHEMA_VERSION: &str = "result.v1";

/// NdArray backend used by the synchronous CLI path.
pub type CliBackend = NdArray;

type MassBlock4 = (
    Tensor<CliBackend, 4>,
    Tensor<CliBackend, 4>,
    Tensor<CliBackend, 4>,
    Tensor<CliBackend, 4>,
);

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
/// - `safety_margin` — admissibility margin from [`IScienceCartridge::compute_topology`]
pub fn predict(spec: &MixSpec) -> Result<PhysicalResult<CliBackend>, CliError> {
    let device = burn_ndarray::NdArrayDevice::default();
    let cartridge = ConcreteCartridge::<CliBackend>::default();

    let mix = mix_tensor_from_spec(spec, &device)?;
    let alpha = hydration_degree_from_mix(&mix, spec, &device)?;
    let alpha_scalar = tensor_mean_scalar(alpha)?;

    let wc_4 = tensor_fill_4d(spec.w_c.value(), &device);
    let alpha_4 = tensor_fill_4d(alpha_scalar, &device);
    let air_4 = tensor_fill_4d(0.02_f32, &device);
    let intrinsic_4 = tensor_fill_4d(2_400_f32, &device);

    let (fc_tensor, _, _) = StrengthEngine::compute_strength_jennings(
        wc_4.clone(),
        alpha_4.clone(),
        air_4,
        intrinsic_4,
    );
    let fc_scalar = tensor_mean_scalar_nd(fc_tensor)?;

    let phi = tensor_fill_4d(spec.aggregate_volume_fraction, &device);
    let phi_m = tensor_fill_4d(0.74_f32, &device);
    let d50 = tensor_fill_4d(120e-6_f32, &device);
    let f_sigma = tensor_fill_4d(
        1e-3_f32 * (1.0 + spec.superplasticiser_pct * 0.05_f32),
        &device,
    );

    let tau_tensor =
        RheologyEngine::compute_yield_stress_yodel(phi.clone(), phi_m.clone(), d50, f_sigma);
    let tau_scalar = tensor_mean_scalar_nd(tau_tensor)?;

    let (mass_cement, mass_scm, mass_agg, mass_water) = masses_from_spec(spec, &device)?;
    let gwp_tensor = SustainabilityEngine::compute_embodied_carbon(
        mass_cement.clone(),
        mass_scm.clone(),
        mass_agg.clone(),
        mass_water.clone(),
        (0.93_f32, 0.05_f32, 0.02_f32, 0.001_f32),
    );
    let gwp_scalar = tensor_mean_scalar_nd(gwp_tensor)?;

    let manifold = minimal_manifold_from_spec(spec, &device)?;
    let topo = cartridge.compute_topology(&manifold);
    let safety_scalar = tensor_mean_scalar(topo.safety_margin.clone())?;

    let row_energy = vec![fc_scalar, tau_scalar];
    let free_energy =
        Tensor::<CliBackend, 2>::from_data(Data::new(row_energy, Shape::new([1, 2])), &device);
    let dissipation = Tensor::<CliBackend, 2>::from_data(
        Data::new(vec![alpha_scalar], Shape::new([1, 1])),
        &device,
    );
    let safety_margin = Tensor::<CliBackend, 2>::from_data(
        Data::new(vec![safety_scalar], Shape::new([1, 1])),
        &device,
    );
    let cost = Tensor::<CliBackend, 2>::from_data(
        Data::new(vec![gwp_scalar], Shape::new([1, 1])),
        &device,
    );

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

fn tensor_fill_4d(value: f32, device: &<CliBackend as Backend>::Device) -> Tensor<CliBackend, 4> {
    Tensor::<CliBackend, 4>::full([1, 1, 1, 1], value, device)
}

fn mix_tensor_from_spec(
    spec: &MixSpec,
    device: &<CliBackend as Backend>::Device,
) -> Result<MixTensor<CliBackend>, CliError> {
    let w_c = spec.w_c.value();
    let fly = spec.fly_ash_pct / 100.0_f32;
    let silica = spec.silica_fume_pct / 100.0_f32;
    let cementish = (1.0_f32 - fly - silica).max(1e-6_f32);
    let denom = w_c + cementish + fly + silica;

    let mut row = vec![0.0_f32; 8];
    row[0] = w_c / denom;
    row[1] = cementish / denom;
    row[5] = silica / denom;
    row[6] = fly / denom;

    let fractions = Tensor::<CliBackend, 2>::from_data(Data::new(row, Shape::new([1, 8])), device);
    Ok(MixTensor { fractions })
}

fn hydration_degree_from_mix(
    mix: &MixTensor<CliBackend>,
    spec: &MixSpec,
    device: &<CliBackend as Backend>::Device,
) -> Result<Tensor<CliBackend, 2>, CliError> {
    let batch = mix.fractions.dims()[0];
    let cement = mix.fractions.clone().slice([0..batch, 1..2]);
    let slag = mix.fractions.clone().slice([0..batch, 5..6]);
    let fly_ash = mix.fractions.clone().slice([0..batch, 6..7]);

    let binder = cement
        .clone()
        .add(slag.clone())
        .add(fly_ash.clone())
        .clamp_min(1e-6_f32);
    let scm_ratio = slag.add(fly_ash).div(binder.clone());

    let alpha_max = scm_ratio.clone().mul_scalar(-0.15_f32).add_scalar(0.95_f32);
    let k_ref = 0.55_f32;

    let t_ref_k = 293.15_f32;
    let temp_c = spec.temperature_k.value() - 273.15_f32;
    let temperature_c_tensor = Tensor::<CliBackend, 2>::full([batch, 1], temp_c, device);
    let t_k = temperature_c_tensor.add_scalar(273.15_f32);
    let e_over_r = 5_000.0_f32;

    let inv_t_ref = 1.0_f32 / t_ref_k;
    let inv_t = t_k.powf_scalar(-1.0_f32);
    let temp_factor = inv_t
        .mul_scalar(-1.0_f32)
        .add_scalar(inv_t_ref)
        .mul_scalar(e_over_r)
        .exp();

    let scm_factor = scm_ratio.mul_scalar(-0.4_f32).add_scalar(1.0_f32);
    let k = temp_factor.mul(scm_factor).mul_scalar(k_ref);

    let age_days =
        Tensor::<CliBackend, 2>::full([batch, 1], spec.target_age_hours / 24.0_f32, device);
    let age_sqrt = age_days.sqrt();
    let decay = k.mul(age_sqrt).mul_scalar(-1.0_f32).exp();
    let alpha = alpha_max.mul(decay.mul_scalar(-1.0_f32).add_scalar(1.0_f32));

    Ok(alpha.clamp(0.0_f32, 1.0_f32))
}

fn masses_from_spec(
    spec: &MixSpec,
    device: &<CliBackend as Backend>::Device,
) -> Result<MassBlock4, CliError> {
    let cement_kg_m3 = 350.0_f32;
    let water_kg_m3 = cement_kg_m3 * spec.w_c.value();
    let agg_vol = spec.aggregate_volume_fraction.clamp(0.0_f32, 0.85_f32);
    let scm_mass = cement_kg_m3 * (spec.fly_ash_pct + spec.silica_fume_pct) / 100.0_f32;
    let cement_net = (cement_kg_m3 - scm_mass).max(10.0_f32);
    let agg_kg_m3 = 2_600.0_f32 * agg_vol;

    Ok((
        tensor_fill_4d(cement_net, device),
        tensor_fill_4d(scm_mass.max(0.0_f32), device),
        tensor_fill_4d(agg_kg_m3, device),
        tensor_fill_4d(water_kg_m3, device),
    ))
}

fn minimal_manifold_from_spec(
    spec: &MixSpec,
    device: &<CliBackend as Backend>::Device,
) -> Result<UnifiedMaterialStateTensor<CliBackend>, CliError> {
    let n = 2_usize;
    let temp_c = spec.temperature_k.value() - 273.15_f32;

    let coords_data: Vec<i64> = vec![0; n * 5];
    let coords =
        Tensor::<CliBackend, 2, Int>::from_data(Data::new(coords_data, Shape::new([n, 5])), device);

    let edges = Tensor::<CliBackend, 2, Int>::from_data(
        Data::new(vec![0_i64, 1_i64], Shape::new([2, 1])),
        device,
    );

    let faces = Tensor::<CliBackend, 2, Int>::from_data(
        Data::new(vec![0_i64, 0_i64], Shape::new([2, 1])),
        device,
    );

    let mut sf = vec![0.0_f32; n * 8];
    for i in 0..n {
        let base = i * 8;
        sf[base + 3] = temp_c;
        sf[base + 4] = 0.0_f32;
    }
    let scalar_features =
        Tensor::<CliBackend, 2>::from_data(Data::new(sf, Shape::new([n, 8])), device);

    let vector_features = Tensor::<CliBackend, 3>::zeros([n, 1, 3], device);
    let matrix_features = Tensor::<CliBackend, 4>::zeros([n, 1, 3, 3], device);

    Ok(UnifiedMaterialStateTensor {
        coords,
        edges_b1: edges,
        faces_b2: faces,
        scalar_features,
        vector_features,
        matrix_features,
        resolution_mm: [1.0_f32, 1.0_f32, 1.0_f32],
    })
}

fn tensor_mean_scalar(t: Tensor<CliBackend, 2>) -> Result<f32, CliError> {
    let m = t.mean();
    Ok(m.into_scalar())
}

fn tensor_mean_scalar_nd(t: Tensor<CliBackend, 4>) -> Result<f32, CliError> {
    let m = t.mean();
    Ok(m.into_scalar())
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
