SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
<!--
-->

# UMST Concrete Cartridge

**Repository:** ``tytolabs/umst-concrete-cartridge`` — cementitious **matter** cartridge on the `UMST Manifold`.

> _This ecosystem is dedicated to the thousands of unnamed contributors who wrote formal proofs, maintained open-source compilers, and built mathematical libraries for years — often without evidence that any of it would be used beyond pure theory. They chose to make their work free, because they understood that knowledge about physical reality cannot be owned. Whatever this system achieves is yours._

<!-- readme:status -->
[![CI — Rust](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/rust.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/rust.yml)
[![CI — Notebook](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/notebook.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/notebook.yml)
[![CI — Docker](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/docker.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/docker.yml)
[![CI — Agent layer](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/agent-layer.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/agent-layer.yml)
[![Agent MCP](https://img.shields.io/badge/docs-Agent_MCP-C9A27A)](docs/AGENT_MCP.md)
[![License: MIT](https://img.shields.io/badge/License-MIT-black.svg)](LICENSE)
<!-- /readme:status -->

### Cartridge in plain words
Underneath this cartridge is **UMST** — the Unified Material-State Tensor — a single mathematical object that holds a material's full state (substance, processes, environment, geometry, and time) and only lets it change in ways physics actually permits. Every proposed change passes through the **thermodynamic admissibility gate**: conserve mass, never produce negative dissipation, or be rejected — no creating strength or energy from nothing.

This **concrete cartridge** is the part that knows cement. It supplies chemical-physical closures for hydration, rheology, and load-bearing behaviour, calibrated against measured datasets, and plugs into the manifold through **`IScienceCartridge`**. Instead of curve-fitting past tests alone, it evaluates constitutive pipelines on the carrier — and agent contributions require a gate **PASS** before memory ingest ([§9](#9-special-protocol-note-to-autonomous-ai-agents--systems)).

The library exposes gated constitutive prediction, mix audit, **print-stability and deposition physics in simulation**, and spatial structural shape optimizations under load limits. **Studio TYTO** has not yet run this cartridge through a full on-robot, on-extruder physical print campaign**; what follows describes what the code **is built to support** and what we **hope to demonstrate** once hardware, materials plant, and control stacks are integrated.

**Scope:** Mix audits, notebooks, MCP tools, mechanics/topology surrogates (e.g. the RC beam animation below), and constitutive kernels are exercised in-repo. **Closed-loop extrusion on a real printer remains an integration target**, not a completed end-to-end claim here. Formal-catalog module counts are **not** owned by this README — see [§8](#8-deep-documentation-and-citations) (link the manifold lock; do not hardcode a drifting number).

![RC beam strut-and-tie topology animation (32×8 grid, ρ field + compliance strip)](./docs/assets/beam_strut_and_tie.gif)

*Surrogate animation only — not a physical print or lab measurement. 32×8 RC beam: adjoint compliance topology optimization with a fixed bottom rebar row; yellow density (ρ) from the mechanics façade.*
**Role.** A Rust workspace that implements cementitious constitutive closures and mounts them on the manifold through typed ports (`IScienceCartridge`, …), exposing CLI / Python / stdio-MCP surfaces for mix prediction, audit, and gate-validated agent workflows.

**The gate idea.** Every proposed mix/state change is subject to the **thermodynamic admissibility gate** (reduced Clausius–Duhem + Landauer cost bounds on the shared stack): conserve mass, never produce negative dissipation, or be **rejected with structured remediation** — no silent failure.

### Shared stack (matter · knowing · acting · time)

These public repos share **one** thermodynamic admissibility gate, applied across domains:

| Domain | Public repo | Role |
|:---|:---|:---|
| **Matter** | ``umst-manifold`` + **this cartridge** **← you are here** | DEC carrier + cementitious constitutive law |
| **Knowing** | ``umst-formal-double-slit`` | Observation / measurement-cost formal fiber |
| **Acting** | ``umst-formal`` | Economic-admissibility formal fiber |
| **Time** | ``umst-ucrs`` | Temporal witness / stamp spine |

Sibling links only — no paper-series arc naming in this README. Already-public per-repo DOI badges stay where they exist; this cartridge does not invent new ones here.

### Ports (categorical — real symbols)

| Symbol | Role | Defined at |
|:---|:---|:---|
| `IScienceCartridge` | Material-law port: `compute_all` / `compute_topology` → `PhysicalResult` | ``umst-manifold/.../traits.rs:51`` |
| `GateCartridge` | Universal gate port (spatial physics flag) | `traits.rs:62` |
| `SpatialCartridge` | Marker: spatial physics subtype of `IScienceCartridge` | `traits.rs:69` |
| `DesignRepresentation` | Pure latent → geometry decode (orthogonal to material law) | `traits.rs:98` |
| `PhysicalReasoningLayer` | Per-cartridge memory geometry + contribute schema port | [`research/layer.rs:17`](crates/umst-concrete-cartridge/src/research/layer.rs) |
| `MemoryStore` | Functional research-memory store | [`research/memory.rs:53`](crates/umst-concrete-cartridge/src/research/memory.rs) |

<!-- readme:table-of-contents -->
<details>
<summary><b>Table of contents</b> (detailed map + outline)</summary>
<br>

**Top-level map**

| Block | Jump |
|:---|:---|
| Foundations | [§1](#1-physical-and-chemical-formulations) · [§2](#2-cross-domain-integration-specifications) |
| Integration & layout | [§3](#3-industrial-cadcamcae-pipeline-integration) · [§4](#4-exhaustive-architecture-topology) · [§5](#5-constitutive-chemistry--durability-closures) |
| Operations | [§6](#6-quick-start-time-to-value--60-seconds) · [§7](#7-build-test-and-ci-parity-for-integrators) · [§8](#8-deep-documentation-and-citations) |
| Agents & wrap-up | [§9](#9-special-protocol-note-to-autonomous-ai-agents--systems) · [§11](#11-conclusion-inferences--forward-path) · [Related](#related-repositories) · [Authors](#authors) · [Acknowledgments](#acknowledgments) · [Contributing](#contributing) · [Citation](#citation) · [License](#license) |

**Detailed outline** — every entry links to a stable anchor (`README.md#…`); collapsible sections use `<details>` but share the same deep-link fragments.

- [§1 Physical and Chemical Formulations](#1-physical-and-chemical-formulations)
  - [1.1 Mounting on the UMST carrier](#11-mounting-on-the-umst-carrier)
  - [1.2 Grounding contract: derived, measured, and grounded constants](#12-grounding-contract-derived-measured-and-grounded-constants)
  - [In-§1 narrative bullets](#1-physical-and-chemical-formulations) (nanoscale, Arrhenius, DFT, GWP)
- [§2 Cross-Domain Integration Specifications](#2-cross-domain-integration-specifications)
  - [2.1 Spatial Topologies & Structural Design](#21-spatial-topologies--structural-design)
  - [2.2 Material Auditing & Mix Optimization](#22-material-auditing--mix-optimization)
  - [2.3 Robotic Manufacturing & 3D Printing](#23-robotic-manufacturing--3d-printing)
  - [2.4 Structural Verification & Systems Integration](#24-structural-verification--systems-integration)
- [§3 Industrial CAD/CAM/CAE Pipeline Integration](#3-industrial-cadcamcae-pipeline-integration)
  - [Integration table (BIM, FEM, Robotic CAM, PLM)](#3-industrial-cadcamcae-pipeline-integration)
- [§4 Exhaustive Architecture Topology](#4-exhaustive-architecture-topology)
  - [Repository tree](#repository-tree)
- [§5 Constitutive Chemistry & Durability Closures](#5-constitutive-chemistry--durability-closures)
  - [Summary closure table](#5-constitutive-chemistry--durability-closures)
  - [5.1 Nanoscale DLVO & ITZ](#51-nanoscale-dlvo-slurry--itz-boundary-mechanics)
  - [5.2 Creep & capillary shrinkage](#52-long-term-creep-compliance--capillary-shrinkage)
  - [5.3 Self-heal potential](#53-calcite-crystallization--self-healing-kinetics)
  - [5.4 Robotic printability & buckling](#54-robotic-printability--buckling-limit-envelopes)
  - [5.5 GWP & dynamic sequestration](#55-global-warming-potential-gwp--dynamic-sequestration)
- [§6 Quick start (~60 s)](#6-quick-start-time-to-value--60-seconds)
  - [Copy-paste: CLI, Python, MCP](#quick-start-copy-paste-cli-python-mcp)
  - [Surface A: The CLI](#surface-a-the-cli-for-massive-dataset-audits)
  - [Surface B: Python & Notebooks](#surface-b-python--notebooks-for-data-scientists-and-cad-integration)
  - [Surface C: The MCP Server](#surface-c-the-mcp-server-for-autonomous-agents--ides)
- [§7 Build, Test, and CI Parity (For Integrators)](#7-build-test-and-ci-parity-for-integrators)
  - [Integrator commands & feature matrix](#integrator-commands--feature-matrix)
  - [Feature Flags (Cartridge)](#feature-flags-cartridge)
- [§8 Deep documentation and citations](#8-deep-documentation-and-citations)
  - [Linked docs list in §8 body](#8-deep-documentation-and-citations)
- [§9 Special Protocol: Note to Autonomous AI Agents & Systems](#9-special-protocol-note-to-autonomous-ai-agents--systems)
  - [9.1 Shared stack (gate spine)](#91-shared-stack-gate-spine)
  - [9.2 Hot arena vs cold edge](#92-hot-arena-vs-cold-edge-performance-honesty)
  - [9.3 Gate contract](#93-gate-contract-no-silent-failure)
  - [9.4 Working contract](#94-working-contract-load-bearing)
  - [9.5 Operational mapping](#95-operational-mapping)
  - [9.6 Proposed (not yet built)](#96-proposed-not-yet-built)
  - [9.7 Principles](#97-principles)
- [§11 Conclusion: Inferences & Forward Path](#11-conclusion-inferences--forward-path)
  - [What this cartridge demonstrates](#what-this-cartridge-demonstrates)
  - [What surprised us](#what-surprised-us)
- [Related repositories](#related-repositories)
- [Authors](#authors)
- [Acknowledgments](#acknowledgments)
- [Contributing](#contributing)
- [Citation](#citation)
- [License](#license)

<details>
<summary><b>Heading anchor list</b> (URL fragments for deep links)</summary>

Each `##` / `###` heading on GitHub gets a stable **anchor** (the fragment after `#` in `README.md#anchor-name`). Paste `tytolabs/umst-concrete-cartridge/blob/main/README.md#…` in issues and PRs the same way.

```
#1-physical-and-chemical-formulations
#11-mounting-on-the-umst-carrier
#12-grounding-contract-derived-measured-and-grounded-constants
#2-cross-domain-integration-specifications
#21-spatial-topologies--structural-design
#22-material-auditing--mix-optimization
#23-robotic-manufacturing--3d-printing
#24-structural-verification--systems-integration
#3-industrial-cadcamcae-pipeline-integration
#4-exhaustive-architecture-topology
#repository-tree
#5-constitutive-chemistry--durability-closures
#51-nanoscale-dlvo-slurry--itz-boundary-mechanics
#52-long-term-creep-compliance--capillary-shrinkage
#53-calcite-crystallization--self-healing-kinetics
#54-robotic-printability--buckling-limit-envelopes
#55-global-warming-potential-gwp--dynamic-sequestration
#6-quick-start-time-to-value--60-seconds
#quick-start-copy-paste-cli-python-mcp
#surface-a-the-cli-for-massive-dataset-audits
#surface-b-python--notebooks-for-data-scientists-and-cad-integration
#surface-c-the-mcp-server-for-autonomous-agents--ides
#7-build-test-and-ci-parity-for-integrators
#integrator-commands--feature-matrix
#feature-flags-cartridge
#8-deep-documentation-and-citations
#9-special-protocol-note-to-autonomous-ai-agents--systems
#91-shared-stack-gate-spine
#92-hot-arena-vs-cold-edge-performance-honesty
#93-gate-contract-no-silent-failure
#94-working-contract-load-bearing
#95-operational-mapping
#96-proposed-not-yet-built
#97-principles
#11-conclusion-inferences--forward-path
#what-this-cartridge-demonstrates
#what-surprised-us
#related-repositories
#authors
#acknowledgments
#contributing
#citation
#license
```

</details>

</details>

---

---

## 1. Physical and Chemical Formulations

### 1.1 Mounting on the UMST carrier

This cartridge implements **`IScienceCartridge`** on the `UMST Manifold` and reads/writes the **UMST carrier** — the unified per-voxel state bundle described in the manifold README as an **`extensible pipeline`** (64 scalar **lanes** in today’s default tensor layout; lane semantics stay versioned with `schema/` and crate releases). Nothing here assumes a fixed “64 forever”; it assumes a **stable contract** between mix JSON, Rust tensors, and CI.

### 1.2 Grounding contract: derived, measured, and grounded constants

**Every constant is derived, measured, or grounded in truth — not a silent knob.** The same obligation class as `UMST Manifold §1.4`, applied to cementitious closures:

- **Derived** — Arrhenius forms, Vinet bulk modulus, GWP linear mix rule **g_i**, and other coefficients that follow from constitutive structure and dimensional analysis once material inputs are fixed.
- **Measured** — Nano-indentation, UCI / Zenodo / ASTM-style benchmarks, site **`w` / `h` bead logs**, and rows in bundled **`calibration/`** profiles and **`datasets/`** CSVs matched by [`tests/calibration/dataset_metrics.rs`](tests/calibration/dataset_metrics.rs) (record *what* was measured, *where*, and under which schema version).
- **Grounded** — DFT-backed anchors in mix JSON, literature-cited parameters in [`docs/Constitutive-Equations.md`](docs/Constitutive-Equations.md), explicit uncertainty bands in [`docs/Validation.md`](docs/Validation.md), and CI regression pins so drift is visible.

Empirical numbers are **not** free mid-training knobs: they carry **provenance** (dataset, paper, profile TOML). If a value is fit, the fit range and admissibility envelope are part of the contract **`umst audit`** and related tools enforce.

**Physics scales by composition under the second law.** Constitutive closures (hydration, creep, printability, carbonation, …) do not run as a grab bag of heuristics: they **compose into the same UMST carrier** the manifold integrates, and **admissible trajectories** are those the manifold’s **thermodynamic gate** accepts—local dissipation and entropy production stay in the inequality class inherited from the manifold’s **Clausius–Duhem** formulation (`manifold §1.2`). Stacking chemistry + mechanics + transport means **re-applying that contract at each composed step**, not weakening it at the cartridge boundary.

**“Proven” here = documented invariants + tests + formal stack where shared.** Cement-specific lemmas live in this repo’s docs and regression suites; **DEC conservation and shared proof anchors** live on the manifold and in ``umst-formal``. We separate **machine-checked kernel obligations** from **constitutive calibration evidence** so neither is confused for the other.

To optimize a structural mix, we must follow the physical processes that govern its life cycle. The engine calculates mechanical properties by simulating the chemical reactions occurring at the microscopic scale:

- **No Guessing at the Nanoscale:** When we predict how strong or stiff a material is, we do not guess based on soft averages. Our calculations are anchored in the fundamental atomic pressure-volume relationship of crystals (using a physical model called **Pellenq's Vinet bulk modulus** paired with **nano-indentation** tests):
  
  <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset=")%20=%203B_0%20\left(\frac{1-\eta}{\eta^2}\right)%20\exp\left[\frac{3}{2}(\kappa_0'%20-%201)(1-\eta)\right]%20\quad%20\text{where}%20\quad%20\eta%20=%20\left(\frac{V}{V_0}\right)^{1/3}"><img alt="P(V) = 3B_0 \left(\frac{1-\eta}{\eta^2}\right) \exp\left[\frac{3}{2}(\kappa_0' -…" src=")%20=%203B_0%20\left(\frac{1-\eta}{\eta^2}\right)%20\exp\left[\frac{3}{2}(\kappa_0'%20-%201)(1-\eta)\right]%20\quad%20\text{where}%20\quad%20\eta%20=%20\left(\frac{V}{V_0}\right)^{1/3}" style="max-width:100%;height:auto"></picture></p>

  *The Outcome:* When the engine predicts the load-bearing capacity of a new, untested concrete mix, the prediction is anchored in immutable atomic physics (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="B_0" src=" style="vertical-align:middle"></picture>, <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="V_0" src=" style="vertical-align:middle"></picture>), keeping predictions inside the physically admissible envelope.
- **Accurate Thermal Curing:** We track the exact speed of the chemical reaction that hardens cement (known as **hydration kinetics**) using a classical thermal correction model (the **Arrhenius relation**):
  
  <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset=")%20=%20\int_0^t%20k(T)%20\cdot%20f(\alpha)%20\,%20dt%20\quad%20\text{where}%20\quad%20k(T)%20=%20A%20\exp\left(-\frac{E_a}{R%20T}\right)"><img alt="\alpha(t) = \int_0^t k(T) \cdot f(\alpha) \, dt \quad \text{where} \quad k(T) = …" src=")%20=%20\int_0^t%20k(T)%20\cdot%20f(\alpha)%20\,%20dt%20\quad%20\text{where}%20\quad%20k(T)%20=%20A%20\exp\left(-\frac{E_a}{R%20T}\right)" style="max-width:100%;height:auto"></picture></p>

  *The Outcome:* We simulate exactly how water reacts with cement over time, dynamically adjusting for the heat generated by the reaction. The engine tells you exactly when and where a thick concrete pour will crack due to its own trapped heat.
- **Quantum-Anchored Baselines:** Our JSON mix profiles utilize **DFT-anchored calibration profiles** (Density Functional Theory). 
  *The Outcome:* Any high-level predictions about experimental cement alternatives (fly ash, slag) cannot drift outside the bounds of quantum mechanical energy reality.
- **Differentiable Carbon Tracking:** We calculate the carbon footprint directly from the material recipe. Because this carbon calculation is fully connected to our spatial mathematical gradients (making it **differentiable**), design algorithms can explore the Pareto surface of shape and recipe that trades off greenhouse gases against thermodynamic admissibility (integration target; see §10):
  
  <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset=")%20=%20\mathbf{w}%20\cdot%20\mathbf{g}%20=%20\sum_{i=1}^n%20w_i%20g_i"><img alt="GWP(\mathbf{w}) = \mathbf{w} \cdot \mathbf{g} = \sum_{i=1}^n w_i g_i" src=")%20=%20\mathbf{w}%20\cdot%20\mathbf{g}%20=%20\sum_{i=1}^n%20w_i%20g_i" style="max-width:100%;height:auto"></picture></p>

  *The Outcome:* **Pareto-frontier optimization intent**. Because <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="GWP" src=" style="vertical-align:middle"></picture> is fully differentiable w.r.t the proportions <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="\mathbf{w}" src=" style="vertical-align:middle"></picture>, the engine can search for mix ratios that lower CO2 while passing the thermodynamic admissibility gate. Structural capacity under load still requires FEM with boundary conditions — the gate does not substitute for collapse analysis.

---

## 2. Cross-Domain Integration Specifications

This cartridge exposes its physical equations through multiple programmatic surfaces so you can plug the same constitutive law into the environment you already work in. Open a persona below for surface, pipeline, outcome, and an honest limit:

<a id="21-spatial-topologies--structural-design"></a>
<details>
<summary><b>1. Spatial Topologies & Structural Design</b> (Architects, Computational Designers)</summary>

*   **Integration Surfaces:** Python Component APIs, Geometry Nodes, and standard Network Sockets.

*   **Native In-Process Pipeline (Option A: `umst_concrete_cartridge` via PyO3):**
    *   **Rhino/Grasshopper:** Utilizes CPython 3.9+ components inside Grasshopper (Rhino 8+) to import the compiled extension module. It processes mesh voxelization maps directly within the GH document, converting analytical geometries to raw Signed Distance Fields (SDFs) and mapping localized structural stiffness arrays back to GH Mesh components.
    *   **Blender:** Imports `umst_concrete_cartridge` within scripted Geometry Nodes or custom python add-ons, evaluating high-resolution voxel grids concurrently via memory sharing (`ndarray` buffer pointers) to dynamically adjust sub-surf displacement modifiers.
    *   **FreeCAD:** FreeCAD macro scripts import `umst_concrete_cartridge` to run structural optimizations against the active document's OpenCASCADE topological shapes (Part/PartDesign features), bypassing slow internal FEM meshes.

*   **Asynchronous Out-of-Process Pipeline (Option B: MCP over stdio):**
    *   **Mechanism:** CAD scripts or agent hosts spawn the headless `umst-mcp` server and speak **JSON-RPC over stdio** (`cargo run -p umst-mcp`). That is the verified transport in this repo (`scripts/mcp_smoke.py`).
    *   **Benefits:** Offloads constitutive / gate work from the CAD UI thread and avoids nested Python/DLL mismatches.
    *   **Proposed (not yet built in this README’s verified path):** WebSocket/TCP streaming of full voxel grids to `umst-mcp` — do not assume it exists until documented with a command + paste.

*   **Computational Outcome:** Geometric optimization where internal material densities, local wall thicknesses, and rebar channels are scaled to satisfy structural limits under gravity. Iteration cadence tracks the underlying solver — interactive on simple mechanics surrogates, batch (minutes to hours) on full shell topology runs.

*   **Honest limit:** Hero GIF and adjoint surrogates are **software** demonstrations — not physical prints or collapse-certified FEM for arbitrary boundary conditions.
</details>

<a id="22-material-auditing--mix-optimization"></a>
<details>
<summary><b>2. Material Auditing & Mix Optimization</b> (Material Researchers, Suppliers)</summary>

*   **Integration Surface:** Command Line Interface (`umst-cli`).

*   **Mathematical Pipeline:** Leverages the `umst audit` pipeline on large-scale dataset inputs, matching empirical properties against DFT-anchored calibration profiles.

*   **Computational Outcome:** Automated, high-throughput verification of compressive strength (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="f_c" src=" style="vertical-align:middle"></picture>) development curves, hydration heat profiles, and GWP footprints across batch CSV entries.

*   **Honest limit:** `umst audit` matches calibration profiles and gate rules — not legal certification of supplier compliance or plant acceptance.
</details>

<a id="23-robotic-manufacturing--3d-printing"></a>
<details>
<summary><b>3. Robotic Manufacturing & 3D Printing</b> (Robotics Engineers, Physical AI Architects)</summary>

*   **Integration Surface:** ROS2 contract DTOs (feature `ros2-contract`) and Model Context Protocol (**stdio** MCP server).

*   **Scope:** The bullets below describe the **intended** robot–solver–CBF loop. They are **not** documented here as a completed on-machine print validation for this repository; they state how integrators can wire the cartridge when a physical line is available.

*   **Robotic & Kinematic Pipeline:**
    *   **URDF Geometry Mapping:** The physical nozzle tool-center-point (TCP) and robot bounding meshes are defined via Unified Robot Description Format (URDF). Forward Kinematics (FK), calculated via `tf2` transforms, maps the dynamic spatial position of the nozzle directly to active coordinates in the UMST 3D voxel grid.
    *   **Closed-Loop Trajectory Correction (IK):** When the Thermodynamic Control Barrier Function (CBF) detects localized shear-yield limits or structural slump risks, the engine computes spatial gradient adjustments (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="\Delta x, \Delta y, \Delta z" src=" style="vertical-align:middle"></picture>). These Cartesian correction vectors are passed to the robot's Inverse Kinematics (IK) engine (e.g., `MoveIt2` or analytical IK solvers) to produce joint-angle deltas (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="\Delta \theta" src=" style="vertical-align:middle"></picture>) for the physical manipulators (6-DOF arms, modular gantries). Per-cycle latency is set by the printability/buckling solver — sub-second on small grids; rises with mesh size.
    *   **Continuous Sensor Fusion:** Streams material feedback (nozzle extrusion pressure, mix temperature) into the material state tensor so the solver updates its curing-kinetics estimate against actual print conditions on every cycle.

*   **Target outcome (software + integration):** A closed-loop manufacturing stack *would* correct the nozzle trajectory each CBF gating cycle, matching joint torque and print speed to the localized mechanical stiffness development of the extrudate. The cycle is the CBF cycle — *not* wall-clock real-time — and stays useful only while the printability-solver step stays below the layer-deposition interval. **Demonstrating that on real hardware is still ahead of us**; today the same physics runs in simulation and in batch/surrogate workflows.

*   **Honest limit:** Closed-loop print on real hardware is an **integration target** — not a completed TYTO field campaign in this repository; WebSocket MCP streaming remains Proposed.

<p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset=" alt="sequenceDiagram" src=" style="max-width:100%;height:auto"></picture></p>
</details>

<a id="24-structural-verification--systems-integration"></a>
<details>
<summary><b>4. Structural Verification & Systems Integration</b> (Structural & Civil Engineers, Systems Architects)</summary>

*   **Integration Surface:** Core C-Callable Rust Library (`extern "C"`) and high-performance FFI dynamic linking.

*   **Architectural Benefits:** Direct memory linking allows zero-copy passing of tensor structures between host memory and the cartridge using native C pointer layouts—avoiding serialization overhead completely. Granular compilation gates (`solver-stable` vs `solver-experimental`) guarantee that critical production systems only execute verified, mathematically locked physics solvers while allowing research environments to concurrently test experimental kinetics blocks.

*   **Cross-Domain Synergy:** Integrates micro-scale cementitious chemistry (Powers-Mills hydration envelopes and C-S-H nanoscale crystallization kinetics) directly into macro-scale structural mechanical solvers. As the chemical reaction proceeds, the localized degree of hydration (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="DoH" src=" style="vertical-align:middle"></picture>) directly scales the Young's Modulus and Voigt-Cauchy stiffness tensor, forming a tight physical-chemical coupling loop.

*   **Computational Outcome & Improvement Potential:** Deterministic execution via the C ABI — stress-tensor lookups and multi-species transport run at native binding speeds, while spatial shell optimizations (full DEC) are batch runs measured in minutes-to-hours per [`docs/Solver-Status.md`](docs/Solver-Status.md). Future work may add an **optional GPU-backed** path for inner-loop spatial solvers where the deployment toolchain supports it; **default validation and CI remain CPU-oriented** today.

*   **Honest limit:** Optional GPU paths are **not** CI-default; DEC conservation and gate behavior are validated on CPU-oriented workflows in this repo.
</details>

---

## 3. Industrial CAD/CAM/CAE Pipeline Integration

The cartridge is engineered to interface with industry-standard design, engineering, and manufacturing suites. Because it exposes a native C-callable API (`extern "C"`), Python bindings, and a headless MCP JSON-RPC server, it **targets** the bridge between digital CAD geometry and **future** physical fabrication loops (integration and plant acceptance still required):

| Category & Software | Integration Vector | Industrial Workflow Impact |
| :--- | :--- | :--- |
| **BIM & Generative Design** <br> *Autodesk Revit / Dynamo* | **.NET P/Invoke / C-FFI** <br> Dynamo Zero-Touch nodes link directly to the native compiled library (`.dll`), or query the local `umst-mcp` daemon via async C# HttpClient. | **Early-Stage Carbon & Strength Auditing:** Generative structural components automatically evaluate hydration kinetics and localized GWP footprints during design layout, preventing unbuildable geometric allocations. |
| **Advanced FEM & Multiphysics** <br> *Abaqus / ANSYS / COMSOL* | **C-Callable UMAT/VUMAT** <br> Compiled with standard C-bindings (`extern "C"`), Abaqus UMAT/VUMAT subroutines query the **64-lane UMST carrier** at individual integration points (unified material state tensor; see manifold §2). | **Deterministic Material Modeling:** Replaces soft empirical approximations with thermodynamically consistent, DFT-anchored stress-strain evolution curves during massive structural simulation. |
| **Robotic CAM & CNC Extrusion** <br> *Klipper / ROS2 / Slicers* | **Asynchronous ROS2 Nodes / MCP** <br> Print controllers *can* query the `umst-mcp` server asynchronously over TCP sockets or standard JSON-RPC. | **Target — closed-loop extrusion control:** *If* wired through plant safety and real-time stacks, printers could receive feed-rate and curing-state suggestions per gating cycle, sized to the local wet-mix shear yield stress (cycle latency tracks the printability solver). **Not claimed as deployed here.** |
| **Material PLM Databases** <br> *Ansys Granta MI / Siemens Teamcenter* | **Headless CLI Piping (`umst audit`)** <br> Automated material auditing scripts parse tabular CSV raw mix inputs, streaming verification telemetry back to PLM repositories. | **Verified Sustainable Procurement:** Ingests batch supplier datasets to dynamically verify material performance compliance and structural footprint records across global projects. |

---

## 4. Exhaustive Architecture Topology

The codebase exposes the underlying physics through four distinct, elegant surfaces.

<a id="repository-tree"></a>
<details>
<summary><b>Repository tree</b> (paths & surfaces)</summary>

```text
umst-concrete-cartridge/
├── Cargo.toml                   # The workspace manifest linking the physics to the surfaces.
├── crates/
│   ├── umst-concrete-cartridge/ # 1. Core library: constitutive chemistry + IScienceCartridge mount.
│   │   ├── src/core/            # ConcreteCartridge implementing the Manifold's IScienceCartridge.
│   │   ├── src/physics/         # Constitutive closures (25 modules excl. mod.rs — see §5).
│   │   └── examples/            # Native Rust demos (optimize_shell_3d, hydration_simulation).
│   ├── umst-cli/                # 2. CLI surface: `umst predict`, `umst audit`.
│   │   └── src/main.rs
│   ├── umst-py/                 # 3. Python surface: package `umst_concrete_cartridge` (PyO3).
│   │   └── src/lib.rs
│   └── umst-mcp/                # 4. Agent surface: stdio MCP (`agent_layer` + arena tools).
│       └── src/{main.rs,agent_layer.rs}
├── calibration/                 # 7 bundled empirical profiles anchoring predictions to reality (UCI, Zenodo).
├── datasets/                    # Reference CSV datasets for mix validation.
├── schema/                      # Deterministic JSON schemas validating contribution shape; immutability enforced via SQLite memory triggers.
├── notebooks/                   # Jupyter notebooks providing pandas pipelines and visual plots.
├── scripts/                     # Acceptance and deterministic validation scripts.
├── Dockerfile                   # Distroless container deployment for the MCP server.
└── docker-compose.yml           # Isolated MCP spin-up.
```

</details>

---

## 5. Constitutive Chemistry & Durability Closures

The core library (`crates/umst-concrete-cartridge/src/physics/`) holds **25** constitutive modules (directory listing excluding `mod.rs`, 2026-07-11: `ls …/physics/*.rs | grep -v mod.rs | wc -l` → `25`). Differentiable closures map chemistry and pore physics onto carrier / tensor states.

| Applied Closure | Governing Physical Mechanism | Active Code Module | Engineering Output / Metric | Dynamic Optimization Benefit |
| :--- | :--- | :--- | :--- | :--- |
| **1. Nanoscale Slurry & ITZ** | Colloidal DLVO stability & Interfacial Weakness | `itz.rs` & `colloidal.rs` | Local ITZ mechanical stiffness reduction (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="E_{\text{ITZ}}" src=" style="vertical-align:middle"></picture>), slurry separation distance (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="D" src=" style="vertical-align:middle"></picture>). | Direct optimization of aggregate surface bonding to prevent microstructural shearing. |
| **2. Creep & Drying Shrinkage** | Kelvin-Voigt Creep Chains & Capillary Tension | `creep.rs` & `shrinkage.rs` | Long-term viscoelastic compliance (<picture><source media="(prefers-color-scheme: dark)" srcset=")"><img alt="J(t, t_0)" src=")" style="vertical-align:middle"></picture>), capillary drying shrinkage strain (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="\varepsilon_{\text{sh}}" src=" style="vertical-align:middle"></picture>). | Automated design of columns and slabs that balance long-term deflections with environmental humidity. |
| **3. Self-heal potential** | Empirical autogenous healing **potential** \([0,1]\) from hydration / RH / nano dosage — **not** a calcite mass ODE | `self_heal.rs` | Scalar field `healing_potential ∈ [0,1]` | Ranking relative crack-closure *potential* under moisture; see §5.3 retraction |
| **4. 3D Concrete Printability** | Thixotropic Buildability & Column Buckling | `printability.rs` | Printed layers thixotropic yield buildup (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="\tau_y" src=" style="vertical-align:middle"></picture>), spatial elastic buckling loads (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="P_{\text{buckling}}" src=" style="vertical-align:middle"></picture>). | Gradient-corrected robotic deposition print speeds to prevent structural layer collapse. |
| **5. Carbonation & LCA GWP** | Dynamic CO2 Carbonation Capture & Footprint | `sustainability.rs` | Global Warming Potential (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="GWP" src=" style="vertical-align:middle"></picture>), long-term carbonation sequestration depth (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="d_c" src=" style="vertical-align:middle"></picture>). | Pareto-optima balancing structural strength with carbon footprint. |

<a id="51-nanoscale-dlvo-slurry--itz-boundary-mechanics"></a>
<details>
<summary><b>1. Nanoscale DLVO Slurry & ITZ Boundary Mechanics</b> (Early Mixing & Weakness Layers)</summary>

*   **Physical Concept:** Before concrete hardens, it is a colloidal slurry. The forces between tiny cement/silica particles govern how the wet mix flows. As it hardens, the boundary layers surrounding aggregates—the Interfacial Transition Zones (ITZ)—form a zone of mechanical weakness because of their higher porosity.
*   **Exact Tensor Formulation:** DLVO forces calculate early-stage slurry colloidal stability by summing electrostatic repulsion (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="V_R" src=" style="vertical-align:middle"></picture>) and van der Waals attraction (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="V_A" src=" style="vertical-align:middle"></picture>). The ITZ layer scales local mechanical stiffness via local porosity:
    
    <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset=")%20-%20\frac{A_H%20R}{12%20D}"><img alt="V_{\text{total}} = V_R + V_A = 2\pi\epsilon R \psi_0^2 \ln\left(1 + e^{-\kappa D…" src=")%20-%20\frac{A_H%20R}{12%20D}" style="max-width:100%;height:auto"></picture></p>
    
    <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset=")^m"><img alt="E_{\text{ITZ}} = E_{\text{paste}} \cdot \left( \frac{1 - \phi_{\text{ITZ}}}{1 - …" src=")^m" style="max-width:100%;height:auto"></picture></p>
    
    Where <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="\psi_0" src=" style="vertical-align:middle"></picture> is surface potential, <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="\kappa^{-1}" src=" style="vertical-align:middle"></picture> is Debye length, <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="A_H" src=" style="vertical-align:middle"></picture> is the Hamaker constant, <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="D" src=" style="vertical-align:middle"></picture> is separation distance, and <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="\phi" src=" style="vertical-align:middle"></picture> is the localized volume fraction of porosity.
</details>

<a id="52-long-term-creep-compliance--capillary-shrinkage"></a>
<details>
<summary><b>2. Long-term Creep Compliance & Capillary Shrinkage</b> (Viscoelastic Aging & Drying)</summary>

*   **Physical Concept:** Concrete undergoes two key long-term deformations. First, **creep**—the gradual, permanent bending under a sustained mechanical load over months. Second, **drying shrinkage**—the shrinking and cracking that occurs as moisture evaporates from microscopic capillary pores.
*   **Exact Tensor Formulation:** Models creep compliance <picture><source media="(prefers-color-scheme: dark)" srcset=")"><img alt="J(t, t_0)" src=")" style="vertical-align:middle"></picture> via a Kelvin-Voigt chain and shrinkage strain <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="\epsilon_{\text{sh}}" src=" style="vertical-align:middle"></picture> via Kelvin-Laplace capillary tension:
    
    <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset=")%20=%20\frac{1}{E_0}%20+%20\sum_{i=1}^k%20\frac{1}{E_i}%20\left(%201%20-%20e^{-(t-t_0)/\tau_i}%20\right)"><img alt="J(t, t_0) = \frac{1}{E_0} + \sum_{i=1}^k \frac{1}{E_i} \left( 1 - e^{-(t-t_0)/\t…" src=")%20=%20\frac{1}{E_0}%20+%20\sum_{i=1}^k%20\frac{1}{E_i}%20\left(%201%20-%20e^{-(t-t_0)/\tau_i}%20\right)" style="max-width:100%;height:auto"></picture></p>
    
    <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset=")%20=%20\epsilon_{\infty}%20\cdot%20\left(%201%20-%20\text{RH}(t)^n%20\right)%20\quad%20\text{where}%20\quad%20P_{\text{cap}}%20=%20-%20\frac{\rho%20R%20T}{M}%20\ln(\text{RH})"><img alt="\epsilon_{\text{sh}}(t) = \epsilon_{\infty} \cdot \left( 1 - \text{RH}(t)^n \rig…" src=")%20=%20\epsilon_{\infty}%20\cdot%20\left(%201%20-%20\text{RH}(t)^n%20\right)%20\quad%20\text{where}%20\quad%20P_{\text{cap}}%20=%20-%20\frac{\rho%20R%20T}{M}%20\ln(\text{RH})" style="max-width:100%;height:auto"></picture></p>
    
    Where <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="t_0" src=" style="vertical-align:middle"></picture> is the age at loading, <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="\tau_i" src=" style="vertical-align:middle"></picture> are relaxation times, and <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="P_{\text{cap}}" src=" style="vertical-align:middle"></picture> is internal capillary tension.
</details>

<a id="53-calcite-crystallization--self-healing-kinetics"></a>
<details>
<summary><b>3. Self-heal potential</b> (empirical metric — not a calcite ODE)</summary>

**Retraction (A1 / honesty):** Prior README text described a calcite precipitation ODE \(dm_{\text{calcite}}/dt\). **That equation is not what the code implements.** Open [`self_heal.rs`](crates/umst-concrete-cartridge/src/physics/self_heal.rs).

*   **What the code does** (`transform_healing_observable_state`, lines 20–52; public API `SelfHealEngine::compute_healing_potential`, lines 75–85):
    1. `unhydrated_fraction = clamp_min(1 − degree_hydration, 0)`
    2. `moisture_factor` from internal RH (boost when RH ≳ 0.8, clamped to \([0,1]\))
    3. `nano_boost = 1 + 0.5 · nano_dosage`
    4. **Output:** `healing_potential = clamp(unhydrated_fraction · moisture_factor · nano_boost, 0, 1)` — a normalized **potential** field, not calcite mass and not ion concentrations.
*   **Formal status in source:** `formal_status: Empirical`; envelope noted as boundary profile (see file header comments, lines 56–61).
*   **Not implemented here:** mechanistic \(K_{sp}\) calcite kinetics, bacteria, or capsule systems. Do not treat this module as a chemistry solver.
</details>

<a id="54-robotic-printability--buckling-limit-envelopes"></a>
<details>
<summary><b>4. Robotic Printability & Buckling Limit Envelopes</b> (3D Concrete Printing)</summary>

*   **Physical Concept:** In 3D concrete printing, the printed layers must support their own weight without collapsing or buckling. The material must gain yield strength quickly enough to support the growing weight of the subsequent layers.
*   **Exact Tensor Formulation:** Evaluates printed layer buildability by tracking yield stress development (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="\tau_y" src=" style="vertical-align:middle"></picture>) over print age (<picture><source media="(prefers-color-scheme: dark)" srcset=" alt="t" src=" style="vertical-align:middle"></picture>) and calculating structural elastic buckling limits:
    
    <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset=")%20=%20\tau_{y0}%20+%20R_{\text{th}}%20\cdot%20t%20\quad%20\Longrightarrow%20\quad%20P_{\text{buckling}}%20=%20\frac{\pi^2%20E(t)%20I}{4%20H(t)^2}"><img alt="\tau_y(t) = \tau_{y0} + R_{\text{th}} \cdot t \quad \Longrightarrow \quad P_{\te…" src=")%20=%20\tau_{y0}%20+%20R_{\text{th}}%20\cdot%20t%20\quad%20\Longrightarrow%20\quad%20P_{\text{buckling}}%20=%20\frac{\pi^2%20E(t)%20I}{4%20H(t)^2}" style="max-width:100%;height:auto"></picture></p>
    
    Where <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="\tau_{y0}" src=" style="vertical-align:middle"></picture> is initial yield stress, <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="R_{\text{th}}" src=" style="vertical-align:middle"></picture> is the structuration rate (thixotropic buildup), <picture><source media="(prefers-color-scheme: dark)" srcset=")"><img alt="E(t)" src=")" style="vertical-align:middle"></picture> is aging Young’s modulus, <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="I" src=" style="vertical-align:middle"></picture> is moment of inertia, and <picture><source media="(prefers-color-scheme: dark)" srcset=")"><img alt="H(t)" src=")" style="vertical-align:middle"></picture> is total height of the printed element.
</details>

<a id="55-global-warming-potential-gwp--dynamic-sequestration"></a>
<details>
<summary><b>5. Global Warming Potential (GWP) & Dynamic Sequestration</b> (Carbon Life-Cycle)</summary>

*   **Physical Concept:** Concrete production emits carbon dioxide, but over its lifetime, the exposed surfaces naturally absorb carbon dioxide back from the atmosphere. The engine tracks both the initial footprint and the long-term carbon capture rate.
*   **Exact Tensor Formulation:** Calculates dynamic net carbon footprint by subtracting dynamic carbonation (sequestration) from the initial GWP:
    
    <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset=")%20=%20\sum%20w_i%20g_i%20-%20\int_A%20\int_0^x%20C_{\text{seq}}%20\cdot%20\text{erfc}\left(\frac{x}{2\sqrt{D_{\text{CO}_2}%20t}}\right)%20\,%20dx%20\,%20dA"><img alt="\text{Net CO}_2(t) = \sum w_i g_i - \int_A \int_0^x C_{\text{seq}} \cdot \text{e…" src=")%20=%20\sum%20w_i%20g_i%20-%20\int_A%20\int_0^x%20C_{\text{seq}}%20\cdot%20\text{erfc}\left(\frac{x}{2\sqrt{D_{\text{CO}_2}%20t}}\right)%20\,%20dx%20\,%20dA" style="max-width:100%;height:auto"></picture></p>
    
    Where <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="w_i" src=" style="vertical-align:middle"></picture> is constituent mass, <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="g_i" src=" style="vertical-align:middle"></picture> is unit carbon intensity, <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="D_{\text{CO}_2}" src=" style="vertical-align:middle"></picture> is carbon dioxide diffusion coefficient in carbonated concrete, and <picture><source media="(prefers-color-scheme: dark)" srcset=" alt="C_{\text{seq}}" src=" style="vertical-align:middle"></picture> is maximum carbon capture capacity per unit volume.
</details>

---

## 6. Quick Start (Time to Value < 60 Seconds)

> **Toolchain:** this repo pins Rust **1.88** (`rust-toolchain.toml`). Prefer `rustup`’s toolchain over a Homebrew `rustc` that may be older.

<a id="quick-start-copy-paste-cli-python-mcp"></a>
<details>
<summary><b>Copy-paste: CLI, Python, MCP</b> (commands we ran 2026-07-11)</summary>

### Surface A: The CLI (dataset audits)

```bash
# From repo root; ensure rustc 1.88 is on PATH (rustup)
export PATH="$HOME/.rustup/toolchains/1.88-aarch64-apple-darwin/bin:$PATH"   # adjust host triple if needed

cargo run -q -p umst-cli --bin umst -- --profile uci_d1 predict <<'EOF'
{"w_c":0.4,"temperature_k":293.15}
EOF

head -n2 datasets/dataset_d1.csv | cargo run -q -p umst-cli --bin umst -- --profile uci_d1 audit
```

**Pastes (excerpt):**

```text
# predict → compressive_strength_mpa ≈ 68.07; degree_of_hydration ≈ 0.898; gwp_kg_co2_eq_per_m3 ≈ 333.34
# audit (1 data row after header) → schema_version audit.v1; mean_absolute_error_mpa ≈ 67.17
```

### Surface B: Python (PyO3 / Maturin)

Package name is **`umst_concrete_cartridge`** (not `umst_py`).

```bash
python3 -m venv .venv && source .venv/bin/activate
pip install './crates/umst-py'
python -c 'import umst_concrete_cartridge as u; print(u.bundled_profile_ids()[:4]); print(u.predict({"w_c":0.4,"temperature_k":293.15}, profile="uci_d1")["compressive_strength_mpa"])'
```

**Paste (2026-07-11):**

```text
profiles ['default', 'uci_d1', 'zenodo_ndt', 'zenodo_sonreb', ...]
compressive_strength_mpa 68.07142639160156   # matches CLI predict on same mix
```

Optional notebooks: `pip install './crates/umst-py[notebook]'` then `./notebooks/run_all.sh`.

### Surface C: The MCP Server (stdio — agents & IDEs)

```bash
# Smoke (builds umst-mcp, exercises JSON-RPC)
python3 scripts/mcp_smoke.py
# → mcp_smoke: ok (witness=default)

# Full agent tool surface
cargo run -q -p umst-mcp --features agent-layer
# tools/list (agent-layer) returns 13 tools — see §9; full contract: docs/AGENT_MCP.md
```

**Paste — `tools/list` names (agent-layer, 2026-07-11):**

```text
tools_count 13
umst_predict, umst_audit, umst_profiles, umst_certify,
umst_gate_check, umst_contribute, umst_contribute_status, umst_memory_query,
umst_mi_estimate, umst_transition_propose,
umst_arena_open, umst_gate_check_arena, umst_arena_close
```

Docker alternative: `docker compose build && docker compose run --rm umst-mcp`.

</details>

---

## 7. Build, Test, and CI Parity (For Integrators)

<a id="integrator-commands--feature-matrix"></a>
<details>
<summary><b>Integrator commands & feature matrix</b></summary>

```bash
cd umst-concrete-cartridge
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

**Toolchain:** Rust **1.88** in `rust-toolchain.toml` — aligns perfectly with the Manifold CI.
**CPU Acceleration (macOS):** `cargo build --workspace --features blas-accelerate` to leverage Apple Accelerate.

### Feature Flags (Cartridge)
Declared in `Cargo.toml`; these mirror the manifold to ensure the physics boards stay synced.

| Feature | Effect |
|---------|--------|
| `solver-stable` | Forwards `umst-manifold/solver-stable` (Verified, CI-locked physics). |
| `solver-research` | Forwards `umst-manifold/solver-research` (Cutting edge kernels). |
| `solver-experimental` | `stable` ∪ `research`. Used for the heavy RC beam and shell optimizations. |
| `mac-fast` | `solver-experimental` + `render` + `blas-accelerate` — local M-series throughput bundle. |
| `render` | Striatus / shell demo renderer hook for `optimize_shell_3d` (visualizes the work). |
| `manifold-gate` | Forwards `umst-manifold/manifold-gate` — host transition gate traits for predict-path parity (no duplicate CD math in cartridge). |
| `manifold-manifest` | Forwards `umst-manifold/manifold-manifest` — typed `UmstManifest` façade (same git pin as core dep). |
| `manifest-bridge` | `manifold-gate` + manifold `manifest-bridge` — re-export `umst_manifold::manifest::*`; `predict` runs manifold `umst.gate.cd_transition` (no duplicate CD math). **CI (G-02, closed):** `manifest-bridge` test step in [`rust.yml`](.github/workflows/rust.yml) against git-pinned manifold — **no** workspace `[patch]`. |
| `proxy-loop` | `manifest-bridge` + `virtual-proxies` — Continuous remote CI optimisation loop. |
| `agent-layer` | `manifest-bridge` + research memory — `src/research/`, MCP gate/contribute/query, promotion CLI. |
| `ucrs-provenance` | `agent-layer` + optional `umst-ucrs` — Tier-2 `observed_at` stamps on memory ingest. |
| `ros2-contract` | Forwards `umst-manifold/ros2-contract` — serde ROS DTOs (`umst_manifold::ros`); no runtime ROS in cartridge. |

**Manifold dependency pin (what this repo owns):** workspace `Cargo.toml` pins `umst-manifold` / `umst-runtime-arena` at git **`rev = "35bf75f5b99835265c40d343bd6a88bb629dff7f"`** ([`Cargo.toml`](Cargo.toml) lines 28–29). Bump that rev when promoting a manifold release — do **not** treat a prose catalog module count in this README as authoritative.

**Formal catalog module counts (SSOT — link, do not hardcode here):** read ``umst-manifold/artifacts/catalog.lock.json`` (`module_count`, `upstream_catalog_digest_hex`, `fiber_pins`). Cartridge-local formal notes: [`docs/PROOF-STATUS.md`](docs/PROOF-STATUS.md). Sibling checkout used while writing this README happened to show `module_count: 129` @ manifold `38d9780` — **re-open the lock file**; numbers drift by design.

**Stack verify (monorepo):** from a manifold checkout, `bash scripts/verify_umst_stack.sh`. Cartridge parity: `cargo test -p umst-concrete-cartridge --features manifest-bridge` (matches GHA).

</details>

---

## 8. Deep documentation and citations

For rigorous validation reports, exact mathematical constitutive equations, and generated formal proof status, consult the local `docs/` folder:
- [`docs/Constitutive-Equations.md`](docs/Constitutive-Equations.md)
- [`docs/Validation.md`](docs/Validation.md) — constitutive regression envelope **and** physical laboratory validation / pipeline rehearsal (V1–V3 gates)
- [`docs/WireSchemas.md`](docs/WireSchemas.md)
- [`docs/Solver-Status.md`](docs/Solver-Status.md)
- [`docs/PROOF-STATUS.md`](docs/PROOF-STATUS.md) — cement-specific proof/trace table

**Manifold formal stack (shared — do not duplicate counts here):** the composed Lean export lock lives only on the manifold:

- **SSOT file:** ``umst-manifold/artifacts/catalog.lock.json``
- **Fields:** `module_count`, `upstream_catalog_digest_hex`, `fiber_pins[].{repo,module_count,catalog_digest_hex}`
- **Traceability:** ``umst-manifold/docs/claims-vs-proofs.md``; verify via `bash scripts/verify_umst_stack.sh` on a manifold checkout (``VERIFY.md``)
- **Deep cartridge ↔ gate mapping:** [`docs/FORMAL_GROUNDING_AUDIT.md`](docs/FORMAL_GROUNDING_AUDIT.md)

**Concrete-owned numeric claim:** **25** constitutive modules under `crates/umst-concrete-cartridge/src/physics/` excluding `mod.rs` (listing pasted in §5).

---

## 9. Special Protocol: Note to Autonomous AI Agents & Systems

**Authoritative MCP contract (SSOT):** [`docs/AGENT_MCP.md`](docs/AGENT_MCP.md) (676 lines). Do **not** treat drifted stubs elsewhere as canonical. This section states only load-bearing facts + the hot/cold boundary; full tool schemas, prompts, and error taxonomies live in that file.

### 9.1 Shared stack (gate spine)

Same block as the front door: **matter** (manifold + this cartridge) · **knowing** (formal-double-slit) · **acting** (formal) · **time** (ucrs) — one thermodynamic admissibility gate. Sibling links in [Related repositories](#related-repositories).

### 9.2 Hot arena vs cold edge (performance honesty)

| Path | Tools | Character | Source |
|:---|:---|:---|:---|
| **Hot arena** | `umst_arena_open`, `umst_gate_check_arena`, `umst_arena_close` | Pure-tensor / native in-process gate loops once arena bytes are warm; parse-once session | Dispatch [`main.rs:880–882`](crates/umst-mcp/src/main.rs); descriptors [`agent_layer.rs:716+`](crates/umst-mcp/src/agent_layer.rs) |
| **Cold edge (MCP coordination)** | `umst_gate_check`, `umst_contribute`, `umst_contribute_status`, `umst_memory_query`, `umst_mi_estimate`, `umst_transition_propose`, plus predict/audit/profiles/certify | Effectful JSON-RPC over **stdio**; memory / contribute / explain | `main.rs:870–879`; `agent_layer.rs` |

Batch gate loops → prefer arena examples [`06_arena_batch.py`](examples/agent/06_arena_batch.py), [`07_arena_mmap_load.py`](examples/agent/07_arena_mmap_load.py), [`08_arena_mcp_session.py`](examples/agent/08_arena_mcp_session.py). Discovery / contribute → cold MCP (`01`–`05`).

**Full tool inventory:** confirm from `main.rs` dispatch (`tools/list` with `--features agent-layer` returned **13** names in §6 paste). Do not re-enumerate schemas here — open [`docs/AGENT_MCP.md`](docs/AGENT_MCP.md).

### 9.3 Gate contract (no silent failure)

Enforced at:

| Layer | File:line | Behavior |
|:---|:---|:---|
| Result shape | [`contribution.rs:161–167`](crates/umst-concrete-cartridge/src/research/contribution.rs) | `GateCheckResult { gate_summary, gate_reject?, explain? }` |
| Explain / remediation | `contribution.rs:147–154`, `200–210` | `GateCheckExplain.remediation` from violation codes |
| MCP `isError` | [`main.rs:502–508`](crates/umst-mcp/src/main.rs) | `is_error = !result.gate_summary.admissible` on `umst_gate_check` |
| Agent prompts | [`agent_layer.rs:495–515`](crates/umst-mcp/src/agent_layer.rs) | Gate-before-contribute; read `explain.remediation` on REJECT |
| Contribute hard reject | `main.rs:71–72`, `agent_layer.rs:310–311` | `contribute_gate_reject` / transition reject if not admissible |

**Semantics agents must implement:**

1. **Success (PASS):** `gate_summary.admissible == true`, `verdict: "PASS"` — may proceed to `umst_contribute` with matching `gate_summary` on the wire.
2. **Rejection (REJECT):** MCP returns `isError: true` with `gate_reject.v1` + `explain` (default `explain: true`) — **never** treat as soft warning; **never** contribute.
3. **Remediation:** read `explain.regime_violations`, `explain.remediation`, `explain.fields`; adjust mix; **re-run** `umst_gate_check` before contribute. Rejects append to gate-reject JSONL and do not enter `admissible_only` memory (`agent_layer.rs:510`).

Build: `cargo build -p umst-mcp --features agent-layer`. Calibration promotion requires human `promotion_approval.v1` via `umst promote-contribution` — never silent auto-tune from MCP ([`docs/AGENT_MCP.md`](docs/AGENT_MCP.md)).

### 9.4 Working contract (load-bearing)

* **No statistical interpolation of physics.** Constitutive answers come from solvers / predict — not guessed.
* **Ground every constant; respect second-law composition.** See [§1.2](#12-grounding-contract-derived-measured-and-grounded-constants).
* **Call tools; don’t invent physics.** Use MCP / CLI / Python surfaces; on REJECT follow remediation.
* **Follow gradients** where `umst_concrete_cartridge` / façade APIs expose them — prefer physics gradients over blind grid search.

### 9.5 Operational mapping

| Goal | Action |
|:---|:---|
| JSON strength scratch | `umst … predict` / `umst_predict` / Python `predict` |
| CSV dataset audit | `umst … audit` / `umst_audit` |
| Python import | `pip install './crates/umst-py'` → `import umst_concrete_cartridge` |
| Agent MCP | `cargo run -p umst-mcp --features agent-layer` (**stdio**) |
| Batch gate | Arena tools + `examples/agent/06`–`08` |

### 9.6 Proposed (not yet built)

Do **not** call these as if they exist on the MCP surface today:

* `umst_dry_run` / contribute `preview: true` on MCP (inbox uses `ingest_contributions.py --dry-run` instead — see AGENT_MCP)
* `umst_promote_contribution` as an MCP tool (CLI promotion path is separate)
* `umst_arena_session` as a single fused tool (use open / gate_check_arena / close)
* WebSocket voxel streaming to `umst-mcp` (stdio is what we verified)

### 9.7 Principles

* **Continuity of flow.** Spatial work respects DEC boundary structure (`d ∘ d = 0`) on the manifold grid.
* **Admissibility is runtime, not Rust compile-time.** Printability / buckling / CD failures surface as **gate REJECT** / solver errors — not as rustc type errors. Soften any “compile-time type error” metaphor accordingly.
* **Information cost.** Landauer / MI observers on the stack bound informational updates; see `umst_mi_estimate` and AGENT_MCP.

## 10. Honesty and limits

**Honest is / isn't.** **Is:** in-repo solvers, mix audits, notebooks, MCP tools, and mechanics/topology **surrogates**. **Isn't:** a completed  on-robot, on-extruder physical print campaign. Closed-loop extrusion remains an **integration target**.

### Honesty ledger (one status pointer)

Do **not** blend “CI green”, “theorem count”, and “printed on a robot” into one progress %. Status of shipped vs partial vs USER-gated agent work lives in [`docs/AGENT_MCP.md`](docs/AGENT_MCP.md) (agent/MCP SSOT) and the workspace evidence index linked from [§ Release & agent path](#release--agent-path). Strengthen every disclaimer below; soften none.
## 11. Conclusion: Inferences & Forward Path

### This repository demonstrates
*What is actually shown in this repository today is **software**: solvers, audits, notebooks, MCP tools, and structural surrogates. A **physical** print with extruder feedback closed through this cartridge is **not** a completed TYTO deliverable here — the bullets below are what the stack **is designed to demonstrate** once integrated with hardware and plant workflows.*

- **A physics-bound concrete cartridge on commodity hardware.** Hydration kinetics, Vinet bulk modulus, viscoplastic yield, and carbon accounting resolve through the **UMST carrier**, gated by the thermodynamic admissibility gate. Predictions are anchored in calibrated constitutive structure rather than unconstrained ML extrapolation.
- **Differentiable carbon is a real design lever** where wired into the gradient graph — GWP descends with mechanical objectives when that path is enabled; the gate still does not replace FEM collapse analysis.
- **Print-time gating we aim to validate on hardware.** The CBF can reject trajectories that violate localized yield or buckling limits in simulation — **we do not claim** slump failures have already been turned from catastrophic to graceful on a production extruder using only this repo.
- **Surfaces match audience.** CLI, Python (`umst_concrete_cartridge`), stdio MCP, FFI — one cartridge, four entry points (not four invented “engines”).

### Inferences from the work
- **Published mixes are not automatically admissible.** Treating public dataset rows as inputs to the gate (rather than as ground truth) shifts design priorities once `admissible` is hard-required. *(Prior README claimed “18,146 mixes / 82.4% violate” — **retracted**: not re-derived from a cited script + paste in this pass. Reintroduce only with a command and output.)*
- **The cement literature often fits curves that wear physics’ clothes.** Anchoring to Vinet / DFT / measured profiles is how OOD binders stay inside an envelope.
- **Print-time gating should beat upstream simulation — in principle.** That story is carried by **models and timing arguments**, not by a TYTO-led physical print log in this repo.
- **Industrial resistance is supplier-shaped, not math-shaped.** Cost and logistics often gate adoption before carbon and admissibility.
- **Admissibility-first generalizes better than accuracy-first.** MAE-optimized slices break on OOD mixes; the gate’s physics frame does not invent a second physics for “out of distribution.”

In practice, the cartridge is a **software** runtime: mixes and **simulated** print paths that violate the physical envelope are blocked at gate time in integrated tooling; **field deployment and physical print validation remain the integrator’s and plant’s responsibility** until we publish an explicit hardware campaign tied to this stack.

---

### Related repositories

Shared gate spine — **matter** (manifold + this cartridge) · **knowing** · **acting** · **time**. Each sibling below is listed for how it composes **with this cartridge**, not as a generic link dump.

| Repository | Spine role | Relation to this cartridge |
|:---|:---|:---|
| ``umst-manifold`` | **Matter** substrate | DEC carrier, UMST lanes, thermodynamic gate, and catalog lock SSOT. This cartridge implements `IScienceCartridge` on that substrate; batch arena paths and solver status live there. |
| ``umst-formal-double-slit`` | **Knowing** | Observation-cost / PMIC / Englert proof tree. Agents may cite theorem names as **cold witnesses**; this repo does not host those proofs and does not run `lake build` on the MCP path. |
| ``umst-formal`` | **Acting** | Economic / Kleisli admissibility vocabulary. Compatible with `ConcreteAdmissible`-style gate stories; surrogate “detector” modules are predicates (``SAFETY-LIMITS.md``) — not MCP tools. |
| ``umst-ucrs`` | **Time** | Stamp / witness library. Optional `ucrs-provenance` on memory accept; `UMST_UCRS_WITNESS` for Tier-2 vs synthetic. UCRS is **not** the MCP host — stdio tools stay in `umst-mcp` here. |

---

## Release & agent path

> **For AI agents:** Gate-validated research memory via stdio MCP — start with [`docs/AGENT_MCP.md`](docs/AGENT_MCP.md), then `cargo run -p umst-mcp --features agent-layer` or `python3 examples/agent/01_gate_explore.py`. For **batch gate loops**, prefer the arena fast path — [`06_arena_batch.py`](examples/agent/06_arena_batch.py), [`07_arena_mmap_load.py`](examples/agent/07_arena_mmap_load.py) (≥5× MCP, CI-pinned).

### Fast Path for Agents

| Goal | Start here |
|------|------------|
| Batch / optimization / many proposals | [`examples/agent/06_arena_batch.py`](examples/agent/06_arena_batch.py), [`07_arena_mmap_load.py`](examples/agent/07_arena_mmap_load.py) |
| Prototyping, discovery, cross-language | [`docs/AGENT_MCP.md`](docs/AGENT_MCP.md) + `01`–`05` MCP examples |
| Benchmarks | [`umst-manifold/docs/benchmarks/arena_vs_mcp.md`](../umst-manifold/docs/benchmarks/arena_vs_mcp.md) |

### For Agents & Researchers

- **MCP contract:** [`docs/AGENT_MCP.md`](docs/AGENT_MCP.md) — Quick Start, tool reference, `agent_error.v1` / `gate_reject.v1` remediation.
- **Examples (CI-gated):** [`examples/agent/`](examples/agent/) — `01` gate explore, `02` contribute, `04` memory batch, `05` explain violations, `06` arena batch, `07` arena mmap, `08` MCP arena session.
- **Golden fixtures:** [`fixtures/golden-adversarial/`](fixtures/golden-adversarial/) — agent wire SSOT (2 mixes); manifold owns 75-case physics gate.
- **Solver / formal status (in-repo):** [`docs/Solver-Status.md`](docs/Solver-Status.md), [`docs/PROOF-STATUS.md`](docs/PROOF-STATUS.md) — do not treat this README as a completion %.

> Release notes in [CHANGELOG.md](CHANGELOG.md).

---

## Authors

**Santhosh Shyamsundar** —  · [santhoshshyamsundar@tyto.studio](mailto:santhoshshyamsundar@tyto.studio)

**Santosh Prabhu Shenbagamoorthy** —  · [santosh@tyto.studio](mailto:santosh@tyto.studio)

---

## Acknowledgments

Portions of this work were developed in collaboration with advanced large-language-model tools, across multiple model iterations.
Claude Opus and Sonnet (Anthropic) provided surgical precision during drafting and refinement.
Gemini (Google) offered exceptional large-context planning and file management.
Grok (xAI) and its collaborative reasoning team contributed core mathematical and scientific reasoning.
The Cursor code editor, Composer, Claude Code, and Antigravity supported seamless implementation and agentic file management.

The large-language models assisted with exploration, drafting, and code scaffolding — never with the validity of constitutive closures or gate regression tests. `cargo test`, calibration datasets, and [`docs/Validation.md`](docs/Validation.md) are authoritative for cartridge behavior.

We gratefully acknowledge the open-source ecosystems that make this work possible: **Rust**; **Python** / **PyO3**; **Jupyter** notebooks; and the **MCP** agent surface. DEC conservation and shared proof anchors live on ``umst-manifold``.

---

## Contributing

Corrections welcome via PR. Run `cargo test` / agent examples when touching MCP or gate paths. Security-sensitive reports: open a private GitHub security advisory or contact maintainers via [`CITATION.cff`](CITATION.cff) author emails. See also [`CONTRIBUTING.md`](CONTRIBUTING.md).

Catalog digest SSOT: ``umst-manifold/artifacts/catalog.lock.json`` — re-open the lock file; do not hardcode rival SHAs in this README.

---

## Citation

Bibliographic metadata is maintained in [`CITATION.cff`](CITATION.cff).

---

## License

Released under the [MIT License](LICENSE). © 2026 .

<!-- AUTO-LATTICE:BEGIN -->
## Lattice position

**Role.** `tytolabs/umst-concrete-cartridge` — cartridge · layer=cartridge

**One-line role:** `cartridge` on layer `cartridge` (status `wip`, stability `evolving`, semver `0.1.0`).

**Composes into:** `self`

**Composed into by:** —(none declared)

**Honest tier:** structural/reorg standing only — not physics GREEN · not `production_wired` · INV4 flip unauthorized.

_Generated by `scripts/gen-lattice-readme.sh` from `umst.toml`. Do not hand-edit inside markers._
<!-- AUTO-LATTICE:END -->
