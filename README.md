<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# UMST Concrete Cartridge

[![CI — Rust](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/rust.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/rust.yml)
[![Notebook](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/notebook.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/notebook.yml)
[![Docker](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/docker.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/docker.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-black.svg)](LICENSE)

> *"The universe does not care about your startup's pitch deck; entropy always wins. We built an AI that understands this. It does not guess. It computes the inevitable."*

**UMST Concrete Cartridge** is the applied specialization of the [UMST Manifold](https://github.com/tytolabs/umst-manifold). It provides the differentiable constitutive equations, empirical calibration, and deployment surfaces for cementitious materials. 

This is not a speculative LLM wrapper. This is a thermodynamically-gated, gradient-based design engine that can optimize a concrete mix's carbon footprint, assess the buildability of a 3D-printed structure, and evolve the topology of a load-bearing shell—all while strictly obeying the laws of physics.

<p align="center">
  <img src="docs/assets/beam_strut_and_tie.gif" alt="RC beam strut-and-tie topology animation (32×8 grid, ρ field + compliance strip)" width="960" />
</p>

*32×8 RC beam surrogate: adjoint compliance topology optimization with a fixed bottom rebar row; viridis density ρ, compliance strip, captions — rendered to `notebooks/_artifacts/beam_strut_and_tie.gif` via the same mechanics façade as the rest of the cartridge.*

## The Physical Truth Behind the Code

We anchor every claim in mathematical and physical truth.
- **No Guessing at the Nanoscale:** Our Young's moduli are derived from Pellenq's 2009 Vinet bulk modulus and Ulm & Constantinides nano-indentation.
- **Accurate Kinetics:** We use Powers/Mills hydration kinetics with Arrhenius temperature corrections. The gradients flow directly through the hydration tensor w.r.t mix fractions.
- **Sustainability as a Computable Metric:** We compute Global Warming Potential (GWP) via differentiable tensor dot products against unit cost factors, enabling true Pareto-frontier optimization for carbon-negative concrete.

## Interoperability: We Play Nice

We do not force you into a proprietary GUI. The intelligence here acts as a "headless" engine designed to integrate flawlessly into your existing, industry-standard workflows.

- **Architects & Designers (Rhino/Grasshopper, FreeCAD, Blender):** Use our Python bindings (`umst-py`) to inject exact hydration and strength physics directly into your geometry nodes or SDF/F-rep workflows.
- **Autonomous Agents & Robotics:** Connect your multi-agent systems directly to the engine via the **Model Context Protocol (MCP)** server. Your agent can query the physical stability of an extrusion path in real-time.
- **Material Scientists:** Use the `umst` CLI to audit massive CSV datasets of mix designs against our DFT-anchored calibration profiles.

## Exhaustive Architecture Topology

The codebase exposes the underlying physics through four distinct, elegant surfaces.

```text
umst-concrete-cartridge/
├── Cargo.toml
├── crates/
│   ├── umst-concrete-cartridge/ # 1. The Core Rust Library
│   │   ├── src/core/            # ConcreteCartridge implementing IScienceCartridge
│   │   ├── src/physics/         # 26 constitutive closures (hydration, strength, optical, cost, sustainability)
│   │   └── examples/            # Native Rust demos (optimize_shell_3d, hydration_simulation)
│   ├── umst-cli/                # 2. The Bash/Scripting Surface
│   │   └── src/main.rs          # Binaries for `umst predict`, `umst audit`, `umst certify`
│   ├── umst-py/                 # 3. The Data Science Surface (Python/Jupyter)
│   │   └── src/lib.rs           # PyO3 bindings exposing the exact Rust physics
│   └── umst-mcp/                # 4. The Agentic Surface (Model Context Protocol)
│       └── src/main.rs          # JSON-RPC server exposing tools to Cursor/Claude
├── calibration/                 # 7 bundled empirical profiles (UCI, Zenodo, etc.)
├── datasets/                    # Ground-truth CSV files for auditing and repro
├── schema/                      # Deterministic JSON schemas for mix and result contracts
├── notebooks/                   # Jupyter notebooks bridging Python with rendering scripts
├── scripts/                     # Acceptance and deterministic validation scripts
├── Dockerfile                   # Distroless container deployment for the MCP server
└── docker-compose.yml           # Instant MCP spin-up
```

Root **`Dockerfile`** / **`docker-compose.yml`** package the MCP service for container deployment.

## Grounded examples (mix, Pareto, sampling)

Each bullet ties a command to a **checked-in artifact** or **in-repo documentation** (no extra benchmark claims beyond those files).

- **Mix inputs → strength JSON (`uci_d1`):** `echo '{"w_c":0.4,"temperature_k":293.15}' | umst --profile uci_d1 predict` (install via `cargo install --path crates/umst-cli`). Pair with [`schema/mix.v1.json`](schema/mix.v1.json) and regime keys in [`docs/Calibration.md`](docs/Calibration.md) (generated by `calibration_report`; see [`docs/CLI.md`](docs/CLI.md)).
- **Row-wise audit / sampling-style CSV passes:** `head -n2 datasets/dataset_d1.csv | umst --profile uci_d1 audit` on [`datasets/dataset_d1.csv`](datasets/dataset_d1.csv) (UCI slice); Zenodo mirrors `dataset_d2.csv`–`dataset_d4.csv` per [`datasets/PROVENANCE.md`](datasets/PROVENANCE.md).
- **Headline calibration trade space (tabular, not a frontier plot):** after running profiles locally, compare against the committed snapshot [`results/canonical/table_per_dataset_metrics.csv`](results/canonical/table_per_dataset_metrics.csv) and the column definitions in [`results/canonical/README.md`](results/canonical/README.md). Regenerate the Markdown twin with `cargo run -p umst-cli -q --bin calibration_report > docs/Calibration.md` (see canonical README).
- **Notebook mix audit (pandas + plots):** `pip install './crates/umst-py[notebook]'` then `./notebooks/run_all.sh` — primary notebook [`notebooks/sustainable_mix_audit.ipynb`](notebooks/sustainable_mix_audit.ipynb); operator notes in [`notebooks/README.md`](notebooks/README.md).
- **Differentiable economic / multi-objective hook (code-level “Pareto” cost):** [`crates/umst-concrete-cartridge/src/physics/cost.rs`](crates/umst-concrete-cartridge/src/physics/cost.rs) (`compute_cost`: tensor dot against unit cost factors for gradient-based mix steps). Tabular calibration summaries that accompany mix stepping live in [`results/canonical/table_per_dataset_metrics.csv`](results/canonical/table_per_dataset_metrics.csv) and [`results/canonical/README.md`](results/canonical/README.md).
- **Topology vs compliance (structural surrogate, animated artifact):** `cargo run --release -p umst-concrete-cartridge --example optimize_rc_beam --features solver-experimental` then render with [`notebooks/render_beam_gif.py`](notebooks/render_beam_gif.py) per [`notebooks/README.md`](notebooks/README.md); the committed reference loop is [`docs/assets/beam_strut_and_tie.gif`](docs/assets/beam_strut_and_tie.gif). Shell/Striatus smoke with small grids: same notebook README (`UMST_SHELL_NX=6` … `optimize_shell_3d`).
- **Hydration forward path (constitutive integration without heavy topology):** `cargo run -p umst-concrete-cartridge --example hydration_simulation`.
- **Agents / CI parity:** `python3 scripts/mcp_smoke.py`, `cargo run -p umst-mcp` — [`crates/umst-mcp/README.md`](crates/umst-mcp/README.md); workspace tests `cargo test --workspace` (see [`.github/workflows/rust.yml`](.github/workflows/rust.yml)). Editable Python: `cd crates/umst-py && maturin develop --release --extras notebook`.

## Surfaces & entrypoints

| Surface | Best for | Copy-paste | Prerequisites |
|--------|----------|------------|-----------------|
| **Rust library** (`umst-concrete-cartridge`) | Striatus hooks, constitutive + manifold composition in Rust | `cargo build -p umst-concrete-cartridge` / `cargo test -p umst-concrete-cartridge` | **Rust 1.88** ([`rust-toolchain.toml`](rust-toolchain.toml)); pulls [`umst-manifold`](https://github.com/tytolabs/umst-manifold) from git `main`. |
| **Cargo examples** | End-to-end demos | `cargo run -p umst-concrete-cartridge --example hydration_simulation` (default); `… optimize_rc_beam --features solver-experimental`; `… optimize_shell_3d --features 'solver-experimental render'` | Examples that need **`solver-experimental`** / **`render`** are declared in [`crates/umst-concrete-cartridge/Cargo.toml`](crates/umst-concrete-cartridge/Cargo.toml). |
| **CLI** (`umst`, `umst-canonical`) | Scripting, CI contracts, canonical JSON | `cargo install --path crates/umst-cli` then `umst --help` | Same Rust toolchain; bins defined in [`crates/umst-cli/Cargo.toml`](crates/umst-cli/Cargo.toml). |
| **Python** (`umst_concrete_cartridge`) | Notebooks, pandas pipelines, pytest | `pip install './crates/umst-py[notebook]'` or `cd crates/umst-py && maturin develop --extras notebook` | Python **≥ 3.10** ([`crates/umst-py/pyproject.toml`](crates/umst-py/pyproject.toml)); optional **`[render]`** / **`[tests]`** extras for shell tooling / `pytest`. |
| **Docker** | MCP image / compose deploy | `docker compose build` then `docker compose run --rm umst-mcp` | Docker engine; [`Dockerfile`](Dockerfile) builds `umst-mcp` release binary; [`docker-compose.yml`](docker-compose.yml) mounts `./calibration` read-only. |
| **MCP** | IDE and agent integrations over stdio | `cargo run -p umst-mcp` (host) or compose command above | JSON-RPC lines on stdin/stdout; tool list in [`crates/umst-mcp/README.md`](crates/umst-mcp/README.md). |

**Manifold-only:** differentiable substrate, solver lanes, and `IScienceCartridge` host APIs live in **[`umst-manifold`](https://github.com/tytolabs/umst-manifold)** — no PyO3/MCP there. This repo owns the concrete cartridge façade plus CLI/Python/MCP/Docker packaging.

## Choose your path

- **Library author (Rust):** Work in `crates/umst-concrete-cartridge`; align manifold features with [`docs/Solver-Status.md`](docs/Solver-Status.md) and the upstream [manifold solver table](https://github.com/tytolabs/umst-manifold/blob/main/docs/Solver-Status.md).
- **Application engineer:** Prefer **`umst` CLI** for quick JSON/CSV contracts, then **`umst-py`** when you need notebooks or Python tests.
- **Researcher / repro:** [`notebooks/README.md`](notebooks/README.md) for headless notebook runs and shell/RC artefact gates; [`docs/Validation.md`](docs/Validation.md) and [`docs/Constitutive-Equations.md`](docs/Constitutive-Equations.md) for science narrative.
- **Integrator / agent host:** Run **`umst-mcp`** via `cargo run -p umst-mcp` locally or **`docker compose`** for the same binary in a distroless image; point clients at the stdio transport documented in [`crates/umst-mcp/README.md`](crates/umst-mcp/README.md).

## For agents

- **Repo root:** `umst-concrete-cartridge/` checkout — run `cargo`, `docker compose`, and `pip` paths relative to this directory unless a sub-crate README specifies `crates/umst-py`.
- **Read first:** [`README.md`](README.md), [`crates/umst-mcp/README.md`](crates/umst-mcp/README.md), [`scripts/mcp_smoke.py`](scripts/mcp_smoke.py), [`notebooks/README.md`](notebooks/README.md), [`docs/Solver-Status.md`](docs/Solver-Status.md), [`.github/workflows/rust.yml`](.github/workflows/rust.yml), [`.github/workflows/docker.yml`](.github/workflows/docker.yml), [`.github/workflows/notebook.yml`](.github/workflows/notebook.yml).
- **Safe, no-GPU defaults:** `cargo test --workspace`, `python3 scripts/mcp_smoke.py`, `cargo run -p umst-concrete-cartridge --example hydration_simulation`, `echo '{"w_c":0.4,"temperature_k":293.15}' | umst --profile uci_d1 predict` (with CLI installed), `pip install './crates/umst-py[notebook]'` + `./notebooks/run_all.sh`, `docker compose build`.
- **If you want X → run Y:** JSON strength scratch → `umst … predict`; CSV sanity → `umst … audit`; Python import smoke → `pip install './crates/umst-py[notebook]'` then `python3 -c "import umst_concrete_cartridge"`; MCP tool calls → `cargo run -p umst-mcp` and drive `umst_predict` / `umst_audit` per [`crates/umst-mcp/README.md`](crates/umst-mcp/README.md); solver-heavy concrete demos → add `--features solver-experimental` (and `render` for `optimize_shell_3d`).
- **Before editing:** read [`docs/WireSchemas.md`](docs/WireSchemas.md) / [`docs/Solver-Status.md`](docs/Solver-Status.md) before changing `schema/`, calibration JSON, or manifold feature forwards; run `cargo clippy --workspace --all-targets --features solver-experimental,render -- -D warnings` to mirror Linux CI.

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
