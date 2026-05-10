SPDX-License-Identifier: MIT  
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO.

<div align="center">

# UMST Concrete Cartridge

### Differentiable constitutive engine for cementitious materials — CLI, Python, MCP, Docker

[![CI — Rust](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/rust.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/rust.yml)
[![Notebook](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/notebook.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/notebook.yml)
[![Docker](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/docker.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/docker.yml)

[![License: MIT](https://img.shields.io/badge/License-MIT-black.svg)](LICENSE)
[![Rust 2021](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)
[![Python 3.10+](https://img.shields.io/badge/python-≥3.10-blue.svg)](https://www.python.org/)

Pure functional **`burn`** tensors · **22** coupled modules · calibrated **profiles** · mechanised lemmas where noted in [`docs/PROOF-STATUS.md`](docs/PROOF-STATUS.md)

*bundled calibration rows (**17 646** total rows per [`docs/SSOT.json`](docs/SSOT.json)) · **26** Rust symbols tracked as **`Mechanised`** ([`docs/PROOF-STATUS.md`](docs/PROOF-STATUS.md))*

*The scientific cartridge for the [UMST manifold](https://github.com/tytolabs/umst-manifold): mix-design scalars in, constitutive tensor bundle out.*

</div>

---

## Manifesto

We ship **one** audited prediction path (`umst_concrete_cartridge::facade`) reused by **`umst`**, PyO3, and MCP/Docker wrappers. Mixed-regime homogeneous routing, optional CSV audit (**`audit.v1`**), deterministic **`result.v2`** JSON with **`calibration_profile`**, **`formal_anchor`**, sorted **`axioms`**, **`physics_pipeline`** summary, and explicit **`warnings`**. Formal statements live in **`umst-formal`**; this crate surfaces traceable **`lean://…`** anchors on the wire.

---

## Who this repository is for

### Engineers integrating mix design tooling

Prefer **`umst predict`** stdin JSON or MCP **`umst_predict`** — same façade as Rust with stable **`canonical_json`** / **`umst-canonical`** semantics for agent-safe byte equality checks.

### Researchers reproducing calibrated envelopes

Eight bundled **`calibration/profiles/*.v1.toml`** packs reference copy-of-record CSVs under **`datasets/`** (see **Dataset provenance** below).

### Teams running notebooks and dashboards

Install the bindings with **`pip install ./crates/umst-py[notebook]`** (pulls in **`pandas`**, **`matplotlib`**, and Jupyter) for **`audit_dataframe`** exploration; regenerate outputs with **`./notebooks/run_all.sh`** (**`--strict`** in CI requires **`jupyter`** on **`PATH`**).

### ML / optimisation loops

Expose gradients downstream of façade predictions only where numeric stability allows; façade errors such as **`OutsideAllRegimes`** become explicit skips in property tests (see **`crates/umst-py/tests/test_property_determinism.py`**).

### Operators packaging containers

Docker / compose stubs live at repo root; MCP stdio harness: **`scripts/mcp_smoke.py`**.

---

## Architecture

```mermaid
flowchart TD
    Surfaces["CLI  ·  Python (PyO3)  ·  MCP stdio  ·  Docker"]
    Facade[["umst_concrete_cartridge :: facade"]]
    Profile[("calibration profile · TOML")]
    Regime{{"regime check"}}
    Predict["Mills hydration → Powers gel-space → safety_margin"]
    Wire[/"result.v2 / audit.v1<br/>calibration_profile · formal_anchor · axioms · warnings"/]
    Canon[["umst-canonical · byte-stable JSON"]]
    Formal["umst-formal · Lean 4<br/>Powers.lean · RegimeSoundness.lean · MeasurementCost.lean"]
    Axiom(["physicalSecondLaw"])
    Data["UCI Yeh 1998 · Zenodo 14921019<br/>17,646 measurement rows · CC-BY"]

    Surfaces --> Facade
    Facade --> Profile
    Profile --> Regime
    Regime -->|in-regime| Predict
    Regime -.->|out-of-regime| Wire
    Predict --> Wire
    Wire --> Canon
    Wire -.->|lean:// URI| Formal
    Formal --> Axiom
    Profile -.->|fitted against| Data

    classDef surf fill:#0a2540,stroke:#5b9bd5,stroke-width:2px,color:#e1f5fe
    classDef facade fill:#0f2a44,stroke:#16e0bd,stroke-width:2px,color:#a7f3d0
    classDef prof fill:#3d2e1a,stroke:#f59e0b,stroke-width:2px,color:#fef3c7
    classDef pred fill:#2d1b69,stroke:#bb86fc,stroke-width:2px,color:#e9d5ff
    classDef out fill:#3d1a3a,stroke:#ec4899,stroke-width:2px,color:#fce7f3
    classDef formal fill:#1f2937,stroke:#a78bfa,stroke-width:2px,color:#e9d5ff
    classDef data fill:#0f3d3d,stroke:#22d3ee,stroke-width:2px,color:#cffafe

    class Surfaces surf
    class Facade,Canon facade
    class Profile prof
    class Regime,Predict pred
    class Wire out
    class Formal,Axiom formal
    class Data data
```

The diagram summarises the cartridge in one view. Four surfaces (CLI / Python / MCP / Docker) share a single **`facade`**, so identical mixes yield byte-identical JSON by design, not only by convention. The active calibration profile drives a regime check tied to **`RegimeSoundness.lean`**; in-regime mixes run through Mills hydration kinetics into the Powers gel-space strength model; out-of-regime mixes still return a wire payload but attach explicit **`warnings`** instead of silent extrapolation. Eight bundled profiles (`default`, `uci_d1`, `zenodo_ndt`, `zenodo_sonreb`, `zenodo_rh`, `uhpc` *(Boundary)*, `selfheal` *(Boundary)*, `highscm`) live under **`calibration/profiles/*.v1.toml`**. The twenty-two constitutive modules each carry one formal-status bucket (**Mechanised** / **Structural** / **Empirical** / **Literature** / **NONE**) — see [`docs/PROOF-STATUS.md`](docs/PROOF-STATUS.md). Every **`result.v2`** / **`audit.v1`** response includes **`calibration_profile`**, **`formal_anchor`**, sorted **`axioms`**, and any regime **`warnings`**; proofs and citations chain toward **`umst-formal`** and the **`physicalSecondLaw`** axiom where listed. Calibration rows total **17 646** CC-BY measurements — UCI (Yeh 1998) plus Zenodo Record **14921019** (TU/e + TNO).

| Surface | Entry | Notes |
|---------|-------|-------|
| Rust library | `crates/umst-concrete-cartridge` | Serde façade; **no** `serde_json` / `tokio` / `clap` inside the core default tree (`cargo tree` hygiene in CI). |
| CLI | `umst` binary | **`predict`**, **`audit`**, **`certify`**, **`profiles`**, **`umst-canonical`**. |
| Python | **`maturin develop`** (`crates/umst-py`) | **`predict`**, **`audit`**, **`audit_rows`**, **`audit_dataframe`**, **`certify`**, **`canonical_json`**, **`bundled_profile_ids()`**; **`[notebook]`**, **`[tests]`** extras. |
| MCP · Docker | `umst-mcp` + Dockerfile | **`umst_predict`**, **`umst_audit`**, **`umst_profiles`**, **`umst_certify`**. |

> [!NOTE]
> Canonical JSON (**sorted keys**, Ryū float literals): run **`umst predict`**, pipe stdout into **`target/$PROFILE/umst-canonical`**, and compare to **`bytes(canonical_json(predict(...)))`** (see **`scripts/check_predict_determinism.py`** — same mix must yield identical byte payloads for **`uci_d1`** and **`zenodo_ndt`** on your platform).

---

## Quickstart snippets (captured **`result.v2` / `audit.v1`**)

CLI prediction (**`uci_d1`**, captured from a local **`cargo build -p umst-cli`** run, mix `{"w_c":0.4,"temperature_k":293.15}`):

```json
{
  "schema_version": "result.v2",
  "calibration_profile": "uci_d1",
  "calibration_model": "powers_gel_space",
  "axioms": ["physicalSecondLaw"],
  "formal_anchor": "lean://umst-formal/Lean/Powers.lean#powers_monotone",
  "compressive_strength_mpa": 68.07142639160156,
  "degree_of_hydration": 0.8982649445533752,
  "warnings": [],
  "physics_pipeline": {
    "schema_version": "physics_pipeline.v1",
    "summary": {
      "hydration_alpha": 0.8982649445533752,
      "effective_water_cement_ratio": 0.4000000059604645,
      "strength_jennings_mpa": 68.07142639160156
    }
  }
}
```

Zenodo-aligned profile (**`zenodo_ndt`**, same mix):

```json
{
  "schema_version": "result.v2",
  "calibration_profile": "zenodo_ndt",
  "axioms": ["physicalSecondLaw"],
  "formal_anchor": "lean://umst-formal/Lean/Powers.lean#powers_monotone",
  "compressive_strength_mpa": 43.61223220825195,
  "warnings": []
}
```

CSV audit (**`uci_d1`**, first **`dataset_d1.csv`** header + row piped into **`umst audit`**):

```json
{
  "schema_version": "audit.v1",
  "calibration_profile": "uci_d1",
  "axioms": ["physicalSecondLaw"],
  "rows": [
    {
      "row_index": 0,
      "profile_used": "uci_d1",
      "predicted_strength_mpa": 147.1542510986328,
      "observed_strength_mpa": 79.98611450195312
    }
  ],
  "summary": {
    "row_count": 1,
    "rows_with_observations": 1
  }
}
```

### Minimal commands

```bash
cargo install --path crates/umst-cli
echo '{"w_c":0.4,"temperature_k":293.15}' | umst --profile uci_d1 predict
head -n2 datasets/dataset_d1.csv | umst --profile uci_d1 audit
```

Python (extension built via **`maturin develop --extras notebook`**):

```python
from umst_concrete_cartridge import predict, canonical_json

out = predict({"w_c": 0.4, "temperature_k": 293.15}, profile="uci_d1")
assert out["schema_version"] == "result.v2"
```

---

## Bundled calibration profiles

| Id | Corpus / intent |
|----|----------------|
| **`default`** | Same routing as uplifted homogeneous default bundle. |
| **`uci_d1`** | **[UCI ML concrete](https://doi.org/10.24432/C5PK67)** (**`datasets/dataset_d1.csv`**). |
| **`zenodo_ndt`**, **`zenodo_sonreb`**, **`zenodo_rh`** | TU/e + TNO reproducibility artefacts (**Zenodo record [14921019](https://zenodo.org/records/14921019)**, CC-BY 4.0; file names **`dataset_d2.csv`**, … unchanged). |
| **`uhpc`**, **`selfheal`** | Boundary **`verification_status`** profiles — headline `[acceptance]` gates omitted by design (see **`tests/calibration/dataset_metrics.rs`** skips). |
| **`highscm`** | High SCM fraction regime CSV **`dataset_highscm.csv`**. |

---

## Dataset provenance (row splits)

| Source | Rows (copy-of-record) | Citation |
|--------|-----------------------|----------|
| **UCI** | **`dataset_d1.csv`** (1 030 mix rows + header) | **`10.24432/C5PK67`** |
| **Zenodo 14921019** | **`dataset_d2.csv`**, **`dataset_d3.csv`**, **`dataset_d4.csv`** | TU/e + TNO; **CC-BY 4.0** |
| Others | **`dataset_highscm.csv`**, **`dataset_selfheal.csv`**, … | Listed per profile **`[provenance]`** |

---

## Formal layer & schemas

Formal anchors **`lean://umst-formal/…`** point at mechanised lemmas in the companion **`umst-formal`** repository ([**github.com/tytolabs/umst-formal**](https://github.com/tytolabs/umst-formal)). Bucket counts (**26 Mechanised / 33 Structural / …**) are regenerated into [**`docs/PROOF-STATUS.md`**](docs/PROOF-STATUS.md).

Human guides:

- **[`docs/FormalProvenance.md`](docs/FormalProvenance.md)** — from URI to **`Lean/*.lean`**
- **[`docs/WireSchemas.md`](docs/WireSchemas.md)** — **`mix.v1`**, **`result.v{1,2}`**, **`audit.v1`**, embedded **`physics_pipeline.v1`**

---

## Coupled physics modules (22)

| Module lanes | Highlights |
|----------------|-------------|
| Hydration & chemistry | Jennings CM-II, Powers–Brownyard pathways |
| Microstructure | Nano C-S-H, colloidal DLVO/YODEL, packing, ITZ |
| Rheology & printability | Château–Ovarlez–Trung, Roussel buildability |
| Durability | Transport, chloride diffusivity, freeze–thaw, shrinkage, creep |

Full equations → [**`docs/Constitutive-Equations.md`**](docs/Constitutive-Equations.md) · benchmarks → [**`docs/Validation.md`**](docs/Validation.md).

---

## Development parity (Rust CI mirrors)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --verbose
cargo test --workspace --verbose
```

Hypothesis determinism (**requires** **`pip install -e 'crates/umst-py[tests]'`** or **`maturin develop --extras tests`**):

```bash
python3 -m unittest discover -s crates/umst-py/tests -p 'test_property_determinism.py' -v
```

---

## Roadmap (conservative **`v0.3`**)

- Tighter regime routing diagnostics on multi-profile overlays.
- Optional WASM façade experiments (blocked on backend choice).
- Continued **`umst-formal`** parity as new lemmas land.

---

## Citation

Prefer **[`CITATION.cff`](CITATION.cff)** for GitHub/Git-Zenodo bots.

```bibtex
@software{umst_concrete_2026,
  author       = {Shyamsundar, Santhosh and Shenbagamoorthy, Santosh Prabhu},
  title        = {UMST Concrete Cartridge: a differentiable constitutive
                  engine for cementitious materials},
  year         = 2026,
  url          = {https://github.com/tytolabs/umst-concrete-cartridge}
}
```

---

## Authors · governance

**Santhosh Shyamsundar** — Studio TYTO; IAAC · [santhoshshyamsundar@tyto.studio](mailto:santhoshshyamsundar@tyto.studio)  
**Santosh Prabhu Shenbagamoorthy** — Studio TYTO · [santosh@tyto.studio](mailto:santosh@tyto.studio)

Issues and PRs: [**`CONTRIBUTING.md`**](CONTRIBUTING.md) · [**`CODE_OF_CONDUCT.md`**](CODE_OF_CONDUCT.md) · security contact via [**`SECURITY.md`**](SECURITY.md) (**do not** file public security issues).

---

Released under [**MIT**](LICENSE).

<div align="center">
<sub><a href="https://github.com/tytolabs">github.com/tytolabs</a></sub>
</div>
