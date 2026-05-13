<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# UMST Concrete Cartridge

[![CI — Rust](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/rust.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/rust.yml)
[![Notebook](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/notebook.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/notebook.yml)
[![Docker](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/docker.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/docker.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-black.svg)](LICENSE)

A differentiable constitutive engine for cementitious materials, built to run on the [UMST manifold](https://github.com/tytolabs/umst-manifold). It provides a single Rust façade (with PyO3 and MCP bindings) for evaluating mix designs, predicting hydration kinetics, and running adjoint-based topology optimization for structural components.

<p align="center">
  <img src="docs/assets/beam_strut_and_tie.gif" alt="RC beam strut-and-tie topology animation (32×8 grid, ρ field + compliance strip)" width="960" />
</p>

*The hero GIF is a 32×8 RC beam surrogate: adjoint compliance topology optimization with a fixed bottom rebar row, rendered to `notebooks/_artifacts/beam_strut_and_tie.gif` (viridis ρ, compliance strip, captions). It is meant for structural engineers and architects reading strut-and-tie–like load paths in the density field—same UMST/MaOS mechanics façade as the rest of the cartridge, not a separate production solver UI.*

## How to Use the Two Repositories Together

The UMST ecosystem is split into two halves by design:
- **`umst-manifold`** provides the domain-agnostic PDE solvers, Discrete Exterior Calculus (DEC) operators, and topology optimization routines.
- **`umst-concrete-cartridge`** (this repo) provides the material science. It holds the constitutive equations, hydration kinetics, and empirical calibration profiles.

**Workflow:** You load the `ConcreteCartridge` into the Manifold. The manifold asks the cartridge for material properties (like strength, porosity, or temperature response) at specific points, and the manifold handles solving the global physics (heat diffusion, structural mechanics) across the entire geometry.

## Overview

This repository packages material science formulas into a computational cartridge. It is intended for structural engineers wiring mix-design tools, researchers reproducing calibration corpora, and operators packaging agent-facing MCP tools. 

Shell topology capabilities, the Striatus artefact contract, and print-ready gates are documented in detail in **[`docs/Solver-Status.md`](docs/Solver-Status.md)** and **[`docs/Striatus.md`](docs/Striatus.md)**.

## Quick CLI Usage

You can use the CLI to predict physical properties from mix parameters:
```bash
echo '{"w_c":0.4,"temperature_k":293.15}' | umst --profile uci_d1 predict
```

Or audit a dataset against the formal calibration bounds:
```bash
head -n2 datasets/dataset_d1.csv | umst --profile uci_d1 audit
```

## Python & Notebooks

To use the Python bindings locally for headless notebooks or scripting:
```bash
cd umst-concrete-cartridge
pip install './crates/umst-py[notebook]'
```
*See [`notebooks/README.md`](notebooks/README.md) for detailed Python usage.*

## Build and Test (Rust)

From the repository root:

```bash
cd umst-concrete-cartridge
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```
*Install the CLI locally: `cargo install --path crates/umst-cli`.*

**Toolchain:** Rust **1.88** is pinned in [`rust-toolchain.toml`](rust-toolchain.toml). Use `rustup default 1.88` to ensure CI parity.
**CPU Acceleration (macOS):** `cargo build --workspace --features blas-accelerate` enables Accelerate linking via `burn-ndarray`. 

## Surfaces & Ecosystem

| Surface | Location | Notes |
|--------|----------|--------|
| **Library** | `crates/umst-concrete-cartridge` | Core façade and constitutive physics modules. |
| **CLI** | `crates/umst-cli` | Standalone binaries (`umst`, `umst-canonical`). |
| **Python** | `crates/umst-py` | PyO3 bindings for prediction and auditing. |
| **MCP** | `crates/umst-mcp` | Model Context Protocol integration. |
| **Docker** | `Dockerfile`, `docker-compose.yml` | Containerized MCP service. |

## Bundled Calibration Profiles

The cartridge ships with built-in profiles calibrated against empirical datasets:

| Id | Notes |
|----|--------|
| `default` | Baseline heuristic bundle. |
| `uci_d1` | UCI concrete strength corpus (`datasets/dataset_d1.csv`). |
| `zenodo_ndt`, `zenodo_sonreb`, `zenodo_rh` | Mirrors of Zenodo record [14921019](https://zenodo.org/records/14921019). |
| `uhpc`, `selfheal` | Boundary profiles (verification status tests). |
| `highscm` | High-SCM / alternative binder regime. |

*See [`docs/SSOT.json`](docs/SSOT.json) and [`datasets/PROVENANCE.md`](datasets/PROVENANCE.md) for exact dataset provenance.*

## Documentation Reference

- **Constitutive Equations:** [`docs/Constitutive-Equations.md`](docs/Constitutive-Equations.md)
- **Validation:** [`docs/Validation.md`](docs/Validation.md)
- **Wire Schemas:** [`docs/WireSchemas.md`](docs/WireSchemas.md)
- **Formal Status & Anchors:** [`docs/PROOF-STATUS.md`](docs/PROOF-STATUS.md), [`docs/FormalProvenance.md`](docs/FormalProvenance.md)

## Citation

Please refer to [`CITATION.cff`](CITATION.cff) or the repository URL for software citations.

## Contributing & License

Contributing: [`CONTRIBUTING.md`](CONTRIBUTING.md), [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). Security: [`SECURITY.md`](SECURITY.md).
Released under the [MIT License](LICENSE).
*© 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO.*
