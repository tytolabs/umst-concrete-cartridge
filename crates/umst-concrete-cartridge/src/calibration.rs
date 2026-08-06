// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Bundled calibration profiles (`calibration/*.toml`): provenance, regime bounds, Powers parameters,
//! and verification contract metadata lifted from prototype-2a (Zenodo 18940933) SSOT JSON (see [`calibration/SCHEMA.md`](../../../calibration/SCHEMA.md)).

use crate::calibration_fit::RheologyCalibrationBlock;
use crate::pipeline::cast_phase::CastLifecycleThresholds;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Ordered manifest of bundled profile ids for `include_str!` routing.
///
/// Ordered names of bundled [`Profile`] artefacts embedded via [`include_str!`].
pub const BUNDLED_PROFILE_IDS: &[&str] = &[
    "default",
    "jennings_gel_space",
    "uci_d1",
    "zenodo_ndt",
    "zenodo_sonreb",
    "zenodo_rh",
    "uhpc",
    "highscm",
    "selfheal",
    "tyto_mortar",
];

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Exhaustive serde enum over calibrated homogeneous model kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    PowersGelSpace,
    JenningsGelSpace,
}

/// formal_anchor: lean://umst-formal/Lean/Concrete/Powers.lean#PowersState
/// catalog_id: thermodynamic_mix
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PowersGelParameters {
    pub s_intrinsic: f64,
    pub k_slag: f64,
    pub k_fly_ash: f64,
    pub k_ref: f64,
    pub early_boost: f64,
}

/// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
/// catalog_id: umst.gate.cd_transition
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
#[derive(Debug, Clone, Deserialize)]
pub struct FormalBlock {
    pub anchor: String,
    pub status: String,
    #[serde(default)]
    pub axioms: Vec<String>,
    #[serde(default)]
    pub rationale: Option<String>,
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Serde lift of TOML `[provenance.formal]`; `status` string is file metadata (may include Boundary scope), not a Rust `formal_status` bucket.
#[derive(Debug, Clone, Deserialize)]
pub struct ProvenanceFormal {
    pub anchor: String,
    pub status: String,
    #[serde(default)]
    pub axioms: Vec<String>,
    #[serde(default)]
    pub rationale: Option<String>,
}

/// formal_anchor: lean://umst-formal/Lean/Concrete/Powers.lean#S_intrinsic
/// catalog_id: thermodynamic_mix
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
#[derive(Debug, Clone, Deserialize)]
pub struct CalibrationMeta {
    pub name: String,
    pub schema: String,
    pub material: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub authors: Option<String>,
    #[serde(default)]
    pub date_fit: Option<String>,
    #[serde(default)]
    pub source_repo: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_axioms: NONE
/// formal_anchor_rationale: Dataset and Zenodo citation bundle parsed from TOML only; no Lean witness on this serde container — see `docs/FormalAnchors.md` “Future formal links” for manifold adjoint context.
pub struct CalibrationProvenance {
    #[serde(default)]
    pub dataset_lift_from: Option<String>,
    pub provenance_sha256: String,
    #[serde(default)]
    pub primary_reference: Option<String>,
    #[serde(default)]
    pub secondary_references: Vec<String>,
    #[serde(default)]
    pub formal: Option<ProvenanceFormal>,
    #[serde(default)]
    pub zenodo_record: Option<String>,
    #[serde(default)]
    pub zenodo_doi: Option<String>,
    #[serde(default)]
    pub zenodo_url: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub subset: Option<String>,
}

/// formal_anchor: lean://umst-formal/Lean/OrderStatisticsBand.lean#order_statistic_concentration
/// catalog_id: umst.cartridge.concrete.acceptance_band
/// formal_status: Mechanised
/// formal_axioms: NONE
#[derive(Debug, Clone, Deserialize)]
pub struct RegimeBounds {
    pub w_c_min: f64,
    pub w_c_max: f64,
    pub temperature_k_min: f64,
    pub temperature_k_max: f64,
    pub age_hours_min: f64,
    pub age_hours_max: f64,
    #[serde(default)]
    pub fly_ash_pct_max: Option<f64>,
    #[serde(default)]
    pub silica_fume_pct_max: Option<f64>,
    #[serde(default)]
    pub slag_pct_max: Option<f64>,
    /// When set (e.g. HIGHSCM), SCM sum must exceed this fraction of binder [%].
    #[serde(default)]
    pub scm_sum_min_pct: Option<f64>,
    /// Specialty: UHPC silica fume cap [%].
    #[serde(default)]
    pub silica_fume_pct_max_special: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_axioms: NONE
/// formal_anchor_rationale: Dispatch metadata only; Jennings gel-space path returns `HomogeneousError::JenningsNotImplemented` until operator boards CC-P-JENNINGS (see `archived/residuals/misc-outputs-tmp/JENNINGS_RESIDUAL_2252.md` TODO-M3-002).
pub struct CalibrationModelSection {
    pub kind: ModelKind,
}

/// formal_anchor: lean://umst-formal/Lean/OrderStatisticsBand.lean#p25_p75_admissibility
/// catalog_id: umst.cartridge.concrete.acceptance_band
/// formal_status: Mechanised
/// formal_axioms: NONE
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AcceptanceBlock {
    pub strength_mae_max: Option<f64>,
    pub strength_rmse_max: Option<f64>,
    pub strength_r2_min: Option<f64>,
    pub strength_max_err_max: Option<f64>,
    #[serde(default)]
    pub formal_anchor: Option<String>,
    #[serde(default, rename = "formal_status")]
    pub acceptance_bucket: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_axioms: NONE
/// formal_anchor_rationale: Contract metadata (`verification_status`); hyperbox regime warnings are soundness-witnessed on `regime_check_scalars` — see RegimeSoundness anchor there.
pub struct ContractBlock {
    pub verification_status: String,
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Parsed TOML aggregate routed by `bundle_id`; field invariants delegated to nested serde structs.
///
/// Calibration profile: parsed TOML plus bundle id used for homogeneous routing branches.
#[derive(Debug, Clone)]
pub struct Profile {
    pub bundle_id: String,
    pub meta: CalibrationMeta,
    pub provenance: CalibrationProvenance,
    pub regime: RegimeBounds,
    pub model_section: CalibrationModelSection,
    pub powers: PowersGelParameters,
    pub acceptance: AcceptanceBlock,
    pub contract: ContractBlock,
    /// Optional single-mix τ₀ bias (`[rheology_calibration]` in TOML).
    pub rheology_calibration: Option<RheologyCalibrationBlock>,
    /// Hydration α thresholds for [`crate::pipeline::cast_phase::classify_cast_phase`] (MP3.1).
    pub cast_lifecycle: CastLifecycleThresholds,
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Named-field regime violation records for CLI warning strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegimeViolation {
    pub field: &'static str,
    pub message: String,
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_axioms: NONE
/// formal_anchor_rationale: Bundled profile IO and TOML parse failures only; DEC mass-conservation witness belongs on the manifold Laplacian — see `docs/FormalAnchors.md` “Future formal links”.
#[derive(Debug)]
pub enum CalibrationError {
    UnknownBundledProfile(String),
    Io(std::io::Error),
    TomlDeserialize(toml::de::Error),
}

impl fmt::Display for CalibrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownBundledProfile(name) => write!(f, "unknown bundled profile `{name}`"),
            Self::Io(e) => write!(f, "{e}"),
            Self::TomlDeserialize(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for CalibrationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::TomlDeserialize(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CalibrationError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::de::Error> for CalibrationError {
    fn from(e: toml::de::Error) -> Self {
        Self::TomlDeserialize(e)
    }
}

impl Profile {
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Bundled `include_str!` loader with normalized bundle id validation.
    pub fn load_bundled(name: &str) -> Result<Self, CalibrationError> {
        let txt = bundled_toml_source(name)?;
        Self::parse_toml(bundle_id_normalized(name)?, txt)
    }

    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Filesystem path IO for non-bundled TOML; parse errors surface as CalibrationError.
    pub fn load_from_path(bundle_id: &str, path: &Path) -> Result<Self, CalibrationError> {
        let txt = fs::read_to_string(path)?;
        Self::parse_toml(bundle_id.trim().to_ascii_lowercase(), &txt)
    }

    fn parse_toml(bundle_id: String, txt: &str) -> Result<Self, CalibrationError> {
        let raw: TomlCalibration = toml::from_str(txt)?;
        Ok(Self::from_parts(bundle_id, raw))
    }

    pub(crate) fn from_parts(bundle_id: String, raw: TomlCalibration) -> Self {
        let powers = match raw.model.kind {
            ModelKind::PowersGelSpace => raw
                .parameters
                .powers_gel_space
                .clone()
                .expect("powers profile must carry [parameters.powers_gel_space]"),
            ModelKind::JenningsGelSpace => {
                raw.parameters
                    .powers_gel_space
                    .clone()
                    .unwrap_or(PowersGelParameters {
                        s_intrinsic: 0.0,
                        k_slag: 0.0,
                        k_fly_ash: 0.0,
                        k_ref: 0.0,
                        early_boost: 1.0,
                    })
            }
        };
        Self {
            meta: raw.meta,
            provenance: raw.provenance,
            regime: raw.regime,
            model_section: raw.model,
            powers,
            acceptance: raw.acceptance,
            contract: raw.contract,
            rheology_calibration: raw.rheology_calibration,
            cast_lifecycle: raw.cast_lifecycle,
            bundle_id,
        }
    }

    /// formal_anchor: lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime
    /// catalog_id: umst.cartridge.concrete.regime
    /// formal_status: Mechanised
    /// formal_axioms: NONE
    ///
    /// Returns non-empty iff inputs lie outside this profile regime (for warnings, not refusal).
    #[must_use]
    pub fn regime_check_scalars(
        &self,
        w_c: f32,
        temperature_k: f32,
        age_hours: f32,
        fly_ash_pct: f32,
        silica_fume_pct: f32,
    ) -> Vec<RegimeViolation> {
        let mut out = Vec::new();
        let w = f64::from(w_c);
        let t_k = f64::from(temperature_k);
        let age = f64::from(age_hours);
        let fa = f64::from(fly_ash_pct);
        let sf = f64::from(silica_fume_pct);
        let r = &self.regime;
        if w < r.w_c_min || w > r.w_c_max {
            out.push(RegimeViolation {
                field: "w_c",
                message: format!("w_c={} outside [{} , {}]", w, r.w_c_min, r.w_c_max),
            });
        }
        if t_k < r.temperature_k_min || t_k > r.temperature_k_max {
            out.push(RegimeViolation {
                field: "temperature_k",
                message: format!(
                    "temperature_k={} outside [{} , {}]",
                    t_k, r.temperature_k_min, r.temperature_k_max
                ),
            });
        }
        if age < r.age_hours_min || age > r.age_hours_max {
            out.push(RegimeViolation {
                field: "target_age_hours",
                message: format!(
                    "target_age_hours={} outside [{} , {}]",
                    age, r.age_hours_min, r.age_hours_max
                ),
            });
        }
        if let Some(m) = r.fly_ash_pct_max {
            if fa > m {
                out.push(RegimeViolation {
                    field: "fly_ash_pct",
                    message: format!("fly_ash_pct={fa} exceeds {m}"),
                });
            }
        }
        if let Some(m) = r.silica_fume_pct_max {
            if sf > m {
                out.push(RegimeViolation {
                    field: "silica_fume_pct",
                    message: format!("silica_fume_pct={sf} exceeds {m}"),
                });
            }
        }
        if let Some(m) = r.silica_fume_pct_max_special {
            if sf > m {
                out.push(RegimeViolation {
                    field: "silica_fume_pct",
                    message: format!("silica_fume_pct={sf} exceeds UHPC cap {m}"),
                });
            }
        }
        if let Some(min_scm) = r.scm_sum_min_pct {
            if fa + sf < min_scm {
                out.push(RegimeViolation {
                    field: "scm_pct",
                    message: format!(
                        "fly_ash_pct+silica_fume_pct={} below required {}%",
                        fa + sf,
                        min_scm
                    ),
                });
            }
        }
        out
    }
}

/// formal_anchor: lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime
/// catalog_id: umst.cartridge.concrete.regime
/// formal_status: Mechanised
/// formal_axioms: NONE
///
/// True iff at least one bundled profile has an empty regime check for the given scalars.
#[must_use]
pub fn any_bundled_profile_covers_scalars(
    w_c: f32,
    temperature_k: f32,
    age_hours: f32,
    fly_ash_pct: f32,
    silica_fume_pct: f32,
) -> bool {
    BUNDLED_PROFILE_IDS.iter().any(|id| {
        Profile::load_bundled(id)
            .map(|p| {
                p.regime_check_scalars(w_c, temperature_k, age_hours, fly_ash_pct, silica_fume_pct)
                    .is_empty()
            })
            .unwrap_or(false)
    })
}

/// Lowest bundled `temperature_k_min` [K] across [`BUNDLED_PROFILE_IDS`].
#[must_use]
pub fn bundled_union_temperature_floor_k() -> f32 {
    BUNDLED_PROFILE_IDS
        .iter()
        .filter_map(|id| Profile::load_bundled(id).ok())
        .map(|p| p.regime.temperature_k_min as f32)
        .fold(f32::INFINITY, f32::min)
}

/// Finding-only three-lens regime posture — **no gate verdict change**.
///
/// Surfaces union envelope vs `MixSpec` wire floor without routing through
/// compose-delegate admissibility (see `RESEARCH_270K_REGIME_2218.md`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegimeUnionDiagnostic {
    /// `any_bundled_profile_covers_scalars` (canonical / predict path).
    pub union_covers: bool,
    /// `MixSpec` wire floor \[273, 353\] K.
    pub wire_temperature_ok: bool,
    /// Lowest bundled `temperature_k_min` [K].
    pub union_floor_k: f32,
    /// `union_floor_k - temperature_k` when below floor; else 0.
    pub gap_k_below_union: f32,
    /// Canonical gate would reject (`!union_covers`).
    pub canonical_regime_reject: bool,
}

/// Three-lens regime diagnostic for operator probes (finding-only).
#[must_use]
pub fn regime_union_diagnostic(
    w_c: f32,
    temperature_k: f32,
    age_hours: f32,
    fly_ash_pct: f32,
    silica_fume_pct: f32,
) -> RegimeUnionDiagnostic {
    let union_covers = any_bundled_profile_covers_scalars(
        w_c,
        temperature_k,
        age_hours,
        fly_ash_pct,
        silica_fume_pct,
    );
    let union_floor_k = bundled_union_temperature_floor_k();
    let wire_temperature_ok = (273.0..=353.0).contains(&temperature_k);
    let gap_k_below_union = if temperature_k < union_floor_k {
        union_floor_k - temperature_k
    } else {
        0.0
    };
    RegimeUnionDiagnostic {
        union_covers,
        wire_temperature_ok,
        union_floor_k,
        gap_k_below_union,
        canonical_regime_reject: !union_covers,
    }
}

#[cfg(test)]
mod six_mix_temp_regime_probe {
    use super::{
        any_bundled_profile_covers_scalars, bundled_union_temperature_floor_k,
        regime_union_diagnostic,
    };

    /// Six-mix row-6 DRAFT scalars — w/c=0.45, age=672 h, SCM=0.
    const W_C: f32 = 0.45;
    const AGE_H: f32 = 672.0;

    /// T=277 K — library PASS (specialty profiles min 273 K cover); finding-only @ 2102.
    #[test]
    fn bundled_union_covers_t277k_six_mix_draft() {
        assert!(
            any_bundled_profile_covers_scalars(W_C, 277.0, AGE_H, 0.0, 0.0),
            "T=277 K must be in-box for at least one bundled profile (highscm/zenodo class)"
        );
    }

    /// T=270 K — below every bundled `temperature_k_min` (lowest 273 K); finding-only @ 2102.
    #[test]
    fn bundled_union_rejects_t270k_six_mix_draft() {
        assert!(
            !any_bundled_profile_covers_scalars(W_C, 270.0, AGE_H, 0.0, 0.0),
            "T=270 K must fall outside all bundled profile hyperboxes"
        );
    }

    /// T=270 K — three-lens gap witness (union REJECT · wire REJECT · compose-delegate PASS).
    #[test]
    fn six_mix_t270k_regime_gap_diagnostic_witness() {
        assert_eq!(bundled_union_temperature_floor_k(), 273.0);
        let d = regime_union_diagnostic(W_C, 270.0, AGE_H, 0.0, 0.0);
        assert!(!d.union_covers);
        assert!(!d.wire_temperature_ok);
        assert!((d.gap_k_below_union - 3.0).abs() < f32::EPSILON);
        assert!(d.canonical_regime_reject);
    }

    /// T=277 K — union PASS aligns with wire OK (compose-delegate also PASS).
    #[test]
    fn six_mix_t277k_regime_diagnostic_union_pass() {
        let d = regime_union_diagnostic(W_C, 277.0, AGE_H, 0.0, 0.0);
        assert!(d.union_covers);
        assert!(d.wire_temperature_ok);
        assert!(!d.canonical_regime_reject);
        assert_eq!(d.gap_k_below_union, 0.0);
    }
}

fn bundle_id_normalized(name: &str) -> Result<String, CalibrationError> {
    let s = name.trim().to_ascii_lowercase();
    if BUNDLED_PROFILE_IDS.contains(&s.as_str()) {
        Ok(s)
    } else {
        Err(CalibrationError::UnknownBundledProfile(name.to_string()))
    }
}

fn bundled_toml_source(name: &str) -> Result<&'static str, CalibrationError> {
    let id = bundle_id_normalized(name)?;
    let s = match id.as_str() {
        "default" => include_str!("../../../calibration/profiles/default.v1.toml"),
        "jennings_gel_space" => {
            include_str!("../../../calibration/profiles/jennings_gel_space.v1.toml")
        }
        "uci_d1" => include_str!("../../../calibration/profiles/uci_d1.v1.toml"),
        "zenodo_ndt" => include_str!("../../../calibration/profiles/zenodo_ndt.v1.toml"),
        "zenodo_sonreb" => include_str!("../../../calibration/profiles/zenodo_sonreb.v1.toml"),
        "zenodo_rh" => include_str!("../../../calibration/profiles/zenodo_rh.v1.toml"),
        "uhpc" => include_str!("../../../calibration/profiles/uhpc.v1.toml"),
        "highscm" => include_str!("../../../calibration/profiles/highscm.v1.toml"),
        "selfheal" => include_str!("../../../calibration/profiles/selfheal.v1.toml"),
        "tyto_mortar" => include_str!("../../../calibration/profiles/tyto_mortar.v1.toml"),
        _ => return Err(CalibrationError::UnknownBundledProfile(name.to_string())),
    };
    Ok(s)
}

#[derive(Debug, Deserialize)]
pub(crate) struct TomlCalibration {
    meta: CalibrationMeta,
    provenance: CalibrationProvenance,
    regime: RegimeBounds,
    model: CalibrationModelSection,
    parameters: ParametersSection,
    #[serde(default)]
    acceptance: AcceptanceBlock,
    contract: ContractBlock,
    #[serde(default)]
    rheology_calibration: Option<RheologyCalibrationBlock>,
    #[serde(default)]
    cast_lifecycle: CastLifecycleThresholds,
}

#[derive(Debug, Deserialize)]
struct ParametersSection {
    #[serde(rename = "powers_gel_space")]
    powers_gel_space: Option<PowersGelParameters>,
}

impl fmt::Display for RegimeViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Static HashMap of tab-separated CLI profile blurbs (human-readable only).
#[must_use]
pub fn profile_descriptions() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert(
        "default",
        "generic OPC fallback (Contract, synthetic anchor)",
    );
    m.insert(
        "jennings_gel_space",
        "Jennings–Brownyard gel-space homogeneous witness (Boundary, G0 pins)",
    );
    m.insert("uci_d1", "Yeh 1998 UCI concrete strength (D1, 1030 rows)");
    m.insert(
        "zenodo_ndt",
        "Zenodo 14921019 NDT subset (dataset_d2.csv, TU/e + TNO)",
    );
    m.insert(
        "zenodo_sonreb",
        "Zenodo 14921019 SonReb subset (dataset_d3.csv, TU/e + TNO)",
    );
    m.insert(
        "zenodo_rh",
        "Zenodo 14921019 RH subset (dataset_d4.csv, TU/e + TNO)",
    );
    m.insert("uhpc", "Ultra-high performance concrete (Boundary)");
    m.insert("highscm", "High SCM blends (Contract on paired CSV)");
    m.insert("selfheal", "Self-healing specialty mix (Boundary)");
    m.insert(
        "tyto_mortar",
        "Tyto S1 proxy-rheology mortar (θ calibration, printable window)",
    );
    m
}
