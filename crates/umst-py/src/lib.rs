// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

#![allow(clippy::doc_lazy_continuation, clippy::useless_conversion)]

//! PyO3 extension: `predict`, `audit`, `certify`, and SSOT schema snippets (transport only).

use csv::WriterBuilder;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyIterator, PyList, PyModule};
use serde_json::{Map, Value};
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

/// Column order aligned with `datasets/dataset_d1.csv` (audit CSV contract).
const AUDIT_CSV_HEADER: &[&str] = &[
    "cement",
    "slag",
    "fly_ash",
    "water",
    "superplasticizer",
    "coarse_agg",
    "fine_agg",
    "age",
    "strength",
    "source",
    "temperature",
    "humidity",
];

fn json_value_to_csv_cell(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(_) | Value::Object(_) => v.to_string(),
    }
}

fn dict_row_to_audit_record(obj: &Map<String, Value>) -> Vec<String> {
    AUDIT_CSV_HEADER
        .iter()
        .map(|h| obj.get(*h).map(json_value_to_csv_cell).unwrap_or_default())
        .collect()
}

fn rows_to_audit_csv(py: Python<'_>, rows: &Bound<'_, PyAny>) -> PyResult<String> {
    let iter = PyIterator::from_bound_object(rows)?;
    let mut wtr = WriterBuilder::new().flexible(true).from_writer(Vec::new());
    wtr.write_record(AUDIT_CSV_HEADER)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    for item_result in iter {
        let item = item_result?;
        let s = json_dumps(py, &item)?;
        let v: Value =
            serde_json::from_str(s.trim()).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let obj = v.as_object().ok_or_else(|| {
            PyValueError::new_err("audit_rows: each row must be a JSON object (dict)")
        })?;
        wtr.write_record(dict_row_to_audit_record(obj))
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
    }
    wtr.flush()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let buf = wtr
        .into_inner()
        .map_err(|e| PyValueError::new_err(format!("{e}")))?;
    String::from_utf8(buf).map_err(|e| PyValueError::new_err(e.to_string()))
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
/// formal_anchor_rationale: Encodes iterable of row dicts into dataset-style CSV then reuses **`audit_csv_buf`** (aligned with **`audit`** string path).
#[pyfunction]
#[pyo3(signature = (rows, *, profile="default", limit=None))]
pub fn audit_rows(
    py: Python<'_>,
    rows: Bound<'_, PyAny>,
    profile: &str,
    limit: Option<usize>,
) -> PyResult<Py<PyDict>> {
    let csv_text = rows_to_audit_csv(py, &rows)?;
    let profile = profile.to_string();
    let prof = umst_concrete_cartridge::calibration::Profile::load_bundled(&profile)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let v =
        audit_csv_buf(&prof, &csv_text, limit).map_err(|e| PyValueError::new_err(e.to_string()))?;
    value_to_py_dict(py, &v)
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
    let bytes = canonical_json_bytes(&v).map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(pyo3::types::PyBytes::new_bound(py, &bytes))
}

#[cfg(feature = "agent-layer")]
/// formal_anchor: lean://umst-formal/Lean/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
/// formal_anchor_rationale: Python transport over [`gate_check_mix`]; admissibility SSOT on manifold gate.
/// Gate-check mix_spec JSON; returns admissibility summary dict.
#[pyfunction]
#[pyo3(signature = (mix, *, profile="default"))]
pub fn gate_check(py: Python<'_>, mix: Bound<'_, PyAny>, profile: &str) -> PyResult<Py<PyDict>> {
    use umst_concrete_cartridge::research::{gate_check_mix, GateSummary};
    let mix_val = value_from_py_mix(py, &mix)?;
    let prof = umst_concrete_cartridge::calibration::Profile::load_bundled(profile)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let summary: GateSummary = gate_check_mix(&prof, &mix_val);
    let v = serde_json::to_value(&summary).map_err(|e| PyValueError::new_err(e.to_string()))?;
    value_to_py_dict(py, &v)
}

#[cfg(feature = "agent-layer")]
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Python transport over pure [`query`] filter; no new physical claim.
/// Query research memory with optional filters (in-memory store per call).
#[pyfunction]
#[pyo3(signature = (*, admissible_only=true, curing_regime=None, limit=None))]
pub fn memory_query(
    py: Python<'_>,
    admissible_only: bool,
    curing_regime: Option<&str>,
    limit: Option<usize>,
) -> PyResult<Py<PyList>> {
    use umst_concrete_cartridge::research::{query, MemoryQuery, ResearchStore};
    let q = MemoryQuery {
        admissible_only,
        curing_regime: curing_regime.map(str::to_string),
        limit,
        ..Default::default()
    };
    let rows = query(&ResearchStore::default(), &q);
    let list = PyList::empty_bound(py);
    for row in rows {
        let v = serde_json::to_value(&row).map_err(|e| PyValueError::new_err(e.to_string()))?;
        list.append(value_to_py_dict(py, &v)?)?;
    }
    Ok(list.into())
}

#[cfg(feature = "agent-layer")]
/// formal_anchor: lean://umst-formal/Lean/Gate.lean#Admissible
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// catalog_id: umst.gate.cd_transition
/// formal_anchor_rationale: Python transport over [`accept`]; gate re-check before memory append.
/// Contribute contribution.v1 JSON into in-memory research store.
#[pyfunction]
#[pyo3(signature = (contribution, *, profile="default"))]
pub fn contribute(
    py: Python<'_>,
    contribution: Bound<'_, PyAny>,
    profile: &str,
) -> PyResult<Py<PyDict>> {
    use umst_concrete_cartridge::research::{
        accept, GateContext, ProvenanceClock, ResearchStore, WallClock,
    };
    let c_val = value_from_py_mix(py, &contribution)?;
    let prof = umst_concrete_cartridge::calibration::Profile::load_bundled(profile)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let ctx = GateContext { profile: &prof };
    let (store, _clock, result) = accept(
        ResearchStore::default(),
        ProvenanceClock::default(),
        WallClock,
        &ctx,
        &c_val,
    )
    .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let _ = store;
    let v = serde_json::to_value(&result).map_err(|e| PyValueError::new_err(e.to_string()))?;
    value_to_py_dict(py, &v)
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Thin extension entry; façade and CLI layers carry formal blocks.
#[pymodule]
fn _umst_concrete_cartridge(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(predict, m)?)?;
    m.add_function(wrap_pyfunction!(audit, m)?)?;
    m.add_function(wrap_pyfunction!(audit_rows, m)?)?;
    m.add_function(wrap_pyfunction!(certify, m)?)?;
    m.add_function(wrap_pyfunction!(schema, m)?)?;
    m.add_function(wrap_pyfunction!(bundled_profile_ids, m)?)?;
    m.add_function(wrap_pyfunction!(canonical_json, m)?)?;
    #[cfg(feature = "agent-layer")]
    {
        m.add_function(wrap_pyfunction!(gate_check, m)?)?;
        m.add_function(wrap_pyfunction!(memory_query, m)?)?;
        m.add_function(wrap_pyfunction!(contribute, m)?)?;
    }
    m.add(
        "__doc__",
        "UMST concrete cartridge Python extension (predict, audit, certify, schema).",
    )?;
    Ok(())
}
