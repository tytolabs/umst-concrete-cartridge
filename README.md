<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# UMST Concrete Cartridge

[![CI — Rust](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/rust.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/rust.yml)
[![Notebook](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/notebook.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/notebook.yml)
[![Docker](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/docker.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/docker.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-black.svg)](LICENSE)

**UMST Concrete Cartridge** is the differentiable constitutive and calibration layer for cementitious materials, built to mount on **[UMST Manifold](https://github.com/tytolabs/umst-manifold)**. The workspace ships a Rust library (**Burn** + **`burn-ndarray`**), a CLI, **PyO3** bindings, and an **MCP** server so the same physics and data contracts surface in notebooks, services, and agent workflows. `ConcreteCartridge` supplies mix-to-field closures; the manifold supplies DEC operators, equilibrium and adjoint mechanics, fracture and transport kernels, and topology evolution—composed through explicit **Cargo features** that forward the manifold’s **`solver-stable` / `solver-research` / `solver-experimental`** lanes.

<p align="center">
  <img src="docs/assets/beam_strut_and_tie.gif" alt="RC beam strut-and-tie topology animation (32×8 grid, ρ field + compliance strip)" width="960" />
</p>

*32×8 RC beam surrogate: adjoint compliance topology optimization with a fixed bottom rebar row; viridis density ρ, compliance strip, captions — rendered to `notebooks/_artifacts/beam_strut_and_tie.gif` via the same mechanics façade as the rest of the cartridge.*

## Composition with the manifold

- **[`umst-manifold`](https://github.com/tytolabs/umst-manifold)** (git dependency on `main`) provides graph/mesh operators, solver implementations, and the **`IScienceCartridge`** host API.
- **This repo** holds hydration kinetics, empirical calibration profiles, Striatus-class shell demos, print-ready export pipelines, and tests that lock cartridge–manifold wiring.

With **`solver-experimental`** disabled (default), `ConcreteCartridge::compute_topology` follows the heat-graph Laplacian path; enabling **`solver-experimental`** pulls the full manifold solver union so examples such as **`optimize_rc_beam`** and **`optimize_shell_3d`** compile and run with phase-field, adjoint, and shell-specific hooks documented in-repo.

## Workspace layout

| Crate | Role |
|-------|------|
| `crates/umst-concrete-cartridge` | Library façade, constitutive modules, topology helpers, Striatus examples. |
| `crates/umst-cli` | Binaries `umst`, `umst-canonical`. |
| `crates/umst-py` | Python bindings (`pip install './crates/umst-py[notebook]'`). |
| `crates/umst-mcp` | Model Context Protocol server for agent-facing tools. |

Root **`Dockerfile`** / **`docker-compose.yml`** package the MCP service for container deployment.

## Feature flags (cartridge)

Declared in [`crates/umst-concrete-cartridge/Cargo.toml`](crates/umst-concrete-cartridge/Cargo.toml); names mirror the manifold.

| Feature | Effect |
|---------|--------|
| `solver-stable` | Forwards `umst-manifold/solver-stable`. |
| `solver-research` | Forwards `umst-manifold/solver-research`. |
| `solver-experimental` | Forwards `umst-manifold/solver-experimental` (stable ∪ research). |
| Granular forwards | e.g. `photonics-fdfd`, `electrochemistry-pnp`, `mechanics-adjoint`, `statistical-mechanics-vinet` — single-kernel pulls. |
| `blas-accelerate` | Apple Accelerate linking for `burn-ndarray` (and manifold). |
| `mac-fast` | `solver-experimental` + `render` + `blas-accelerate` — local M-series throughput bundle. |
| `render` | Striatus / shell demo renderer hook for `optimize_shell_3d` (artefact layout + visualization). |

Authoritative solver ↔ verification narrative: [`docs/Solver-Status.md`](docs/Solver-Status.md) here (Striatus, gates, runbooks) with the **master solver table** in [`umst-manifold/docs/Solver-Status.md`](https://github.com/tytolabs/umst-manifold/blob/main/docs/Solver-Status.md). Striatus artefact semantics: [`docs/Striatus.md`](docs/Striatus.md).

## Quick CLI

```bash
echo '{"w_c":0.4,"temperature_k":293.15}' | umst --profile uci_d1 predict
```

```bash
head -n2 datasets/dataset_d1.csv | umst --profile uci_d1 audit
```

Install: `cargo install --path crates/umst-cli`.

## Python and notebooks

```bash
cd umst-concrete-cartridge
pip install './crates/umst-py[notebook]'
```

[`notebooks/README.md`](notebooks/README.md) covers notebook workflows.

## Build and test (Rust)

```bash
cd umst-concrete-cartridge
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

**Toolchain:** **1.88** in [`rust-toolchain.toml`](rust-toolchain.toml) — aligns with manifold CI and optional **`--all-features`** MSRV paths.  
**CPU acceleration (macOS):** `cargo build --workspace --features blas-accelerate`.

## Bundled calibration profiles

| Id | Notes |
|----|-------|
| `default` | Baseline heuristic bundle. |
| `uci_d1` | UCI concrete strength corpus (`datasets/dataset_d1.csv`). |
| `zenodo_ndt`, `zenodo_sonreb`, `zenodo_rh` | Zenodo record [14921019](https://zenodo.org/records/14921019). |
| `uhpc`, `selfheal` | Boundary profiles exercised in verification tests. |
| `highscm` | High-SCM / alternative binder regime. |

Provenance: [`docs/SSOT.json`](docs/SSOT.json), [`datasets/PROVENANCE.md`](datasets/PROVENANCE.md).

## Documentation

- **Constitutive equations:** [`docs/Constitutive-Equations.md`](docs/Constitutive-Equations.md)
- **Validation:** [`docs/Validation.md`](docs/Validation.md)
- **Wire schemas:** [`docs/WireSchemas.md`](docs/WireSchemas.md)
- **Formal anchors / generated proof status:** [`docs/PROOF-STATUS.md`](docs/PROOF-STATUS.md), [`docs/FormalProvenance.md`](docs/FormalProvenance.md)

## Citation

[`CITATION.cff`](CITATION.cff) and the repository URL.

## Contributing and license

[`CONTRIBUTING.md`](CONTRIBUTING.md), [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md), [`SECURITY.md`](SECURITY.md).  
Released under the [MIT License](LICENSE). © 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO.
