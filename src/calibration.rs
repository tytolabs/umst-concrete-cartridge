// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Bundled calibration profiles (`calibration/*.toml`): provenance, regime bounds, Powers parameters,
//! and verification contract metadata lifted from prototype-3 SSOT JSON (see [`calibration/SCHEMA.md`](../../calibration/SCHEMA.md)).

use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// formal_anchor: lean://umst-formal/Lean/Constitutional.lean#KleisliArrow
/// formal_status: Structural
/// formal_axioms: NONE
///
/// Ordered names of bundled [`Profile`] artefacts embedded via [`include_str!`].
pub const BUNDLED_PROFILE_IDS: &[&str] = &[
    "default", "uci_d1", "uci_d2", "uci_d3", "uci_d4", "uhpc", "highscm", "selfheal", "lunar",
];

/// formal_anchor: lean://umst-formal/Lean/Constitutional.lean#kleisliComposeWellTypedN
/// formal_status: Structural
/// formal_axioms: NONE
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    PowersGelSpace,
    JenningsGelSpace,
}

/// formal_anchor: lean://umst-formal/Lean/Powers.lean#PowersState
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

/// formal_anchor: lean://umst-formal/Lean/Gate.lean#Admissible
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

/// formal_anchor: lean://umst-formal/Lean/MeasurementCost.lean#zero_info_zero_energy
/// formal_status: Boundary
/// formal_axioms: NONE
#[derive(Debug, Clone, Deserialize)]
pub struct ProvenanceFormal {
    pub anchor: String,
    pub status: String,
    #[serde(default)]
    pub axioms: Vec<String>,
    #[serde(default)]
    pub rationale: Option<String>,
}

/// formal_anchor: lean://umst-formal/Lean/Powers.lean#S_intrinsic
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
/// formal_status: Library
/// formal_axioms: NONE
/// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
pub struct CalibrationProvenance {
    #[serde(default)]
    pub dataset_lift_from: Option<String>,
    pub prototype_3_sha256: String,
    #[serde(default)]
    pub primary_reference: Option<String>,
    #[serde(default)]
    pub secondary_references: Vec<String>,
    #[serde(default)]
    pub formal: Option<ProvenanceFormal>,
}

/// formal_anchor: lean://umst-formal/Lean/OrderStatisticsBand.lean#order_statistic_concentration
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
/// formal_status: Library
/// formal_axioms: NONE
/// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
pub struct CalibrationModelSection {
    pub kind: ModelKind,
}

/// formal_anchor: lean://umst-formal/Lean/OrderStatisticsBand.lean#p25_p75_admissibility
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
    #[serde(default)]
    pub formal_status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
/// formal_anchor: NONE
/// formal_status: Library
/// formal_axioms: NONE
/// formal_anchor_rationale: Differentiable training pathway; mechanised gate lemmas apply at manifold orchestration layer.
pub struct ContractBlock {
    pub verification_status: String,
}

/// formal_anchor: lean://umst-formal/Lean/Naturality.lean#gateMaterialAgnostic
/// formal_status: Structural
/// formal_axioms: NONE
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
}

/// formal_anchor: lean://umst-formal/Lean/Gate.lean#Admissible
/// formal_status: Structural
/// formal_axioms: NONE
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegimeViolation {
    pub field: &'static str,
    pub message: String,
}

/// formal_anchor: NONE
/// formal_anchor_rationale: IO and parse errors for bundled calibration artefacts.
#[derive(Debug, Error)]
pub enum CalibrationError {
    #[error("unknown bundled profile `{0}`")]
    UnknownBundledProfile(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    TomlDeserialize(#[from] toml::de::Error),
}

impl Profile {
    /// formal_anchor: lean://umst-formal/Lean/Constitutional.lean#kleisliCompose
    /// formal_status: Structural
    /// formal_axioms: NONE
    pub fn load_bundled(name: &str) -> Result<Self, CalibrationError> {
        let txt = bundled_toml_source(name)?;
        Self::parse_toml(bundle_id_normalized(name)?, txt)
    }

    /// formal_anchor: lean://umst-formal/Lean/LandauerLaw.lean#ErasureProcess
    /// formal_status: Boundary
    /// formal_axioms: physicalSecondLaw
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
            bundle_id,
        }
    }

    /// formal_anchor: lean://umst-formal/Lean/OrderStatisticsBand.lean#order_statistic_concentration
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
                    message: format!("fly_ash_pct={} exceeds {}", fa, m),
                });
            }
        }
        if let Some(m) = r.silica_fume_pct_max {
            if sf > m {
                out.push(RegimeViolation {
                    field: "silica_fume_pct",
                    message: format!("silica_fume_pct={} exceeds {}", sf, m),
                });
            }
        }
        if let Some(m) = r.silica_fume_pct_max_special {
            if sf > m {
                out.push(RegimeViolation {
                    field: "silica_fume_pct",
                    message: format!("silica_fume_pct={} exceeds UHPC cap {}", sf, m),
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

/// formal_anchor: lean://umst-formal/Lean/Naturality.lean#naturalitySquare
/// formal_status: Structural
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
        "default" => include_str!("../calibration/profiles/default.v1.toml"),
        "uci_d1" => include_str!("../calibration/profiles/uci_d1.v1.toml"),
        "uci_d2" => include_str!("../calibration/profiles/uci_d2.v1.toml"),
        "uci_d3" => include_str!("../calibration/profiles/uci_d3.v1.toml"),
        "uci_d4" => include_str!("../calibration/profiles/uci_d4.v1.toml"),
        "uhpc" => include_str!("../calibration/profiles/uhpc.v1.toml"),
        "highscm" => include_str!("../calibration/profiles/highscm.v1.toml"),
        "selfheal" => include_str!("../calibration/profiles/selfheal.v1.toml"),
        "lunar" => include_str!("../calibration/profiles/lunar.v1.toml"),
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

/// formal_anchor: lean://umst-formal/Lean/Constitutional.lean#KleisliArrow
/// formal_status: Structural
/// formal_axioms: NONE
#[must_use]
pub fn profile_descriptions() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert(
        "default",
        "generic OPC fallback (Contract, synthetic anchor)",
    );
    m.insert("uci_d1", "Yeh 1998 UCI concrete strength (D1, 1030 rows)");
    m.insert("uci_d2", "Augmented UCI-style strength dataset D2");
    m.insert("uci_d3", "Augmented UCI-style strength dataset D3");
    m.insert("uci_d4", "Augmented UCI-style strength dataset D4");
    m.insert("uhpc", "Ultra-high performance concrete (Boundary)");
    m.insert("highscm", "High SCM blends (Contract on paired CSV)");
    m.insert("selfheal", "Self-healing specialty mix (Boundary)");
    m.insert("lunar", "Lunar geopolymer-style stub (Boundary)");
    m
}
