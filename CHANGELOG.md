SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
<!--
-->

# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added (PRL Openings Finish — 2026-06-19)

- **`umst memory export`** CLI — signed `memory_export_bundle.v1` + JCS hash chain.
- **`umst_transition_propose`** MCP tool; prompt **`audit_mix_csv`**; Phase 8 **`umst-py`** parity.
- Docs: [`CARTRIDGE_PORT.md`](docs/CARTRIDGE_PORT.md), [`MEMORY_REPLICATION.md`](docs/MEMORY_REPLICATION.md).

### Prepare [1.0.0-agent-layer] — **no git tag without maintainer approval**

- PRL MCP P0/P1 surface complete; 16,146-row audit bootstrap; live witness promotion gate.

**Impact:** Cementitious closures stay on the manifold’s pinned catalog and witness ladder — `manifest-bridge` aligns predict-path gates with manifold `catalog_id` evaluators without duplicating CD math in the cartridge. Catalog SSOT is the **119-module** unified export (`module_count` in `artifacts/catalog.lock.json`), digest **`0697014fb5b90a3aca4db3e5cc226896ca198802c910d5395f254e4262aa6227`** (`0697014f…` in logs), aligned with manifold @ [`fe22437`](https://github.com/tytolabs/umst-manifold/commit/fe22437).

### Added

- **`manifest-bridge`** optional feature on `umst-concrete-cartridge`: forwards `umst_manifold::manifest` types from git-pinned **`umst-manifold`** (`rev = fe22437`); `predict` can run the manifold gate bundle when enabled (see manifold [`docs/GateUnificationSpec.md`](../umst-manifold/docs/GateUnificationSpec.md) and [`docs/VERIFY.md`](../umst-manifold/docs/VERIFY.md)).
- **G-02 (closed 2026-05-29):** GitHub Actions runs `cargo test -p umst-concrete-cartridge --features manifest-bridge` on the **git dependency only** — no workspace `[patch]` to sibling manifold (see [`docs/FORMAL_GROUNDING_AUDIT.md`](docs/FORMAL_GROUNDING_AUDIT.md)).
- Cross-link to unified catalog lock digest **`0697014f…`** (**119** Lean modules, dual-pin merge) in manifold `artifacts/catalog.lock.json`; fiber pin **`c1d9ba2a…`** (**69** modules, `umst-formal-double-slit`) retained for provenance only; coverage audit (when published): [`../umst-manifold/docs/CATALOG_COVERAGE_AUDIT.md`](../umst-manifold/docs/CATALOG_COVERAGE_AUDIT.md).

### Changed

- **Documentation:** `README.md` and [`docs/FORMAL_GROUNDING_AUDIT.md`](docs/FORMAL_GROUNDING_AUDIT.md) — **119-module** catalog SSOT, manifold pin **`fe22437`**, W8 done / **G-02** git-only `manifest-bridge` CI (no workspace `[patch]`); corrected lock bundle SHA-256 `1d0d1ed6…`.
- **Documentation:** [`docs/FORMAL_GROUNDING_AUDIT.md`](docs/FORMAL_GROUNDING_AUDIT.md) — G-02 skip-rule prose matches `manifest_bridge_catalog_grounding` (git-embed checks always run; sibling doc cross-checks optional); pins **`6742fa3`** / **`fe22437`** recorded.

### Added (continued)

- Striatus-class shell topology demo: `optimize_shell_3d` (with `solver-experimental` + `render`), `shell_demo_smoke`, Z2×Z2 symmetry helper in `print_ready::symmetry`, and driver `notebooks/_run_shell_demo.sh`.
- PyVista render pipeline (`render_shell_gif.py`, `overlay_final_isostatics.py`, `stitch_gif.py`), STL export (`export_print_ready.py`), and `notebooks/tests/test_print_ready.py`.
- Documentation pack `docs/Striatus.md` and `docs/References.bib`; README hero embed for `notebooks/_artifacts/striatus_emergence.gif`; optional `crates/umst-py[render]` extra in `pyproject.toml`.
- Acceptance helpers: `notebooks/check_shell_artifact_budgets.sh` (GIF/STL size budgets) and `notebooks/check_shell_determinism.sh` (two-run STL hash); composer prompt documents CI subset commands.

### Changed

- Workspace Rust crates set `publish = false` (distribution via Git tags, Docker, and maturin wheels—not crates.io).

### Added

- **`result.v2` `axioms`**: sorted axiom identifiers on every prediction wire payload; JSON Schema **`schema/result.v2.json`** allowlists the Lean-facing closure alongside **`audit.v1`**.
- **`audit_rows`** (Rust PyO3) and **`audit_dataframe`** (optional pandas via **`[notebook]`** extra): helpers for programmatic CSV audit without hand-serialisation.
- **`tests`** Python extra (**`hypothesis>=6,<7`**) plus **`tests/test_property_determinism.py`** exercising CLI vs Python **`canonical_json`** parity across bundled profiles (bounded random mix scalars).
- **`docs/WireSchemas.md`** and **`docs/FormalProvenance.md`**: SSOT/schema status table and **`formal_anchor`** → **`umst-formal`** workflow.
- **`notebooks/run_all.sh --strict`** for CI/notebook pipelines; audited notebook corpus with refreshed provenance (**UCI D1** + **Zenodo 14921019** rows).

### Changed

- **`audit.v1`** schema axioms tightened to enumerated allow-list + **`uniqueItems`** on the **`axioms`** array (matches certification chain vocabulary).
- Python **`pyproject.toml` optional-dependencies**: standalone **`pandas`** extra removed (**`notebook`** still ships pandas/matplotlib/Jupyter).

### Fixed

- CI / Docker: **`umst-manifold`** git dependency follows branch **`main`** (upstream default renamed from **`master`**).
- **`maturin develop`** jobs create a repo-root **`.venv`** (maturin refuses bare system interpreters).
- **`Dockerfile`** no longer copies **`Cargo.lock`** (lockfile is gitignored — image build resolves deps during **`cargo build`**).
- **Notebook** workflow also triggers on **`push`** to **`main`** (parity with **`rust.yml`**).
- **Notebook** CI prepends **`.venv/bin`** to **`PATH`** before **`run_all.sh --strict`** so **`jupyter`** / **`nbconvert`** resolve (venv installs are not on the default PATH).
- **Docker** workflow builds on every **`main`** push (**`push: false`**); GHCR login + image push remain **version tags (`v*`)** only.

## [0.2.0] — 2026-05-07

### Added

- **Virtual workspace** root with the library in `crates/umst-concrete-cartridge` and members `umst-cli`, `umst-mcp`, `umst-py`.
- **`umst_concrete_cartridge::facade`**: serde-only transport surface + `predict` / audit / certify helpers without `serde_json` / `tokio` / `clap` in the core crate.
- **`umst audit`** + **`audit.v1`** schema; CSV header synonyms for `datasets/dataset_d1.csv`-style corpora.
- **`umst-canonical`** binary: deterministic JSON bytes (sorted object keys, Ryū float literals, rejects non-finite numbers).
- **Python** (`crates/umst-py`, PyO3 0.22, `abi3-py310`): `predict`, `audit`, `certify`, `schema`, `canonical_json`; maturin `pyproject.toml` + `python/umst_concrete_cartridge` shim.
- **MCP** (`umst-mcp`): **`umst_predict`**, **`umst_audit`**, **`umst_profiles`**, **`umst_certify`** with optional `canonical` flag.
- **Docker / compose** + **`scripts/mcp_smoke.py`**; **`scripts/acceptance_v02.sh`** bundles smoke steps.
- **Notebooks** (`notebooks/sustainable_mix_audit.ipynb`, `run_all.sh`) with approved envelope claim language.
- **CI**: `python-wheels.yml`, `docker.yml`, `notebook.yml`; `rust.yml` scans `crates/**`, `cargo check --all-features`, `cargo tree -p umst-concrete-cartridge` dep guard, MCP smoke, maturin develop job.
- Formal hygiene: **`naturalitySquare`** marker check when **`UMST_FORMAL_ROOT`** points at `umst-formal`.

### Changed

- README / **`CITATION.cff`** now document CLI · Python · MCP · Docker surfaces.
- **`audit.v1` row objects** are contract-shaped: nested **`input`** mix map, **`profile_used`**, per-row **`formal_anchor`**, **`predicted_strength_mpa`**, optional **`observed_strength_mpa`** / **`abs_error_mpa`**, **`safety_margin`**, and **`regime_warnings`** (JSON Schema enforces the stable row shape).
- **Python `predict`**: `predict(spec, *, profile="default", schema_version="v2")` — mix dict first, keyword-only profile/schema; `compare_homogeneous` removed from the public binding surface.
- **`scripts/acceptance_v02.sh`** step **[7]** asserts **`umst predict` → `umst-canonical`** bytes match **`canonical_json(predict(...))`** (after `maturin develop` if needed).

### Fixed

- **`CITATION.cff` `date-released`** advanced past the v0.1.0 stamp to reflect the v0.2.0 citation cut.

## [Unreleased]

### Added
- Versioned **calibration** artefacts: eight bundled `calibration/profiles/*.v1.toml` profiles (lifted from umst-prototype-2a (Zenodo 18940933); SHA-256 traceability documented in `calibration/SCHEMA.md`), copy-of-record CSVs under `datasets/`, and homogeneous **Powers gel-space** routing via `Profile`.
- **result.v2** JSON schema (`schema/result.v2.json`): default `umst predict` payload now includes `calibration_profile`, `calibration_model`, `formal_anchor`, and `warnings`; `result.v1` remains available via `--schema-version v1` for one minor cycle.
- **`umst` CLI** extensions: global `--profile` / `--profile-file`, `umst profiles {list,describe,regime}`, `umst certify NAME`, `umst schema result-v2`, and stderr notice when the default profile applies.
- Regression tests: CSV-backed dataset metrics (`tests/calibration/dataset_metrics.rs`), formal-anchor doc lint (`tests/formal_anchors.rs`), regime warnings, migration anchor sampling.
- **`calibration_report`** binary (`--features cli,calibration`) regenerates committed `docs/Calibration.md`.
- Documentation: `docs/FormalAnchors.md`, `docs/Calibration.md`, and expanded calibration notes in `README`, `docs/CLI.md`, `docs/Validation.md`.
- Optional **`sha2`** dependency (behind `calibration` feature) for deterministic profile file digests in the report binary.

### Changed
- Formal-anchor lint extended to a five-status grammar (**`Mechanised`**, **`Structural`**, **`Empirical`**, **`Literature`**, **`NONE`**). The placeholder **`Library`** status and the boilerplate “Differentiable training pathway…” rationale are removed; every public symbol carries a precise per-symbol classification (see [`docs/PROOF-STATUS.md`](docs/PROOF-STATUS.md)).
- Four tensor pathways flipped to **`Mechanised`** against `umst-formal@8a6b372`: **`chemo_water`**, **`set_time`**, **`thermo`**, **`transport`** (chloride diffusivity → `MeasurementCost.lean#zero_info_zero_energy`).
- Ten modules reclassified to **`Empirical`**: **`colloidal`**, **`creep`**, **`freeze_thaw`**, **`itz`**, **`nano`**, **`polymer`**, **`printability`**, **`rheology`**, **`self_heal`**, **`shrinkage`**.
- Four modules reclassified to **`Literature`**: **`fiber`**, **`fracture`**, **`packing`**, **`sustainability`** (plus wire-schema version constants and Mills / ACI / EN closures as cited).
- **`cost`** remains **`NONE`** with an explicit auxiliary-objective rationale.
- Renamed profiles `uci_d2` / `uci_d3` / `uci_d4` → **`zenodo_ndt`** / **`zenodo_sonreb`** / **`zenodo_rh`** to match the IROS Paper 2 reproducibility manifest (TU/e + TNO, Zenodo Record **14921019**, CC-BY 4.0). `[provenance]` blocks carry the full attribution; on-disk CSV filenames stay `dataset_d2.csv`, ….
- **`Profile::regime_check_scalars`** documents a mechanised anchor to **`RegimeSoundness.warnings_empty_iff_in_regime`** (`umst-formal`).
- **`umst certify`** JSON includes a string **`formal_status`** (one of the five buckets) plus optional **`zenodo_record`** / **`zenodo_doi`** / **`zenodo_url`** / **`license`** / **`subset`** when present on the profile.
- **`docs/SSOT.json`** uses **`total_data_rows`** (= **17646**) as the summed row-count field name.
- **Boundary calibration profiles** (`uhpc`, `selfheal`) omit `[acceptance]` headline metrics; `tests/calibration/dataset_metrics.rs` skips any profile with `verification_status = "Boundary"`.
- Integration tests [`tests/cli/public_contract.rs`](tests/cli/public_contract.rs) lock in live-binary acceptance checks 7–10 (`profiles list`, `predict` v2 fields, temperature regime `warnings`, `certify` JSON).

### Fixed
- Lint hygiene: doc-line parsing no longer false-matches **`Option<…>`** as **`formal_status: Option`** (pattern requires a whitelisted single-token status).
- Lint hygiene: **`Boundary`** is rejected as a Rust **`formal_status`** — it remains a **`verification_status`** field in calibration TOML only; **`umst certify`** maps unknown/boundary metadata to wire **`NONE`** when needed.

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
