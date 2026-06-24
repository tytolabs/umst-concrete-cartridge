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
| **Mechanised** | 34 |
| **Structural** | 93 |
| **Empirical** | 34 |
| **Literature** | 50 |
| **NONE** | 274 |

## Mechanised

| Symbol | File | formal_anchor | catalog_id | Citation / envelope / rationale |
|--------|------|---------------|------------|-----------------------------------|
| `PowersGelParameters` | `crates/umst-concrete-cartridge/src/calibration.rs:42` | `lean://umst-formal/Lean/Powers.lean#PowersState` | thermodynamic_mix | physicalSecondLaw |
| `FormalBlock` | `crates/umst-concrete-cartridge/src/calibration.rs:56` | `lean://umst-formal/Lean/Gate.lean#Admissible` | umst.gate.cd_transition | physicalSecondLaw |
| `CalibrationMeta` | `crates/umst-concrete-cartridge/src/calibration.rs:83` | `lean://umst-formal/Lean/Powers.lean#S_intrinsic` | thermodynamic_mix | physicalSecondLaw |
| `RegimeBounds` | `crates/umst-concrete-cartridge/src/calibration.rs:129` | `lean://umst-formal/Lean/OrderStatisticsBand.lean#order_statistic_concentration` | umst.cartridge.concrete.acceptance_band | NONE |
| `AcceptanceBlock` | `crates/umst-concrete-cartridge/src/calibration.rs:164` | `lean://umst-formal/Lean/OrderStatisticsBand.lean#p25_p75_admissibility` | umst.cartridge.concrete.acceptance_band | NONE |
| `regime_check_scalars` | `crates/umst-concrete-cartridge/src/calibration.rs:315` | `lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | umst.cartridge.concrete.regime | NONE |
| `any_bundled_profile_covers_scalars` | `crates/umst-concrete-cartridge/src/calibration.rs:401` | `lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | umst.cartridge.concrete.regime | NONE |
| `WaterCementRatio` | `crates/umst-concrete-cartridge/src/facade/mod.rs:90` | `lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | umst.cartridge.concrete.regime | NONE |
| `TemperatureK` | `crates/umst-concrete-cartridge/src/facade/mod.rs:119` | `lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | umst.cartridge.concrete.regime | NONE |
| `powers_compressive_strength_mpa` | `crates/umst-concrete-cartridge/src/homogeneous.rs:139` | `lean://umst-formal/Lean/Powers.lean#powers_monotone` | thermodynamic_mix | physicalSecondLaw |
| `compressive_strength_mpa` | `crates/umst-concrete-cartridge/src/homogeneous.rs:191` | `lean://umst-formal/Lean/Powers.lean#PowersState` | thermodynamic_mix | physicalSecondLaw |
| `degree_of_hydration_alpha` | `crates/umst-concrete-cartridge/src/homogeneous.rs:200` | `lean://umst-formal/Lean/Powers.lean#powers_monotone` | thermodynamic_mix | physicalSecondLaw |
| `capillary_porosity` | `crates/umst-concrete-cartridge/src/homogeneous.rs:208` | `lean://umst-formal/Lean/Powers.lean#PowersState` | thermodynamic_mix | NONE |
| `safety_margin` | `crates/umst-concrete-cartridge/src/homogeneous.rs:258` | `lean://umst-formal/Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | umst.cartridge.concrete.regime | NONE |
| `ChemoWaterEngine` | `crates/umst-concrete-cartridge/src/physics/chemo_water.rs:6` | `lean://umst-formal/Lean/Powers.lean#PowersState` | thermodynamic_mix | physicalSecondLaw |
| `compute_moisture_transport` | `crates/umst-concrete-cartridge/src/physics/chemo_water.rs:17` | `lean://umst-formal/Lean/Powers.lean#PowersState` | thermodynamic_mix | physicalSecondLaw |
| `compute_hydration_degree` | `crates/umst-concrete-cartridge/src/physics/hydration.rs:7` | `lean://umst-formal/Lean/JenningsGelSpace.lean#jennings_strength_monotone` | umst.cartridge.concrete.jennings_gel | NONE |
| `compute_capillary_porosity` | `crates/umst-concrete-cartridge/src/physics/porosity.rs:6` | `lean://umst-formal/Lean/Powers.lean#PowersState` | thermodynamic_mix | NONE |
| `SetTimeEngine` | `crates/umst-concrete-cartridge/src/physics/set_time.rs:6` | `lean://umst-formal/Lean/Helmholtz.lean#ψAntitoneHelmholtz` | umst.gate.cd_transition | NONE |
| `compute_setting_time` | `crates/umst-concrete-cartridge/src/physics/set_time.rs:18` | `lean://umst-formal/Lean/Helmholtz.lean#ψAntitoneHelmholtz` | umst.gate.cd_transition | NONE |
| `StrengthEngine` | `crates/umst-concrete-cartridge/src/physics/strength.rs:41` | `lean://umst-formal/Lean/Powers.lean#powers_monotone` | thermodynamic_mix | physicalSecondLaw |
| `compute_strength_jennings` | `crates/umst-concrete-cartridge/src/physics/strength.rs:53` | `lean://umst-formal/Lean/Powers.lean#powers_monotone` | thermodynamic_mix | physicalSecondLaw |
| `ThermoEngine` | `crates/umst-concrete-cartridge/src/physics/thermo.rs:6` | `lean://umst-formal/Lean/Helmholtz.lean#ψAntitoneHelmholtz` | umst.gate.cd_transition | NONE |
| `compute_heat_rate` | `crates/umst-concrete-cartridge/src/physics/thermo.rs:17` | `lean://umst-formal/Lean/Helmholtz.lean#ψAntitoneHelmholtz` | umst.gate.cd_transition | NONE |
| `compute_capillary_porosity` | `crates/umst-concrete-cartridge/src/physics/transport.rs:17` | `lean://umst-formal/Lean/Powers.lean#PowersState` | thermodynamic_mix | NONE |
| `compute_chloride_diffusivity` | `crates/umst-concrete-cartridge/src/physics/transport.rs:46` | `lean://umst-formal/Lean/MeasurementCost.lean#zero_info_zero_energy` | umst.gate.landauer_cbf | NONE |
| `CD_TRANSITION_CATALOG_ID` | `crates/umst-concrete-cartridge/src/pipeline/dual_gate.rs:21` | `lean://umst-formal/Lean/Gate.lean#Admissible` | umst.gate.cd_transition | physicalSecondLaw |
| `thermodynamic_ok` | `crates/umst-concrete-cartridge/src/pipeline/dual_gate.rs:97` | `lean://umst-formal/Lean/Gate.lean#Admissible` | umst.gate.cd_transition | physicalSecondLaw |
| `gate_check_mix` | `crates/umst-concrete-cartridge/src/research/contribution.rs:100` | `lean://umst-formal/Lean/Gate.lean#Admissible` | umst.gate.cd_transition | physicalSecondLaw |
| `gate_check_mix_result` | `crates/umst-concrete-cartridge/src/research/contribution.rs:167` | `lean://umst-formal/Lean/Gate.lean#Admissible` | umst.gate.cd_transition | physicalSecondLaw |
| `gate_recheck` | `crates/umst-concrete-cartridge/src/research/contribution.rs:438` | `lean://umst-formal/Lean/Gate.lean#Admissible` | umst.gate.cd_transition | physicalSecondLaw |
| `accept` | `crates/umst-concrete-cartridge/src/research/contribution.rs:508` | `lean://umst-formal/Lean/Gate.lean#Admissible` | umst.gate.cd_transition | physicalSecondLaw |
| `gate_check` | `crates/umst-py/src/lib.rs:244` | `lean://umst-formal/Lean/Gate.lean#Admissible` | umst.gate.cd_transition | physicalSecondLaw |
| `contribute` | `crates/umst-py/src/lib.rs:336` | `lean://umst-formal/Lean/Gate.lean#Admissible` | umst.gate.cd_transition | physicalSecondLaw |

## Structural

| Symbol | File | formal_anchor | catalog_id | Citation / envelope / rationale |
|--------|------|---------------|------------|-----------------------------------|
| `certify_profile_chain, mix_spec_wire_out, predict, predict_with_options, prediction_wire_v1, prediction_wire_v2, tensor_element_at, CertifyChain, FacadeBackend, FacadeError, HomogeneousCompareWire, MixSpec, MixSpecError, MixSpecWire, MixSpecWireOut, PredictBundle, PredictOptions, PredictionWireV1, PredictionWireV2, PredictionWireVersion` | `crates/umst-cli/src/cli.rs:13` | `STRUCTURAL` | — | Thin transport re-export of `umst_concrete_cartridge::facade`; authoritative formal blocks live on facade definitions. |
| `CliBackend` | `crates/umst-cli/src/cli.rs:38` | `STRUCTURAL` | — | CLI re-export; ndarray backend alias for historical `CliBackend` name. |
| `CliError` | `crates/umst-cli/src/cli.rs:43` | `STRUCTURAL` | — | Binary-boundary error aggregation; extends [`FacadeError`] with JSON/optimise glue. |
| `certify_profile_json` | `crates/umst-cli/src/cli.rs:142` | `STRUCTURAL` | — | JSON Value view of certify chain; structural wrapper over [`certify_profile_chain`]. |
| `OptimizeField` | `crates/umst-cli/src/cli.rs:159` | `STRUCTURAL` | — | Exhaustive enum of optimisation targets for the CLI bisection / coordinate-descent driver. |
| `BUNDLED_PROFILE_IDS` | `crates/umst-concrete-cartridge/src/calibration.rs:15` | `STRUCTURAL` | — | Ordered manifest of bundled profile ids for `include_str!` routing. |
| `ModelKind` | `crates/umst-concrete-cartridge/src/calibration.rs:32` | `STRUCTURAL` | — | Exhaustive serde enum over calibrated homogeneous model kinds. |
| `Profile` | `crates/umst-concrete-cartridge/src/calibration.rs:189` | `STRUCTURAL` | — | Parsed TOML aggregate routed by `bundle_id`; field invariants delegated to nested serde structs. |
| `RegimeViolation` | `crates/umst-concrete-cartridge/src/calibration.rs:208` | `STRUCTURAL` | — | Named-field regime violation records for CLI warning strings. |
| `load_bundled` | `crates/umst-concrete-cartridge/src/calibration.rs:261` | `STRUCTURAL` | — | Bundled `include_str!` loader with normalized bundle id validation. |
| `profile_descriptions` | `crates/umst-concrete-cartridge/src/calibration.rs:477` | `STRUCTURAL` | — | Static HashMap of tab-separated CLI profile blurbs (human-readable only). |
| `FacadeBackend` | `crates/umst-concrete-cartridge/src/facade/mod.rs:75` | `STRUCTURAL` | — | Burn backend selection; structural type alias to the ndarray tensor runtime. |
| `PredictionWireVersion` | `crates/umst-concrete-cartridge/src/facade/mod.rs:80` | `STRUCTURAL` | — | Exhaustive enum over wire-schema variants; pattern matching guarantees both tags handled. |
| `MixSpec` | `crates/umst-concrete-cartridge/src/facade/mod.rs:147` | `STRUCTURAL` | — | Field invariants enforced by `WaterCementRatio` / `TemperatureK` newtypes and range-checked fractions. |
| `MixSpecWire` | `crates/umst-concrete-cartridge/src/facade/mod.rs:162` | `STRUCTURAL` | — | Serde shape for mix.v1 JSON; field validation on conversion to [`MixSpec`]. |
| `FacadeError` | `crates/umst-concrete-cartridge/src/facade/mod.rs:260` | `STRUCTURAL` | — | Binary-boundary error aggregation for facade calls (no vendor IO). |
| `HomogeneousCompareWire` | `crates/umst-concrete-cartridge/src/facade/mod.rs:315` | `STRUCTURAL` | — | Homogeneous sidecar scalars for optional regression diff (serde-friendly). |
| `PredictBundle` | `crates/umst-concrete-cartridge/src/facade/mod.rs:337` | `STRUCTURAL` | — | Bundle of physical tensors plus calibration metadata returned by [`predict`] / [`predict_with_options`]. |
| `predict` | `crates/umst-concrete-cartridge/src/facade/mod.rs:425` | `STRUCTURAL` | — | Natural transformation φ ∘ F ∘ ψ over the cartridge functor (facade orchestration entry). |
| `predict_from_mix_row` | `crates/umst-concrete-cartridge/src/facade/mod.rs:522` | `STRUCTURAL` | — | Tensor prediction from dataset-style [`homog::MixRow`] masses; regime gates use binder-normalised SCM splits (slag routed through the silica regime channel). |
| `PreparedAuditRow` | `crates/umst-concrete-cartridge/src/facade/mod.rs:617` | `STRUCTURAL` | — | One CSV row wired for corpus audit alongside aggregate packing fraction derived in CLI from coarse/fine masses (ρ=2600 kg/m³, same surrogate as homogeneous layout). |
| `audit_build_report_v1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:686` | `STRUCTURAL` | — | Deterministic corpus audit projection over prepared rows (tensor strength channel). |
| `CertifyChain` | `crates/umst-concrete-cartridge/src/facade/mod.rs:758` | `STRUCTURAL` | — | JSON payload schema for `umst certify` output (profile, anchors, mapped formal bucket). |
| `certify_profile_chain` | `crates/umst-concrete-cartridge/src/facade/mod.rs:783` | `STRUCTURAL` | — | Builds the certify view including wire `formal_status` mapped from profile metadata. |
| `schema_mix_v1_json` | `crates/umst-concrete-cartridge/src/facade/mod.rs:818` | `STRUCTURAL` | — | SSOT `include_str!` of repo-root schema for CLI/MCP/Python. |
| `schema_result_v1_json` | `crates/umst-concrete-cartridge/src/facade/mod.rs:827` | `STRUCTURAL` | — | SSOT `include_str!` of repo-root schema for CLI/MCP/Python. |
| `schema_result_v2_json` | `crates/umst-concrete-cartridge/src/facade/mod.rs:836` | `STRUCTURAL` | — | SSOT `include_str!` of repo-root schema for CLI/MCP/Python. |
| `schema_audit_v1_json` | `crates/umst-concrete-cartridge/src/facade/mod.rs:845` | `STRUCTURAL` | — | SSOT `include_str!` of repo-root schema for CLI `umst audit`. |
| `ConcreteTransitionWitness` | `crates/umst-concrete-cartridge/src/gate_evidence.rs:14` | `STRUCTURAL` | — | Telemetry envelope around manifold [`TransitionEvidence`]; CD admissibility on `core` leg. |
| `ConcreteTransitionCartridge` | `crates/umst-concrete-cartridge/src/gate_evidence.rs:27` | `STRUCTURAL` | — | Zero-sized [`GateCartridge`] witness binding cement SSOT into manifold transition evidence. |
| `snapshot_from_mix` | `crates/umst-concrete-cartridge/src/gate_evidence.rs:35` | `STRUCTURAL` | — | Deterministic lift of mix scalars into [`ThermodynamicStateSnapshot`] via cartridge cement params. |
| `dissipation_joules` | `crates/umst-concrete-cartridge/src/gate_evidence.rs:54` | `STRUCTURAL` | — | Delegates dissipation to manifold `transition_outcome` host path for telemetry parity. |
| `transition_witness` | `crates/umst-concrete-cartridge/src/gate_evidence.rs:68` | `STRUCTURAL` | — | Composes manifold `transition_evidence` with cartridge dissipation telemetry. |
| `MixRow` | `crates/umst-concrete-cartridge/src/homogeneous.rs:14` | `STRUCTURAL` | — | kg/m³ tagged scalars; structural carrier of mix design components for homogeneous routing. |
| `mix_row_from_scalar_spec` | `crates/umst-concrete-cartridge/src/homogeneous.rs:294` | `STRUCTURAL` | — | Deterministic projection of `MixSpec` scalar inputs into `MixRow` mass fractions. |
| `query` | `crates/umst-concrete-cartridge/src/research/contribution.rs:65` | `STRUCTURAL` | — | Functor to store `query`; filter semantics on `filter_records`. |
| `content_hash_preimage` | `crates/umst-concrete-cartridge/src/research/contribution.rs:74` | `STRUCTURAL` | — | Deterministic field bundle for SHA-256; no admissibility claim. |
| `rational_to_f64` | `crates/umst-concrete-cartridge/src/research/contribution.rs:383` | `STRUCTURAL` | — | Wire decode only; physical units validated downstream on `MixSpec`. |
| `mix_wire_from_spec_value` | `crates/umst-concrete-cartridge/src/research/contribution.rs:406` | `STRUCTURAL` | — | Serde routing to facade wire; gate on `MixSpec::try_from`. |
| `content_id` | `crates/umst-concrete-cartridge/src/research/contribution.rs:464` | `STRUCTURAL` | — | Content-addressed memory key; hash of wire fields not physics. |
| `memory_record_from_contribution` | `crates/umst-concrete-cartridge/src/research/contribution.rs:475` | `STRUCTURAL` | — | memory_record.v1 projection; mix_geometry from Morton on mix_spec. |
| `PhysicalReasoningLayer` | `crates/umst-concrete-cartridge/src/research/layer.rs:10` | `STRUCTURAL` | — | Cartridge port trait; geometry hook defers to `mix_geometry_key`. |
| `cartridge_slug` | `crates/umst-concrete-cartridge/src/research/layer.rs:18` | `STRUCTURAL` | — | Namespace id for MCP resource URIs; no physics claim. |
| `contribution_schema` | `crates/umst-concrete-cartridge/src/research/layer.rs:24` | `STRUCTURAL` | — | Default contribution.v1 schema id re-export. |
| `memory_schema` | `crates/umst-concrete-cartridge/src/research/layer.rs:32` | `STRUCTURAL` | — | Default memory_record.v1 schema id re-export. |
| `mix_geometry` | `crates/umst-concrete-cartridge/src/research/layer.rs:40` | `STRUCTURAL` | — | Delegates to Morton geometry; not thermodynamic gate. |
| `default_memory_query` | `crates/umst-concrete-cartridge/src/research/layer.rs:52` | `STRUCTURAL` | — | Default filter record for agent memory queries. |
| `matches_query` | `crates/umst-concrete-cartridge/src/research/layer.rs:63` | `STRUCTURAL` | — | Optional cartridge-specific query predicate hook. |
| `ConcretePhysicalReasoningLayer` | `crates/umst-concrete-cartridge/src/research/layer.rs:72` | `STRUCTURAL` | — | Zero-sized port impl for umst-concrete-cartridge agent layer. |
| `find_by_memory_id` | `crates/umst-concrete-cartridge/src/research/memory.rs:33` | `STRUCTURAL` | — | Linear scan morphism; no interior mutation. |
| `MemoryStore` | `crates/umst-concrete-cartridge/src/research/memory.rs:49` | `STRUCTURAL` | — | Type-class for functional append; no global mutable store. |
| `append` | `crates/umst-concrete-cartridge/src/research/memory.rs:56` | `STRUCTURAL` | — | Functional append morphism; duplicate content_id rejected. |
| `rows` | `crates/umst-concrete-cartridge/src/research/memory.rs:64` | `STRUCTURAL` | — | Immutable row snapshot for pure `filter_records`. |
| `filter_records` | `crates/umst-concrete-cartridge/src/research/memory.rs:135` | `STRUCTURAL` | — | Query filter over row slice; L1/Morton sort when requested. |
| `query_page` | `crates/umst-concrete-cartridge/src/research/memory.rs:144` | `STRUCTURAL` | — | Page envelope for MCP `umst_memory_query`; cursor is prior `content_id`. |
| `InMemoryStore` | `crates/umst-concrete-cartridge/src/research/memory.rs:197` | `STRUCTURAL` | — | Functional in-memory store; rows + idempotency keys owned by value. |
| `new` | `crates/umst-concrete-cartridge/src/research/memory.rs:208` | `STRUCTURAL` | — | Zero-row initializer for tests and default session. |
| `from_rows` | `crates/umst-concrete-cartridge/src/research/memory.rs:220` | `STRUCTURAL` | — | Test/fixture constructor; no duplicate checks on load. |
| `query` | `crates/umst-concrete-cartridge/src/research/memory.rs:232` | `STRUCTURAL` | — | Delegates to `filter_records` on owned row slice. |
| `in_memory` | `crates/umst-concrete-cartridge/src/research/memory.rs:275` | `STRUCTURAL` | — | Constructs functional in-memory arm only. |
| `UcrsStampMode` | `crates/umst-concrete-cartridge/src/research/provenance.rs:41` | `STRUCTURAL` | — | Live vs synthetic stamp functor selection on accept. |
| `ProvenanceClock` | `crates/umst-concrete-cartridge/src/research/provenance.rs:68` | `STRUCTURAL` | — | Immutable clock state; `advance` is pure given injected wall. |
| `from_env` | `crates/umst-concrete-cartridge/src/research/provenance.rs:102` | `STRUCTURAL` | — | Session boundary initializer; IO on env read only. |
| `new` | `crates/umst-concrete-cartridge/src/research/provenance.rs:111` | `STRUCTURAL` | — | Clock initializer; sequence threaded through accept. |
| `with_mode` | `crates/umst-concrete-cartridge/src/research/provenance.rs:120` | `STRUCTURAL` | — | Clock initializer with live/synthetic witness functor. |
| `mode` | `crates/umst-concrete-cartridge/src/research/provenance.rs:144` | `STRUCTURAL` | — | Read-only stamp mode for session threading. |
| `sequence` | `crates/umst-concrete-cartridge/src/research/provenance.rs:158` | `STRUCTURAL` | — | Read-only access to threaded sequence state. |
| `advance` | `crates/umst-concrete-cartridge/src/research/provenance.rs:167` | `STRUCTURAL` | — | Functional clock step; wall_ms injected at boundary only. |
| `observed_at_for_tick` | `crates/umst-concrete-cartridge/src/research/provenance.rs:205` | `STRUCTURAL` | — | Pure stamp from seq + wall; UCRS fields when feature enabled. |
| `ensure_observed_at` | `crates/umst-concrete-cartridge/src/research/provenance.rs:231` | `STRUCTURAL` | — | Stamp merge on accept; monotonicity checked via `is_monotonic_after`. |
| `is_monotonic_after` | `crates/umst-concrete-cartridge/src/research/provenance.rs:252` | `STRUCTURAL` | — | Monotonic ordering on UCRS seq with wall_ms and phase_q tie-break. |
| `GateVerdict` | `crates/umst-concrete-cartridge/src/research/types.rs:26` | `STRUCTURAL` | — | Serde-shaped verdict tag; admissibility on `GateSummary.admissible`. |
| `GateSummary` | `crates/umst-concrete-cartridge/src/research/types.rs:38` | `STRUCTURAL` | — | Wire bundle of verdict + catalog_id witnesses from gate path. |
| `ObservedAt` | `crates/umst-concrete-cartridge/src/research/types.rs:53` | `STRUCTURAL` | — | observed_at.v1/v2 wire; monotonicity checked in provenance. |
| `Contribution` | `crates/umst-concrete-cartridge/src/research/types.rs:74` | `STRUCTURAL` | — | Agent ingest wire; gate fields validated before accept. |
| `MemoryRecord` | `crates/umst-concrete-cartridge/src/research/types.rs:96` | `STRUCTURAL` | — | Gate-validated memory shape; query filters on payload + mix_geometry. |
| `MemoryPayload` | `crates/umst-concrete-cartridge/src/research/types.rs:115` | `STRUCTURAL` | — | Nested wire bundle inside memory_record.v1. |
| `MemoryQuery` | `crates/umst-concrete-cartridge/src/research/types.rs:127` | `STRUCTURAL` | — | Pure filter record; semantics in `memory::filter_records`. |
| `MemoryQueryPage` | `crates/umst-concrete-cartridge/src/research/types.rs:158` | `STRUCTURAL` | — | Stable-sort page envelope; `next_cursor` for MCP agents. |
| `AcceptResult` | `crates/umst-concrete-cartridge/src/research/types.rs:169` | `STRUCTURAL` | — | MCP contribute response wire; ids assigned at accept boundary. |
| `ValidationError` | `crates/umst-concrete-cartridge/src/research/validation.rs:10` | `STRUCTURAL` | — | Schema routing for contribution.v1 / memory_record.v1 wire shapes. |
| `is_valid_rational` | `crates/umst-concrete-cartridge/src/research/validation.rs:34` | `STRUCTURAL` | — | Rational grammar for mix_spec rationals; no physics claim. |
| `validate_contribution_value` | `crates/umst-concrete-cartridge/src/research/validation.rs:106` | `STRUCTURAL` | — | contribution.v1 schema routing; gate fields checked before accept. |
| `validate_for_accept` | `crates/umst-concrete-cartridge/src/research/validation.rs:140` | `STRUCTURAL` | — | Accept-path guard; thermodynamic re-check on `gate_recheck` after parse. |
| `parse_contribution_json` | `crates/umst-concrete-cartridge/src/research/validation.rs:152` | `STRUCTURAL` | — | JSON parse + `validate_contribution_value`; no store I/O. |
| `ObservedAtV2` | `crates/umst-concrete-cartridge/src/research/wire_v2.rs:15` | `STRUCTURAL` | — | UCRS wire functor; monotonicity checked on v1 `provenance` path. |
| `observed_at_to_v2` | `crates/umst-concrete-cartridge/src/research/wire_v2.rs:37` | `STRUCTURAL` | — | Field-preserving projection to observed_at.v2; no new stamp semantics. |
| `ucrs_observed_at_to_v2` | `crates/umst-concrete-cartridge/src/research/wire_v2.rs:56` | `STRUCTURAL` | — | UCRS crate → v2 wire functor; monotonicity on `is_monotonic_after`. |
| `with_gate_cartridge` | `crates/umst-concrete-cartridge/src/thmc_gate.rs:12` | `STRUCTURAL` | — | Configures [`ThmcSolver`] intrinsic strength from cement SSOT before coupled step. |
| `gate_cartridge_witness` | `crates/umst-concrete-cartridge/src/thmc_gate.rs:25` | `STRUCTURAL` | — | Routes post-step evidence through cartridge [`GateCartridge`] witness. |
| `predict` | `crates/umst-py/src/lib.rs:115` | `STRUCTURAL` | — | Python transport wrapper over **[`predict_with_options`]**; anchored on facade predict path. |
| `certify` | `crates/umst-py/src/lib.rs:186` | `STRUCTURAL` | — | Dict view of **[`certify_profile_json`]**; structural mirror of CLI `umst certify`. |
| `schema` | `crates/umst-py/src/lib.rs:198` | `STRUCTURAL` | — | SSOT schema text from facade `include_str!` for notebooks and packaging checks. |

## Empirical

| Symbol | File | formal_anchor | catalog_id | Citation / envelope / rationale |
|--------|------|---------------|------------|-----------------------------------|
| `optimize_mix` | `crates/umst-cli/src/cli.rs:196` | `empirical://datasets/cli-optimize-wc-bisection.v1.csv` | — | "Driver-only inverse search on w/c holding other mix fields fixed" \| "tests/cli/optimize.rs" |
| `RheologyCalibrationBlock` | `crates/umst-concrete-cartridge/src/calibration_fit.rs:11` | `empirical://datasets/printability-rheology-yield-proxy.v1.csv` | — | "In-house Tyto mortar yield proxy calibration" \| "tests/calibration_tyto_mortar.rs" |
| `apply_tau0_calibration` | `crates/umst-concrete-cartridge/src/calibration_fit.rs:36` | `empirical://datasets/printability-rheology-yield-proxy.v1.csv` | — | "In-house Tyto mortar yield proxy calibration" \| "tests/calibration_tyto_mortar.rs" |
| `fit_theta_tau0_single_mix` | `crates/umst-concrete-cartridge/src/calibration_fit.rs:48` | `empirical://datasets/printability-rheology-yield-proxy.v1.csv` | — | "In-house Tyto mortar yield proxy calibration" \| "tests/calibration_tyto_mortar.rs" |
| `effective_theta_tau0` | `crates/umst-concrete-cartridge/src/calibration_fit.rs:66` | `empirical://datasets/printability-rheology-yield-proxy.v1.csv` | — | "In-house Tyto mortar yield proxy calibration" \| "tests/calibration_tyto_mortar.rs" |
| `calibrated_tau0_pa` | `crates/umst-concrete-cartridge/src/calibration_fit.rs:88` | `empirical://datasets/printability-rheology-yield-proxy.v1.csv` | — | "In-house Tyto mortar yield proxy calibration" \| "tests/calibration_tyto_mortar.rs" |
| `hydration_degree_calibrated` | `crates/umst-concrete-cartridge/src/formulas.rs:24` | `empirical://datasets/hydration-kinetics-calibration-grid.v1.csv` | — | "Mills (1966) ultimate cap with stretched-exponential √t kinetics and Arrhenius temperature factor (calibrated multipliers from profile TOML)" \| "tests/hydration.rs::powers_doh_envelope" |
| `yield_stress_pa` | `crates/umst-concrete-cartridge/src/homogeneous.rs:217` | `empirical://datasets/printability-rheology-yield-proxy.v1.csv` | — | "Roussel (2018) Cem. Concr. Res. 112, 76; Château, Ovarlez & Trung (2008) J. Rheol. 52, 489" \| "tests/printability.rs" |
| `ColloidalEngine` | `crates/umst-concrete-cartridge/src/physics/colloidal.rs:6` | `empirical://datasets/dataset_d1.csv` | — | "Flatt & Bowen (2007) J. Am. Ceram. Soc. 89, 1244 (YODEL)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); DLVO pathway exercised under tests/realism/adversarial_physics.rs" |
| `compute_dlvo_potential` | `crates/umst-concrete-cartridge/src/physics/colloidal.rs:20` | `empirical://datasets/dataset_d1.csv` | — | "Flatt & Bowen (2007) J. Am. Ceram. Soc. 89, 1244 (YODEL)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); DLVO pathway exercised under tests/realism/adversarial_physics.rs" |
| `compute_flocculation_multiplier` | `crates/umst-concrete-cartridge/src/physics/colloidal.rs:82` | `empirical://datasets/dataset_d1.csv` | — | "Flatt & Bowen (2007) J. Am. Ceram. Soc. 89, 1244 (YODEL)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); DLVO pathway exercised under tests/realism/adversarial_physics.rs" |
| `CreepEngine` | `crates/umst-concrete-cartridge/src/physics/creep.rs:8` | `empirical://datasets/dataset_d1.csv` | — | "Bažant et al. (2015) Mater. Struct. 48, 753 (RILEM B4)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); RILEM B4 creep pathway exercised under tests/creep.rs + adversarial harness" |
| `compute_compliance` | `crates/umst-concrete-cartridge/src/physics/creep.rs:21` | `empirical://datasets/dataset_d1.csv` | — | "Bažant et al. (2015) Mater. Struct. 48, 753 (RILEM B4)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); RILEM B4 creep pathway exercised under tests/creep.rs + adversarial harness" |
| `FreezeThawEngine` | `crates/umst-concrete-cartridge/src/physics/freeze_thaw.rs:8` | `empirical://datasets/dataset_d1.csv` | — | "Powers (1949) Highw. Res. Board Proc. 29, 184 (spacing factor)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); freeze–thaw durability pathway exercised under tests/freeze_thaw.rs + adversarial harness" |
| `compute_durability` | `crates/umst-concrete-cartridge/src/physics/freeze_thaw.rs:21` | `empirical://datasets/dataset_d1.csv` | — | "Powers (1949) Highw. Res. Board Proc. 29, 184 (spacing factor)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); freeze–thaw durability pathway exercised under tests/freeze_thaw.rs + adversarial harness" |
| `compute_itz_thickness_microns` | `crates/umst-concrete-cartridge/src/physics/itz.rs:6` | `empirical://datasets/dataset_d1.csv` | — | "Scrivener et al. (2004) Interface Sci. 12, 411" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); ITZ thickness/porosity pathway exercised under tests/realism/adversarial_physics.rs" |
| `compute_itz_porosity` | `crates/umst-concrete-cartridge/src/physics/itz.rs:25` | `empirical://datasets/dataset_d1.csv` | — | "Scrivener et al. (2004) Interface Sci. 12, 411" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); ITZ thickness/porosity pathway exercised under tests/realism/adversarial_physics.rs" |
| `compute_itz_percolation_factor` | `crates/umst-concrete-cartridge/src/physics/itz.rs:40` | `empirical://datasets/dataset_d1.csv` | — | "Scrivener et al. (2004) Interface Sci. 12, 411" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); ITZ thickness/porosity pathway exercised under tests/realism/adversarial_physics.rs" |
| `NanoEngine` | `crates/umst-concrete-cartridge/src/physics/nano.rs:6` | `empirical://datasets/csh-nano-calibration-grid.v1.csv` | — | "Pellenq et al. (2009) PNAS 106, 16102" \| "tests/realism/adversarial_physics.rs" |
| `compute_enhancements` | `crates/umst-concrete-cartridge/src/physics/nano.rs:18` | `empirical://datasets/csh-nano-calibration-grid.v1.csv` | — | "Pellenq et al. (2009) PNAS 106, 16102" \| "tests/realism/adversarial_physics.rs" |
| `PolymerEngine` | `crates/umst-concrete-cartridge/src/physics/polymer.rs:6` | `empirical://datasets/dataset_highscm.csv` | — | "Su, van Breugel & Bijen (1991) Cem. Concr. Res. 21" \| "Headline compressive strength vs dataset_highscm.csv: MAE ≤ 60 MPa, RMSE ≤ 80 MPa, R² ≥ −10 ([acceptance] highscm.v1.toml); polymer modifiers exercised under tests/realism/adversarial_physics.rs" |
| `compute_modifiers` | `crates/umst-concrete-cartridge/src/physics/polymer.rs:19` | `empirical://datasets/dataset_highscm.csv` | — | "Su, van Breugel & Bijen (1991) Cem. Concr. Res. 21" \| "Headline compressive strength vs dataset_highscm.csv: MAE ≤ 60 MPa, RMSE ≤ 80 MPa, R² ≥ −10 ([acceptance] highscm.v1.toml); polymer modifiers exercised under tests/realism/adversarial_physics.rs" |
| `PrintabilityEngine` | `crates/umst-concrete-cartridge/src/physics/printability.rs:6` | `empirical://datasets/dataset_d1.csv` | — | "Roussel (2018) Cem. Concr. Res. 112, 76" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel buildability pathway exercised under tests/printability.rs + adversarial harness" |
| `compute_buildability` | `crates/umst-concrete-cartridge/src/physics/printability.rs:20` | `empirical://datasets/dataset_d1.csv` | — | "Roussel (2018) Cem. Concr. Res. 112, 76" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel buildability pathway exercised under tests/printability.rs + adversarial harness" |
| `compute_extrudability` | `crates/umst-concrete-cartridge/src/physics/printability.rs:85` | `empirical://datasets/dataset_d1.csv` | — | "Roussel (2018) Cem. Concr. Res. 112, 76" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel buildability pathway exercised under tests/printability.rs + adversarial harness" |
| `RheologyEngine` | `crates/umst-concrete-cartridge/src/physics/rheology.rs:6` | `empirical://datasets/dataset_d1.csv` | — | "Château, Ovarlez & Trung (2008) J. Rheol. 52, 489" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel/YODEL pathway exercised under tests/rheology.rs + adversarial harness" |
| `compute_chateau_ovarlez` | `crates/umst-concrete-cartridge/src/physics/rheology.rs:19` | `empirical://datasets/dataset_d1.csv` | — | "Château, Ovarlez & Trung (2008) J. Rheol. 52, 489" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel/YODEL pathway exercised under tests/rheology.rs + adversarial harness" |
| `compute_yield_stress_yodel` | `crates/umst-concrete-cartridge/src/physics/rheology.rs:69` | `empirical://datasets/dataset_d1.csv` | — | "Château, Ovarlez & Trung (2008) J. Rheol. 52, 489" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Roussel/YODEL pathway exercised under tests/rheology.rs + adversarial harness" |
| `SelfHealEngine` | `crates/umst-concrete-cartridge/src/physics/self_heal.rs:54` | `empirical://datasets/dataset_selfheal.csv` | — | "Edvardsen (1999) ACI Mater. J. 96, 448" \| "Boundary profile (no [acceptance] strength gate); paired CSV still listed in dataset_metrics skip — healing kinetics exercised under tests/realism/adversarial_physics.rs" |
| `compute_healing_potential` | `crates/umst-concrete-cartridge/src/physics/self_heal.rs:67` | `empirical://datasets/dataset_selfheal.csv` | — | "Edvardsen (1999) ACI Mater. J. 96, 448" \| "Boundary profile (no [acceptance] strength gate); paired CSV still listed in dataset_metrics skip — healing kinetics exercised under tests/realism/adversarial_physics.rs" |
| `ShrinkageEngine` | `crates/umst-concrete-cartridge/src/physics/shrinkage.rs:8` | `empirical://datasets/dataset_d1.csv` | — | "Bažant et al. (2015) Mater. Struct. 48, 753 (B4 shrinkage model)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Bažant–Baweja shrinkage pathway exercised under tests/shrinkage.rs + adversarial harness" |
| `compute_autogenous_shrinkage` | `crates/umst-concrete-cartridge/src/physics/shrinkage.rs:21` | `empirical://datasets/dataset_d1.csv` | — | "Bažant et al. (2015) Mater. Struct. 48, 753 (B4 shrinkage model)" \| "Headline compressive strength vs dataset_d1.csv: MAE ≤ 35 MPa, RMSE ≤ 45 MPa, R² ≥ −5 ([acceptance] uci_d1.v1.toml); Bažant–Baweja shrinkage pathway exercised under tests/shrinkage.rs + adversarial harness" |
| `calibrated_yield_stress_pa` | `crates/umst-concrete-cartridge/src/pipeline/track_a.rs:84` | `empirical://datasets/printability-rheology-yield-proxy.v1.csv` | — | "In-house Tyto mortar yield proxy calibration" \| "tests/calibration_tyto_mortar.rs" |
| `coordinate_descent_optimize` | `crates/umst-concrete-cartridge/src/pipeline/track_a.rs:203` | `empirical://datasets/cli-optimize-wc-bisection.v1.csv` | — | "Proxy-loop coordinate descent envelope (CLI tests)" \| "crates/umst-cli/tests/proxy_loop_optimize.rs" |

## Literature

| Symbol | File | formal_anchor | catalog_id | Citation / envelope / rationale |
|--------|------|---------------|------------|-----------------------------------|
| `RESULT_SCHEMA_VERSION_V1` | `crates/umst-cli/src/cli.rs:26` | `literature://wire-schema-result-v1` | — | "UMST concrete cartridge JSON wire schema tag (`result.v1`)" \| "`result.v1` — version tag for deprecated prediction JSON envelope" |
| `RESULT_SCHEMA_VERSION_V2` | `crates/umst-cli/src/cli.rs:32` | `literature://wire-schema-result-v2` | — | "UMST concrete cartridge JSON wire schema tag (`result.v2`)" \| "`result.v2` — version tag for current prediction JSON envelope" |
| `RESULT_SCHEMA_VERSION_V1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:63` | `literature://wire-schema-result-v1` | — | "UMST concrete cartridge JSON wire schema tag (`result.v1`)" \| "`result.v1` — version tag for deprecated prediction JSON envelope" |
| `RESULT_SCHEMA_VERSION_V2` | `crates/umst-concrete-cartridge/src/facade/mod.rs:69` | `literature://wire-schema-result-v2` | — | "UMST concrete cartridge JSON wire schema tag (`result.v2`)" \| "`result.v2` — version tag for current prediction JSON envelope" |
| `AUDIT_SCHEMA_VERSION` | `crates/umst-concrete-cartridge/src/facade/mod.rs:610` | `literature://wire-schema-audit-v1` | — | "UMST concrete cartridge JSON wire schema tag (`audit.v1`)" \| "`audit.v1` — batch CSV audit envelope with tensor predictions vs optional CSV strength" |
| `OpticalAuditWireV1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:912` | `literature://optics/fresnel-simpson-portland-paste` | — | "ASTM E903 spectrum; Sihvola 1999 dielectric database; v0.4 Track H3 acceptance gate" \| "(R_solar, A_uv, ε_lwir) integrated from plain-Portland (λ, ε_r) profile at 50 mm" |
| `ultimate_doh_wc` | `crates/umst-concrete-cartridge/src/formulas.rs:13` | `literature://Mills-1966-gel-stiffness-closure` | — | "Mills (1966); OPC gel stiffness / ultimate hydration cap closure used in routing" \| "α_inf(w/c) = 1.031·w/c / (0.194 + w/c)" |
| `ultimate_doh` | `crates/umst-concrete-cartridge/src/homogeneous.rs:67` | `literature://Mills-1966-gel-stiffness-closure` | — | "Mills (1966); α_inf = 1.031 w/c / (0.194 + w/c)" \| "α_inf(w/c) = 1.031·w/c / (0.194 + w/c)" |
| `embodied_co2_kg_per_m3` | `crates/umst-concrete-cartridge/src/homogeneous.rs:243` | `literature://EN-15804+A2-indicative-EPD-intensities` | — | "EN 15804+A2 (2019) environmental product declarations — indicative cradle-to-gate CO₂e intensities per constituent class" \| "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3); inline coefficients match bundled EPD intensity convention" |
| `constituent_masses_kg_m3` | `crates/umst-concrete-cartridge/src/homogeneous.rs:271` | `literature://ACI-211.1-binder-dosage-convention` | — | "ACI 211.1 — Standard Practice for Selecting Proportions for Normal, Heavyweight, and Mass Concrete" \| "350 kg/m³ binder dosage convention for constituent mass reconstruction from scalar mix spec" |
| `PHYSICS_PIPELINE_SCHEMA_VERSION` | `crates/umst-concrete-cartridge/src/lib.rs:96` | `literature://wire-schema-physics-pipeline-v1` | — | "physics_pipeline schema tag (`physics_pipeline.v1`)" \| "`schema_version` string on serde `PhysicsPipelineReport` — bump tag when breaking report shape." |
| `CEMENT_REACTION_ENTHALPY_J_PER_KG` | `crates/umst-concrete-cartridge/src/material_transition.rs:12` | `literature://cement-hydration-enthalpy-order-of-magnitude` | — | "Representative cementitious hydration enthalpy scale (order 450 J/g binder mass basis)." \| "`CEMENT_REACTION_ENTHALPY_J_PER_KG` feeds Clausius–Duhem transition gate via [`MaterialTransitionParams`]." |
| `CEMENT_DEFAULT_S_INTRINSIC_MPA` | `crates/umst-concrete-cartridge/src/material_transition.rs:19` | `literature://powers-intrinsic-strength-scale` | — | "Powers-style monotonic strength closure intrinsic scale (order 240 MPa)." \| "`CEMENT_DEFAULT_S_INTRINSIC_MPA` upper-bounds admissible strength jumps in transition gate." |
| `ClinkerPhase` | `crates/umst-concrete-cartridge/src/physics/clinker_eos.rs:22` | `literature://stat-mech/vinet-clinker-phase-enum` | — | "Manzano et al. 2009 J. Am. Chem. Soc. 131:7416; Speziale et al. 2008 Phys. Chem. Miner. 35:573; Clark et al. 2008 Cem. Concr. Res. 38:19; Pellenq et al. 2009 PNAS 106:16102" \| "Discrete phase tags carrying (V0, K0, K0') for Vinet P(V) calibration" |
| `VinetPhaseParams` | `crates/umst-concrete-cartridge/src/physics/clinker_eos.rs:36` | `literature://stat-mech/vinet-phase-params` | — | "Vinet et al. 1986 J. Phys. C 19:L467" \| "(V0 [Å³/f.u.], K0 [GPa], K0' [1]) parameter triple" |
| `params` | `crates/umst-concrete-cartridge/src/physics/clinker_eos.rs:49` | `literature://stat-mech/vinet-clinker-table` | — | "Manzano et al. 2009; Speziale et al. 2008; Clark et al. 2008; Pellenq et al. 2009" \| "VinetPhaseParams { v0_per_fu_ang3, bulk_modulus_gpa, k0_prime }" |
| `bulk_modulus_ambient_gpa` | `crates/umst-concrete-cartridge/src/physics/clinker_eos.rs:84` | `literature://stat-mech/vinet-k0-ambient` | — | "Vinet et al. 1986 J. Phys. C 19:L467" \| "K0 from tabulated EOS fit at P \\approx 0" |
| `vinet_pressure_gpa` | `crates/umst-concrete-cartridge/src/physics/clinker_eos.rs:94` | `literature://stat-mech/vinet-pressure-closed-form` | — | "Vinet et al. 1986 J. Phys. C 19:L467" \| "P(V) = 3 K0 ((1-x)/x²) exp(η(1-x)), x=(V/V0)^(1/3), η=(3/2)(K0'-1)" |
| `voigt_bulk_modulus_gpa` | `crates/umst-concrete-cartridge/src/physics/clinker_eos.rs:109` | `literature://micromechanics/voigt-upper-bound` | — | "Voigt W. 1887 Ann. Phys. 274:573 (rule of mixtures)" \| "K_Voigt = f K_a + (1-f) K_b" |
| `FiberEngine` | `crates/umst-concrete-cartridge/src/physics/fiber.rs:6` | `literature://Naaman-2006-ACI-SP235-pullout` | — | "Naaman (2006) ACI SP-235 — fiber pullout and crack-bridging micromechanics" \| "V_{f,crit} = σ_cu / (η_l η_o τ_b l_f / d_f)" |
| `compute_micromechanics` | `crates/umst-concrete-cartridge/src/physics/fiber.rs:17` | `literature://Naaman-2006-ACI-SP235-pullout` | — | "Naaman (2006) ACI SP-235 — fiber pullout and crack-bridging micromechanics" \| "V_{f,crit} = σ_cu / (η_l η_o τ_b l_f / d_f)" |
| `FractureEngine` | `crates/umst-concrete-cartridge/src/physics/fracture.rs:6` | `literature://Ulm-Coussy-2003-micromechanics` | — | "Ulm & Coussy (2003) Mechanics of Porous Continua (MIT Press); micromechanics derivation" \| "K_Ic = √(2 γ_s E_eff); E_eff = E_0 (1 − φ)^n" |
| `compute_effective_modulus_mt` | `crates/umst-concrete-cartridge/src/physics/fracture.rs:18` | `literature://Ulm-Coussy-2003-micromechanics` | — | "Ulm & Coussy (2003) Mechanics of Porous Continua (MIT Press); micromechanics derivation" \| "K_Ic = √(2 γ_s E_eff); E_eff = E_0 (1 − φ)^n" |
| `compute_fracture_toughness` | `crates/umst-concrete-cartridge/src/physics/fracture.rs:88` | `literature://Ulm-Coussy-2003-micromechanics` | — | "Ulm & Coussy (2003) Mechanics of Porous Continua (MIT Press); micromechanics derivation" \| "K_Ic = √(2 γ_s E_eff); E_eff = E_0 (1 − φ)^n" |
| `ManifoldPhotonicsSolver` | `crates/umst-concrete-cartridge/src/physics/optical.rs:25` | `literature://electromagnetics/photonics-phasor-driver` | — | "Rumpf M. 2022 Computational Electromagnetics in MATLAB; Taflove & Hagness 2005 FDTD handbook" \| "Alias to PhotonicsSolver { frequency_hz } — Maxwell phasor placeholder pending FDFD Helmholtz" |
| `refractive_index_real` | `crates/umst-concrete-cartridge/src/physics/optical.rs:36` | `literature://optics/refractive-index-dielectric` | — | "Born & Wolf 1999 Principles of Optics" \| "n = sqrt(ε_r) for non-magnetic dielectric" |
| `fresnel_power_reflectance_air_to_medium` | `crates/umst-concrete-cartridge/src/physics/optical.rs:46` | `literature://optics/fresnel-normal-incidence` | — | "Born & Wolf 1999 Principles of Optics" \| "R = ((n1-n2)/(n1+n2))² with n1=1" |
| `solar_reflectance` | `crates/umst-concrete-cartridge/src/physics/optical.rs:102` | `literature://concrete/solar-reflectance-cool-roof` | — | "ASTM E903 standard practice for solar absorptance; Track H3 UMST v0.4 brief" \| "Simpson average of Fresnel R(λ) + diffuse fraction (rough paste); Helmholtz interior open" |
| `photocatalytic_uv_absorption` | `crates/umst-concrete-cartridge/src/physics/optical.rs:121` | `literature://photocatalysis/uv-absorption-cement` | — | "Beer 1852; Lambert 1760 absorption law; ISO photocatalytic concrete test lines (~365 nm)" \| "A = 1 - exp(-4π k z / λ) - R(n(ε_r)), k lower-bounded per paste UV anchor" |
| `radiative_cooling_emissivity` | `crates/umst-concrete-cartridge/src/physics/optical.rs:143` | `literature://radiative-cooling/lwir-emissivity` | — | "Zhai et al. 2017 Joule 1:359 (radiative cooling); Kirchhoff 1860" \| "ε ≈ 1 - R at λ = 10.5 μm, ε_r(λ) from piecewise-linear profile" |
| `default_extinction_k_uv` | `crates/umst-concrete-cartridge/src/physics/optical.rs:159` | `literature://dielectrics/sihvola-cement-complex-permittivity` | — | "Sihvola A. 1999 Electromagnetic Mixing Formulas and Applications" \| "k ≈ (n tan δ)/2 with tan δ = 0.018 anchor" |
| `plain_portland_visible_profile` | `crates/umst-concrete-cartridge/src/physics/optical.rs:173` | `literature://dielectrics/portland-cement-permittivity-band` | — | "Track H3 UMST v0.4 brief (ε_r = 5.6 plain paste); LWIR ε_r order-of-magnitude" \| "Piecewise-linear (λ_nm, ε_r) knots for solar / UV / atmospheric-window interpolation" |
| `paste_bulk_modulus_voigt_from_wc_gpa` | `crates/umst-concrete-cartridge/src/physics/optical.rs:183` | `literature://micromechanics/voigt-csh-bulk-from-wc` | — | "Jennings H.M. 2000 Cem. Concr. Res.; Pellenq et al. 2009 PNAS (C-S-H modulus)" \| "K = f_ld K_ld + (1-f_ld) K_hd with f_ld(w/c) from Jennings linear fit" |
| `compute_packing_density` | `crates/umst-concrete-cartridge/src/physics/packing.rs:6` | `literature://Andreasen-Andersen-1930-Fuller-curve` | — | "Andreasen & Andersen (1930), Kolloid-Z. 50, 217" \| "P(D) = (D^q - D_min^q) / (D_max^q - D_min^q), q in [0.30, 0.45]" |
| `paste_csh_youngs_moduli_gpa` | `crates/umst-concrete-cartridge/src/physics/strength.rs:26` | `literature://micromechanics/csh-vinet-anchored-gel-moduli` | — | "Pellenq et al. 2009 PNAS 106:16102; Ulm & Constantinides 2004; Jennings 2000" \| "(E_LD, E_HD) = (CSH_LD_SCALE_OF_BULK, CSH_HD_SCALE_OF_BULK) * K_csh_vinet" |
| `SustainabilityEngine` | `crates/umst-concrete-cartridge/src/physics/sustainability.rs:6` | `literature://EN-15804+A2-GWP-and-unit-costs` | — | "EN 15804+A2 (2019) cradle-to-gate / modules A2 — indicative EPD-style CO₂e intensities; financial row uses linear $/kg mass factors" \| "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3)" |
| `compute_embodied_carbon` | `crates/umst-concrete-cartridge/src/physics/sustainability.rs:19` | `literature://EN-15804+A2-GWP-and-unit-costs` | — | "EN 15804+A2 (2019) cradle-to-gate / modules A2 — indicative EPD-style CO₂e intensities; financial row uses linear $/kg mass factors" \| "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3)" |
| `compute_financial_cost` | `crates/umst-concrete-cartridge/src/physics/sustainability.rs:51` | `literature://EN-15804+A2-GWP-and-unit-costs` | — | "EN 15804+A2 (2019) cradle-to-gate / modules A2 — indicative EPD-style CO₂e intensities; financial row uses linear $/kg mass factors" \| "GWP_mix = sum_i m_i * e_i  (kg CO2-eq / m^3)" |
| `PRINTABLE_TAU_LO` | `crates/umst-concrete-cartridge/src/pipeline/dual_gate.rs:28` | `literature://roussel-2018-buildability-window` | — | "Roussel (2018) Cem. Concr. Res. 112, 76 — printable τ₀ band" \| "τ₀ ∈ [180, 360] Pa extrusion window" |
| `PRINTABLE_TAU_HI` | `crates/umst-concrete-cartridge/src/pipeline/dual_gate.rs:33` | `literature://roussel-2018-buildability-window` | — | "Roussel (2018) Cem. Concr. Res. 112, 76 — printable τ₀ band" \| "τ₀ ∈ [180, 360] Pa extrusion window" |
| `printability_window_ok` | `crates/umst-concrete-cartridge/src/pipeline/dual_gate.rs:59` | `literature://roussel-2018-buildability-window` | — | "Roussel (2018) Cem. Concr. Res. 112, 76 — printable τ₀ band" \| "τ₀ band AND extrudability ≥ 0.35" |
| `reflection_xy_partner_indices` | `crates/umst-concrete-cartridge/src/print_ready/symmetry.rs:12` | `literature://symmetry-density-topology-sheet` | — | "Sigmund & Maute 2013, Struct. Multidisc. Optim. 48:1031-1055" \| "Index tensor `[1, N, 4]` listing the four xy-reflection partners of each primal vertex" |
| `apply_reflection_xy_average` | `crates/umst-concrete-cartridge/src/print_ready/symmetry.rs:45` | `literature://symmetry-density-topology-sheet` | — | "Sigmund & Maute 2013, Struct. Multidisc. Optim. 48:1031-1055" \| "Arithmetic mean of `rho` over the four xy-reflection partners (gather + mean)" |
| `EXTRUDABLE_TAU_LO_PA` | `crates/umst-concrete-cartridge/src/proxies/virtual_extrusion.rs:4` | `literature://roussel-2018-buildability-window` | — | "Roussel (2018) Cem. Concr. Res. 112, 76 — lower τ₀ bound" \| "τ₀ ≥ 180 Pa" |
| `EXTRUDABLE_TAU_HI_PA` | `crates/umst-concrete-cartridge/src/proxies/virtual_extrusion.rs:10` | `literature://roussel-2018-buildability-window` | — | "Roussel (2018) Cem. Concr. Res. 112, 76 — upper τ₀ bound" \| "τ₀ ≤ 360 Pa" |
| `extrusion_band_score` | `crates/umst-concrete-cartridge/src/proxies/virtual_extrusion.rs:16` | `literature://roussel-2018-buildability-window` | — | "Roussel (2018) Cem. Concr. Res. 112, 76 — printable τ₀ band" \| "0.5 when τ₀ ∈ band else 0" |
| `VIRTUAL_STACK_HEIGHT_M` | `crates/umst-concrete-cartridge/src/proxies/virtual_stack.rs:4` | `literature://roussel-2016-buildability` | — | "Roussel et al. (2016) Cem. Concr. Res. 85 — buildability height" \| "H = 0.012 m fresh layer (mortar printable proxy; full 0.30 m column uses tests/printability.rs)" |
| `VIRTUAL_STACK_RHO_KG_M3` | `crates/umst-concrete-cartridge/src/proxies/virtual_stack.rs:11` | `literature://aci-211-density-nominal` | — | "ACI 211.1 nominal fresh density for mortar/concrete" \| "ρ = 2300 kg/m³ surrogate" |
| `roussel_min_yield_pa` | `crates/umst-concrete-cartridge/src/proxies/virtual_stack.rs:20` | `literature://roussel-2016-buildability` | — | "Roussel et al. (2016) Cem. Concr. Res. 85 — τ_min = ρ g H / √3" \| "τ_min(H, ρ)" |
| `virtual_stack_score_in_band` | `crates/umst-concrete-cartridge/src/proxies/virtual_stack.rs:43` | `literature://roussel-2018-buildability-window` | — | "Roussel (2018) Cem. Concr. Res. 112, 76 — printable τ₀ band" \| "τ₀ ∈ [180, 360] Pa → score 1.0 else Roussel stack proxy" |

## NONE

| Symbol | File | formal_anchor | catalog_id | Citation / envelope / rationale |
|--------|------|---------------|------------|-----------------------------------|
| `canon_header` | `crates/umst-cli/src/audit.rs:24` | `NONE` | — | Mechanical header synonym routing for CLI CSV audit ergonomics only. |
| `audit_csv_buf` | `crates/umst-cli/src/audit.rs:93` | `NONE` | — | Glue from CSV text to facade [`audit_build_report_v1`] without physical claims. |
| `stdin_to_string` | `crates/umst-cli/src/audit.rs:147` | `NONE` | — | IO helper for MCP/CLI corpus workflows. |
| `audit_csv_file` | `crates/umst-cli/src/audit.rs:158` | `NONE` | — | File IO adapter for corpus audit CLI. |
| `canonical_json_value` | `crates/umst-cli/src/canonical.rs:11` | `NONE` | — | Transport-only deterministic JSON contract; physical claims live upstream of this layer. |
| `CanonicalJsonError` | `crates/umst-cli/src/canonical.rs:46` | `NONE` | — | Structural transport error for malformed or non-finite JSON numbers. |
| `canonical_json_bytes` | `crates/umst-cli/src/canonical.rs:90` | `NONE` | — | Byte-stable wire encoding for MCP / CLI / acceptance scripts. |
| `mix_spec_from_json_value` | `crates/umst-cli/src/cli.rs:100` | `NONE` | — | JSON boundary helper; validation uses [`MixSpecWire`] + [`MixSpec::try_from`]. |
| `serialize_prediction` | `crates/umst-cli/src/cli.rs:115` | `NONE` | — | JSON-serialise glue; no physical claim. |
| `serialize_mix_spec` | `crates/umst-cli/src/cli.rs:151` | `NONE` | — | JSON-serialise glue. |
| `parse_optimize_target` | `crates/umst-cli/src/cli.rs:183` | `NONE` | — | String-parse glue for `FIELD=VALUE` optimise CLI syntax. |
| `proposed_next_mix_value` | `crates/umst-cli/src/cli.rs:247` | `NONE` | — | Track A sidecar for experiment loop. |
| `optimize_mix_with_gate` | `crates/umst-cli/src/cli.rs:284` | `NONE` | — | CLI driver; gate semantics from `pipeline::dual_gate`. |
| `run_memory_export` | `crates/umst-cli/src/memory_export.rs:12` | `NONE` | — | Operator filesystem IO; rows already gate-validated at ingest. |
| `default_memory_record_path` | `crates/umst-cli/src/promote.rs:13` | `NONE` | — | CLI filesystem path helper; human-gated promotion only. |
| `load_memory_record` | `crates/umst-cli/src/promote.rs:21` | `NONE` | — | Filesystem read for promote-contribution; row already gate-validated. |
| `run_promote_contribution` | `crates/umst-cli/src/promote.rs:30` | `NONE` | — | CLI orchestration; never MCP-exposed; approval file required. |
| `run_propose_promotion` | `crates/umst-cli/src/propose_promotion.rs:14` | `NONE` | — | Human-only CLI proposal; calibration metrics not MCP gate. |
| `bool_and` | `crates/umst-concrete-cartridge/src/burn_compat.rs:8` | `NONE` | — | Burn-version compatibility shim for boolean tensor AND across crate semver skew. |
| `ProvenanceFormal` | `crates/umst-concrete-cartridge/src/calibration.rs:70` | `NONE` | — | Serde lift of TOML `[provenance.formal]`; `status` string is file metadata (may include Boundary scope), not a Rust `formal_status` bucket. |
| `CalibrationProvenance` | `crates/umst-concrete-cartridge/src/calibration.rs:103` | `NONE` | — | Dataset and Zenodo citation bundle parsed from TOML only; no Lean witness on this serde container — see `docs/FormalAnchors.md` “Future formal links” for manifold adjoint context. |
| `CalibrationModelSection` | `crates/umst-concrete-cartridge/src/calibration.rs:156` | `NONE` | — | Dispatch metadata only; Jennings gel-space monotone strength witness applies once `powers_compressive_strength_mpa` ships a Jennings branch (TODO_FORMAL note on that function). |
| `ContractBlock` | `crates/umst-concrete-cartridge/src/calibration.rs:181` | `NONE` | — | Contract metadata (`verification_status`); hyperbox regime warnings are soundness-witnessed on `regime_check_scalars` — see RegimeSoundness anchor there. |
| `CalibrationError` | `crates/umst-concrete-cartridge/src/calibration.rs:217` | `NONE` | — | Bundled profile IO and TOML parse failures only; DEC mass-conservation witness belongs on the manifold Laplacian — see `docs/FormalAnchors.md` “Future formal links”. |
| `load_from_path` | `crates/umst-concrete-cartridge/src/calibration.rs:269` | `NONE` | — | Filesystem path IO for non-bundled TOML; parse errors surface as CalibrationError. |
| `RegressionMetrics` | `crates/umst-concrete-cartridge/src/calibration_metrics.rs:7` | `NONE` | — | Ordinary least-squares aggregates over paired CSV predictions; QA helper without Lean witness on this surface. |
| `regression_metrics` | `crates/umst-concrete-cartridge/src/calibration_metrics.rs:19` | `NONE` | — | Same as `RegressionMetrics`; computes MAE/RMSE/R² slices for calibration reports. |
| `ConcreteCartridge` | `crates/umst-concrete-cartridge/src/core/implementation.rs:188` | `NONE` | — | Cartridge functor F: mix layout → constitutive summaries; topology pass remains separate DEC hook. |
| `new` | `crates/umst-concrete-cartridge/src/core/implementation.rs:201` | `NONE` | — | Deterministic bundled baseline when callers omit explicit calibration. |
| `with_profile` | `crates/umst-concrete-cartridge/src/core/implementation.rs:209` | `NONE` | — | Avoids silently mixing heterogeneous tensor kinetics with unrelated gel-space coefficients. |
| `with_topology_nominal` | `crates/umst-concrete-cartridge/src/core/implementation.rs:222` | `NONE` | — | Avoids silent regime-midpoint surrogate when a design is known. |
| `apply_topology_result_to_umst` | `crates/umst-concrete-cartridge/src/core/implementation.rs:232` | `NONE` | — | Mutable UMST merge path for topology tensors; proof witnesses remain caller-owned. |
| `ConcreteCartridge` | `crates/umst-concrete-cartridge/src/core/mod.rs:6` | `NONE` | — | Re-export; classification follows the underlying symbol. |
| `apply_physics_to_umst` | `crates/umst-concrete-cartridge/src/core/mod.rs:10` | `NONE` | — | Forwards manifold UMST write-back helper used after topology physics. |
| `MaterialCompositionTensor` | `crates/umst-concrete-cartridge/src/core/mod.rs:14` | `NONE` | — | W9 agnostic composition tensor re-export; classification follows manifold `MaterialCompositionTensor`. |
| `IScienceCartridge, PhysicalResult` | `crates/umst-concrete-cartridge/src/core/mod.rs:18` | `NONE` | — | Forwards manifold cartridge façade trait and tensor bundles. |
| `MixTensor` | `crates/umst-concrete-cartridge/src/core/mod.rs:22` | `NONE` | — | Deprecated W9 alias for `MaterialCompositionTensor`; retained one release for cartridge callers. |
| `StatePoint` | `crates/umst-concrete-cartridge/src/core/mod.rs:28` | `NONE` | — | Deprecated W9 alias for `MaterialCompositionTensor`; retained one release for cartridge callers. |
| `umst_manifold::manifest::*` | `crates/umst-concrete-cartridge/src/facade/mod.rs:46` | `NONE` | — | Forwards manifold manifest types when `manifest-bridge` is enabled. |
| `UmstManifest` | `crates/umst-concrete-cartridge/src/facade/mod.rs:53` | `NONE` | — | Local serde mirror until `manifest-bridge` pins manifold wire types. |
| `value` | `crates/umst-concrete-cartridge/src/facade/mod.rs:110` | `NONE` | — | Trivial accessor; getter for the wrapped `f32`. |
| `value` | `crates/umst-concrete-cartridge/src/facade/mod.rs:139` | `NONE` | — | Trivial accessor. |
| `MixSpecError` | `crates/umst-concrete-cartridge/src/facade/mod.rs:232` | `NONE` | — | Mix-spec rejection causes without JSON parse errors (handled at transport boundary). |
| `PredictOptions` | `crates/umst-concrete-cartridge/src/facade/mod.rs:327` | `NONE` | — | Behavioral flags only — no Lean witness. |
| `predict_with_options` | `crates/umst-concrete-cartridge/src/facade/mod.rs:432` | `NONE` | — | Feature flag glue for MCP/CLI; no standalone formal claim. |
| `AuditSummaryV1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:628` | `NONE` | — | JSON summary stats for auditors; aggregates row-level residuals only. |
| `AuditReportV1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:639` | `NONE` | — | Top-level serde envelope for CLI/MCP corpus audit tooling. |
| `AuditRowInputV1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:653` | `NONE` | — | Normalised mix scalars carried from the CSV row for audit consumers (`audit.v1` row `input`). |
| `AuditRowWireV1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:668` | `NONE` | — | One audited CSV row projection with tensor headline strength. |
| `tensor_element_at` | `crates/umst-concrete-cartridge/src/facade/mod.rs:854` | `NONE` | — | Internal tensor scalar read for wire projection; index contract from pipeline layout. |
| `PredictionWireV1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:870` | `NONE` | — | Serde wire projection for `result.v1` scalars; versioning tagged in `schema_version`. |
| `PredictionWireV2` | `crates/umst-concrete-cartridge/src/facade/mod.rs:885` | `NONE` | — | Serde wire projection for `result.v2` scalars; `physics_pipeline` merged at JSON boundary. |
| `prediction_wire_v1` | `crates/umst-concrete-cartridge/src/facade/mod.rs:951` | `NONE` | — | Pure wire projection; transport encoding is caller-owned. |
| `prediction_wire_v2` | `crates/umst-concrete-cartridge/src/facade/mod.rs:973` | `NONE` | — | Pure wire projection; nested objects merged by CLI/MCP `serde_json`. |
| `MixSpecWireOut` | `crates/umst-concrete-cartridge/src/facade/mod.rs:1094` | `NONE` | — | Round-trip mix spec view for CLI `mix print` / MCP. |
| `mix_spec_wire_out` | `crates/umst-concrete-cartridge/src/facade/mod.rs:1107` | `NONE` | — | Serialize-friendly mix view without JSON crate in core. |
| `ConcretePolicyEvaluator` | `crates/umst-concrete-cartridge/src/gate_policy.rs:9` | `NONE` | — | Zero-sized policy evaluator; HTTP gate catalog row without spatial physics. |
| `gate_manifest_from` | `crates/umst-concrete-cartridge/src/gate_policy.rs:16` | `NONE` | — | Injection-only HTTP gate manifest from explicit [`UmstManifest`] (see `docs/RUNTIME_TOPOLOGY.md` § HTTP gate defaults). |
| `HomogeneousError` | `crates/umst-concrete-cartridge/src/homogeneous.rs:28` | `NONE` | — | Dispatch error: Jennings-not-yet, invalid mix; no formal claim. |
| `mix_hydration_state` | `crates/umst-concrete-cartridge/src/homogeneous.rs:76` | `NONE` | — | Internal homogeneous helper composing calibrated α(t,T,scm) and effective w/c from profile parameters. |
| `ConcreteTransitionCartridge, ConcreteTransitionWitness` | `crates/umst-concrete-cartridge/src/lib.rs:43` | `NONE` | — | Re-exports concrete transition gate cartridge when `manifold-gate` is enabled. |
| `cement_reaction_extent_kinetics_spec, CementMaterialParams, CEMENT_DEFAULT_S_INTRINSIC_MPA, CEMENT_REACTION_ENTHALPY_J_PER_KG` | `crates/umst-concrete-cartridge/src/lib.rs:48` | `NONE` | — | Re-exports W9 Tier-2c cement closure SSOT when `tier2c-handshake` is enabled. |
| `gate_cartridge_witness, with_gate_cartridge` | `crates/umst-concrete-cartridge/src/lib.rs:56` | `NONE` | — | Re-exports THMC injectable gate helpers when `solver-experimental` enabled. |
| `apply_physics_to_umst, ConcreteCartridge, IScienceCartridge, MaterialCompositionTensor, PhysicalResult` | `crates/umst-concrete-cartridge/src/lib.rs:64` | `NONE` | — | Re-exports manifold façade symbols for ergonomics only. |
| `ConcretePolicyEvaluator` | `crates/umst-concrete-cartridge/src/lib.rs:71` | `NONE` | — | Re-exports cartridge HTTP gate policy evaluator. |
| `run_full_physics_pipeline` | `crates/umst-concrete-cartridge/src/lib.rs:75` | `NONE` | — | Stable import path for MCP/CLI integration tests. |
| `PhysicsPipelineReport` | `crates/umst-concrete-cartridge/src/lib.rs:79` | `NONE` | — | JSON envelope for staged tensor outputs. |
| `PhysicsPipelineSummary` | `crates/umst-concrete-cartridge/src/lib.rs:84` | `NONE` | — | Scalar digest accompanying report JSON. |
| `PipelineStageRecord` | `crates/umst-concrete-cartridge/src/lib.rs:88` | `NONE` | — | Stage record type embedded in [`PhysicsPipelineReport`]. |
| `PipelineStageStatus` | `crates/umst-concrete-cartridge/src/lib.rs:92` | `NONE` | — | Serialized stage disposition enum for MCP/CLI audit trails. |
| `GateEvaluator, ThermodynamicTransitionEvaluator, TransitionGateEvaluator` | `crates/umst-concrete-cartridge/src/lib.rs:102` | `NONE` | — | Re-exports host transition gate traits when `manifold-gate` is enabled. |
| `UmstManifest` | `crates/umst-concrete-cartridge/src/lib.rs:109` | `NONE` | — | Re-exports manifold deployment manifest when `manifold-manifest` is enabled. |
| `ros` | `crates/umst-concrete-cartridge/src/lib.rs:114` | `NONE` | — | Re-exports manifold ROS serde DTOs when `ros2-contract` is enabled. |
| `cement_reaction_extent_kinetics_spec` | `crates/umst-concrete-cartridge/src/material_transition.rs:26` | `NONE` | — | Bundled THMC kinetics witness for parity harnesses; values mirror cartridge calibration lane. |
| `CementMaterialParams` | `crates/umst-concrete-cartridge/src/material_transition.rs:58` | `NONE` | — | Zero-sized cement closure witness for harnesses without a loaded [`Profile`]. |
| `MIX_FEATURE_COUNT` | `crates/umst-concrete-cartridge/src/mix_layout.rs:13` | `NONE` | — | Structural convention for CLI ↔ tensor engines; documented here as SSOT for column indices. |
| `IDX_WATER_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:19` | `NONE` | — | Structural column tag for wire ↔ tensor bridging. |
| `IDX_CEMENT_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:24` | `NONE` | — | Must stay aligned with `physics::hydration` slicers. |
| `IDX_AGG_COARSE_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:29` | `NONE` | — | Reserved packing-engine input when recipe supplies split gradation. |
| `IDX_AGG_FINE_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:34` | `NONE` | — | Reserved packing-engine input when recipe supplies split gradation. |
| `IDX_RESERVED_4` | `crates/umst-concrete-cartridge/src/mix_layout.rs:39` | `NONE` | — | Keeps hydration column indices historically stable (`hydration.rs` assumptions). |
| `IDX_SLAG_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:44` | `NONE` | — | Structural column aligned with SCM tensor hydrates. |
| `IDX_FLY_ASH_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:49` | `NONE` | — | Structural column for pozzolan mass routing. |
| `IDX_SUPERPLASTICIZER_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:54` | `NONE` | — | Admixture mass channel for homogeneous + rheology stubs. |
| `IDX_AGE_DAYS` | `crates/umst-concrete-cartridge/src/mix_layout.rs:59` | `NONE` | — | Scalar age carried on-layout for `compute_all` without extra manifold state. |
| `IDX_TEMPERATURE_C` | `crates/umst-concrete-cartridge/src/mix_layout.rs:64` | `NONE` | — | Arrhenius-style tensor engines consume this lane. |
| `IDX_SILICA_FUME_KG_M3` | `crates/umst-concrete-cartridge/src/mix_layout.rs:69` | `NONE` | — | SCM extension field for supplemental micro-silica mass. |
| `IDX_AGGREGATE_VOLUME_FRACTION` | `crates/umst-concrete-cartridge/src/mix_layout.rs:74` | `NONE` | — | Drives packing + paste fraction surrogates on collapsed paths. |
| `fractions_from_mix_row` | `crates/umst-concrete-cartridge/src/mix_layout.rs:85` | `NONE` | — | Deterministic CSV-style encoding; aligns with homogeneous `MixRow` used by CLI. |
| `mix_tensor_from_layout` | `crates/umst-concrete-cartridge/src/mix_layout.rs:106` | `NONE` | — | Constructor replacing non-existent `MixTensor::from_proportions`; see README Quick start. |
| `collapsed_rank4_from_rank2_scalar` | `crates/umst-concrete-cartridge/src/mix_layout.rs:123` | `NONE` | — | Engine APIs require rank-4 tensors; singleton spatial dims document batch-collapsed mode. |
| `compute_cost` | `crates/umst-concrete-cartridge/src/physics/cost.rs:7` | `NONE` | — | Auxiliary objective; linear cost vector, no physical claim. |
| `TransportEngine` | `crates/umst-concrete-cartridge/src/physics/transport.rs:6` | `NONE` | — | Tensor facade grouping porosity and chloride diffusivity kernels documented on methods. |
| `DualGateVerdict` | `crates/umst-concrete-cartridge/src/pipeline/dual_gate.rs:39` | `NONE` | — | Composite verdict; legs documented on helper fns. |
| `passes` | `crates/umst-concrete-cartridge/src/pipeline/dual_gate.rs:50` | `NONE` | — | Equal-weight AND of printability and thermodynamic legs. |
| `printability_from_summary` | `crates/umst-concrete-cartridge/src/pipeline/dual_gate.rs:71` | `NONE` | — | Summary-scalar wrapper over [`printability_window_ok`]. |
| `printability_with_virtual_proxies` | `crates/umst-concrete-cartridge/src/pipeline/dual_gate.rs:82` | `NONE` | — | Lazy AND of summary band + Roussel stack/extrusion surrogates. |
| `evaluate_dual_gate` | `crates/umst-concrete-cartridge/src/pipeline/dual_gate.rs:110` | `NONE` | — | Track A composite gate; legs carry individual anchors. |
| `evaluate_dual_gate, DualGateVerdict, PRINTABLE_TAU_HI, PRINTABLE_TAU_LO` | `crates/umst-concrete-cartridge/src/pipeline/mod.rs:15` | `NONE` | — | Re-export dual-gate verdict for MCP/CLI Track A. |
| `run_full_physics_pipeline` | `crates/umst-concrete-cartridge/src/pipeline/mod.rs:19` | `NONE` | — | Stable import path for staged tensor physics. |
| `nominal_mix_tensor_for_mix_spec, nominal_mix_tensor_for_topology, physical_result_from_report, topology_pipeline_headlines, topology_pipeline_report, TopologyNominalMix` | `crates/umst-concrete-cartridge/src/pipeline/mod.rs:23` | `NONE` | — | Topology / predict policy maps from pipeline report. |
| `PhysicsPipelineReport, PhysicsPipelineSummary, PipelineStageRecord, PipelineStageStatus, PHYSICS_PIPELINE_SCHEMA_VERSION` | `crates/umst-concrete-cartridge/src/pipeline/mod.rs:30` | `NONE` | — | JSON envelope types for MCP/CLI audit trails. |
| `coordinate_descent_optimize, evaluate_mix_dual_gate, proposed_next_mix_json, ProposedNextMix, TrackAObjective` | `crates/umst-concrete-cartridge/src/pipeline/mod.rs:38` | `NONE` | — | Track A coordinate-descent + proposed mix JSON assembly. |
| `run_full_physics_pipeline` | `crates/umst-concrete-cartridge/src/pipeline/orchestrator.rs:127` | `NONE` | — | Cartridge functor composition root exercised by tooling + tests. |
| `TopologyNominalMix` | `crates/umst-concrete-cartridge/src/pipeline/physical_summary.rs:25` | `NONE` | — | Decouples `core` from `facade::MixSpec` while sharing mix_layout semantics. |
| `topology_pipeline_report` | `crates/umst-concrete-cartridge/src/pipeline/physical_summary.rs:54` | `NONE` | — | When `nominal` is set, uses caller recipe instead of regime midpoint. |
| `physical_result_from_report` | `crates/umst-concrete-cartridge/src/pipeline/physical_summary.rs:105` | `NONE` | — | Encodes CLI tensor summary contract; see module-level policy mapping. |
| `nominal_mix_tensor_for_mix_spec` | `crates/umst-concrete-cartridge/src/pipeline/physical_summary.rs:145` | `NONE` | — | Shared layout path for predict and topology when recipe is known. |
| `nominal_mix_tensor_for_topology` | `crates/umst-concrete-cartridge/src/pipeline/physical_summary.rs:169` | `NONE` | — | Deterministic surrogate mix so staged tensor engines match `compute_all` semantics. |
| `topology_pipeline_headlines` | `crates/umst-concrete-cartridge/src/pipeline/physical_summary.rs:204` | `NONE` | — | Single SSOT with Jennings/YODEL/GWP tensor pipeline used by bulk predict. |
| `PHYSICS_PIPELINE_SCHEMA_VERSION` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:11` | `NONE` | — | Version discriminator for additive JSON fields. |
| `PipelineStageStatus` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:17` | `NONE` | — | Enumerates audited stage outcomes (`Executed` vs honest skips/failures). |
| `PipelineStageRecord` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:29` | `NONE` | — | Serialized evidence that a stage ran, skipped, or failed. |
| `ok` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:43` | `NONE` | — | Successful stage marker helper. |
| `skip_missing` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:55` | `NONE` | — | Honest skip when inputs/constants are absent by design. |
| `fail` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:67` | `NONE` | — | Propagates panics-avoiding error strings for observability. |
| `PhysicsPipelineSummary` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:79` | `NONE` | — | Human/tooling digest; not a substitute for full tensor fields. |
| `PhysicsPipelineReport` | `crates/umst-concrete-cartridge/src/pipeline/report.rs:105` | `NONE` | — | Cartridge-local rich JSON envelope parallel to manifold tensors. |
| `ProposedNextMix` | `crates/umst-concrete-cartridge/src/pipeline/track_a.rs:27` | `NONE` | — | Wire envelope for Track A CLI; physics claims live on nested gate fields. |
| `MixSpecWireOut` | `crates/umst-concrete-cartridge/src/pipeline/track_a.rs:42` | `NONE` | — | Mix JSON mirror without newtype wrappers for serde output. |
| `DualGateWire` | `crates/umst-concrete-cartridge/src/pipeline/track_a.rs:57` | `NONE` | — | Dual-gate audit block for proposed mix JSON sidecar. |
| `summary_with_calibrated_tau` | `crates/umst-concrete-cartridge/src/pipeline/track_a.rs:98` | `NONE` | — | Applies θ bias before dual-gate printability leg. |
| `evaluate_mix_dual_gate` | `crates/umst-concrete-cartridge/src/pipeline/track_a.rs:156` | `NONE` | — | Track A scoring helper; gate semantics from `dual_gate`. |
| `TrackAObjective` | `crates/umst-concrete-cartridge/src/pipeline/track_a.rs:173` | `NONE` | — | Track A optimise targets mirrored from CLI `OptimizeField`. |
| `proposed_next_mix_json` | `crates/umst-concrete-cartridge/src/pipeline/track_a.rs:386` | `NONE` | — | JSON sidecar assembly for Track A CLI. |
| `thermodynamic_gate_ok` | `crates/umst-concrete-cartridge/src/pipeline/track_a.rs:417` | `NONE` | — | Thin wrapper for comparison example Track A path. |
| `extrusion_tensor_score` | `crates/umst-concrete-cartridge/src/proxies/virtual_extrusion.rs:30` | `NONE` | — | Headline scalar from pipeline printability stage. |
| `virtual_extrusion_score` | `crates/umst-concrete-cartridge/src/proxies/virtual_extrusion.rs:43` | `NONE` | — | Combines τ₀ band with tensor extrudability headline. |
| `virtual_stack_score` | `crates/umst-concrete-cartridge/src/proxies/virtual_stack.rs:30` | `NONE` | — | Normalized score from [`roussel_min_yield_pa`]; not a standalone Lean witness. |
| `CHECKPOINTS_JSONL_DEFAULT` | `crates/umst-concrete-cartridge/src/research/checkpoint.rs:13` | `NONE` | — | Filesystem path constant; Merkle batch is audit metadata only. |
| `CheckpointError` | `crates/umst-concrete-cartridge/src/research/checkpoint.rs:19` | `NONE` | — | JSONL sidecar error sum type; not admissibility gate. |
| `CheckpointRecord` | `crates/umst-concrete-cartridge/src/research/checkpoint.rs:31` | `NONE` | — | Audit batch metadata; memory admissibility on `accept` path. |
| `merkle_root_from_leaves` | `crates/umst-concrete-cartridge/src/research/checkpoint.rs:44` | `NONE` | — | Hash combiner for batch audit; no thermodynamic claim. |
| `build_checkpoint` | `crates/umst-concrete-cartridge/src/research/checkpoint.rs:80` | `NONE` | — | Pure record morphism over content_id leaves; persistence is IO. |
| `append_checkpoint_jsonl` | `crates/umst-concrete-cartridge/src/research/checkpoint.rs:99` | `NONE` | — | Append-only checkpoint JSONL; not queryable admissible memory. |
| `AcceptError` | `crates/umst-concrete-cartridge/src/research/contribution.rs:34` | `NONE` | — | Error sum type; thermodynamic verdict on `GateReject` leg only. |
| `ContributeError` | `crates/umst-concrete-cartridge/src/research/contribution.rs:52` | `NONE` | — | Type alias for wire ergonomics; gate reject leg documented on `AcceptError`. |
| `DEFAULT_CATALOG_HASH` | `crates/umst-concrete-cartridge/src/research/contribution.rs:58` | `NONE` | — | Build-time pin override in manifest_bridge tests; not a physics claim. |
| `GateContext` | `crates/umst-concrete-cartridge/src/research/contribution.rs:92` | `NONE` | — | Profile carrier for `gate_recheck`; CD math on manifold when manifest-bridge on. |
| `GateFieldIssue` | `crates/umst-concrete-cartridge/src/research/contribution.rs:130` | `NONE` | — | Operator diagnostics; not admissibility proof. |
| `GateCheckExplain` | `crates/umst-concrete-cartridge/src/research/contribution.rs:140` | `NONE` | — | Operator diagnostics; not admissibility proof. |
| `GateCheckResult` | `crates/umst-concrete-cartridge/src/research/contribution.rs:154` | `NONE` | — | Structured tool result; reject row matches `gate_reject.v1`. |
| `gate_reject_row_for_mix` | `crates/umst-concrete-cartridge/src/research/contribution.rs:330` | `NONE` | — | Audit stream morphism; reject rows excluded from `admissible_only` query. |
| `encode` | `crates/umst-concrete-cartridge/src/research/contribution.rs:569` | `NONE` | — | Formatting helper; content addressing on `content_id`. |
| `ExportError` | `crates/umst-concrete-cartridge/src/research/export.rs:14` | `NONE` | — | Bundle handoff error sum type; rows already gate-validated. |
| `hash_chain_for_rows` | `crates/umst-concrete-cartridge/src/research/export.rs:26` | `NONE` | — | Deterministic audit chain over exported rows; not admissibility. |
| `MemoryExportBundle` | `crates/umst-concrete-cartridge/src/research/export.rs:56` | `NONE` | — | Signed export envelope; physics witnesses on row `gate_summary`. |
| `build_memory_export_bundle` | `crates/umst-concrete-cartridge/src/research/export.rs:71` | `NONE` | — | Pure bundle morphism; operator handoff artifact not MCP gate. |
| `write_memory_export_bundle` | `crates/umst-concrete-cartridge/src/research/export.rs:91` | `NONE` | — | Filesystem export write; never exposed via MCP tools. |
| `fields_for_code, remediation_for_code, MANIFEST_BRIDGE_DISABLED, MIX_SPEC_RATIONAL_PARSE_FAIL, MIX_SPEC_WIRE_INVALID, THERMODYNAMIC_CD_FAIL, THERMODYNAMIC_FAIL, TOP_GATE_EXPLAIN_CODES` | `crates/umst-concrete-cartridge/src/research/gate_explain_ssot.rs:6` | `NONE` | — | Cartridge mirror of manifold gate reject codes and field/remediation SSOT for MCP explain parity. |
| `MixGeometryKey` | `crates/umst-concrete-cartridge/src/research/geometry.rs:13` | `NONE` | — | Design-coordinate locality index; no thermodynamic claim. |
| `quantize_w_c` | `crates/umst-concrete-cartridge/src/research/geometry.rs:28` | `NONE` | — | Morton grid quantization on w_c; retrieval heuristic only. |
| `quantize_temperature_k` | `crates/umst-concrete-cartridge/src/research/geometry.rs:38` | `NONE` | — | Morton grid quantization on temperature; no constitutive law. |
| `morton_index` | `crates/umst-concrete-cartridge/src/research/geometry.rs:48` | `NONE` | — | Z-order curve index for memory locality queries. |
| `mix_geometry_key` | `crates/umst-concrete-cartridge/src/research/geometry.rs:64` | `NONE` | — | Mix-space Morton key for `memory_query`; not admissibility gate. |
| `mix_l1_distance` | `crates/umst-concrete-cartridge/src/research/geometry.rs:84` | `NONE` | — | Nearest-neighbor mix distance for query sorting; advisory geometry. |
| `morton_index_distance` | `crates/umst-concrete-cartridge/src/research/geometry.rs:101` | `NONE` | — | Hilbert-index distance for locality filter; no physics claim. |
| `ScopeError` | `crates/umst-concrete-cartridge/src/research/governance.rs:8` | `NONE` | — | Operator allowlist gate; not thermodynamic admissibility. |
| `validate_scope_token` | `crates/umst-concrete-cartridge/src/research/governance.rs:20` | `NONE` | — | YAML/env governance allowlist; physics on `gate_recheck` only. |
| `StoreError` | `crates/umst-concrete-cartridge/src/research/memory.rs:11` | `NONE` | — | Store boundary error sum type; admissibility on accept path. |
| `MemoryError` | `crates/umst-concrete-cartridge/src/research/memory.rs:27` | `NONE` | — | Type alias for ergonomic imports; same variants as `StoreError`. |
| `ResearchStore` | `crates/umst-concrete-cartridge/src/research/memory.rs:258` | `NONE` | — | Store enum dispatch; SQLite arm is IO boundary. |
| `open_sqlite` | `crates/umst-concrete-cartridge/src/research/memory.rs:284` | `NONE` | — | Filesystem + SQLite connection open; not pure morphism. |
| `from_env` | `crates/umst-concrete-cartridge/src/research/memory.rs:292` | `NONE` | — | Environment-driven store selection at session start. |
| `query` | `crates/umst-concrete-cartridge/src/research/memory.rs:303` | `NONE` | — | Dispatches to in-memory or SQLite IO arm. |
| `append` | `crates/umst-concrete-cartridge/src/research/memory.rs:315` | `NONE` | — | Store dispatch + idempotency check at IO boundary. |
| `rows` | `crates/umst-concrete-cartridge/src/research/memory.rs:344` | `NONE` | — | May load from SQLite; pure filter on result in `filter_records`. |
| `load_contribute_jobs_sqlite` | `crates/umst-concrete-cartridge/src/research/memory.rs:363` | `NONE` | — | IO boundary for MCP job poll; JSON sidecar dual-write. |
| `persist_contribute_jobs_sqlite` | `crates/umst-concrete-cartridge/src/research/memory.rs:374` | `NONE` | — | IO boundary; jobs are ephemeral operator state. |
| `SqliteStore` | `crates/umst-concrete-cartridge/src/research/memory.rs:423` | `NONE` | — | Durable store IO; query uses pure `filter_records` on load. |
| `open` | `crates/umst-concrete-cartridge/src/research/memory.rs:434` | `NONE` | — | Connection + schema migration IO boundary. |
| `path` | `crates/umst-concrete-cartridge/src/research/memory.rs:451` | `NONE` | — | Path accessor for operator diagnostics only. |
| `has_idempotency_key` | `crates/umst-concrete-cartridge/src/research/memory.rs:477` | `NONE` | — | SQLite read for duplicate suppression at append. |
| `append` | `crates/umst-concrete-cartridge/src/research/memory.rs:492` | `NONE` | — | SQLite INSERT; rows must pass accept gate before call. |
| `query` | `crates/umst-concrete-cartridge/src/research/memory.rs:516` | `NONE` | — | SQLite SELECT + pure filter morphism. |
| `rows` | `crates/umst-concrete-cartridge/src/research/memory.rs:525` | `NONE` | — | Full table scan IO; filter in caller if needed. |
| `load_contribute_jobs` | `crates/umst-concrete-cartridge/src/research/memory.rs:533` | `NONE` | — | SQLite read for MCP job poll; JSON sidecar remains dual-written. |
| `replace_contribute_jobs` | `crates/umst-concrete-cartridge/src/research/memory.rs:558` | `NONE` | — | SQLite transaction boundary for ephemeral job map. |
| `SqliteStore` | `crates/umst-concrete-cartridge/src/research/memory.rs:598` | `NONE` | — | Re-export of IO-backed store; see `sqlite_store::SqliteStore`. |
| `estimate_mi_bits_rational` | `crates/umst-concrete-cartridge/src/research/mi.rs:17` | `NONE` | — | Epistemic surrogate for MCP enrichment; not admissibility. |
| `estimate_mi_bits_from_mix` | `crates/umst-concrete-cartridge/src/research/mi.rs:35` | `NONE` | — | `umst_mi_estimate` advisory wire; histogram MI on manifold PPO path. |
| `fields_for_code as gate_fields_for_code_ssot, remediation_for_code as gate_remediation_for_code, MANIFEST_BRIDGE_DISABLED, MIX_SPEC_RATIONAL_PARSE_FAIL, MIX_SPEC_WIRE_INVALID, THERMODYNAMIC_CD_FAIL, THERMODYNAMIC_FAIL, TOP_GATE_EXPLAIN_CODES` | `crates/umst-concrete-cartridge/src/research/mod.rs:16` | `NONE` | — | SSOT re-exports of manifold gate explain vocabulary. |
| `append_checkpoint_jsonl, build_checkpoint, merkle_root_from_leaves, CheckpointError, CheckpointRecord, CHECKPOINTS_JSONL_DEFAULT` | `crates/umst-concrete-cartridge/src/research/mod.rs:42` | `NONE` | — | Re-exports Merkle checkpoint helpers; audit IO on defining module. |
| `accept, content_hash_preimage, content_id, gate_check_mix, gate_check_mix_result, gate_recheck, gate_reject_row_for_mix, memory_record_from_contribution, mix_wire_from_spec_value, query, rational_to_f64, AcceptError, ContributeError, GateCheckExplain, GateCheckResult, GateContext, GateFieldIssue, DEFAULT_CATALOG_HASH` | `crates/umst-concrete-cartridge/src/research/mod.rs:49` | `NONE` | — | Re-exports gate/accept morphisms; Mechanised on `gate_check_mix` / `accept` in `contribution`. |
| `build_memory_export_bundle, hash_chain_for_rows, write_memory_export_bundle, ExportError, MemoryExportBundle` | `crates/umst-concrete-cartridge/src/research/mod.rs:58` | `NONE` | — | Re-exports memory export bundle builders; human handoff only. |
| `mix_geometry_key, mix_l1_distance, morton_index, morton_index_distance, MixGeometryKey` | `crates/umst-concrete-cartridge/src/research/mod.rs:65` | `NONE` | — | Re-exports Morton mix geometry; locality index not admissibility gate. |
| `validate_scope_token, ScopeError` | `crates/umst-concrete-cartridge/src/research/mod.rs:71` | `NONE` | — | Re-exports scope token validation; operator governance only. |
| `ConcretePhysicalReasoningLayer, PhysicalReasoningLayer` | `crates/umst-concrete-cartridge/src/research/mod.rs:75` | `NONE` | — | Re-exports Physical Reasoning Layer port trait and concrete impl. |
| `SqliteStore` | `crates/umst-concrete-cartridge/src/research/mod.rs:80` | `NONE` | — | Re-export of SQLite IO store when `agent-layer` enabled. |
| `filter_records, find_by_memory_id, query_page, InMemoryStore, MemoryError, MemoryStore, ResearchStore, StoreError` | `crates/umst-concrete-cartridge/src/research/mod.rs:84` | `NONE` | — | Re-exports functional memory store port and filter morphisms. |
| `estimate_mi_bits_from_mix, estimate_mi_bits_rational` | `crates/umst-concrete-cartridge/src/research/mod.rs:91` | `NONE` | — | Re-exports Landauer advisory MI surrogates; not admissibility gate. |
| `holdout_rmse_passes, live_witness_enabled, parse_promotion_policy_yaml, validate_promotion_policy, validate_track_a_stamp_tier, PolicyError, PromotionPolicy` | `crates/umst-concrete-cartridge/src/research/mod.rs:95` | `NONE` | — | Re-exports promotion policy YAML validation; human-gated CLI only. |
| `apply_promotion_writes, build_promotion_record, promote_contribution, PromotionApproval, PromotionError, PromotionRecordOut` | `crates/umst-concrete-cartridge/src/research/mod.rs:102` | `NONE` | — | Re-exports human-gated promotion record morphisms; never MCP. |
| `ensure_observed_at, is_monotonic_after, observed_at_for_tick, synthetic_observed_at, ProvenanceClock, UcrsStampMode, WallClock` | `crates/umst-concrete-cartridge/src/research/mod.rs:109` | `NONE` | — | Re-exports UCRS provenance clock and monotonic stamp helpers. |
| `append_gate_reject_jsonl, build_gate_reject, build_gate_reject_from_contribution, GateRejectRow, RejectError, GATE_REJECT_SCHEMA` | `crates/umst-concrete-cartridge/src/research/mod.rs:116` | `NONE` | — | Re-exports gate reject audit stream; excluded from admissible memory. |
| `append_memory_jsonl, SidecarError, MEMORY_JSONL_DEFAULT` | `crates/umst-concrete-cartridge/src/research/mod.rs:123` | `NONE` | — | Re-exports accepted-memory JSONL sidecar append hook. |
| `AcceptResult, Contribution, GateSummary, GateVerdict, MemoryPayload, MemoryQuery, MemoryQueryPage, MemoryRecord, ObservedAt, CANON_VERSION, CONTRIBUTION_SCHEMA, MEMORY_SCHEMA` | `crates/umst-concrete-cartridge/src/research/mod.rs:127` | `NONE` | — | Re-exports contribution.v1 / memory_record.v1 wire types. |
| `parse_contribution_json, validate_contribution_value, validate_for_accept, ValidationError` | `crates/umst-concrete-cartridge/src/research/mod.rs:134` | `NONE` | — | Re-exports contribution.v1 schema validators. |
| `ucrs_observed_at_to_v2` | `crates/umst-concrete-cartridge/src/research/mod.rs:141` | `NONE` | — | Re-export UCRS → v2 wire when `ucrs-provenance` feature enabled. |
| `observed_at_to_v2, ObservedAtV2, OBSERVED_AT_V2_SCHEMA` | `crates/umst-concrete-cartridge/src/research/mod.rs:145` | `NONE` | — | Re-exports observed_at.v2 integer wire mapping. |
| `memory_record_from_contribution as build_memory_record` | `crates/umst-concrete-cartridge/src/research/mod.rs:150` | `NONE` | — | Re-export alias for `memory_record_from_contribution`; see `contribution`. |
| `PolicyError` | `crates/umst-concrete-cartridge/src/research/policy.rs:9` | `NONE` | — | Human-gated calibration governance; not MCP admissibility. |
| `PromotionPolicy` | `crates/umst-concrete-cartridge/src/research/policy.rs:21` | `NONE` | — | Operator policy record; hold-out metrics are empirical gates only. |
| `parse_promotion_policy_yaml` | `crates/umst-concrete-cartridge/src/research/policy.rs:35` | `NONE` | — | YAML ingest for propose-promotion CLI; never MCP-exposed. |
| `validate_promotion_policy` | `crates/umst-concrete-cartridge/src/research/policy.rs:46` | `NONE` | — | Schema shape check on policy wire; not thermodynamic gate. |
| `live_witness_enabled` | `crates/umst-concrete-cartridge/src/research/policy.rs:70` | `NONE` | — | Env IO at session boundary; promotion policy gate only. |
| `validate_track_a_stamp_tier` | `crates/umst-concrete-cartridge/src/research/policy.rs:79` | `NONE` | — | Human-gated calibration policy; not thermodynamic admissibility. |
| `holdout_rmse_passes` | `crates/umst-concrete-cartridge/src/research/policy.rs:97` | `NONE` | — | Calibration hold-out metric; human review still required. |
| `PromotionError` | `crates/umst-concrete-cartridge/src/research/promotion.rs:20` | `NONE` | — | CLI-only promotion path; never MCP-exposed. |
| `PromotionApproval` | `crates/umst-concrete-cartridge/src/research/promotion.rs:36` | `NONE` | — | JWS-backed human decision; not agent auto-promotion. |
| `PromotionRecordOut` | `crates/umst-concrete-cartridge/src/research/promotion.rs:52` | `NONE` | — | Calibration handoff bundle; admissibility already on memory row. |
| `build_promotion_record` | `crates/umst-concrete-cartridge/src/research/promotion.rs:74` | `NONE` | — | Record morphism from gate-validated memory + human approval. |
| `apply_promotion_writes` | `crates/umst-concrete-cartridge/src/research/promotion.rs:163` | `NONE` | — | Filesystem writes for human-gated calibration promotion. |
| `promote_contribution` | `crates/umst-concrete-cartridge/src/research/promotion.rs:188` | `NONE` | — | CLI orchestration; physics gate already on accepted memory row. |
| `WallClock` | `crates/umst-concrete-cartridge/src/research/provenance.rs:13` | `NONE` | — | System time IO injection; stamp semantics on `ProvenanceClock`. |
| `epoch_ms` | `crates/umst-concrete-cartridge/src/research/provenance.rs:21` | `NONE` | — | `SystemTime` / UCRS wall hook; not monotonic sequence logic. |
| `from_env` | `crates/umst-concrete-cartridge/src/research/provenance.rs:55` | `NONE` | — | Env IO at session boundary; stamp mode selection only. |
| `synthetic_observed_at` | `crates/umst-concrete-cartridge/src/research/provenance.rs:290` | `NONE` | — | Convenience wrapper calling wall IO; reject stream not memory. |
| `GATE_REJECT_SCHEMA` | `crates/umst-concrete-cartridge/src/research/reject.rs:16` | `NONE` | — | Reject stream schema label; rows excluded from admissible memory. |
| `RejectError` | `crates/umst-concrete-cartridge/src/research/reject.rs:22` | `NONE` | — | Audit stream IO errors; not admissible memory store. |
| `GateRejectRow` | `crates/umst-concrete-cartridge/src/research/reject.rs:34` | `NONE` | — | Audit wire for failed gate; never in `admissible_only` query. |
| `mix_content_hash` | `crates/umst-concrete-cartridge/src/research/reject.rs:53` | `NONE` | — | Content hash for reject audit; gate verdict on manifold path. |
| `build_gate_reject` | `crates/umst-concrete-cartridge/src/research/reject.rs:68` | `NONE` | — | Reject row morphism; thermodynamic fail codes from gate path. |
| `build_gate_reject_from_contribution` | `crates/umst-concrete-cartridge/src/research/reject.rs:98` | `NONE` | — | Maps failed accept contribution to gate_reject.v1 audit row. |
| `append_gate_reject_jsonl` | `crates/umst-concrete-cartridge/src/research/reject.rs:117` | `NONE` | — | Append-only reject audit sink; not admissible memory. |
| `mix_spec_hashable` | `crates/umst-concrete-cartridge/src/research/reject.rs:137` | `NONE` | — | Pre-hash shape check; admissibility on `gate_recheck`. |
| `MEMORY_JSONL_DEFAULT` | `crates/umst-concrete-cartridge/src/research/sidecar.rs:12` | `NONE` | — | Filesystem path constant; not admissible memory query surface. |
| `SidecarError` | `crates/umst-concrete-cartridge/src/research/sidecar.rs:18` | `NONE` | — | Sidecar IO error sum type; rows validated before append. |
| `append_memory_jsonl` | `crates/umst-concrete-cartridge/src/research/sidecar.rs:30` | `NONE` | — | Append-only audit sink; admissible rows from `accept` only. |
| `CONTRIBUTION_SCHEMA` | `crates/umst-concrete-cartridge/src/research/types.rs:10` | `NONE` | — | Schema version string; validation in `validation` module. |
| `MEMORY_SCHEMA` | `crates/umst-concrete-cartridge/src/research/types.rs:15` | `NONE` | — | Schema version string for persisted rows. |
| `CANON_VERSION` | `crates/umst-concrete-cartridge/src/research/types.rs:20` | `NONE` | — | JCS profile label on wire; not a physics claim. |
| `OBSERVED_AT_V2_SCHEMA` | `crates/umst-concrete-cartridge/src/research/wire_v2.rs:9` | `NONE` | — | Schema version string for integer-only UCRS wire. |
| `with_witness_mode` | `crates/umst-concrete-cartridge/src/research/witness_test_lock.rs:10` | `NONE` | — | Test-only env serialization; not production gate semantics. |
| `ContributeJobStatus` | `crates/umst-mcp/src/agent_layer.rs:85` | `NONE` | — | MCP job status wire; physics on `gate_check_mix` / `accept`. |
| `ContributeJob` | `crates/umst-mcp/src/agent_layer.rs:98` | `NONE` | — | MCP async job envelope; gate on synchronous `contribute` delegate. |
| `AgentSession` | `crates/umst-mcp/src/agent_layer.rs:112` | `NONE` | — | MCP session state carrier; thermodynamic gate on cartridge `accept`. |
| `gate_check` | `crates/umst-mcp/src/agent_layer.rs:136` | `NONE` | — | stdio JSON-RPC transport; CD admissibility on `gate_check_mix_result`. |
| `mi_estimate` | `crates/umst-mcp/src/agent_layer.rs:150` | `NONE` | — | MI enrichment wire; not admissibility gate. |
| `contribute` | `crates/umst-mcp/src/agent_layer.rs:162` | `NONE` | — | stdio transport; gate re-check before memory append on cartridge. |
| `contribute_async` | `crates/umst-mcp/src/agent_layer.rs:192` | `NONE` | — | In-process async wrapper; same gate path as `contribute`. |
| `contribute_status` | `crates/umst-mcp/src/agent_layer.rs:265` | `NONE` | — | In-memory job map lookup; no new physics claim. |
| `memory_query` | `crates/umst-mcp/src/agent_layer.rs:274` | `NONE` | — | stdio transport over cartridge `query_page`; stable `(ucrs_seq, content_id)` sort. |
| `transition_propose` | `crates/umst-mcp/src/agent_layer.rs:283` | `NONE` | — | Chained operator workflow; gate must pass before async ingest. |
| `arena_open` | `crates/umst-mcp/src/agent_layer.rs:343` | `NONE` | — | stdio transport; parse-once arena session for hot gate loops. |
| `arena_open_bytes` | `crates/umst-mcp/src/agent_layer.rs:352` | `NONE` | — | Warm-boundary parse-once; gate physics on `gate_check_arena` delegate. |
| `gate_check_arena` | `crates/umst-mcp/src/agent_layer.rs:378` | `NONE` | — | Warm parse + in-process gate; no per-iter MCP round-trip. |
| `arena_close` | `crates/umst-mcp/src/agent_layer.rs:417` | `NONE` | — | Session lifecycle; releases mmap-owned buffer handle. |
| `SCHEMA_RESOURCES` | `crates/umst-mcp/src/agent_layer.rs:453` | `NONE` | — | Schema fixture bytes for MCP discovery; versioned wire only. |
| `AGENT_PROMPTS` | `crates/umst-mcp/src/agent_layer.rs:488` | `NONE` | — | MCP prompt templates; operational guidance only. |
| `resources_list_result` | `crates/umst-mcp/src/agent_layer.rs:534` | `NONE` | — | MCP resource enumeration; schema bytes are versioned fixtures. |
| `resources_read_result` | `crates/umst-mcp/src/agent_layer.rs:551` | `NONE` | — | MCP resource read; returns pinned JSON schema text. |
| `prompts_list_result` | `crates/umst-mcp/src/agent_layer.rs:571` | `NONE` | — | MCP prompts list; names only. |
| `prompts_get_result` | `crates/umst-mcp/src/agent_layer.rs:587` | `NONE` | — | MCP prompt body fetch; operational text. |
| `agent_tools_schema` | `crates/umst-mcp/src/agent_layer.rs:606` | `NONE` | — | MCP tool schema export; delegates to gate/memory/contribute impls. |
| `saturate01` | `crates/umst-mcp/src/soft_gate.rs:25` | `NONE` | — | Scalar clamp utility; no thermodynamic gate claim. |
| `smoothstep01` | `crates/umst-mcp/src/soft_gate.rs:34` | `NONE` | — | Hermite smoothstep kernel; training slack only. |
| `smoothstep` | `crates/umst-mcp/src/soft_gate.rs:44` | `NONE` | — | Bounded Hermite ramp; not a hard admissibility witness. |
| `soft_lower_gate` | `crates/umst-mcp/src/soft_gate.rs:59` | `NONE` | — | Soft lower-bound multiplier for policy loss; hard gate is separate. |
| `soft_upper_gate` | `crates/umst-mcp/src/soft_gate.rs:71` | `NONE` | — | Soft upper-bound multiplier for policy loss; hard gate is separate. |
| `soft_band_gate` | `crates/umst-mcp/src/soft_gate.rs:83` | `NONE` | — | Product of soft lower/upper ramps; exploration slack only. |
| `connected_fraction_gate` | `crates/umst-mcp/src/soft_gate.rs:92` | `NONE` | — | Supercap percolation soft ramp; not connected-fraction witness. |
| `network_conductivity_factor` | `crates/umst-mcp/src/soft_gate.rs:101` | `NONE` | — | Alias for supercap conductivity nomenclature; soft ramp only. |
| `soft_violation_penalty` | `crates/umst-mcp/src/soft_gate.rs:110` | `NONE` | — | Upper slack penalty for policy loss; not CD reject. |
| `soft_deficit_penalty` | `crates/umst-mcp/src/soft_gate.rs:119` | `NONE` | — | Lower slack penalty for policy loss; not CD reject. |
| `band_margin_penalty` | `crates/umst-mcp/src/soft_gate.rs:128` | `NONE` | — | Combined band margin penalty; hard band gate remains on cartridge. |
| `printability_tau_gate` | `crates/umst-mcp/src/soft_gate.rs:137` | `NONE` | — | Roussel τ₀ window soft multiplier; printability gate is hard path. |
| `extrudability_gate` | `crates/umst-mcp/src/soft_gate.rs:146` | `NONE` | — | Extrudability floor soft ramp; hard extrudability gate is separate. |
| `printability_dual_gate` | `crates/umst-mcp/src/soft_gate.rs:155` | `NONE` | — | Product of printability soft gates; dual hard gate on cartridge path. |
| `audit_rows` | `crates/umst-py/src/lib.rs:147` | `NONE` | — | Encodes iterable of row dicts into dataset-style CSV then reuses **`audit_csv_buf`** (aligned with **`audit`** string path). |
| `audit` | `crates/umst-py/src/lib.rs:167` | `NONE` | — | Python transport over CLI audit glue; no extra physical claim beyond CSV→facade audit. |
| `bundled_profile_ids` | `crates/umst-py/src/lib.rs:217` | `NONE` | — | Bundled id manifest for packaging smoke tests. |
| `canonical_json` | `crates/umst-py/src/lib.rs:229` | `NONE` | — | Byte-stable JSON for golden tests; matches **`umst-canonical`** binary. |
| `memory_query` | `crates/umst-py/src/lib.rs:271` | `NONE` | — | Python transport over [`query_page`]; paginated filter over env-backed store. |

