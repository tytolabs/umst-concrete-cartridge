<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Proof status (Rust cartridge sources)

Generated from `crates/umst-concrete-cartridge/src/**/*.rs`, `crates/umst-cli/src/**/*.rs`, `crates/umst-mcp/src/**/*.rs`, and `crates/umst-py/src/**/*.rs` formal documentation blocks. Regenerate with:

```bash
cargo test -p umst-concrete-cartridge --test proof_status_doc \
  proof_status_refresh_markdown_on_disk -- --ignored --nocapture
```

## Bucket counts

| formal_status | Symbols |
|---------------|---------|
| **Mechanised** | 26 |
| **Structural** | 33 |
| **Empirical** | 27 |
| **Literature** | 19 |
| **NONE** | 92 |

## Mechanised

| Symbol | File | formal_anchor | Citation / envelope / rationale |
|--------|------|---------------|-----------------------------------|
| `PowersGelParameters` | `crates/umst-concrete-cartridge/src/calibration.rs:40` | `lean://umst-formal/Lean/Powers.lean#PowersState` | physicalSecondLaw |
| `FormalBlock` | `crates/umst-concrete-cartridge/src/calibration.rs:53` | `lean://umst-formal/Lean/Gate.lean#Admissible` | physicalSecondLaw |
| `CalibrationMeta` | `crates/umst-concrete-cartridge/src/calibration.rs:79` | `lean://umst-formal/Lean/Powers.lean#S_intrinsic` | physicalSecondLaw |
| `RegimeBounds` | `crates/umst-concrete-cartridge/src/calibration.rs:124` | `lean://umst-formal/Lean/OrderStatisticsBand.lean#order_statistic_concentration` | NONE |
| `AcceptanceBlock` | `crates/umst-concrete-cartridge/src/calibration.rs:158` | `lean://umst-formal/Lean/OrderStatisticsBand.lean#p25_p75_admissibility` | NONE |
| `regime_check_scalars` | `crates/umst-concrete-cartridge/src/calibration.rs:305` | `lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | NONE |
| `any_bundled_profile_covers_scalars` | `crates/umst-concrete-cartridge/src/calibration.rs:390` | `lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | NONE |
| `WaterCementRatio` | `crates/umst-concrete-cartridge/src/facade/mod.rs:50` | `lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | NONE |
| `TemperatureK` | `crates/umst-concrete-cartridge/src/facade/mod.rs:78` | `lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | NONE |
| `powers_compressive_strength_mpa` | `crates/umst-concrete-cartridge/src/homogeneous.rs:139` | `lean://umst-formal/Lean/Powers.lean#powers_monotone` | physicalSecondLaw |
| `compressive_strength_mpa` | `crates/umst-concrete-cartridge/src/homogeneous.rs:190` | `lean://umst-formal/Lean/Powers.lean#PowersState` | physicalSecondLaw |
| `degree_of_hydration_alpha` | `crates/umst-concrete-cartridge/src/homogeneous.rs:198` | `lean://umst-formal/Lean/Powers.lean#powers_monotone` | physicalSecondLaw |
| `capillary_porosity` | `crates/umst-concrete-cartridge/src/homogeneous.rs:205` | `lean://umst-formal/Lean/Powers.lean#PowersState` | NONE |
| `safety_margin` | `crates/umst-concrete-cartridge/src/homogeneous.rs:254` | `lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | NONE |
| `ChemoWaterEngine` | `crates/umst-concrete-cartridge/src/physics/chemo_water.rs:6` | `lean://umst-formal/Lean/Powers.lean#PowersState` | physicalSecondLaw |
| `compute_moisture_transport` | `crates/umst-concrete-cartridge/src/physics/chemo_water.rs:16` | `lean://umst-formal/Lean/Powers.lean#PowersState` | physicalSecondLaw |
| `compute_hydration_degree` | `crates/umst-concrete-cartridge/src/physics/hydration.rs:7` | `lean://umst-formal/Lean/JenningsGelSpace.lean#jennings_strength_monotone` | NONE |
| `compute_capillary_porosity` | `crates/umst-concrete-cartridge/src/physics/porosity.rs:6` | `lean://umst-formal/Lean/Powers.lean#PowersState` | NONE |
| `SetTimeEngine` | `crates/umst-concrete-cartridge/src/physics/set_time.rs:6` | `lean://umst-formal/Lean/Helmholtz.lean#ψAntitoneHelmholtz` | NONE |
| `compute_setting_time` | `crates/umst-concrete-cartridge/src/physics/set_time.rs:17` | `lean://umst-formal/Lean/Helmholtz.lean#ψAntitoneHelmholtz` | NONE |
| `StrengthEngine` | `crates/umst-concrete-cartridge/src/physics/strength.rs:6` | `lean://umst-formal/Lean/Powers.lean#powers_monotone` | physicalSecondLaw |
| `compute_strength_jennings` | `crates/umst-concrete-cartridge/src/physics/strength.rs:17` | `lean://umst-formal/Lean/Powers.lean#powers_monotone` | physicalSecondLaw |
| `ThermoEngine` | `crates/umst-concrete-cartridge/src/physics/thermo.rs:6` | `lean://umst-formal/Lean/Helmholtz.lean#ψAntitoneHelmholtz` | NONE |
| `compute_heat_rate` | `crates/umst-concrete-cartridge/src/physics/thermo.rs:16` | `lean://umst-formal/Lean/Helmholtz.lean#ψAntitoneHelmholtz` | NONE |
| `compute_capillary_porosity` | `crates/umst-concrete-cartridge/src/physics/transport.rs:17` | `lean://umst-formal/Lean/Powers.lean#PowersState` | NONE |
| `compute_chloride_diffusivity` | `crates/umst-concrete-cartridge/src/physics/transport.rs:45` | `lean://umst-formal/Lean/MeasurementCost.lean#zero_info_zero_energy` | NONE |

## Structural

| Symbol | File | formal_anchor | Citation / envelope / rationale |
|--------|------|---------------|-----------------------------------|
| `certify_profile_chain, mix_spec_wire_out, predict, predict_with_options, prediction_wire_v1, prediction_wire_v2, tensor_element_at, CertifyChain, FacadeBackend, FacadeError, HomogeneousCompareWire, MixSpec, MixSpecError, MixSpecWire, MixSpecWireOut, PredictBundle, PredictOptions, PredictionWireV1, PredictionWireV2, PredictionWireVersion` | `crates/umst-cli/src/cli.rs:13` | `STRUCTURAL` | Thin transport re-export of `umst_concrete_cartridge::facade`; authoritative formal blocks live on facade definitions. |
| `CliBackend` | `crates/umst-cli/src/cli.rs:38` | `STRUCTURAL` | CLI re-export; ndarray backend alias for historical `CliBackend` name. |
| `CliError` | `crates/umst-cli/src/cli.rs:43` | `STRUCTURAL` | Binary-boundary error aggregation; extends [`FacadeError`] with JSON/optimise glue. |
| `certify_profile_json` | `crates/umst-cli/src/cli.rs:142` | `STRUCTURAL` | JSON Value view of certify chain; structural wrapper over [`certify_profile_chain`]. |
| `OptimizeField` | `crates/umst-cli/src/cli.rs:159` | `STRUCTURAL` | Exhaustive enum of optimisation targets for the CLI bisection driver. |
| `BUNDLED_PROFILE_IDS` | `crates/umst-concrete-cartridge/src/calibration.rs:14` | `STRUCTURAL` | Ordered manifest of bundled profile ids for `include_str!` routing. |
| `ModelKind` | `crates/umst-concrete-cartridge/src/calibration.rs:30` | `STRUCTURAL` | Exhaustive serde enum over calibrated homogeneous model kinds. |
| `Profile` | `crates/umst-concrete-cartridge/src/calibration.rs:182` | `STRUCTURAL` | Parsed TOML aggregate routed by `bundle_id`; field invariants delegated to nested serde structs. |
| `RegimeViolation` | `crates/umst-concrete-cartridge/src/calibration.rs:199` | `STRUCTURAL` | Named-field regime violation records for CLI warning strings. |
| `load_bundled` | `crates/umst-concrete-cartridge/src/calibration.rs:252` | `STRUCTURAL` | Bundled `include_str!` loader with normalized bundle id validation. |
| `profile_descriptions` | `crates/umst-concrete-cartridge/src/calibration.rs:462` | `STRUCTURAL` | Static HashMap of tab-separated CLI profile blurbs (human-readable only). |
| `FacadeBackend` | `crates/umst-concrete-cartridge/src/facade/mod.rs:35` | `STRUCTURAL` | Burn backend selection; structural type alias to the ndarray tensor runtime. |
| `PredictionWireVersion` | `crates/umst-concrete-cartridge/src/facade/mod.rs:40` | `STRUCTURAL` | Exhaustive enum over wire-schema variants; pattern matching guarantees both tags handled. |
| `MixSpec` | `crates/umst-concrete-cartridge/src/facade/mod.rs:105` | `STRUCTURAL` | Field invariants enforced by `WaterCementRatio` / `TemperatureK` newtypes and range-checked fractions. |
| `MixSpecWire` | `crates/umst-concrete-cartridge/src/facade/mod.rs:120` | `STRUCTURAL` | Serde shape for mix.v1 JSON; field validation on conversion to [`MixSpec`]. |
| `FacadeError` | `crates/umst-concrete-cartridge/src/facade/mod.rs:218` | `STRUCTURAL` | Binary-boundary error aggregation for facade calls (no vendor IO). |
| `HomogeneousCompareWire` | `crates/umst-concrete-cartridge/src/facade/mod.rs:273` | `STRUCTURAL` | Homogeneous sidecar scalars for optional regression diff (serde-friendly). |
| `PredictBundle` | `crates/umst-concrete-cartridge/src/facade/mod.rs:295` | `STRUCTURAL` | Bundle of physical tensors plus calibration metadata returned by [`predict`] / [`predict_with_options`]. |
| `predict` | `crates/umst-concrete-cartridge/src/facade/mod.rs:357` | `STRUCTURAL` | Natural transformation φ ∘ F ∘ ψ over the cartridge functor (facade orchestration entry). |
| `predict_from_mix_row` | `crates/umst-concrete-cartridge/src/facade/mod.rs:451` | `STRUCTURAL` | Tensor prediction from dataset-style [`homog::MixRow`] masses; regime gates use binder-normalised SCM splits (slag routed through the silica regime channel). |
| `PreparedAuditRow` | `crates/umst-concrete-cartridge/src/facade/mod.rs:543` | `STRUCTURAL` | One CSV row wired for corpus audit alongside aggregate packing fraction derived in CLI from coarse/fine masses (ρ=2600 kg/m³, same surrogate as homogeneous layout). |
| `audit_build_report_v1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:612` | `STRUCTURAL` | Deterministic corpus audit projection over prepared rows (tensor strength channel). |
| `CertifyChain` | `crates/umst-concrete-cartridge/src/facade/mod.rs:684` | `STRUCTURAL` | JSON payload schema for `umst certify` output (profile, anchors, mapped formal bucket). |
| `certify_profile_chain` | `crates/umst-concrete-cartridge/src/facade/mod.rs:709` | `STRUCTURAL` | Builds the certify view including wire `formal_status` mapped from profile metadata. |
| `schema_mix_v1_json` | `crates/umst-concrete-cartridge/src/facade/mod.rs:744` | `STRUCTURAL` | SSOT `include_str!` of repo-root schema for CLI/MCP/Python. |
| `schema_result_v1_json` | `crates/umst-concrete-cartridge/src/facade/mod.rs:753` | `STRUCTURAL` | SSOT `include_str!` of repo-root schema for CLI/MCP/Python. |
| `schema_result_v2_json` | `crates/umst-concrete-cartridge/src/facade/mod.rs:762` | `STRUCTURAL` | SSOT `include_str!` of repo-root schema for CLI/MCP/Python. |
| `schema_audit_v1_json` | `crates/umst-concrete-cartridge/src/facade/mod.rs:771` | `STRUCTURAL` | SSOT `include_str!` of repo-root schema for CLI `umst audit`. |
| `MixRow` | `crates/umst-concrete-cartridge/src/homogeneous.rs:14` | `STRUCTURAL` | kg/m³ tagged scalars; structural carrier of mix design components for homogeneous routing. |
| `mix_row_from_scalar_spec` | `crates/umst-concrete-cartridge/src/homogeneous.rs:289` | `STRUCTURAL` | Deterministic projection of `MixSpec` scalar inputs into `MixRow` mass fractions. |
| `predict` | `crates/umst-py/src/lib.rs:115` | `STRUCTURAL` | Python transport wrapper over **[`predict_with_options`]**; anchored on facade predict path. |
| `certify` | `crates/umst-py/src/lib.rs:186` | `STRUCTURAL` | Dict view of **[`certify_profile_json`]**; structural mirror of CLI `umst certify`. |
| `schema` | `crates/umst-py/src/lib.rs:198` | `STRUCTURAL` | SSOT schema text from facade `include_str!` for notebooks and packaging checks. |

## Empirical

| Symbol | File | formal_anchor | Citation / envelope / rationale |
|--------|------|---------------|-----------------------------------|
| `optimize_mix` | `crates/umst-cli/src/cli.rs:190` | `empirical://datasets/cli-optimize-wc-bisection.v1.csv` | "Driver-only inverse search on w/c holding other mix fields fixed" \| "tests/cli/optimize.rs" |
| `hydration_degree_calibrated` | `crates/umst-concrete-cartridge/src/formulas.rs:24` | `empirical://datasets/hydration-kinetics-calibration-grid.v1.csv` | "Mills (1966) ultimate cap with stretched-exponential √t kinetics and Arrhenius temperature factor (calibrated multipliers from profile TOML)" \| "tests/hydration.rs::powers_doh_envelope" |
| `yield_stress_pa` | `crates/umst-concrete-cartridge/src/homogeneous.rs:213` | `empirical://datasets/printability-rheology-yield-proxy.v1.csv` | "Roussel (2018) Cem. Concr. Res. 112, 76; Château, Ovarlez & Trung (2008) J. Rheol. 52, 489" \| "tests/printability.rs" |
| `ColloidalEngine` | `crates/umst-concrete-cartridge/src/physics/colloidal.rs:6` | `empirical://datasets/dataset_d1.csv` | "Flatt & Bowen (2007) J. Am. Ceram. Soc. 89, 1244 (YODEL)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); DLVO pathway exercised under tests/realism/adversarial_physics.rs" |
| `compute_dlvo_potential` | `crates/umst-concrete-cartridge/src/physics/colloidal.rs:20` | `empirical://datasets/dataset_d1.csv` | "Flatt & Bowen (2007) J. Am. Ceram. Soc. 89, 1244 (YODEL)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); DLVO pathway exercised under tests/realism/adversarial_physics.rs" |
| `compute_flocculation_multiplier` | `crates/umst-concrete-cartridge/src/physics/colloidal.rs:82` | `empirical://datasets/dataset_d1.csv` | "Flatt & Bowen (2007) J. Am. Ceram. Soc. 89, 1244 (YODEL)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); DLVO pathway exercised under tests/realism/adversarial_physics.rs" |
| `CreepEngine` | `crates/umst-concrete-cartridge/src/physics/creep.rs:8` | `empirical://datasets/dataset_d1.csv` | "Bažant et al. (2015) Mater. Struct. 48, 753 (RILEM B4)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); RILEM B4 creep pathway exercised under tests/creep.rs + adversarial harness" |
| `compute_compliance` | `crates/umst-concrete-cartridge/src/physics/creep.rs:21` | `empirical://datasets/dataset_d1.csv` | "Bažant et al. (2015) Mater. Struct. 48, 753 (RILEM B4)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); RILEM B4 creep pathway exercised under tests/creep.rs + adversarial harness" |
| `FreezeThawEngine` | `crates/umst-concrete-cartridge/src/physics/freeze_thaw.rs:8` | `empirical://datasets/dataset_d1.csv` | "Powers (1949) Highw. Res. Board Proc. 29, 184 (spacing factor)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); freeze–thaw durability pathway exercised under tests/freeze_thaw.rs + adversarial harness" |
| `compute_durability` | `crates/umst-concrete-cartridge/src/physics/freeze_thaw.rs:21` | `empirical://datasets/dataset_d1.csv` | "Powers (1949) Highw. Res. Board Proc. 29, 184 (spacing factor)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); freeze–thaw durability pathway exercised under tests/freeze_thaw.rs + adversarial harness" |
| `compute_itz_thickness_microns` | `crates/umst-concrete-cartridge/src/physics/itz.rs:6` | `empirical://datasets/dataset_d1.csv` | "Scrivener et al. (2004) Interface Sci. 12, 411" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); ITZ thickness/porosity pathway exercised under tests/realism/adversarial_physics.rs" |
| `compute_itz_porosity` | `crates/umst-concrete-cartridge/src/physics/itz.rs:25` | `empirical://datasets/dataset_d1.csv` | "Scrivener et al. (2004) Interface Sci. 12, 411" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); ITZ thickness/porosity pathway exercised under tests/realism/adversarial_physics.rs" |
| `compute_itz_percolation_factor` | `crates/umst-concrete-cartridge/src/physics/itz.rs:40` | `empirical://datasets/dataset_d1.csv` | "Scrivener et al. (2004) Interface Sci. 12, 411" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); ITZ thickness/porosity pathway exercised under tests/realism/adversarial_physics.rs" |
| `NanoEngine` | `crates/umst-concrete-cartridge/src/physics/nano.rs:6` | `empirical://datasets/csh-nano-calibration-grid.v1.csv` | "Pellenq et al. (2009) PNAS 106, 16102" \| "tests/realism/adversarial_physics.rs" |
| `compute_enhancements` | `crates/umst-concrete-cartridge/src/physics/nano.rs:18` | `empirical://datasets/csh-nano-calibration-grid.v1.csv` | "Pellenq et al. (2009) PNAS 106, 16102" \| "tests/realism/adversarial_physics.rs" |
| `PolymerEngine` | `crates/umst-concrete-cartridge/src/physics/polymer.rs:6` | `empirical://datasets/dataset_highscm.csv` | "Su, van Breugel & Bijen (1991) Cem. Concr. Res. 21" \| "Headline compressive strength vs dataset_highscm.csv: MAE ≤ 60 MPa, RMSE ≤ 80 MPa, R² ≥ −10 ([acceptance] highscm.v1.toml); polymer modifiers exercised under tests/realism/adversarial_physics.rs" |
| `compute_modifiers` | `crates/umst-concrete-cartridge/src/physics/polymer.rs:19` | `empirical://datasets/dataset_highscm.csv` | "Su, van Breugel & Bijen (1991) Cem. Concr. Res. 21" \| "Headline compressive strength vs dataset_highscm.csv: MAE ≤ 60 MPa, RMSE ≤ 80 MPa, R² ≥ −10 ([acceptance] highscm.v1.toml); polymer modifiers exercised under tests/realism/adversarial_physics.rs" |
| `PrintabilityEngine` | `crates/umst-concrete-cartridge/src/physics/printability.rs:6` | `empirical://datasets/dataset_d1.csv` | "Roussel (2018) Cem. Concr. Res. 112, 76" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel buildability pathway exercised under tests/printability.rs + adversarial harness" |
| `compute_buildability` | `crates/umst-concrete-cartridge/src/physics/printability.rs:20` | `empirical://datasets/dataset_d1.csv` | "Roussel (2018) Cem. Concr. Res. 112, 76" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel buildability pathway exercised under tests/printability.rs + adversarial harness" |
| `compute_extrudability` | `crates/umst-concrete-cartridge/src/physics/printability.rs:85` | `empirical://datasets/dataset_d1.csv` | "Roussel (2018) Cem. Concr. Res. 112, 76" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel buildability pathway exercised under tests/printability.rs + adversarial harness" |
| `RheologyEngine` | `crates/umst-concrete-cartridge/src/physics/rheology.rs:6` | `empirical://datasets/dataset_d1.csv` | "Château, Ovarlez & Trung (2008) J. Rheol. 52, 489" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel/YODEL pathway exercised under tests/rheology.rs + adversarial harness" |
| `compute_chateau_ovarlez` | `crates/umst-concrete-cartridge/src/physics/rheology.rs:19` | `empirical://datasets/dataset_d1.csv` | "Château, Ovarlez & Trung (2008) J. Rheol. 52, 489" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel/YODEL pathway exercised under tests/rheology.rs + adversarial harness" |
| `compute_yield_stress_yodel` | `crates/umst-concrete-cartridge/src/physics/rheology.rs:69` | `empirical://datasets/dataset_d1.csv` | "Château, Ovarlez & Trung (2008) J. Rheol. 52, 489" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel/YODEL pathway exercised under tests/rheology.rs + adversarial harness" |
| `SelfHealEngine` | `crates/umst-concrete-cartridge/src/physics/self_heal.rs:6` | `empirical://datasets/dataset_selfheal.csv` | "Edvardsen (1999) ACI Mater. J. 96, 448" \| "Boundary profile (no [acceptance] strength gate); paired CSV still listed in dataset_metrics skip — healing kinetics exercised under tests/realism/adversarial_physics.rs" |
| `compute_healing_potential` | `crates/umst-concrete-cartridge/src/physics/self_heal.rs:19` | `empirical://datasets/dataset_selfheal.csv` | "Edvardsen (1999) ACI Mater. J. 96, 448" \| "Boundary profile (no [acceptance] strength gate); paired CSV still listed in dataset_metrics skip — healing kinetics exercised under tests/realism/adversarial_physics.rs" |
| `ShrinkageEngine` | `crates/umst-concrete-cartridge/src/physics/shrinkage.rs:8` | `empirical://datasets/dataset_d1.csv` | "Bažant et al. (2015) Mater. Struct. 48, 753 (B4 shrinkage model)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Bažant–Baweja shrinkage pathway exercised under tests/shrinkage.rs + adversarial harness" |
| `compute_autogenous_shrinkage` | `crates/umst-concrete-cartridge/src/physics/shrinkage.rs:21` | `empirical://datasets/dataset_d1.csv` | "Bažant et al. (2015) Mater. Struct. 48, 753 (B4 shrinkage model)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Bažant–Baweja shrinkage pathway exercised under tests/shrinkage.rs + adversarial harness" |

## Literature

| Symbol | File | formal_anchor | Citation / envelope / rationale |
|--------|------|---------------|-----------------------------------|
| `RESULT_SCHEMA_VERSION_V1` | `crates/umst-cli/src/cli.rs:26` | `literature://wire-schema-result-v1` | "UMST concrete cartridge JSON wire schema tag (`result.v1`)" \| "`result.v1` — version tag for deprecated prediction JSON envelope" |
| `RESULT_SCHEMA_VERSION_V2` | `crates/umst-cli/src/cli.rs:32` | `literature://wire-schema-result-v2` | "UMST concrete cartridge JSON wire schema tag (`result.v2`)" \| "`result.v2` — version tag for current prediction JSON envelope" |
| `RESULT_SCHEMA_VERSION_V1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:23` | `literature://wire-schema-result-v1` | "UMST concrete cartridge JSON wire schema tag (`result.v1`)" \| "`result.v1` — version tag for deprecated prediction JSON envelope" |
| `RESULT_SCHEMA_VERSION_V2` | `crates/umst-concrete-cartridge/src/facade/mod.rs:29` | `literature://wire-schema-result-v2` | "UMST concrete cartridge JSON wire schema tag (`result.v2`)" \| "`result.v2` — version tag for current prediction JSON envelope" |
| `AUDIT_SCHEMA_VERSION` | `crates/umst-concrete-cartridge/src/facade/mod.rs:536` | `literature://wire-schema-audit-v1` | "UMST concrete cartridge JSON wire schema tag (`audit.v1`)" \| "`audit.v1` — batch CSV audit envelope with tensor predictions vs optional CSV strength" |
| `ultimate_doh_wc` | `crates/umst-concrete-cartridge/src/formulas.rs:13` | `literature://Mills-1966-gel-stiffness-closure` | "Mills (1966); OPC gel stiffness / ultimate hydration cap closure used in routing" \| "α_inf(w/c) = 1.031·w/c / (0.194 + w/c)" |
| `ultimate_doh` | `crates/umst-concrete-cartridge/src/homogeneous.rs:67` | `literature://Mills-1966-gel-stiffness-closure` | "Mills (1966); α_inf = 1.031 w/c / (0.194 + w/c)" \| "α_inf(w/c) = 1.031·w/c / (0.194 + w/c)" |
| `embodied_co2_kg_per_m3` | `crates/umst-concrete-cartridge/src/homogeneous.rs:239` | `literature://EN-15804+A2-indicative-EPD-intensities` | "EN 15804+A2 (2019) environmental product declarations — indicative cradle-to-gate CO₂e intensities per constituent class" \| "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3); inline coefficients match bundled EPD intensity convention" |
| `constituent_masses_kg_m3` | `crates/umst-concrete-cartridge/src/homogeneous.rs:266` | `literature://ACI-211.1-binder-dosage-convention` | "ACI 211.1 — Standard Practice for Selecting Proportions for Normal, Heavyweight, and Mass Concrete" \| "350 kg/m³ binder dosage convention for constituent mass reconstruction from scalar mix spec" |
| `PHYSICS_PIPELINE_SCHEMA_VERSION` | `crates/umst-concrete-cartridge/src/lib.rs:49` | `literature://wire-schema-physics-pipeline-v1` | "physics_pipeline schema tag (`physics_pipeline.v1`)" \| "`schema_version` string on serde `PhysicsPipelineReport` — bump tag when breaking report shape." |
| `FiberEngine` | `crates/umst-concrete-cartridge/src/physics/fiber.rs:6` | `literature://Naaman-2006-ACI-SP235-pullout` | "Naaman (2006) ACI SP-235 — fiber pullout and crack-bridging micromechanics" \| "V_{f,crit} = σ_cu / (η_l η_o τ_b l_f / d_f)" |
| `compute_micromechanics` | `crates/umst-concrete-cartridge/src/physics/fiber.rs:17` | `literature://Naaman-2006-ACI-SP235-pullout` | "Naaman (2006) ACI SP-235 — fiber pullout and crack-bridging micromechanics" \| "V_{f,crit} = σ_cu / (η_l η_o τ_b l_f / d_f)" |
| `FractureEngine` | `crates/umst-concrete-cartridge/src/physics/fracture.rs:6` | `literature://Ulm-Coussy-2003-micromechanics` | "Ulm & Coussy (2003) Mechanics of Porous Continua (MIT Press); micromechanics derivation" \| "K_Ic = √(2 γ_s E_eff); E_eff = E_0 (1 − φ)^n" |
| `compute_effective_modulus_mt` | `crates/umst-concrete-cartridge/src/physics/fracture.rs:18` | `literature://Ulm-Coussy-2003-micromechanics` | "Ulm & Coussy (2003) Mechanics of Porous Continua (MIT Press); micromechanics derivation" \| "K_Ic = √(2 γ_s E_eff); E_eff = E_0 (1 − φ)^n" |
| `compute_fracture_toughness` | `crates/umst-concrete-cartridge/src/physics/fracture.rs:88` | `literature://Ulm-Coussy-2003-micromechanics` | "Ulm & Coussy (2003) Mechanics of Porous Continua (MIT Press); micromechanics derivation" \| "K_Ic = √(2 γ_s E_eff); E_eff = E_0 (1 − φ)^n" |
| `compute_packing_density` | `crates/umst-concrete-cartridge/src/physics/packing.rs:6` | `literature://Andreasen-Andersen-1930-Fuller-curve` | "Andreasen & Andersen (1930), Kolloid-Z. 50, 217" \| "P(D) = (D^q - D_min^q) / (D_max^q - D_min^q), q in [0.30, 0.45]" |
| `SustainabilityEngine` | `crates/umst-concrete-cartridge/src/physics/sustainability.rs:6` | `literature://EN-15804+A2-GWP-and-unit-costs` | "EN 15804+A2 (2019) cradle-to-gate / modules A2 — indicative EPD-style CO₂e intensities; financial row uses linear $/kg mass factors" \| "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3)" |
| `compute_embodied_carbon` | `crates/umst-concrete-cartridge/src/physics/sustainability.rs:19` | `literature://EN-15804+A2-GWP-and-unit-costs` | "EN 15804+A2 (2019) cradle-to-gate / modules A2 — indicative EPD-style CO₂e intensities; financial row uses linear $/kg mass factors" \| "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3)" |
| `compute_financial_cost` | `crates/umst-concrete-cartridge/src/physics/sustainability.rs:51` | `literature://EN-15804+A2-GWP-and-unit-costs` | "EN 15804+A2 (2019) cradle-to-gate / modules A2 — indicative EPD-style CO₂e intensities; financial row uses linear $/kg mass factors" \| "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3)" |

## NONE

| Symbol | File | formal_anchor | Citation / envelope / rationale |
|--------|------|---------------|-----------------------------------|
| `canon_header` | `crates/umst-cli/src/audit.rs:24` | `NONE` | Mechanical header synonym routing for CLI CSV audit ergonomics only. |
| `audit_csv_buf` | `crates/umst-cli/src/audit.rs:93` | `NONE` | Glue from CSV text to facade [`audit_build_report_v1`] without physical claims. |
| `stdin_to_string` | `crates/umst-cli/src/audit.rs:147` | `NONE` | IO helper for MCP/CLI corpus workflows. |
| `audit_csv_file` | `crates/umst-cli/src/audit.rs:158` | `NONE` | File IO adapter for corpus audit CLI. |
| `canonical_json_value` | `crates/umst-cli/src/canonical.rs:11` | `NONE` | Transport-only deterministic JSON contract; physical claims live upstream of this layer. |
| `CanonicalJsonError` | `crates/umst-cli/src/canonical.rs:46` | `NONE` | Structural transport error for malformed or non-finite JSON numbers. |
| `canonical_json_bytes` | `crates/umst-cli/src/canonical.rs:90` | `NONE` | Byte-stable wire encoding for MCP / CLI / acceptance scripts. |
| `mix_spec_from_json_value` | `crates/umst-cli/src/cli.rs:100` | `NONE` | JSON boundary helper; validation uses [`MixSpecWire`] + [`MixSpec::try_from`]. |
| `serialize_prediction` | `crates/umst-cli/src/cli.rs:115` | `NONE` | JSON-serialise glue; no physical claim. |
| `serialize_mix_spec` | `crates/umst-cli/src/cli.rs:151` | `NONE` | JSON-serialise glue. |
| `parse_optimize_target` | `crates/umst-cli/src/cli.rs:177` | `NONE` | String-parse glue for `FIELD=VALUE` optimise CLI syntax. |
| `bool_and` | `crates/umst-concrete-cartridge/src/burn_compat.rs:8` | `NONE` | Burn-version compatibility shim for boolean tensor AND across crate semver skew. |
| `ProvenanceFormal` | `crates/umst-concrete-cartridge/src/calibration.rs:66` | `NONE` | Serde lift of TOML `[provenance.formal]`; `status` string is file metadata (may include Boundary scope), not a Rust `formal_status` bucket. |
| `CalibrationProvenance` | `crates/umst-concrete-cartridge/src/calibration.rs:98` | `NONE` | Dataset and Zenodo citation bundle parsed from TOML only; no Lean witness on this serde container — see `docs/FormalAnchors.md` “Future formal links” for manifold adjoint context. |
| `CalibrationModelSection` | `crates/umst-concrete-cartridge/src/calibration.rs:150` | `NONE` | Dispatch metadata only; Jennings gel-space monotone strength witness applies once `powers_compressive_strength_mpa` ships a Jennings branch (TODO_FORMAL note on that function). |
| `ContractBlock` | `crates/umst-concrete-cartridge/src/calibration.rs:174` | `NONE` | Contract metadata (`verification_status`); hyperbox regime warnings are soundness-witnessed on `regime_check_scalars` — see RegimeSoundness anchor there. |
| `CalibrationError` | `crates/umst-concrete-cartridge/src/calibration.rs:208` | `NONE` | Bundled profile IO and TOML parse failures only; DEC mass-conservation witness belongs on the manifold Laplacian — see `docs/FormalAnchors.md` “Future formal links”. |
| `load_from_path` | `crates/umst-concrete-cartridge/src/calibration.rs:260` | `NONE` | Filesystem path IO for non-bundled TOML; parse errors surface as CalibrationError. |
| `RegressionMetrics` | `crates/umst-concrete-cartridge/src/calibration_metrics.rs:7` | `NONE` | Ordinary least-squares aggregates over paired CSV predictions; QA helper without Lean witness on this surface. |
| `regression_metrics` | `crates/umst-concrete-cartridge/src/calibration_metrics.rs:19` | `NONE` | Same as `RegressionMetrics`; computes MAE/RMSE/R² slices for calibration reports. |
| `ConcreteCartridge` | `crates/umst-concrete-cartridge/src/core/implementation.rs:183` | `NONE` | Cartridge functor F: mix layout → constitutive summaries; topology pass remains separate DEC hook. |
| `new` | `crates/umst-concrete-cartridge/src/core/implementation.rs:194` | `NONE` | Deterministic bundled baseline when callers omit explicit calibration. |
| `with_profile` | `crates/umst-concrete-cartridge/src/core/implementation.rs:202` | `NONE` | Avoids silently mixing heterogeneous tensor kinetics with unrelated gel-space coefficients. |
| `apply_topology_result_to_umst` | `crates/umst-concrete-cartridge/src/core/implementation.rs:214` | `NONE` | Mutable UMST merge path for topology tensors; proof witnesses remain caller-owned. |
| `ConcreteCartridge` | `crates/umst-concrete-cartridge/src/core/mod.rs:6` | `NONE` | Re-export; classification follows the underlying symbol. |
| `apply_physics_to_umst` | `crates/umst-concrete-cartridge/src/core/mod.rs:10` | `NONE` | Forwards manifold UMST write-back helper used after topology physics. |
| `IScienceCartridge, MixTensor, PhysicalResult` | `crates/umst-concrete-cartridge/src/core/mod.rs:14` | `NONE` | Forwards manifold cartridge façade trait and tensor bundles. |
| `value` | `crates/umst-concrete-cartridge/src/facade/mod.rs:69` | `NONE` | Trivial accessor; getter for the wrapped `f32`. |
| `value` | `crates/umst-concrete-cartridge/src/facade/mod.rs:97` | `NONE` | Trivial accessor. |
| `MixSpecError` | `crates/umst-concrete-cartridge/src/facade/mod.rs:190` | `NONE` | Mix-spec rejection causes without JSON parse errors (handled at transport boundary). |
| `PredictOptions` | `crates/umst-concrete-cartridge/src/facade/mod.rs:285` | `NONE` | Behavioral flags only — no Lean witness. |
| `predict_with_options` | `crates/umst-concrete-cartridge/src/facade/mod.rs:364` | `NONE` | Feature flag glue for MCP/CLI; no standalone formal claim. |
| `AuditSummaryV1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:554` | `NONE` | JSON summary stats for auditors; aggregates row-level residuals only. |
| `AuditReportV1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:565` | `NONE` | Top-level serde envelope for CLI/MCP corpus audit tooling. |
| `AuditRowInputV1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:579` | `NONE` | Normalised mix scalars carried from the CSV row for audit consumers (`audit.v1` row `input`). |
| `AuditRowWireV1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:594` | `NONE` | One audited CSV row projection with tensor headline strength. |
| `tensor_element_at` | `crates/umst-concrete-cartridge/src/facade/mod.rs:780` | `NONE` | Internal tensor scalar read for wire projection; index contract from pipeline layout. |
| `PredictionWireV1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:796` | `NONE` | Serde wire projection for `result.v1` scalars; versioning tagged in `schema_version`. |
| `PredictionWireV2` | `crates/umst-concrete-cartridge/src/facade/mod.rs:811` | `NONE` | Serde wire projection for `result.v2` scalars; `physics_pipeline` merged at JSON boundary. |
| `prediction_wire_v1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:829` | `NONE` | Pure wire projection; transport encoding is caller-owned. |
| `prediction_wire_v2` | `crates/umst-concrete-cartridge/src/facade/mod.rs:851` | `NONE` | Pure wire projection; nested objects merged by CLI/MCP `serde_json`. |
| `MixSpecWireOut` | `crates/umst-concrete-cartridge/src/facade/mod.rs:878` | `NONE` | Round-trip mix spec view for CLI `mix print` / MCP. |
| `mix_spec_wire_out` | `crates/umst-concrete-cartridge/src/facade/mod.rs:891` | `NONE` | Serialize-friendly mix view without JSON crate in core. |
| `HomogeneousError` | `crates/umst-concrete-cartridge/src/homogeneous.rs:28` | `NONE` | Dispatch error: Jennings-not-yet, invalid mix; no formal claim. |
| `mix_hydration_state` | `crates/umst-concrete-cartridge/src/homogeneous.rs:76` | `NONE` | Internal homogeneous helper composing calibrated α(t,T,scm) and effective w/c from profile parameters. |
| `apply_physics_to_umst, ConcreteCartridge, IScienceCartridge, MixTensor, PhysicalResult` | `crates/umst-concrete-cartridge/src/lib.rs:23` | `NONE` | Re-exports manifold façade symbols for ergonomics only. |
| `run_full_physics_pipeline` | `crates/umst-concrete-cartridge/src/lib.rs:29` | `NONE` | Stable import path for MCP/CLI integration tests. |
| `PhysicsPipelineReport` | `crates/umst-concrete-cartridge/src/lib.rs:33` | `NONE` | JSON envelope for staged tensor outputs. |
| `PhysicsPipelineSummary` | `crates/umst-concrete-cartridge/src/lib.rs:37` | `NONE` | Scalar digest accompanying report JSON. |
| `PipelineStageRecord` | `crates/umst-concrete-cartridge/src/lib.rs:41` | `NONE` | Stage record type embedded in [`PhysicsPipelineReport`]. |
| `PipelineStageStatus` | `crates/umst-concrete-cartridge/src/lib.rs:45` | `NONE` | Serialized stage disposition enum for MCP/CLI audit trails. |
| `MIX_FEATURE_COUNT` | `crates/umst-concrete-cartridge/src/mix_layout.rs:13` | `NONE` | Structural convention for CLI ↔ tensor engines; documented here as SSOT for column indices. |
| `IDX_WATER_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:19` | `NONE` | Structural column tag for wire ↔ tensor bridging. |
| `IDX_CEMENT_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:24` | `NONE` | Must stay aligned with `physics::hydration` slicers. |
| `IDX_AGG_COARSE_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:29` | `NONE` | Reserved packing-engine input when recipe supplies split gradation. |
| `IDX_AGG_FINE_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:34` | `NONE` | Reserved packing-engine input when recipe supplies split gradation. |
| `IDX_RESERVED_4` | `crates/umst-concrete-cartridge/src/mix_layout.rs:39` | `NONE` | Keeps hydration column indices historically stable (`hydration.rs` assumptions). |
| `IDX_SLAG_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:44` | `NONE` | Structural column aligned with SCM tensor hydrates. |
| `IDX_FLY_ASH_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:49` | `NONE` | Structural column for pozzolan mass routing. |
| `IDX_SUPERPLASTICIZER_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:54` | `NONE` | Admixture mass channel for homogeneous + rheology stubs. |
| `IDX_AGE_DAYS` | `crates/umst-concrete-cartridge/src/mix_layout.rs:59` | `NONE` | Scalar age carried on-layout for `compute_all` without extra manifold state. |
| `IDX_TEMPERATURE_C` | `crates/umst-concrete-cartridge/src/mix_layout.rs:64` | `NONE` | Arrhenius-style tensor engines consume this lane. |
| `IDX_SILICA_FUME_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:69` | `NONE` | SCM extension field for supplemental micro-silica mass. |
| `IDX_AGGREGATE_VOLUME_FRACTION` | `crates/umst-concrete-cartridge/src/mix_layout.rs:74` | `NONE` | Drives packing + paste fraction surrogates on collapsed paths. |
| `fractions_from_mix_row` | `crates/umst-concrete-cartridge/src/mix_layout.rs:85` | `NONE` | Deterministic CSV-style encoding; aligns with homogeneous `MixRow` used by CLI. |
| `mix_tensor_from_layout` | `crates/umst-concrete-cartridge/src/mix_layout.rs:106` | `NONE` | Constructor replacing non-existent `MixTensor::from_proportions`; see README Quick start. |
| `collapsed_rank4_from_rank2_scalar` | `crates/umst-concrete-cartridge/src/mix_layout.rs:123` | `NONE` | Engine APIs require rank-4 tensors; singleton spatial dims document batch-collapsed mode. |
| `compute_cost` | `crates/umst-concrete-cartridge/src/physics/cost.rs:7` | `NONE` | Auxiliary objective; linear cost vector, no physical claim. |
| `TransportEngine` | `crates/umst-concrete-cartridge/src/physics/transport.rs:6` | `NONE` | Tensor facade grouping porosity and chloride diffusivity kernels documented on methods. |
| `run_full_physics_pipeline` | `crates/umst-concrete-cartridge/src/pipeline/mod.rs:13` | `NONE` | Public entry to staged tensor physics used by `ConcreteCartridge::compute_all`. |
| `nominal_mix_tensor_for_topology, physical_result_from_report, topology_pipeline_headlines` | `crates/umst-concrete-cartridge/src/pipeline/mod.rs:17` | `NONE` | Policy map from rich report to manifold `PhysicalResult`. |
| `PhysicsPipelineReport` | `crates/umst-concrete-cartridge/src/pipeline/mod.rs:23` | `NONE` | Re-export pipeline wire types for CLI/MCP consumers. |
| `PhysicsPipelineSummary` | `crates/umst-concrete-cartridge/src/pipeline/mod.rs:27` | `NONE` | Re-export pipeline wire types for CLI/MCP consumers. |
| `PipelineStageRecord` | `crates/umst-concrete-cartridge/src/pipeline/mod.rs:31` | `NONE` | Re-export pipeline wire types for CLI/MCP consumers. |
| `PipelineStageStatus` | `crates/umst-concrete-cartridge/src/pipeline/mod.rs:35` | `NONE` | Re-export pipeline wire types for CLI/MCP consumers. |
| `PHYSICS_PIPELINE_SCHEMA_VERSION` | `crates/umst-concrete-cartridge/src/pipeline/mod.rs:39` | `NONE` | Re-export pipeline wire types for CLI/MCP consumers. |
| `run_full_physics_pipeline` | `crates/umst-concrete-cartridge/src/pipeline/orchestrator.rs:107` | `NONE` | Cartridge functor composition root exercised by tooling + tests. |
| `physical_result_from_report` | `crates/umst-concrete-cartridge/src/pipeline/physical_summary.rs:52` | `NONE` | Encodes CLI tensor summary contract; see module-level policy mapping. |
| `nominal_mix_tensor_for_topology` | `crates/umst-concrete-cartridge/src/pipeline/physical_summary.rs:89` | `NONE` | Deterministic surrogate mix so staged tensor engines (`StrengthEngine`, fracture headline, sustainability) match `compute_all` semantics. |
| `topology_pipeline_headlines` | `crates/umst-concrete-cartridge/src/pipeline/physical_summary.rs:124` | `NONE` | Single SSOT with Jennings/YODEL/GWP tensor pipeline used by bulk predict. |
| `PHYSICS_PIPELINE_SCHEMA_VERSION` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:11` | `NONE` | Version discriminator for additive JSON fields. |
| `PipelineStageStatus` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:17` | `NONE` | Enumerates audited stage outcomes (`Executed` vs honest skips/failures). |
| `PipelineStageRecord` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:29` | `NONE` | Serialized evidence that a stage ran, skipped, or failed. |
| `ok` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:43` | `NONE` | Successful stage marker helper. |
| `skip_missing` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:55` | `NONE` | Honest skip when inputs/constants are absent by design. |
| `fail` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:67` | `NONE` | Propagates panics-avoiding error strings for observability. |
| `PhysicsPipelineSummary` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:79` | `NONE` | Human/tooling digest; not a substitute for full tensor fields. |
| `PhysicsPipelineReport` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:103` | `NONE` | Cartridge-local rich JSON envelope parallel to manifold tensors. |
| `audit_rows` | `crates/umst-py/src/lib.rs:147` | `NONE` | Encodes iterable of row dicts into dataset-style CSV then reuses **`audit_csv_buf`** (aligned with **`audit`** string path). |
| `audit` | `crates/umst-py/src/lib.rs:167` | `NONE` | Python transport over CLI audit glue; no extra physical claim beyond CSV→facade audit. |
| `bundled_profile_ids` | `crates/umst-py/src/lib.rs:217` | `NONE` | Bundled id manifest for packaging smoke tests. |
| `canonical_json` | `crates/umst-py/src/lib.rs:229` | `NONE` | Byte-stable JSON for golden tests; matches **`umst-canonical`** binary. |

