<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Versioned **calibration** artefacts: eight bundled `calibration/profiles/*.v1.toml` profiles (lifted from prototype-3 JSON SHA-256 documented in `calibration/SCHEMA.md`), copy-of-record CSVs under `datasets/`, and homogeneous **Powers gel-space** routing via `Profile`.
- **result.v2** JSON schema (`schema/result.v2.json`): default `umst predict` payload now includes `calibration_profile`, `calibration_model`, `formal_anchor`, and `warnings`; `result.v1` remains available via `--schema-version v1` for one minor cycle.
- **`umst` CLI** extensions: global `--profile` / `--profile-file`, `umst profiles {list,describe,regime}`, `umst certify NAME`, `umst schema result-v2`, and stderr notice when the default profile applies.
- Regression tests: CSV-backed dataset metrics (`tests/calibration/dataset_metrics.rs`), formal-anchor doc lint (`tests/formal_anchors.rs`), regime warnings, migration anchor sampling.
- **`calibration_report`** binary (`--features cli,calibration`) regenerates committed `docs/Calibration.md`.
- Documentation: `docs/FormalAnchors.md`, `docs/Calibration.md`, and expanded calibration notes in `README`, `docs/CLI.md`, `docs/Validation.md`.
- Optional **`sha2`** dependency (behind `calibration` feature) for deterministic profile file digests in the report binary.

### Changed
- Renamed profiles `uci_d2` / `uci_d3` / `uci_d4` → **`zenodo_ndt`** / **`zenodo_sonreb`** / **`zenodo_rh`** to match the IROS Paper 2 reproducibility manifest (TU/e + TNO, Zenodo Record **14921019**, CC-BY 4.0). `[provenance]` blocks carry the full attribution; on-disk CSV filenames stay `dataset_d2.csv`, ….
- **`Profile::regime_check_scalars`** now documents a mechanised anchor to **`RegimeSoundness.warnings_empty_iff_in_regime`** (`umst-formal`). Other calibration-layer structs keep explicit **`NONE`** anchors where the witness lives in another crate or awaits a future homogeneous Jennings branch — see **`docs/FormalAnchors.md`** (“Future formal links”) and the **`TODO_FORMAL`** note on **`powers_compressive_strength_mpa`**.
- **`umst certify`** JSON includes optional **`zenodo_record`** / **`zenodo_doi`** / **`zenodo_url`** / **`license`** / **`subset`** when present on the profile.
- **`docs/SSOT.json`** uses **`total_data_rows`** (= **17646**) as the summed row-count field name.
- **Boundary calibration profiles** (`uhpc`, `selfheal`) no longer carry `[acceptance]` headline metrics; `tests/calibration/dataset_metrics.rs` skips any profile with `verification_status = "Boundary"`. UHPC TOML documents that Powers gel-space is the wrong model class for dense UHPC microstructure.
- New integration tests [`tests/cli/public_contract.rs`](tests/cli/public_contract.rs) lock in live-binary acceptance checks 7–10 (`profiles list`, `predict` v2 fields, temperature regime `warnings`, `certify` JSON).

### Removed
- **`lunar`** bundled calibration profile and **`datasets/dataset_lunar.csv`**. No real lunar concrete corpus exists at the v0.1 horizon; shipping a synthetic placeholder violated the measurement-or-derivation provenance rule.

## [0.1.0] — 2026-05-07

### Added
- Initial public release of the UMST Concrete Cartridge.
- `core::ConcreteCartridge` — implements
  `umst_manifold::core::traits::IScienceCartridge` over the manifold's
  cellular sheaf.
- 22 constitutive modules under `src/physics/`:
  - **Hydration & chemistry:** `hydration`, `chemo_water`, `set_time`, `thermo`.
  - **Microstructure:** `nano`, `colloidal`, `porosity`, `itz`, `packing`.
  - **Rheology & printing:** `rheology` (Chateau–Ovarlez / YODEL),
    `printability` (Roussel constraints).
  - **Mechanics:** `strength` (Jennings CM-II), `fracture` (Ulm
    micromechanics), `creep` (Bažant B4), `shrinkage`, `fiber`, `polymer`.
  - **Durability:** `freeze_thaw`, `transport`, `self_heal`.
  - **Sustainability and economics:** `sustainability` (embodied CO₂),
    `cost`.
- Documentation: README, `docs/Constitutive-Equations.md`, `docs/Validation.md`.
- Worked example: `examples/hydration_simulation.rs`.
- CI: `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo doc -D warnings`
  with sibling-checkout of `umst-manifold`.
- Governance: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1),
  `SECURITY.md`, `CITATION.cff`, issue and pull-request templates, dependabot.

[Unreleased]: https://github.com/tytolabs/umst-concrete-cartridge/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/tytolabs/umst-concrete-cartridge/releases/tag/v0.1.0
