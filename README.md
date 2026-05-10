<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# UMST Concrete Cartridge

[![CI — Rust](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/rust.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/rust.yml)
[![Notebook](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/notebook.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/notebook.yml)
[![Docker](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/docker.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/docker.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-black.svg)](LICENSE)

**Differentiable constitutive engine for cementitious materials** on the [UMST manifold](https://github.com/tytolabs/umst-manifold): one Rust façade drives the `umst` CLI, PyO3 bindings, and MCP server, emitting stable `result.v2` / `audit.v1` JSON with calibration profile metadata, formal anchors, and explicit warnings. For engineers wiring mix-design tools, researchers reproducing bundled calibration corpora, and operators packaging containers or agent-facing MCP tools.

**Repository:** [github.com/tytolabs/umst-concrete-cartridge](https://github.com/tytolabs/umst-concrete-cartridge)

## Build and test

From this repository root:

```bash
cd umst-concrete-cartridge
cargo build --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Install the CLI locally: `cargo install --path crates/umst-cli`.

**Toolchain:** Rust **1.88** is verified in this workspace (recommended when using a path patch to `umst-manifold`, which pins 1.88). Workspace metadata declares **1.75** as `rust-version`; upgrade the toolchain if dependency resolution requires it.

## Cargo features (`umst-concrete-cartridge`)

| Feature | Purpose |
|--------|---------|
| *(default)* | Standard façade and topology path without experimental manifold solvers pulled into the cartridge. |
| `solver-experimental` | Enables `umst-manifold` experimental solver stack (e.g. AT2 damage + coupled THMC stubs) inside `ConcreteCartridge::compute_topology`. |

Other workspace crates (`umst-cli`, `umst-mcp`, `umst-py`) consume the library; they do not add separate Cargo feature flags beyond dependency edges.

## Python and notebooks

```bash
cd umst-concrete-cartridge
pip install './crates/umst-py[notebook]'
# or: cd crates/umst-py && maturin develop --extras notebook
```

Headless notebooks: [`notebooks/README.md`](notebooks/README.md). Determinism checks: `python3 -m unittest discover -s crates/umst-py/tests -p 'test_property_determinism.py' -v` after `pip install -e 'crates/umst-py[tests]'`.

## Quick CLI usage

```bash
echo '{"w_c":0.4,"temperature_k":293.15}' | umst --profile uci_d1 predict
head -n2 datasets/dataset_d1.csv | umst --profile uci_d1 audit
```

Canonical byte-stable JSON: pipe CLI output through `target/release/umst-canonical` (after `cargo build -p umst-cli --release`) and compare with `canonical_json()` from Python; see [`scripts/check_predict_determinism.py`](scripts/check_predict_determinism.py).

## Surfaces

| Surface | Location | Notes |
|--------|----------|--------|
| Library | `crates/umst-concrete-cartridge` | Core façade and constitutive modules. |
| CLI | `crates/umst-cli` (`umst`, `umst-canonical`) | `predict`, `audit`, `certify`, `profiles`. |
| Python | `crates/umst-py` | `predict`, `audit`, `canonical_json`, optional notebook extras. |
| MCP | `crates/umst-mcp` | Stdio MCP tools; [`crates/umst-mcp/README.md`](crates/umst-mcp/README.md). |
| Docker | `Dockerfile`, [`docker-compose.yml`](docker-compose.yml) | MCP-oriented image and compose service. |

## Bundled calibration profiles

| Id | Notes |
|----|--------|
| `default` | Baseline bundle. |
| `uci_d1` | UCI concrete strength corpus (`datasets/dataset_d1.csv`). |
| `zenodo_ndt`, `zenodo_sonreb`, `zenodo_rh` | Zenodo record [14921019](https://zenodo.org/records/14921019) mirrors (`dataset_d2`–`d4`). |
| `uhpc`, `selfheal` | Boundary `verification_status` profiles (dataset tests may skip strict acceptance gates). |
| `highscm` | High-SCM regime (`dataset_highscm.csv`). |

Dataset row counts and citations: [`datasets/PROVENANCE.md`](datasets/PROVENANCE.md). Single source of truth for row totals: [`docs/SSOT.json`](docs/SSOT.json).

## Documentation

- Wire schemas (`mix.v1`, `result.v2`, `audit.v1`): [`docs/WireSchemas.md`](docs/WireSchemas.md)
- Constitutive equations and validation: [`docs/Constitutive-Equations.md`](docs/Constitutive-Equations.md), [`docs/Validation.md`](docs/Validation.md)
- Formal anchor status by symbol: [`docs/PROOF-STATUS.md`](docs/PROOF-STATUS.md)
- Formal provenance (Lean URIs): [`docs/FormalProvenance.md`](docs/FormalProvenance.md)

Formal lemmas live in [umst-formal](https://github.com/tytolabs/umst-formal).

## Citation

Use [`CITATION.cff`](CITATION.cff) or the repository URL for software citations.

## Contributing · security · license

**Santhosh Shyamsundar** — [santhoshshyamsundar@tyto.studio](mailto:santhoshshyamsundar@tyto.studio) · **Santosh Prabhu Shenbagamoorthy** — [santosh@tyto.studio](mailto:santosh@tyto.studio)

Contributing: [`CONTRIBUTING.md`](CONTRIBUTING.md), [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). Security: [`SECURITY.md`](SECURITY.md).

Released under the [MIT License](LICENSE).
