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

The hero GIF is a **32×8** RC beam surrogate: **adjoint compliance** topology optimization with a fixed bottom **rebar** row, rendered to `notebooks/_artifacts/beam_strut_and_tie.gif` (viridis **ρ**, compliance strip, captions). It is meant for **structural engineers and architects** reading **strut-and-tie–like** load paths in the density field—same UMST/MaOS mechanics façade as the rest of the cartridge, not a separate production solver UI.

<p align="center">
  <img src="notebooks/_artifacts/beam_strut_and_tie.gif" alt="RC beam strut-and-tie topology animation (32×8 grid, ρ field + compliance strip)" width="960" />
</p>
<p align="center"><sub>Run <code>bash notebooks/_run_beam_demo.sh</code> (defaults: 90 iterations, dump stride 3; override <code>UMST_BEAM_ITERS</code> / <code>UMST_BEAM_DUMP_STRIDE</code>). GIF pacing: <code>UMST_BEAM_GIF_FRAME_MS</code> (default 200), <code>UMST_BEAM_GIF_HOLD_MS</code> / <code>UMST_BEAM_GIF_HOLD_FRAMES</code> for the final frame. On GitHub or before the first run, the image path may be missing—the broken icon is expected until the script writes the file.</sub></p>

**Shell / Striatus artefacts (honesty):** this repo’s `notebooks/_artifacts/` may contain **v0.3-only** checked-in names (e.g. `striatus_shell_v0.3.obj`, `striatus_shell_v0.3.print_ready.json`) while **v0.4** target filenames from the brief — `striatus_emergence.gif`, `striatus_shell_v0.4.stl`, `striatus_shell_v0.4.print_ready.json`, optional `striatus_shell_v0.4.obj` — are **not** claimed here until they exist on disk and pass Track L / B6 gates. See **[`docs/Solver-Status.md`](docs/Solver-Status.md) → DEFERRAL — Topology / shell (Tracks B + L)** and, when this repo sits beside `composer_prompts/` in the parent workspace, **[`../composer_prompts/v0.4_phase_3_followup_for_composer.md`](../composer_prompts/v0.4_phase_3_followup_for_composer.md)** (Ring 1). When regenerating, use the command block in that follow-up doc; design narrative lives in [`docs/Striatus.md`](docs/Striatus.md). **Do not** embed or assert a shipped hero GIF/STL until `notebooks/_artifacts/` contains those target files.

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

**Toolchain:** Rust **1.88** is pinned at [`rust-toolchain.toml`](rust-toolchain.toml); use `rustup default 1.88` or `cargo +1.88 …` for CI parity. Workspace metadata declares **1.75** as `rust-version` (floor); upgrade rustc if Cargo reports MSRV violations.

**CPU matmul (macOS):** optional `cargo build --workspace --features blas-accelerate` (crate `blas-accelerate` enables `burn-ndarray` Accelerate linking). Tune **`VECLIB_MAXIMUM_THREADS`** (Accelerate) or **`OPENBLAS_NUM_THREADS`** (OpenBLAS builds elsewhere).

**Solver verification matrix (lanes + benchmark paths):** [`docs/Solver-Status.md`](docs/Solver-Status.md) (upstream detail in `umst-manifold/docs/Solver-Status.md`).

## Cargo features (`umst-concrete-cartridge`)

| Feature | Purpose |
|--------|---------|
| *(default)* | Standard façade and topology path without experimental manifold solvers pulled into the cartridge. |
| `solver-experimental` | Enables `umst-manifold` experimental solver stack (e.g. AT2 damage + coupled THMC stubs) inside `ConcreteCartridge::compute_topology`. |
| `blas-accelerate` | Faster CPU GEMM via Apple Accelerate on macOS (`burn-ndarray`). |

Other workspace crates (`umst-cli`, `umst-mcp`, `umst-py`) consume the library; they do not add separate Cargo feature flags beyond dependency edges.

## Python and notebooks

```bash
cd umst-concrete-cartridge
pip install './crates/umst-py[notebook]'
pip install './crates/umst-py[render,tests]'
# Print-readiness (after `bash notebooks/_run_shell_demo.sh`): pytest notebooks/tests/test_print_ready.py -v
# or: cd crates/umst-py && maturin develop --extras notebook,render,tests
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
- Striatus shell lineage + artefact contract: [`docs/Striatus.md`](docs/Striatus.md), [`docs/References.bib`](docs/References.bib)
- Formal provenance (Lean URIs): [`docs/FormalProvenance.md`](docs/FormalProvenance.md)

Formal lemmas live in [umst-formal](https://github.com/tytolabs/umst-formal).

## Citation

Use [`CITATION.cff`](CITATION.cff) or the repository URL for software citations.

## Contributing · security · license

**Santhosh Shyamsundar** — [santhoshshyamsundar@tyto.studio](mailto:santhoshshyamsundar@tyto.studio) · **Santosh Prabhu Shenbagamoorthy** — [santosh@tyto.studio](mailto:santosh@tyto.studio)

Contributing: [`CONTRIBUTING.md`](CONTRIBUTING.md), [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). Security: [`SECURITY.md`](SECURITY.md).

Released under the [MIT License](LICENSE).

Workspace Rust crates set `publish = false` in `Cargo.toml` to avoid accidental `cargo publish` to crates.io; distribution is via Git tags, Docker, and the `maturin` wheel workflow—not the crates.io registry.
