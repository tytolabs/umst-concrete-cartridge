// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

#![allow(clippy::doc_lazy_continuation, clippy::useless_conversion)]

//! PyO3 extension: `predict`, `audit`, `certify`, and SSOT schema snippets (transport only).

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyModule};
use serde_json::Value;
use umst_cli::audit::audit_csv_buf;
use umst_cli::canonical::canonical_json_bytes;
use umst_cli::cli::{
    certify_profile_json, mix_spec_from_json_value, predict_with_options, serialize_prediction,
    MixSpec, PredictOptions, PredictionWireVersion,
};
use umst_concrete_cartridge::calibration::BUNDLED_PROFILE_IDS;
use umst_concrete_cartridge::facade::{
    schema_audit_v1_json, schema_mix_v1_json, schema_result_v1_json, schema_result_v2_json,
};

fn json_dumps(py: Python<'_>, obj: &Bound<'_, PyAny>) -> PyResult<String> {
    let json_mod = PyModule::import_bound(py, "json")?;
    let dumps = json_mod.getattr("dumps")?;
    dumps.call1((obj,))?.extract()
}

fn value_from_py_mix(py: Python<'_>, mix: &Bound<'_, PyAny>) -> PyResult<Value> {
    let s = if let Ok(txt) = mix.extract::<String>() {
        txt
    } else {
        json_dumps(py, mix)?
    };
    serde_json::from_str(s.trim()).map_err(|e| PyValueError::new_err(e.to_string()))
}

fn wire_from_str(schema_version: &str) -> PyResult<PredictionWireVersion> {
    match schema_version {
        "v1" | "V1" => Ok(PredictionWireVersion::V1),
        "v2" | "V2" => Ok(PredictionWireVersion::V2),
        other => Err(PyValueError::new_err(format!(
            "schema_version must be 'v1' or 'v2', got `{other}`"
        ))),
    }
}

fn value_to_py_dict(py: Python<'_>, v: &Value) -> PyResult<Py<PyDict>> {
    let json_mod = PyModule::import_bound(py, "json")?;
    let loads = json_mod.getattr("loads")?;
    let s = serde_json::to_string(v).map_err(|e| PyValueError::new_err(e.to_string()))?;
    let dict = loads.call1((s,))?;
    let dict = dict.downcast_into::<PyDict>()?;
    Ok(dict.unbind())
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Python transport wrapper over **[`predict_with_options`]**; anchored on facade predict path.
#[pyfunction]
#[pyo3(signature = (spec, *, profile="default", schema_version="v2"))]
pub fn predict(
    py: Python<'_>,
    spec: Bound<'_, PyAny>,
    profile: &str,
    schema_version: &str,
) -> PyResult<Py<PyDict>> {
    let mix_val = value_from_py_mix(py, &spec)?;
    let profile = profile.to_string();
    let wire = wire_from_str(schema_version)?;
    let mut mix_spec: MixSpec =
        mix_spec_from_json_value(mix_val).map_err(|e| PyValueError::new_err(e.to_string()))?;
    mix_spec.profile_name = profile.clone();
    let prof = umst_concrete_cartridge::calibration::Profile::load_bundled(&profile)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let bundle = predict_with_options(
        &prof,
        &mix_spec,
        PredictOptions {
            compare_homogeneous: false,
        },
    )
    .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let out =
        serialize_prediction(&bundle, wire).map_err(|e| PyValueError::new_err(e.to_string()))?;
    value_to_py_dict(py, &out)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Python transport over CLI audit glue; no extra physical claim beyond CSV→facade audit.
#[pyfunction]
#[pyo3(signature = (profile, csv_text, limit=None))]
pub fn audit(
    py: Python<'_>,
    profile: &str,
    csv_text: String,
    limit: Option<usize>,
) -> PyResult<Py<PyDict>> {
    let profile = profile.to_string();
    let prof = umst_concrete_cartridge::calibration::Profile::load_bundled(&profile)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let v =
        audit_csv_buf(&prof, &csv_text, limit).map_err(|e| PyValueError::new_err(e.to_string()))?;
    value_to_py_dict(py, &v)
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Dict view of **[`certify_profile_json`]**; structural mirror of CLI `umst certify`.
#[pyfunction]
pub fn certify(py: Python<'_>, profile: &str) -> PyResult<Py<PyDict>> {
    let profile = profile.to_string();
    let p = umst_concrete_cartridge::calibration::Profile::load_bundled(&profile)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let v = certify_profile_json(&p);
    value_to_py_dict(py, &v)
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: SSOT schema text from facade `include_str!` for notebooks and packaging checks.
#[pyfunction]
pub fn schema(kind: &str) -> PyResult<String> {
    let s = match kind {
        "mix" | "mix.v1" => schema_mix_v1_json(),
        "result" | "result.v1" => schema_result_v1_json(),
        "result_v2" | "result.v2" => schema_result_v2_json(),
        "audit" | "audit.v1" => schema_audit_v1_json(),
        other => {
            return Err(PyValueError::new_err(format!(
                "unknown schema kind `{other}` (mix, result, result_v2, audit)"
            )));
        }
    };
    Ok(s.to_string())
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Bundled id manifest for packaging smoke tests.
#[pyfunction]
pub fn bundled_profile_ids(py: Python<'_>) -> PyResult<Py<PyList>> {
    let list = PyList::empty_bound(py);
    for id in BUNDLED_PROFILE_IDS {
        list.append(id)?;
    }
    Ok(list.into())
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Byte-stable JSON for golden tests; matches **`umst-canonical`** binary.
#[pyfunction]
pub fn canonical_json<'py>(
    py: Python<'py>,
    obj: Bound<'_, PyAny>,
) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
    let s = json_dumps(py, &obj)?;
    let v: Value = serde_json::from_str(&s).map_err(|e| PyValueError::new_err(e.to_string()))?;
    let bytes =
        canonical_json_bytes(&v).map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(pyo3::types::PyBytes::new_bound(py, &bytes))
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Thin extension entry; façade and CLI layers carry formal blocks.
#[pymodule]
fn _umst_concrete_cartridge(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(predict, m)?)?;
    m.add_function(wrap_pyfunction!(audit, m)?)?;
    m.add_function(wrap_pyfunction!(certify, m)?)?;
    m.add_function(wrap_pyfunction!(schema, m)?)?;
    m.add_function(wrap_pyfunction!(bundled_profile_ids, m)?)?;
    m.add_function(wrap_pyfunction!(canonical_json, m)?)?;
    m.add(
        "__doc__",
        "UMST concrete cartridge Python extension (predict, audit, certify, schema).",
    )?;
    Ok(())
}
