<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Proof status (Rust cartridge sources)

Generated from `src/**/*.rs` and `crates/umst-cli/src/**/*.rs` formal documentation blocks. Regenerate with:

```bash
cargo test -p umst-concrete-cartridge --test proof_status_doc \
  proof_status_refresh_markdown_on_disk -- --ignored --nocapture
```

## Bucket counts

| formal_status | Symbols |
|---------------|---------|
| **Mechanised** | 26 |
| **Structural** | 17 |
| **Empirical** | 27 |
| **Literature** | 16 |
| **NONE** | 64 |

## Mechanised

| Symbol | File | formal_anchor | Citation / envelope / rationale |
|--------|------|---------------|-----------------------------------|
| `WaterCementRatio` | `crates/umst-cli/src/cli.rs:51` | `lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | NONE |
| `TemperatureK` | `crates/umst-cli/src/cli.rs:79` | `lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | NONE |
| `PowersGelParameters` | `src/calibration.rs:40` | `lean://umst-formal/Lean/Powers.lean#PowersState` | physicalSecondLaw |
| `FormalBlock` | `src/calibration.rs:53` | `lean://umst-formal/Lean/Gate.lean#Admissible` | physicalSecondLaw |
| `CalibrationMeta` | `src/calibration.rs:79` | `lean://umst-formal/Lean/Powers.lean#S_intrinsic` | physicalSecondLaw |
| `RegimeBounds` | `src/calibration.rs:124` | `lean://umst-formal/Lean/OrderStatisticsBand.lean#order_statistic_concentration` | NONE |
| `AcceptanceBlock` | `src/calibration.rs:158` | `lean://umst-formal/Lean/OrderStatisticsBand.lean#p25_p75_admissibility` | NONE |
| `regime_check_scalars` | `src/calibration.rs:305` | `lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | NONE |
| `any_bundled_profile_covers_scalars` | `src/calibration.rs:390` | `lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | NONE |
| `powers_compressive_strength_mpa` | `src/homogeneous.rs:139` | `lean://umst-formal/Lean/Powers.lean#powers_monotone` | physicalSecondLaw |
| `compressive_strength_mpa` | `src/homogeneous.rs:190` | `lean://umst-formal/Lean/Powers.lean#PowersState` | physicalSecondLaw |
| `degree_of_hydration_alpha` | `src/homogeneous.rs:198` | `lean://umst-formal/Lean/Powers.lean#powers_monotone` | physicalSecondLaw |
| `capillary_porosity` | `src/homogeneous.rs:205` | `lean://umst-formal/Lean/Powers.lean#PowersState` | NONE |
| `safety_margin` | `src/homogeneous.rs:254` | `lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | NONE |
| `ChemoWaterEngine` | `src/physics/chemo_water.rs:6` | `lean://umst-formal/Lean/Powers.lean#PowersState` | physicalSecondLaw |
| `compute_moisture_transport` | `src/physics/chemo_water.rs:16` | `lean://umst-formal/Lean/Powers.lean#PowersState` | physicalSecondLaw |
| `compute_hydration_degree` | `src/physics/hydration.rs:7` | `lean://umst-formal/Lean/JenningsGelSpace.lean#jennings_strength_monotone` | NONE |
| `compute_capillary_porosity` | `src/physics/porosity.rs:6` | `lean://umst-formal/Lean/Powers.lean#PowersState` | NONE |
| `SetTimeEngine` | `src/physics/set_time.rs:6` | `lean://umst-formal/Lean/Helmholtz.lean#ψAntitoneHelmholtz` | NONE |
| `compute_setting_time` | `src/physics/set_time.rs:17` | `lean://umst-formal/Lean/Helmholtz.lean#ψAntitoneHelmholtz` | NONE |
| `StrengthEngine` | `src/physics/strength.rs:6` | `lean://umst-formal/Lean/Powers.lean#powers_monotone` | physicalSecondLaw |
| `compute_strength_jennings` | `src/physics/strength.rs:17` | `lean://umst-formal/Lean/Powers.lean#powers_monotone` | physicalSecondLaw |
| `ThermoEngine` | `src/physics/thermo.rs:6` | `lean://umst-formal/Lean/Helmholtz.lean#ψAntitoneHelmholtz` | NONE |
| `compute_heat_rate` | `src/physics/thermo.rs:16` | `lean://umst-formal/Lean/Helmholtz.lean#ψAntitoneHelmholtz` | NONE |
| `compute_capillary_porosity` | `src/physics/transport.rs:17` | `lean://umst-formal/Lean/Powers.lean#PowersState` | NONE |
| `compute_chloride_diffusivity` | `src/physics/transport.rs:45` | `lean://umst-formal/Lean/MeasurementCost.lean#zero_info_zero_energy` | NONE |

## Structural

| Symbol | File | formal_anchor | Citation / envelope / rationale |
|--------|------|---------------|-----------------------------------|
| `CliBackend` | `crates/umst-cli/src/cli.rs:36` | `STRUCTURAL` | Burn backend selection; structural type alias to the ndarray tensor runtime. |
| `PredictionWireVersion` | `crates/umst-cli/src/cli.rs:41` | `STRUCTURAL` | Exhaustive enum over wire-schema variants; pattern matching guarantees both tags handled. |
| `MixSpec` | `crates/umst-cli/src/cli.rs:106` | `STRUCTURAL` | Field invariants enforced by `WaterCementRatio` / `TemperatureK` newtypes and range-checked fractions. |
| `CliError` | `crates/umst-cli/src/cli.rs:232` | `STRUCTURAL` | Binary-boundary error aggregation; sum-type over `MixSpecError`, calibration, tensor IO, and routing failures. |
| `PredictBundle` | `crates/umst-cli/src/cli.rs:332` | `STRUCTURAL` | Bundle of physical tensors plus calibration metadata returned by [`predict`] / [`predict_with_options`]. |
| `predict` | `crates/umst-cli/src/cli.rs:381` | `STRUCTURAL` | Natural transformation φ ∘ F ∘ ψ over the cartridge functor (CLI orchestration entry). |
| `CertifyChain` | `crates/umst-cli/src/cli.rs:528` | `STRUCTURAL` | JSON payload schema for `umst certify` output (profile, anchors, mapped formal bucket). |
| `certify_profile_json` | `crates/umst-cli/src/cli.rs:552` | `STRUCTURAL` | Builds the certify JSON view including wire `formal_status` mapped from profile metadata. |
| `OptimizeField` | `crates/umst-cli/src/cli.rs:625` | `STRUCTURAL` | Exhaustive enum of optimisation targets for the CLI bisection driver. |
| `BUNDLED_PROFILE_IDS` | `src/calibration.rs:14` | `STRUCTURAL` | Ordered manifest of bundled profile ids for `include_str!` routing. |
| `ModelKind` | `src/calibration.rs:30` | `STRUCTURAL` | Exhaustive serde enum over calibrated homogeneous model kinds. |
| `Profile` | `src/calibration.rs:182` | `STRUCTURAL` | Parsed TOML aggregate routed by `bundle_id`; field invariants delegated to nested serde structs. |
| `RegimeViolation` | `src/calibration.rs:199` | `STRUCTURAL` | Named-field regime violation records for CLI warning strings. |
| `load_bundled` | `src/calibration.rs:252` | `STRUCTURAL` | Bundled `include_str!` loader with normalized bundle id validation. |
| `profile_descriptions` | `src/calibration.rs:462` | `STRUCTURAL` | Static HashMap of tab-separated CLI profile blurbs (human-readable only). |
| `MixRow` | `src/homogeneous.rs:14` | `STRUCTURAL` | kg/m³ tagged scalars; structural carrier of mix design components for homogeneous routing. |
| `mix_row_from_scalar_spec` | `src/homogeneous.rs:289` | `STRUCTURAL` | Deterministic projection of `MixSpec` scalar inputs into `MixRow` mass fractions. |

## Empirical

| Symbol | File | formal_anchor | Citation / envelope / rationale |
|--------|------|---------------|-----------------------------------|
| `optimize_mix` | `crates/umst-cli/src/cli.rs:656` | `empirical://datasets/cli-optimize-wc-bisection.v1.csv` | "Driver-only inverse search on w/c holding other mix fields fixed" \| "tests/cli/optimize.rs" |
| `hydration_degree_calibrated` | `src/formulas.rs:24` | `empirical://datasets/hydration-kinetics-calibration-grid.v1.csv` | "Mills (1966) ultimate cap with stretched-exponential √t kinetics and Arrhenius temperature factor (calibrated multipliers from profile TOML)" \| "tests/hydration.rs::powers_doh_envelope" |
| `yield_stress_pa` | `src/homogeneous.rs:213` | `empirical://datasets/printability-rheology-yield-proxy.v1.csv` | "Roussel (2018) Cem. Concr. Res. 112, 76; Château, Ovarlez & Trung (2008) J. Rheol. 52, 489" \| "tests/printability.rs" |
| `ColloidalEngine` | `src/physics/colloidal.rs:6` | `empirical://datasets/dataset_d1.csv` | "Flatt & Bowen (2007) J. Am. Ceram. Soc. 89, 1244 (YODEL)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); DLVO pathway exercised under tests/realism/adversarial_physics.rs" |
| `compute_dlvo_potential` | `src/physics/colloidal.rs:20` | `empirical://datasets/dataset_d1.csv` | "Flatt & Bowen (2007) J. Am. Ceram. Soc. 89, 1244 (YODEL)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); DLVO pathway exercised under tests/realism/adversarial_physics.rs" |
| `compute_flocculation_multiplier` | `src/physics/colloidal.rs:82` | `empirical://datasets/dataset_d1.csv` | "Flatt & Bowen (2007) J. Am. Ceram. Soc. 89, 1244 (YODEL)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); DLVO pathway exercised under tests/realism/adversarial_physics.rs" |
| `CreepEngine` | `src/physics/creep.rs:8` | `empirical://datasets/dataset_d1.csv` | "Bažant et al. (2015) Mater. Struct. 48, 753 (RILEM B4)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); RILEM B4 creep pathway exercised under tests/creep.rs + adversarial harness" |
| `compute_compliance` | `src/physics/creep.rs:21` | `empirical://datasets/dataset_d1.csv` | "Bažant et al. (2015) Mater. Struct. 48, 753 (RILEM B4)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); RILEM B4 creep pathway exercised under tests/creep.rs + adversarial harness" |
| `FreezeThawEngine` | `src/physics/freeze_thaw.rs:8` | `empirical://datasets/dataset_d1.csv` | "Powers (1949) Highw. Res. Board Proc. 29, 184 (spacing factor)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); freeze–thaw durability pathway exercised under tests/freeze_thaw.rs + adversarial harness" |
| `compute_durability` | `src/physics/freeze_thaw.rs:21` | `empirical://datasets/dataset_d1.csv` | "Powers (1949) Highw. Res. Board Proc. 29, 184 (spacing factor)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); freeze–thaw durability pathway exercised under tests/freeze_thaw.rs + adversarial harness" |
| `compute_itz_thickness_microns` | `src/physics/itz.rs:6` | `empirical://datasets/dataset_d1.csv` | "Scrivener et al. (2004) Interface Sci. 12, 411" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); ITZ thickness/porosity pathway exercised under tests/realism/adversarial_physics.rs" |
| `compute_itz_porosity` | `src/physics/itz.rs:25` | `empirical://datasets/dataset_d1.csv` | "Scrivener et al. (2004) Interface Sci. 12, 411" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); ITZ thickness/porosity pathway exercised under tests/realism/adversarial_physics.rs" |
| `compute_itz_percolation_factor` | `src/physics/itz.rs:40` | `empirical://datasets/dataset_d1.csv` | "Scrivener et al. (2004) Interface Sci. 12, 411" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); ITZ thickness/porosity pathway exercised under tests/realism/adversarial_physics.rs" |
| `NanoEngine` | `src/physics/nano.rs:6` | `empirical://datasets/csh-nano-calibration-grid.v1.csv` | "Pellenq et al. (2009) PNAS 106, 16102" \| "tests/realism/adversarial_physics.rs" |
| `compute_enhancements` | `src/physics/nano.rs:18` | `empirical://datasets/csh-nano-calibration-grid.v1.csv` | "Pellenq et al. (2009) PNAS 106, 16102" \| "tests/realism/adversarial_physics.rs" |
| `PolymerEngine` | `src/physics/polymer.rs:6` | `empirical://datasets/dataset_highscm.csv` | "Su, van Breugel & Bijen (1991) Cem. Concr. Res. 21" \| "Headline compressive strength vs dataset_highscm.csv: MAE ≤ 60 MPa, RMSE ≤ 80 MPa, R² ≥ −10 ([acceptance] highscm.v1.toml); polymer modifiers exercised under tests/realism/adversarial_physics.rs" |
| `compute_modifiers` | `src/physics/polymer.rs:19` | `empirical://datasets/dataset_highscm.csv` | "Su, van Breugel & Bijen (1991) Cem. Concr. Res. 21" \| "Headline compressive strength vs dataset_highscm.csv: MAE ≤ 60 MPa, RMSE ≤ 80 MPa, R² ≥ −10 ([acceptance] highscm.v1.toml); polymer modifiers exercised under tests/realism/adversarial_physics.rs" |
| `PrintabilityEngine` | `src/physics/printability.rs:6` | `empirical://datasets/dataset_d1.csv` | "Roussel (2018) Cem. Concr. Res. 112, 76" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel buildability pathway exercised under tests/printability.rs + adversarial harness" |
| `compute_buildability` | `src/physics/printability.rs:20` | `empirical://datasets/dataset_d1.csv` | "Roussel (2018) Cem. Concr. Res. 112, 76" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel buildability pathway exercised under tests/printability.rs + adversarial harness" |
| `compute_extrudability` | `src/physics/printability.rs:85` | `empirical://datasets/dataset_d1.csv` | "Roussel (2018) Cem. Concr. Res. 112, 76" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel buildability pathway exercised under tests/printability.rs + adversarial harness" |
| `RheologyEngine` | `src/physics/rheology.rs:6` | `empirical://datasets/dataset_d1.csv` | "Château, Ovarlez & Trung (2008) J. Rheol. 52, 489" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel/YODEL pathway exercised under tests/rheology.rs + adversarial harness" |
| `compute_chateau_ovarlez` | `src/physics/rheology.rs:19` | `empirical://datasets/dataset_d1.csv` | "Château, Ovarlez & Trung (2008) J. Rheol. 52, 489" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel/YODEL pathway exercised under tests/rheology.rs + adversarial harness" |
| `compute_yield_stress_yodel` | `src/physics/rheology.rs:69` | `empirical://datasets/dataset_d1.csv` | "Château, Ovarlez & Trung (2008) J. Rheol. 52, 489" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel/YODEL pathway exercised under tests/rheology.rs + adversarial harness" |
| `SelfHealEngine` | `src/physics/self_heal.rs:6` | `empirical://datasets/dataset_selfheal.csv` | "Edvardsen (1999) ACI Mater. J. 96, 448" \| "Boundary profile (no [acceptance] strength gate); paired CSV still listed in dataset_metrics skip — healing kinetics exercised under tests/realism/adversarial_physics.rs" |
| `compute_healing_potential` | `src/physics/self_heal.rs:19` | `empirical://datasets/dataset_selfheal.csv` | "Edvardsen (1999) ACI Mater. J. 96, 448" \| "Boundary profile (no [acceptance] strength gate); paired CSV still listed in dataset_metrics skip — healing kinetics exercised under tests/realism/adversarial_physics.rs" |
| `ShrinkageEngine` | `src/physics/shrinkage.rs:8` | `empirical://datasets/dataset_d1.csv` | "Bažant et al. (2015) Mater. Struct. 48, 753 (B4 shrinkage model)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Bažant–Baweja shrinkage pathway exercised under tests/shrinkage.rs + adversarial harness" |
| `compute_autogenous_shrinkage` | `src/physics/shrinkage.rs:21` | `empirical://datasets/dataset_d1.csv` | "Bažant et al. (2015) Mater. Struct. 48, 753 (B4 shrinkage model)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Bažant–Baweja shrinkage pathway exercised under tests/shrinkage.rs + adversarial harness" |

## Literature

| Symbol | File | formal_anchor | Citation / envelope / rationale |
|--------|------|---------------|-----------------------------------|
| `RESULT_SCHEMA_VERSION_V1` | `crates/umst-cli/src/cli.rs:24` | `literature://wire-schema-result-v1` | "UMST concrete cartridge JSON wire schema tag (`result.v1`)" \| "`result.v1` — version tag for deprecated prediction JSON envelope" |
| `RESULT_SCHEMA_VERSION_V2` | `crates/umst-cli/src/cli.rs:30` | `literature://wire-schema-result-v2` | "UMST concrete cartridge JSON wire schema tag (`result.v2`)" \| "`result.v2` — version tag for current prediction JSON envelope" |
| `ultimate_doh_wc` | `src/formulas.rs:13` | `literature://Mills-1966-gel-stiffness-closure` | "Mills (1966); OPC gel stiffness / ultimate hydration cap closure used in routing" \| "α_inf(w/c) = 1.031·w/c / (0.194 + w/c)" |
| `ultimate_doh` | `src/homogeneous.rs:67` | `literature://Mills-1966-gel-stiffness-closure` | "Mills (1966); α_inf = 1.031 w/c / (0.194 + w/c)" \| "α_inf(w/c) = 1.031·w/c / (0.194 + w/c)" |
| `embodied_co2_kg_per_m3` | `src/homogeneous.rs:239` | `literature://EN-15804+A2-indicative-EPD-intensities` | "EN 15804+A2 (2019) environmental product declarations — indicative cradle-to-gate CO₂e intensities per constituent class" \| "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3); inline coefficients match bundled EPD intensity convention" |
| `constituent_masses_kg_m3` | `src/homogeneous.rs:266` | `literature://ACI-211.1-binder-dosage-convention` | "ACI 211.1 — Standard Practice for Selecting Proportions for Normal, Heavyweight, and Mass Concrete" \| "350 kg/m³ binder dosage convention for constituent mass reconstruction from scalar mix spec" |
| `PHYSICS_PIPELINE_SCHEMA_VERSION` | `src/lib.rs:40` | `literature://wire-schema-physics-pipeline-v1` | "physics_pipeline schema tag (`physics_pipeline.v1`)" \| "`schema_version` string on serde `PhysicsPipelineReport` — bump tag when breaking report shape." |
| `FiberEngine` | `src/physics/fiber.rs:6` | `literature://Naaman-2006-ACI-SP235-pullout` | "Naaman (2006) ACI SP-235 — fiber pullout and crack-bridging micromechanics" \| "V_{f,crit} = σ_cu / (η_l η_o τ_b l_f / d_f)" |
| `compute_micromechanics` | `src/physics/fiber.rs:17` | `literature://Naaman-2006-ACI-SP235-pullout` | "Naaman (2006) ACI SP-235 — fiber pullout and crack-bridging micromechanics" \| "V_{f,crit} = σ_cu / (η_l η_o τ_b l_f / d_f)" |
| `FractureEngine` | `src/physics/fracture.rs:6` | `literature://Ulm-Coussy-2003-micromechanics` | "Ulm & Coussy (2003) Mechanics of Porous Continua (MIT Press); micromechanics derivation" \| "K_Ic = √(2 γ_s E_eff); E_eff = E_0 (1 − φ)^n" |
| `compute_effective_modulus_mt` | `src/physics/fracture.rs:18` | `literature://Ulm-Coussy-2003-micromechanics` | "Ulm & Coussy (2003) Mechanics of Porous Continua (MIT Press); micromechanics derivation" \| "K_Ic = √(2 γ_s E_eff); E_eff = E_0 (1 − φ)^n" |
| `compute_fracture_toughness` | `src/physics/fracture.rs:88` | `literature://Ulm-Coussy-2003-micromechanics` | "Ulm & Coussy (2003) Mechanics of Porous Continua (MIT Press); micromechanics derivation" \| "K_Ic = √(2 γ_s E_eff); E_eff = E_0 (1 − φ)^n" |
| `compute_packing_density` | `src/physics/packing.rs:6` | `literature://Andreasen-Andersen-1930-Fuller-curve` | "Andreasen & Andersen (1930), Kolloid-Z. 50, 217" \| "P(D) = (D^q - D_min^q) / (D_max^q - D_min^q), q in [0.30, 0.45]" |
| `SustainabilityEngine` | `src/physics/sustainability.rs:6` | `literature://EN-15804+A2-GWP-and-unit-costs` | "EN 15804+A2 (2019) cradle-to-gate / modules A2 — indicative EPD-style CO₂e intensities; financial row uses linear $/kg mass factors" \| "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3)" |
| `compute_embodied_carbon` | `src/physics/sustainability.rs:19` | `literature://EN-15804+A2-GWP-and-unit-costs` | "EN 15804+A2 (2019) cradle-to-gate / modules A2 — indicative EPD-style CO₂e intensities; financial row uses linear $/kg mass factors" \| "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3)" |
| `compute_financial_cost` | `src/physics/sustainability.rs:51` | `literature://EN-15804+A2-GWP-and-unit-costs` | "EN 15804+A2 (2019) cradle-to-gate / modules A2 — indicative EPD-style CO₂e intensities; financial row uses linear $/kg mass factors" \| "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3)" |

## NONE

| Symbol | File | formal_anchor | Citation / envelope / rationale |
|--------|------|---------------|-----------------------------------|
| `value` | `crates/umst-cli/src/cli.rs:70` | `NONE` | Trivial accessor; getter for the wrapped `f32`. |
| `value` | `crates/umst-cli/src/cli.rs:98` | `NONE` | Trivial accessor. |
| `MixSpecError` | `crates/umst-cli/src/cli.rs:189` | `NONE` | IO / parsing error variants; classification of mix-spec rejection causes. |
| `PredictOptions` | `crates/umst-cli/src/cli.rs:322` | `NONE` | Behavioral flags only — no Lean witness. |
| `predict_with_options` | `crates/umst-cli/src/cli.rs:388` | `NONE` | Feature flag glue for MCP/CLI; no standalone formal claim. |
| `serialize_prediction` | `crates/umst-cli/src/cli.rs:473` | `NONE` | JSON-serialise glue; no physical claim. |
| `serialize_mix_spec` | `crates/umst-cli/src/cli.rs:599` | `NONE` | JSON-serialise glue. |
| `parse_optimize_target` | `crates/umst-cli/src/cli.rs:643` | `NONE` | String-parse glue for `FIELD=VALUE` optimise CLI syntax. |
| `bool_and` | `src/burn_compat.rs:8` | `NONE` | Burn-version compatibility shim for boolean tensor AND across crate semver skew. |
| `ProvenanceFormal` | `src/calibration.rs:66` | `NONE` | Serde lift of TOML `[provenance.formal]`; `status` string is file metadata (may include Boundary scope), not a Rust `formal_status` bucket. |
| `CalibrationProvenance` | `src/calibration.rs:98` | `NONE` | Dataset and Zenodo citation bundle parsed from TOML only; no Lean witness on this serde container — see `docs/FormalAnchors.md` “Future formal links” for manifold adjoint context. |
| `CalibrationModelSection` | `src/calibration.rs:150` | `NONE` | Dispatch metadata only; Jennings gel-space monotone strength witness applies once `powers_compressive_strength_mpa` ships a Jennings branch (TODO_FORMAL note on that function). |
| `ContractBlock` | `src/calibration.rs:174` | `NONE` | Contract metadata (`verification_status`); hyperbox regime warnings are soundness-witnessed on `regime_check_scalars` — see RegimeSoundness anchor there. |
| `CalibrationError` | `src/calibration.rs:208` | `NONE` | Bundled profile IO and TOML parse failures only; DEC mass-conservation witness belongs on the manifold Laplacian — see `docs/FormalAnchors.md` “Future formal links”. |
| `load_from_path` | `src/calibration.rs:260` | `NONE` | Filesystem path IO for non-bundled TOML; parse errors surface as CalibrationError. |
| `RegressionMetrics` | `src/calibration_metrics.rs:7` | `NONE` | Ordinary least-squares aggregates over paired CSV predictions; QA helper without Lean witness on this surface. |
| `regression_metrics` | `src/calibration_metrics.rs:19` | `NONE` | Same as `RegressionMetrics`; computes MAE/RMSE/R² slices for calibration reports. |
| `ConcreteCartridge` | `src/core/implementation.rs:11` | `NONE` | Cartridge functor F: mix layout → constitutive summaries; topology pass remains separate DEC hook. |
| `new` | `src/core/implementation.rs:22` | `NONE` | Deterministic bundled baseline when callers omit explicit calibration. |
| `with_profile` | `src/core/implementation.rs:30` | `NONE` | Avoids silently mixing heterogeneous tensor kinetics with unrelated gel-space coefficients. |
| `ConcreteCartridge` | `src/core/mod.rs:6` | `NONE` | Re-export; classification follows the underlying symbol. |
| `HomogeneousError` | `src/homogeneous.rs:28` | `NONE` | Dispatch error: Jennings-not-yet, invalid mix; no formal claim. |
| `mix_hydration_state` | `src/homogeneous.rs:76` | `NONE` | Internal homogeneous helper composing calibrated α(t,T,scm) and effective w/c from profile parameters. |
| `run_full_physics_pipeline` | `src/lib.rs:20` | `NONE` | Stable import path for MCP/CLI integration tests. |
| `PhysicsPipelineReport` | `src/lib.rs:24` | `NONE` | JSON envelope for staged tensor outputs. |
| `PhysicsPipelineSummary` | `src/lib.rs:28` | `NONE` | Scalar digest accompanying report JSON. |
| `PipelineStageRecord` | `src/lib.rs:32` | `NONE` | Stage record type embedded in [`PhysicsPipelineReport`]. |
| `PipelineStageStatus` | `src/lib.rs:36` | `NONE` | Serialized stage disposition enum for MCP/CLI audit trails. |
| `IScienceCartridge, MixTensor, PhysicalResult` | `src/lib.rs:46` | `NONE` | Re-export manifold façade symbols for ergonomics only. |
| `MIX_FEATURE_COUNT` | `src/mix_layout.rs:13` | `NONE` | Structural convention for CLI ↔ tensor engines; documented here as SSOT for column indices. |
| `IDX_WATER_KG_M3` | `src/mix_layout.rs:19` | `NONE` | Structural column tag for wire ↔ tensor bridging. |
| `IDX_CEMENT_KG_M3` | `src/mix_layout.rs:24` | `NONE` | Must stay aligned with `physics::hydration` slicers. |
| `IDX_AGG_COARSE_KG_M3` | `src/mix_layout.rs:29` | `NONE` | Reserved packing-engine input when recipe supplies split gradation. |
| `IDX_AGG_FINE_KG_M3` | `src/mix_layout.rs:34` | `NONE` | Reserved packing-engine input when recipe supplies split gradation. |
| `IDX_RESERVED_4` | `src/mix_layout.rs:39` | `NONE` | Keeps hydration column indices historically stable (`hydration.rs` assumptions). |
| `IDX_SLAG_KG_M3` | `src/mix_layout.rs:44` | `NONE` | Structural column aligned with SCM tensor hydrates. |
| `IDX_FLY_ASH_KG_M3` | `src/mix_layout.rs:49` | `NONE` | Structural column for pozzolan mass routing. |
| `IDX_SUPERPLASTICIZER_KG_M3` | `src/mix_layout.rs:54` | `NONE` | Admixture mass channel for homogeneous + rheology stubs. |
| `IDX_AGE_DAYS` | `src/mix_layout.rs:59` | `NONE` | Scalar age carried on-layout for `compute_all` without extra manifold state. |
| `IDX_TEMPERATURE_C` | `src/mix_layout.rs:64` | `NONE` | Arrhenius-style tensor engines consume this lane. |
| `IDX_SILICA_FUME_KG_M3` | `src/mix_layout.rs:69` | `NONE` | SCM extension field for supplemental micro-silica mass. |
| `IDX_AGGREGATE_VOLUME_FRACTION` | `src/mix_layout.rs:74` | `NONE` | Drives packing + paste fraction surrogates on collapsed paths. |
| `fractions_from_mix_row` | `src/mix_layout.rs:85` | `NONE` | Deterministic CSV-style encoding; aligns with homogeneous `MixRow` used by CLI. |
| `mix_tensor_from_layout` | `src/mix_layout.rs:106` | `NONE` | Constructor replacing non-existent `MixTensor::from_proportions`; see README Quick start. |
| `collapsed_rank4_from_rank2_scalar` | `src/mix_layout.rs:123` | `NONE` | Engine APIs require rank-4 tensors; singleton spatial dims document batch-collapsed mode. |
| `compute_cost` | `src/physics/cost.rs:7` | `NONE` | Auxiliary objective; linear cost vector, no physical claim. |
| `TransportEngine` | `src/physics/transport.rs:6` | `NONE` | Tensor facade grouping porosity and chloride diffusivity kernels documented on methods. |
| `run_full_physics_pipeline` | `src/pipeline/mod.rs:13` | `NONE` | Public entry to staged tensor physics used by `ConcreteCartridge::compute_all`. |
| `physical_result_from_report` | `src/pipeline/mod.rs:17` | `NONE` | Policy map from rich report to manifold `PhysicalResult`. |
| `PhysicsPipelineReport` | `src/pipeline/mod.rs:21` | `NONE` | Re-export pipeline wire types for CLI/MCP consumers. |
| `PhysicsPipelineSummary` | `src/pipeline/mod.rs:25` | `NONE` | Re-export pipeline wire types for CLI/MCP consumers. |
| `PipelineStageRecord` | `src/pipeline/mod.rs:29` | `NONE` | Re-export pipeline wire types for CLI/MCP consumers. |
| `PipelineStageStatus` | `src/pipeline/mod.rs:33` | `NONE` | Re-export pipeline wire types for CLI/MCP consumers. |
| `PHYSICS_PIPELINE_SCHEMA_VERSION` | `src/pipeline/mod.rs:37` | `NONE` | Re-export pipeline wire types for CLI/MCP consumers. |
| `run_full_physics_pipeline` | `src/pipeline/orchestrator.rs:106` | `NONE` | Cartridge functor composition root exercised by tooling + tests. |
| `physical_result_from_report` | `src/pipeline/physical_summary.rs:23` | `NONE` | Encodes CLI tensor summary contract; see module-level policy mapping. |
| `PHYSICS_PIPELINE_SCHEMA_VERSION` | `src/pipeline/report.rs:11` | `NONE` | Version discriminator for additive JSON fields. |
| `PipelineStageStatus` | `src/pipeline/report.rs:17` | `NONE` | Enumerates audited stage outcomes (`Executed` vs honest skips/failures). |
| `PipelineStageRecord` | `src/pipeline/report.rs:29` | `NONE` | Serialized evidence that a stage ran, skipped, or failed. |
| `ok` | `src/pipeline/report.rs:43` | `NONE` | Successful stage marker helper. |
| `skip_missing` | `src/pipeline/report.rs:55` | `NONE` | Honest skip when inputs/constants are absent by design. |
| `fail` | `src/pipeline/report.rs:67` | `NONE` | Propagates panics-avoiding error strings for observability. |
| `PhysicsPipelineSummary` | `src/pipeline/report.rs:79` | `NONE` | Human/tooling digest; not a substitute for full tensor fields. |
| `PhysicsPipelineReport` | `src/pipeline/report.rs:103` | `NONE` | Cartridge-local rich JSON envelope parallel to manifold tensors. |

