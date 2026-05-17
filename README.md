<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# UMST Concrete Cartridge: The Applied Intelligence

[![CI — Rust](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/rust.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/rust.yml)
[![Notebook](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/notebook.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/notebook.yml)
[![Docker](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/docker.yml/badge.svg)](https://github.com/tytolabs/umst-concrete-cartridge/actions/workflows/docker.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-black.svg)](LICENSE)

> *When water meets cement, nanoscale crystals grow, heat is released, moisture moves through microscopic pores, and the liquid hardens into a load-bearing structure. If the temperature or chemistry is off, the material cracks. The cartridge does not regress this from past test data; it simulates the chemical reactions and stresses directly.*

**UMST Concrete Cartridge** is the applied physical brain of the [UMST Manifold](https://github.com/tytolabs/umst-manifold). It provides the specific chemical-physical equations, real-world data calibration, and programming connections designed specifically for cement and concrete materials. 

The library exposes a physical-chemical design engine—gated by thermodynamic safety boundaries—to optimize concrete recipes, check print stability states in robotic manufacturing, and execute spatial structural shape optimizations under strict load limits.

<p align="center">
  <img src="docs/assets/beam_strut_and_tie.gif" alt="RC beam strut-and-tie topology animation (32×8 grid, ρ field + compliance strip)" width="960" />
</p>

*32×8 RC beam surrogate: adjoint compliance topology optimization with a fixed bottom rebar row. The yellow density (ρ) shows exactly where the engine placed material, guided entirely by mechanical force gradients—rendered via the mechanics façade.*

---

## 1. Physical and Chemical Formulations

To optimize a structural mix, we must follow the physical processes that govern its life cycle. The engine calculates mechanical properties by simulating the chemical reactions occurring at the microscopic scale:

- **No Guessing at the Nanoscale:** When we predict how strong or stiff a material is, we do not guess based on soft averages. Our calculations are anchored in the fundamental atomic pressure-volume relationship of crystals (using a physical model called **Pellenq's Vinet bulk modulus** paired with **nano-indentation** tests):
  
  <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_black%5Ccolor%7Bwhite%7D&space;P(V)%20=%203B_0%20\left(\frac{1-\eta}{\eta^2}\right)%20\exp\left[\frac{3}{2}(\kappa_0'%20-%201)(1-\eta)\right]%20\quad%20\text{where}%20\quad%20\eta%20=%20\left(\frac{V}{V_0}\right)^{1/3}"><img alt="P(V) = 3B_0 \left(\frac{1-\eta}{\eta^2}\right) \exp\left[\frac{3}{2}(\kappa_0' -…" src="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_white&space;P(V)%20=%203B_0%20\left(\frac{1-\eta}{\eta^2}\right)%20\exp\left[\frac{3}{2}(\kappa_0'%20-%201)(1-\eta)\right]%20\quad%20\text{where}%20\quad%20\eta%20=%20\left(\frac{V}{V_0}\right)^{1/3}"></picture></p>

  *The Outcome:* When the engine predicts the load-bearing capacity of a new, untested concrete mix, the prediction is anchored in immutable atomic physics (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;B_0"><img alt="B_0" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;B_0" style="vertical-align:middle"></picture>, <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;V_0"><img alt="V_0" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;V_0" style="vertical-align:middle"></picture>), keeping predictions inside the physically admissible envelope.
- **Accurate Thermal Curing:** We track the exact speed of the chemical reaction that hardens cement (known as **hydration kinetics**) using a classical thermal correction model (the **Arrhenius relation**):
  
  <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_black%5Ccolor%7Bwhite%7D&space;\alpha(t)%20=%20\int_0^t%20k(T)%20\cdot%20f(\alpha)%20\,%20dt%20\quad%20\text{where}%20\quad%20k(T)%20=%20A%20\exp\left(-\frac{E_a}{R%20T}\right)"><img alt="\alpha(t) = \int_0^t k(T) \cdot f(\alpha) \, dt \quad \text{where} \quad k(T) = …" src="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_white&space;\alpha(t)%20=%20\int_0^t%20k(T)%20\cdot%20f(\alpha)%20\,%20dt%20\quad%20\text{where}%20\quad%20k(T)%20=%20A%20\exp\left(-\frac{E_a}{R%20T}\right)"></picture></p>

  *The Outcome:* We simulate exactly how water reacts with cement over time, dynamically adjusting for the heat generated by the reaction. The engine tells you exactly when and where a thick concrete pour will crack due to its own trapped heat.
- **Quantum-Anchored Baselines:** Our JSON mix profiles utilize **DFT-anchored calibration profiles** (Density Functional Theory). 
  *The Outcome:* Any high-level predictions about experimental cement alternatives (fly ash, slag) cannot drift outside the bounds of quantum mechanical energy reality.
- **Differentiable Carbon Tracking:** We calculate the carbon footprint directly from the material recipe. Because this carbon calculation is fully connected to our spatial mathematical gradients (making it **differentiable**), design algorithms can automatically discover the singular, optimal shape and recipe that minimizes greenhouse gases while guaranteeing the structure will not collapse:
  
  <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_black%5Ccolor%7Bwhite%7D&space;GWP(\mathbf{w})%20=%20\mathbf{w}%20\cdot%20\mathbf{g}%20=%20\sum_{i=1}^n%20w_i%20g_i"><img alt="GWP(\mathbf{w}) = \mathbf{w} \cdot \mathbf{g} = \sum_{i=1}^n w_i g_i" src="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_white&space;GWP(\mathbf{w})%20=%20\mathbf{w}%20\cdot%20\mathbf{g}%20=%20\sum_{i=1}^n%20w_i%20g_i"></picture></p>

  *The Outcome:* True **Pareto-frontier optimization**. Because <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;GWP"><img alt="GWP" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;GWP" style="vertical-align:middle"></picture> is fully differentiable w.r.t the proportions <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\mathbf{w}"><img alt="\mathbf{w}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\mathbf{w}" style="vertical-align:middle"></picture>, the engine finds the singular, mathematically optimal structural shape and mix ratio that minimizes CO2 emissions while guaranteeing the structure will not collapse.

---

## 2. Cross-Domain Integration Specifications

This cartridge exposes its physical equations through multiple programmatic surfaces to seamlessly integrate into your specific development environment:

<details>
<summary><b>1. Spatial Topologies & Structural Design</b> (Architects, Computational Designers)</summary>

*   **Integration Surfaces:** Python Component APIs, Geometry Nodes, and standard Network Sockets.

*   **Native In-Process Pipeline (Option A: `umst-py` via PyO3):**
    *   **Rhino/Grasshopper:** Utilizes CPython 3.9+ components inside Grasshopper (Rhino 8+) to import the compiled `.pyd` or `.so` binary module. It processes mesh voxelization maps directly within the GH document, converting analytical geometries to raw Signed Distance Fields (SDFs) and mapping localized structural stiffness arrays back to GH Mesh components.
    *   **Blender:** Imports `umst_py` directly within scripted Geometry Nodes or custom python add-ons, evaluating high-resolution voxel grids concurrently via memory sharing (`ndarray` buffer pointers) to dynamically adjust sub-surf displacement modifiers.
    *   **FreeCAD:** FreeCAD macro scripts import `umst_py` to run structural optimizations against the active document's OpenCASCADE topological shapes (Part/PartDesign features), bypassing slow internal FEM meshes.

*   **Asynchronous Out-of-Process Pipeline (Option B: MCP over WebSocket / TCP):**
    *   **Mechanism:** Lightweight CAD scripts or custom C# components initiate a local or remote WebSocket/stdio connection to the headless `umst-mcp` server. 
    *   **Benefits:** Offloads compute-heavy physical solver calculations (hydration thermodynamic kinetics and Voigt-Cauchy stress tensors) completely from the CAD package's main UI thread. This prevents viewport freezing, scales to cloud compute instances, and bypasses nested Python environment/DLL dependency version mismatches.
    *   **Execution:** JSON-RPC messages stream the structural voxel arrays to `umst-mcp`, returning continuous gradient vectors and density updates directly back into CAD parameters.

*   **Computational Outcome:** Real-time geometric optimization where internal material densities, local wall thicknesses, and rebar channels are dynamically scaled to satisfy structural limits under gravity.
</details>

<details>
<summary><b>2. Material Auditing & Mix Optimization</b> (Material Researchers, Suppliers)</summary>

*   **Integration Surface:** Command Line Interface (`umst-cli`).

*   **Mathematical Pipeline:** Leverages the `umst audit` pipeline on large-scale dataset inputs, matching empirical properties against DFT-anchored calibration profiles.

*   **Computational Outcome:** Automated, high-throughput verification of compressive strength (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;f_c"><img alt="f_c" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;f_c" style="vertical-align:middle"></picture>) development curves, hydration heat profiles, and GWP footprints across batch CSV entries.
</details>

<details>
<summary><b>3. Robotic Manufacturing & 3D Printing</b> (Robotics Engineers, Physical AI Architects)</summary>

*   **Integration Surface:** ROS2 Nodes (C++/Python) and Model Context Protocol (MCP) WebSocket Server.

*   **Robotic & Kinematic Pipeline:**
    *   **URDF Geometry Mapping:** The physical nozzle tool-center-point (TCP) and robot bounding meshes are defined via Unified Robot Description Format (URDF). Forward Kinematics (FK), calculated via `tf2` transforms, maps the dynamic spatial position of the nozzle directly to active coordinates in the UMST 3D voxel grid.
    *   **Closed-Loop Trajectory Correction (IK):** When the Thermodynamic Control Barrier Function (CBF) detects localized shear yield limits or structural slump risks, the engine computes spatial gradient adjustments (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\Delta%20x,%20\Delta%20y,%20\Delta%20z"><img alt="\Delta x, \Delta y, \Delta z" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\Delta%20x,%20\Delta%20y,%20\Delta%20z" style="vertical-align:middle"></picture>). These Cartesian correction vectors are passed directly to the robot's Inverse Kinematics (IK) engine (e.g., `MoveIt2` or analytical IK solvers) to compute real-time joint-angle modifications (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\Delta%20\theta"><img alt="\Delta \theta" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\Delta%20\theta" style="vertical-align:middle"></picture>) on the physical manipulators (6-DOF arms, modular gantries).
    *   **Real-time Sensor Fusion:** Streams material feedback (nozzle extrusion pressure, mix temperature) into the material state tensor, allowing the solver to dynamically predict curing kinetics based on actual print speeds.

*   **Computational Outcome:** Resilient, closed-loop physical manufacturing. The robot adapts its Cartesian trajectory dynamically in real-time, matching joint torque limits and print speeds to the localized mechanical stiffness development of the extrudate.

<p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://mermaid.ink/svg/eyJjb2RlIjoic2VxdWVuY2VEaWFncmFtXG4gICAgYXV0b251bWJlclxuICAgIHBhcnRpY2lwYW50IE5venpsZSBhcyBOb3p6bGUgKFRDUClcbiAgICBwYXJ0aWNpcGFudCBTZW5zb3JzIGFzIFNlbnNvcnMgKFAsIFQpXG4gICAgcGFydGljaXBhbnQgQ2FydHJpZGdlIGFzIENhcnRyaWRnZVxuICAgIHBhcnRpY2lwYW50IFNvbHZlciBhcyBQcmludGFiaWxpdHkgU29sdmVyXG4gICAgcGFydGljaXBhbnQgQ0JGIGFzIFRoZXJtbyBDQkZcbiAgICBwYXJ0aWNpcGFudCBJSyBhcyBNb3ZlSXQyIElLXG4gICAgcGFydGljaXBhbnQgSm9pbnQgYXMgUm9ib3QgSm9pbnRzXG5cbiAgICBOb3p6bGUtPj5TZW5zb3JzOiBTdHJlYW0gZXh0cnVzaW9uIHByZXNzdXJlICYgdGVtcGVyYXR1cmVcbiAgICBTZW5zb3JzLT4-Q2FydHJpZGdlOiBGZWVkIHNlbnNvcnMgdG8gM0QgVm94ZWwgR3JpZFxuICAgIENhcnRyaWRnZS0-PlNvbHZlcjogVXBkYXRlIGxvY2FsaXplZCBzdGlmZm5lc3MgJiBhZ2UgcGFyYW1ldGVyc1xuICAgIFNvbHZlci0-PkNCRjogQ2FsY3VsYXRlIHRoaXhvdHJvcGljIHlpZWxkICYgc2x1bXAgcmlzayBsaW1pdHNcbiAgICBhbHQgTGltaXQgRXhjZWVkZWQgKFNsdW1wL0J1Y2tsaW5nIFJpc2spXG4gICAgICAgIENCRi0-PkNhcnRyaWRnZTogQ29tcHV0ZSBzcGF0aWFsIGdyYWRpZW50IGNvcnJlY3Rpb25zIChcdTAzOTR4LCBcdTAzOTR5LCBcdTAzOTR6KVxuICAgICAgICBDYXJ0cmlkZ2UtPj5JSzogU2VuZCBDYXJ0ZXNpYW4gY29ycmVjdGlvbiB2ZWN0b3JcbiAgICAgICAgSUstPj5Kb2ludDogQ29tcHV0ZSAmIGFwcGx5IHJlYWwtdGltZSBqb2ludCBhbmdsZXMgKFx1MDM5NFx1MDNiOClcbiAgICAgICAgSm9pbnQtPj5Ob3p6bGU6IEFkanVzdCBub3p6bGUgc3BlZWQgJiBwb3NpdGlvbiBkeW5hbWljYWxseVxuICAgIGVsc2UgU3RhYmxlIFByaW50IFN0YXRlXG4gICAgICAgIENCRi0-Pk5venpsZTogTWFpbnRhaW4gcGxhbm5lZCBwcmludCB0cmFqZWN0b3J5XG4gICAgZW5kIiwibWVybWFpZCI6IntcInRoZW1lXCI6IFwiZGFya1wifSJ9"><img alt="sequenceDiagram" src="https://mermaid.ink/svg/eyJjb2RlIjoic2VxdWVuY2VEaWFncmFtXG4gICAgYXV0b251bWJlclxuICAgIHBhcnRpY2lwYW50IE5venpsZSBhcyBOb3p6bGUgKFRDUClcbiAgICBwYXJ0aWNpcGFudCBTZW5zb3JzIGFzIFNlbnNvcnMgKFAsIFQpXG4gICAgcGFydGljaXBhbnQgQ2FydHJpZGdlIGFzIENhcnRyaWRnZVxuICAgIHBhcnRpY2lwYW50IFNvbHZlciBhcyBQcmludGFiaWxpdHkgU29sdmVyXG4gICAgcGFydGljaXBhbnQgQ0JGIGFzIFRoZXJtbyBDQkZcbiAgICBwYXJ0aWNpcGFudCBJSyBhcyBNb3ZlSXQyIElLXG4gICAgcGFydGljaXBhbnQgSm9pbnQgYXMgUm9ib3QgSm9pbnRzXG5cbiAgICBOb3p6bGUtPj5TZW5zb3JzOiBTdHJlYW0gZXh0cnVzaW9uIHByZXNzdXJlICYgdGVtcGVyYXR1cmVcbiAgICBTZW5zb3JzLT4-Q2FydHJpZGdlOiBGZWVkIHNlbnNvcnMgdG8gM0QgVm94ZWwgR3JpZFxuICAgIENhcnRyaWRnZS0-PlNvbHZlcjogVXBkYXRlIGxvY2FsaXplZCBzdGlmZm5lc3MgJiBhZ2UgcGFyYW1ldGVyc1xuICAgIFNvbHZlci0-PkNCRjogQ2FsY3VsYXRlIHRoaXhvdHJvcGljIHlpZWxkICYgc2x1bXAgcmlzayBsaW1pdHNcbiAgICBhbHQgTGltaXQgRXhjZWVkZWQgKFNsdW1wL0J1Y2tsaW5nIFJpc2spXG4gICAgICAgIENCRi0-PkNhcnRyaWRnZTogQ29tcHV0ZSBzcGF0aWFsIGdyYWRpZW50IGNvcnJlY3Rpb25zIChcdTAzOTR4LCBcdTAzOTR5LCBcdTAzOTR6KVxuICAgICAgICBDYXJ0cmlkZ2UtPj5JSzogU2VuZCBDYXJ0ZXNpYW4gY29ycmVjdGlvbiB2ZWN0b3JcbiAgICAgICAgSUstPj5Kb2ludDogQ29tcHV0ZSAmIGFwcGx5IHJlYWwtdGltZSBqb2ludCBhbmdsZXMgKFx1MDM5NFx1MDNiOClcbiAgICAgICAgSm9pbnQtPj5Ob3p6bGU6IEFkanVzdCBub3p6bGUgc3BlZWQgJiBwb3NpdGlvbiBkeW5hbWljYWxseVxuICAgIGVsc2UgU3RhYmxlIFByaW50IFN0YXRlXG4gICAgICAgIENCRi0-Pk5venpsZTogTWFpbnRhaW4gcGxhbm5lZCBwcmludCB0cmFqZWN0b3J5XG4gICAgZW5kIiwibWVybWFpZCI6IntcInRoZW1lXCI6IFwiZGVmYXVsdFwifSJ9"></picture></p>
</details>

<details>
<summary><b>4. Structural Verification & Systems Integration</b> (Structural & Civil Engineers, Systems Architects)</summary>

*   **Integration Surface:** Core C-Callable Rust Library (`extern "C"`) and high-performance FFI dynamic linking.

*   **Architectural Benefits:** Direct memory linking allows zero-copy passing of tensor structures between host memory and the cartridge using native C pointer layouts—avoiding serialization overhead completely. Granular compilation gates (`solver-stable` vs `solver-experimental`) guarantee that critical production systems only execute verified, mathematically locked physics solvers while allowing research environments to concurrently test experimental kinetics blocks.

*   **Cross-Domain Synergy:** Integrates micro-scale cementitious chemistry (Powers-Mills hydration envelopes and C-S-H nanoscale crystallization kinetics) directly into macro-scale structural mechanical solvers. As the chemical reaction proceeds, the localized degree of hydration (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;DoH"><img alt="DoH" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;DoH" style="vertical-align:middle"></picture>) directly scales the Young's Modulus and Voigt-Cauchy stiffness tensor, forming a tight physical-chemical coupling loop.

*   **Computational Outcome & Improvement Potential:** Deterministic, low-latency execution of stress tensors, multi-species transport modeling, and spatial shell optimizations at compilation-level execution speeds. Future performance updates target direct transition of inner-loop spatial solvers to parallel GPU compute shaders (`wgpu` feature lane) as compiler-level JIT blocks are patched.
</details>

---

## 3. Industrial CAD/CAM/CAE Pipeline Integration

The cartridge is engineered to interface directly with industry-standard design, engineering, and manufacturing suites. Because it exposes a native C-callable API (`extern "C"`), Python bindings, and a headless MCP JSON-RPC server, it bridges the gap between digital CAD geometry and physical fabrication loops:

| Category & Software | Integration Vector | Industrial Workflow Impact |
| :--- | :--- | :--- |
| **BIM & Generative Design** <br> *Autodesk Revit / Dynamo* | **.NET P/Invoke / C-FFI** <br> Dynamo Zero-Touch nodes link directly to the native compiled library (`.dll`), or query the local `umst-mcp` daemon via async C# HttpClient. | **Early-Stage Carbon & Strength Auditing:** Generative structural components automatically evaluate hydration kinetics and localized GWP footprints during design layout, preventing unbuildable geometric allocations. |
| **Advanced FEM & Multiphysics** <br> *Abaqus / ANSYS / COMSOL* | **C-Callable UMAT/VUMAT** <br> Compiled with standard C-bindings (`extern "C"`), Abaqus UMAT/VUMAT subroutines query the 64-channel Unified Material State Tensor at individual integration points. | **Deterministic Material Modeling:** Replaces soft empirical approximations with thermodynamically consistent, DFT-anchored stress-strain evolution curves during massive structural simulation. |
| **Robotic CAM & CNC Extrusion** <br> *Klipper / ROS2 / Slicers* | **Asynchronous ROS2 Nodes / MCP** <br> Print controllers query the `umst-mcp` server asynchronously over TCP sockets or standard JSON-RPC. | **Closed-Loop Extrusion Control:** Robotic printers adjust travel velocity, extrusion feed rates, and auxiliary curing states dynamically based on the local wet-mix shear yield stress. |
| **Material PLM Databases** <br> *Ansys Granta MI / Siemens Teamcenter* | **Headless CLI Piping (`umst audit`)** <br> Automated material auditing scripts parse tabular CSV raw mix inputs, streaming verification telemetry back to PLM repositories. | **Verified Sustainable Procurement:** Ingests batch supplier datasets to dynamically verify material performance compliance and structural footprint records across global projects. |

---

## 4. Exhaustive Architecture Topology

The codebase exposes the underlying physics through four distinct, elegant surfaces.

```text
umst-concrete-cartridge/
├── Cargo.toml                   # The workspace manifest linking the physics to the surfaces.
├── crates/
│   ├── umst-concrete-cartridge/ # 1. The Core Rust Library: Constitutive chemistry and mechanical models.
│   │   ├── src/core/            # ConcreteCartridge implementing the Manifold's IScienceCartridge.
│   │   ├── src/physics/         # 26 constitutive closures (calculating hydration, strength, cost, GWP).
│   │   └── examples/            # Native Rust demos (optimize_shell_3d, hydration_simulation).
│   ├── umst-cli/                # 2. The Bash Surface: Fast, pipeline-ready tools.
│   │   └── src/main.rs          # Binaries for `umst predict`, `umst audit` (Dataset verification).
│   ├── umst-py/                 # 3. The Data Science Surface: PyO3 bindings for Python and Jupyter.
│   │   └── src/lib.rs           # PyO3 bindings so Jupyter and Blender can access the math.
│   └── umst-mcp/                # 4. The Agentic Surface: Real-time intuition for AI and Robotics.
│       └── src/main.rs          # JSON-RPC server exposing tools directly to Cursor, Claude, or ROS.
├── calibration/                 # 7 bundled empirical profiles anchoring predictions to reality (UCI, Zenodo).
├── datasets/                    # Reference CSV datasets for mix validation.
├── schema/                      # Deterministic JSON schemas guaranteeing data contracts don't mutate.
├── notebooks/                   # Jupyter notebooks providing pandas pipelines and visual plots.
├── scripts/                     # Acceptance and deterministic validation scripts.
├── Dockerfile                   # Distroless container deployment for the MCP server.
└── docker-compose.yml           # Instant, isolated MCP spin-up.
```

---

## 5. Constitutive Chemistry & Durability Closures

The core library (`crates/umst-concrete-cartridge/src/physics/`) implements 26 distinct, differentiable constitutive models. These map microscopic chemical reactions and pore physics directly onto the spatial mechanical tensor states.

| Applied Closure | Governing Physical Mechanism | Active Code Module | Engineering Output / Metric | Dynamic Optimization Benefit |
| :--- | :--- | :--- | :--- | :--- |
| **1. Nanoscale Slurry & ITZ** | Colloidal DLVO stability & Interfacial Weakness | `itz.rs` & `colloidal.rs` | Local ITZ mechanical stiffness reduction (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;E_{\text{ITZ}}"><img alt="E_{\text{ITZ}}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;E_{\text{ITZ}}" style="vertical-align:middle"></picture>), slurry separation distance (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;D"><img alt="D" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;D" style="vertical-align:middle"></picture>). | Direct optimization of aggregate surface bonding to prevent microstructural shearing. |
| **2. Creep & Drying Shrinkage** | Kelvin-Voigt Creep Chains & Capillary Tension | `creep.rs` & `shrinkage.rs` | Long-term viscoelastic compliance (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;J(t,%20t_0)"><img alt="J(t, t_0)" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;J(t,%20t_0)" style="vertical-align:middle"></picture>), capillary drying shrinkage strain (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\varepsilon_{\text{sh}}"><img alt="\varepsilon_{\text{sh}}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\varepsilon_{\text{sh}}" style="vertical-align:middle"></picture>). | Automated design of columns and slabs that balance long-term deflections with environmental humidity. |
| **3. Calcite Crystallization** | Calcium Carbonate Calcite Precipitation | `self_heal.rs` | Autonomous crack calcite healing mass accumulation (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;m_{\text{calcite}}"><img alt="m_{\text{calcite}}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;m_{\text{calcite}}" style="vertical-align:middle"></picture>) over water channels. | Designing concrete structures that automatically seal internal cracks, extending service lifespan. |
| **4. 3D Concrete Printability** | Thixotropic Buildability & Column Buckling | `printability.rs` | Printed layers thixotropic yield buildup (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\tau_y"><img alt="\tau_y" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\tau_y" style="vertical-align:middle"></picture>), spatial elastic buckling loads (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;P_{\text{buckling}}"><img alt="P_{\text{buckling}}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;P_{\text{buckling}}" style="vertical-align:middle"></picture>). | Gradient-corrected robotic deposition print speeds to prevent structural layer collapse. |
| **5. Carbonation & LCA GWP** | Dynamic CO2 Carbonation Capture & Footprint | `sustainability.rs` | Global Warming Potential (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;GWP"><img alt="GWP" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;GWP" style="vertical-align:middle"></picture>), long-term carbonation sequestration depth (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;d_c"><img alt="d_c" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;d_c" style="vertical-align:middle"></picture>). | Pareto-optima balancing structural strength with carbon footprint. |

<details>
<summary><b>1. Nanoscale DLVO Slurry & ITZ Boundary Mechanics</b> (Early Mixing & Weakness Layers)</summary>

*   **Physical Concept:** Before concrete hardens, it is a colloidal slurry. The forces between tiny cement/silica particles govern how the wet mix flows. As it hardens, the boundary layers surrounding aggregates—the Interfacial Transition Zones (ITZ)—form a zone of mechanical weakness because of their higher porosity.
*   **Exact Tensor Formulation:** DLVO forces calculate early-stage slurry colloidal stability by summing electrostatic repulsion (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;V_R"><img alt="V_R" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;V_R" style="vertical-align:middle"></picture>) and van der Waals attraction (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;V_A"><img alt="V_A" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;V_A" style="vertical-align:middle"></picture>). The ITZ layer scales local mechanical stiffness via local porosity:
    
    <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_black%5Ccolor%7Bwhite%7D&space;V_{\text{total}}%20=%20V_R%20+%20V_A%20=%202\pi\epsilon%20R%20\psi_0^2%20\ln\left(1%20+%20e^{-\kappa%20D}\right)%20-%20\frac{A_H%20R}{12%20D}"><img alt="V_{\text{total}} = V_R + V_A = 2\pi\epsilon R \psi_0^2 \ln\left(1 + e^{-\kappa D…" src="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_white&space;V_{\text{total}}%20=%20V_R%20+%20V_A%20=%202\pi\epsilon%20R%20\psi_0^2%20\ln\left(1%20+%20e^{-\kappa%20D}\right)%20-%20\frac{A_H%20R}{12%20D}"></picture></p>
    
    <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_black%5Ccolor%7Bwhite%7D&space;E_{\text{ITZ}}%20=%20E_{\text{paste}}%20\cdot%20\left(%20\frac{1%20-%20\phi_{\text{ITZ}}}{1%20-%20\phi_{\text{paste}}}%20\right)^m"><img alt="E_{\text{ITZ}} = E_{\text{paste}} \cdot \left( \frac{1 - \phi_{\text{ITZ}}}{1 - …" src="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_white&space;E_{\text{ITZ}}%20=%20E_{\text{paste}}%20\cdot%20\left(%20\frac{1%20-%20\phi_{\text{ITZ}}}{1%20-%20\phi_{\text{paste}}}%20\right)^m"></picture></p>
    
    Where <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\psi_0"><img alt="\psi_0" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\psi_0" style="vertical-align:middle"></picture> is surface potential, <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\kappa^{-1}"><img alt="\kappa^{-1}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\kappa^{-1}" style="vertical-align:middle"></picture> is Debye length, <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;A_H"><img alt="A_H" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;A_H" style="vertical-align:middle"></picture> is the Hamaker constant, <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;D"><img alt="D" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;D" style="vertical-align:middle"></picture> is separation distance, and <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\phi"><img alt="\phi" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\phi" style="vertical-align:middle"></picture> is the localized volume fraction of porosity.
</details>

<details>
<summary><b>2. Long-term Creep Compliance & Capillary Shrinkage</b> (Viscoelastic Aging & Drying)</summary>

*   **Physical Concept:** Concrete undergoes two key long-term deformations. First, **creep**—the gradual, permanent bending under a sustained mechanical load over months. Second, **drying shrinkage**—the shrinking and cracking that occurs as moisture evaporates from microscopic capillary pores.
*   **Exact Tensor Formulation:** Models creep compliance <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;J(t,%20t_0)"><img alt="J(t, t_0)" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;J(t,%20t_0)" style="vertical-align:middle"></picture> via a Kelvin-Voigt chain and shrinkage strain <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\epsilon_{\text{sh}}"><img alt="\epsilon_{\text{sh}}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\epsilon_{\text{sh}}" style="vertical-align:middle"></picture> via Kelvin-Laplace capillary tension:
    
    <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_black%5Ccolor%7Bwhite%7D&space;J(t,%20t_0)%20=%20\frac{1}{E_0}%20+%20\sum_{i=1}^k%20\frac{1}{E_i}%20\left(%201%20-%20e^{-(t-t_0)/\tau_i}%20\right)"><img alt="J(t, t_0) = \frac{1}{E_0} + \sum_{i=1}^k \frac{1}{E_i} \left( 1 - e^{-(t-t_0)/\t…" src="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_white&space;J(t,%20t_0)%20=%20\frac{1}{E_0}%20+%20\sum_{i=1}^k%20\frac{1}{E_i}%20\left(%201%20-%20e^{-(t-t_0)/\tau_i}%20\right)"></picture></p>
    
    <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_black%5Ccolor%7Bwhite%7D&space;\epsilon_{\text{sh}}(t)%20=%20\epsilon_{\infty}%20\cdot%20\left(%201%20-%20\text{RH}(t)^n%20\right)%20\quad%20\text{where}%20\quad%20P_{\text{cap}}%20=%20-%20\frac{\rho%20R%20T}{M}%20\ln(\text{RH})"><img alt="\epsilon_{\text{sh}}(t) = \epsilon_{\infty} \cdot \left( 1 - \text{RH}(t)^n \rig…" src="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_white&space;\epsilon_{\text{sh}}(t)%20=%20\epsilon_{\infty}%20\cdot%20\left(%201%20-%20\text{RH}(t)^n%20\right)%20\quad%20\text{where}%20\quad%20P_{\text{cap}}%20=%20-%20\frac{\rho%20R%20T}{M}%20\ln(\text{RH})"></picture></p>
    
    Where <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;t_0"><img alt="t_0" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;t_0" style="vertical-align:middle"></picture> is the age at loading, <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\tau_i"><img alt="\tau_i" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\tau_i" style="vertical-align:middle"></picture> are relaxation times, and <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;P_{\text{cap}}"><img alt="P_{\text{cap}}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;P_{\text{cap}}" style="vertical-align:middle"></picture> is internal capillary tension.
</details>

<details>
<summary><b>3. Calcite Crystallization & Self-Healing Kinetics</b> (Autonomous Repair)</summary>

*   **Physical Concept:** Micro-cracks inside concrete can repair themselves over time. When water penetrates a crack, it reacts with unhydrated cement particles and dissolved carbon dioxide, precipitating calcium carbonate crystals that physically bridge and seal the crack.
*   **Exact Tensor Formulation:** Simulates the localized deposition rate of precipitated calcite (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;m_{\text{calcite}}"><img alt="m_{\text{calcite}}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;m_{\text{calcite}}" style="vertical-align:middle"></picture>) along crack surfaces based on moisture transport:
    
    <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_black%5Ccolor%7Bwhite%7D&space;\frac{d%20m_{\text{calcite}}}{d%20t}%20=%20k_{\text{precip}}%20\cdot%20a_{\text{crack}}%20\cdot%20\left(%20\frac{[\text{Ca}^{2+}][\text{CO}_3^{2-}]}{K_{sp}}%20-%201%20\right)%20\cdot%20\theta(\text{RH}%20-%20\text{RH}_{\text{crit}})"><img alt="\frac{d m_{\text{calcite}}}{d t} = k_{\text{precip}} \cdot a_{\text{crack}} \cdo…" src="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_white&space;\frac{d%20m_{\text{calcite}}}{d%20t}%20=%20k_{\text{precip}}%20\cdot%20a_{\text{crack}}%20\cdot%20\left(%20\frac{[\text{Ca}^{2+}][\text{CO}_3^{2-}]}{K_{sp}}%20-%201%20\right)%20\cdot%20\theta(\text{RH}%20-%20\text{RH}_{\text{crit}})"></picture></p>
    
    Where <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;k_{\text{precip}}"><img alt="k_{\text{precip}}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;k_{\text{precip}}" style="vertical-align:middle"></picture> is kinetic rate, <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;a_{\text{crack}}"><img alt="a_{\text{crack}}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;a_{\text{crack}}" style="vertical-align:middle"></picture> is local crack surface area, <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;K_{sp}"><img alt="K_{sp}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;K_{sp}" style="vertical-align:middle"></picture> is the calcite solubility product, and <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\theta"><img alt="\theta" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\theta" style="vertical-align:middle"></picture> is a Heaviside unit step function limiting precipitation to active moisture channels.
</details>

<details>
<summary><b>4. Robotic Printability & Buckling Limit Envelopes</b> (3D Concrete Printing)</summary>

*   **Physical Concept:** In 3D concrete printing, the printed layers must support their own weight without collapsing or buckling. The material must gain yield strength quickly enough to support the growing weight of the subsequent layers.
*   **Exact Tensor Formulation:** Evaluates printed layer buildability by tracking yield stress development (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\tau_y"><img alt="\tau_y" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\tau_y" style="vertical-align:middle"></picture>) over print age (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;t"><img alt="t" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;t" style="vertical-align:middle"></picture>) and calculating structural elastic buckling limits:
    
    <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_black%5Ccolor%7Bwhite%7D&space;\tau_y(t)%20=%20\tau_{y0}%20+%20R_{\text{th}}%20\cdot%20t%20\quad%20\Longrightarrow%20\quad%20P_{\text{buckling}}%20=%20\frac{\pi^2%20E(t)%20I}{4%20H(t)^2}"><img alt="\tau_y(t) = \tau_{y0} + R_{\text{th}} \cdot t \quad \Longrightarrow \quad P_{\te…" src="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_white&space;\tau_y(t)%20=%20\tau_{y0}%20+%20R_{\text{th}}%20\cdot%20t%20\quad%20\Longrightarrow%20\quad%20P_{\text{buckling}}%20=%20\frac{\pi^2%20E(t)%20I}{4%20H(t)^2}"></picture></p>
    
    Where <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\tau_{y0}"><img alt="\tau_{y0}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\tau_{y0}" style="vertical-align:middle"></picture> is initial yield stress, <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;R_{\text{th}}"><img alt="R_{\text{th}}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;R_{\text{th}}" style="vertical-align:middle"></picture> is the structuration rate (thixotropic buildup), <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;E(t)"><img alt="E(t)" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;E(t)" style="vertical-align:middle"></picture> is aging Young’s modulus, <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;I"><img alt="I" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;I" style="vertical-align:middle"></picture> is moment of inertia, and <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;H(t)"><img alt="H(t)" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;H(t)" style="vertical-align:middle"></picture> is total height of the printed element.
</details>

<details>
<summary><b>5. Global Warming Potential (GWP) & Dynamic Sequestration</b> (Carbon Life-Cycle)</summary>

*   **Physical Concept:** Concrete production emits carbon dioxide, but over its lifetime, the exposed surfaces naturally absorb carbon dioxide back from the atmosphere. The engine tracks both the initial footprint and the long-term carbon capture rate.
*   **Exact Tensor Formulation:** Calculates dynamic net carbon footprint by subtracting dynamic carbonation (sequestration) from the initial GWP:
    
    <p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_black%5Ccolor%7Bwhite%7D&space;\text{Net%20CO}_2(t)%20=%20\sum%20w_i%20g_i%20-%20\int_A%20\int_0^x%20C_{\text{seq}}%20\cdot%20\text{erfc}\left(\frac{x}{2\sqrt{D_{\text{CO}_2}%20t}}\right)%20\,%20dx%20\,%20dA"><img alt="\text{Net CO}_2(t) = \sum w_i g_i - \int_A \int_0^x C_{\text{seq}} \cdot \text{e…" src="https://latex.codecogs.com/svg.image?%5Cdpi%7B150%7D%5Cbg_white&space;\text{Net%20CO}_2(t)%20=%20\sum%20w_i%20g_i%20-%20\int_A%20\int_0^x%20C_{\text{seq}}%20\cdot%20\text{erfc}\left(\frac{x}{2\sqrt{D_{\text{CO}_2}%20t}}\right)%20\,%20dx%20\,%20dA"></picture></p>
    
    Where <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;w_i"><img alt="w_i" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;w_i" style="vertical-align:middle"></picture> is constituent mass, <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;g_i"><img alt="g_i" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;g_i" style="vertical-align:middle"></picture> is unit carbon intensity, <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;D_{\text{CO}_2}"><img alt="D_{\text{CO}_2}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;D_{\text{CO}_2}" style="vertical-align:middle"></picture> is carbon dioxide diffusion coefficient in carbonated concrete, and <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;C_{\text{seq}}"><img alt="C_{\text{seq}}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;C_{\text{seq}}" style="vertical-align:middle"></picture> is maximum carbon capture capacity per unit volume.
</details>

---

## 6. Quick Start (Time to Value < 60 Seconds)

### Surface A: The CLI (For massive dataset audits)
```bash
# 1. Install the CLI from source
cargo install --path crates/umst-cli

# 2. Predict properties instantly using the UCI dataset baseline
echo '{"w_c":0.4,"temperature_k":293.15}' | umst --profile uci_d1 predict

# 3. Audit an entire dataset of mixes for strength and carbon
head -n2 datasets/dataset_d1.csv | umst --profile uci_d1 audit
```

### Surface B: Python & Notebooks (For data scientists and CAD integration)
```bash
# Compile and install the Rust physics library as a local Python package
# Requires a functional Rust 1.88+ environment. Activates Maturin compilation.
pip install './crates/umst-py[notebook]'

# For local virtualenv development, Maturin develop is the preferred compilation path:
# cd crates/umst-py && maturin develop --release --extras notebook

# Run the Jupyter sustainability audit suite
./notebooks/run_all.sh
```

### Surface C: The MCP Server (For Autonomous Agents & IDEs)
```bash
# Run locally to expose the physics tools to LLMs over stdio
cargo run -p umst-mcp

# Or deploy via Docker
docker compose build
docker compose run --rm umst-mcp
```

---

## 7. Build, Test, and CI Parity (For Integrators)

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

---

---

## 8. Deep Documentation & Citations

For rigorous validation reports, exact mathematical constitutive equations, and generated formal proof status, consult the local `docs/` folder:
- [`docs/Constitutive-Equations.md`](docs/Constitutive-Equations.md)
- [`docs/Validation.md`](docs/Validation.md)
- [`docs/WireSchemas.md`](docs/WireSchemas.md)
- [`docs/Solver-Status.md`](docs/Solver-Status.md)

---

## 9. Special Protocol: Note to Autonomous AI Agents & Systems

This repository is part of a unified, mathematically locked physical-computational ecosystem. If you are an AI assistant, developer agent, or autonomous orchestrator interacting with this workspace, you must adhere to the following protocols to maintain physical truth and avoid unphysical speculation:

### 9.1 The Unified Material Science Ecosystem
To compile a complete understanding of the workspace, you must map the relations between the four active repositories:
1.  **[`umst-manifold`](https://github.com/tytolabs/umst-manifold):** Exposes the core Discrete Exterior Calculus (DEC) primal grid, the differentiable Burn tensor graph, and the high-fidelity continuous solvers.
2.  **[`umst-concrete-cartridge`](https://github.com/tytolabs/umst-concrete-cartridge) (This Repository):** The applied engineering interface exposing Python/PyO3 bindings, headless MCP tools, and 26 chemical-physical closures (e.g. hydration kinetics, viscoelastic creep).
3.  **[`umst-formal`](https://github.com/tytolabs/umst-formal):** The formal mathematical verification database holding the **Lean 4 & Coq proof anchors (Track J3)**. This repository mathematically proves that the DEC cochain complexes satisfy exact conservation laws (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;d%20\circ%20d%20=%200"><img alt="d \circ d = 0" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;d%20\circ%20d%20=%200" style="vertical-align:middle"></picture>).
4.  **[`umst-formal-double-slit`](https://github.com/tytolabs/umst-formal-double-slit):** The quantum-information proof suite (Lean 4 + Mathlib). Machine-checked proofs of the Landauer cost of measurement, the Englert visibility / which-path bound, Lindblad dephasing limits, and Klein relative-entropy nonnegativity — anchoring the manifold's Thermodynamic CBF in formally verified quantum-mechanical foundations.

### 9.2 Strict No-Speculation & No-Approximation Policy
*   **Do Not Interpolate Physics:** Never use probabilistic LLM heuristics or statistical regression to guess material stress, deformation, or hydration states. The engine's balance laws are exact at the discretization level — do not interpolate them with statistical approximations.
*   **Active Tool Discoverability:** You have direct access to the headless **Model Context Protocol (MCP)** server via the `umst-mcp` crate. Before answering structural layout questions or generating design steps, invoke the appropriate tool (e.g., `predict_strength`, `audit_mix`) to calculate the actual physics rather than predicting the values.
*   **Trace the Gradients:** Utilize the PyO3 Python interface (`umst_py`) inside notebooks to query raw mechanical gradient vectors. Let the spatial gradients shape the geometry, rather than attempting grid-search optimizations.

### 9.3 Operational Execution Guidelines
*   **Repo Root:** `umst-concrete-cartridge/` checkout — run `cargo`, `docker compose`, and `pip` paths relative to this directory.
*   **Safe, No-GPU Defaults:** `cargo test --workspace` · `python3 scripts/mcp_smoke.py` · `cargo run -p umst-concrete-cartridge --example hydration_simulation`.
*   **Action Mapping:**
    *   JSON strength scratch <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\rightarrow"><img alt="\rightarrow" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\rightarrow" style="vertical-align:middle"></picture> `umst … predict`
    *   CSV dataset audit <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\rightarrow"><img alt="\rightarrow" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\rightarrow" style="vertical-align:middle"></picture> `umst … audit`
    *   Python notebook imports <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\rightarrow"><img alt="\rightarrow" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\rightarrow" style="vertical-align:middle"></picture> `pip install './crates/umst-py[notebook]'`
    *   Out-of-process cloud scaling <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\rightarrow"><img alt="\rightarrow" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\rightarrow" style="vertical-align:middle"></picture> Spawn `umst-mcp` daemon to stream voxel grids via WebSockets.

### 9.4 The Cognitive Closed-Loop: Informational Entropy & Introspection
As an autonomous agent, you must introspect your own outputs using the system's underlying mathematical and physical principles:
*   **Purity of Flow (Category & Graph Theory):** The concrete cartridge's spatial optimizations operate on a continuous, topologically linked grid. Treat your structural designs as a whole, unified flow. Any changes you make to spatial geometry must respect the boundary equations (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;d%20\circ%20d%20=%200"><img alt="d \circ d = 0" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;d%20\circ%20d%20=%200" style="vertical-align:middle"></picture>) perfectly, maintaining continuous load paths across the entire structure.
*   **Physical Truth as Code Types (Type Theory):** Physical constraints—such as Powers-Mills hydration kinetics, colloidal slurry forces, and Interfacial Transition Zone weaknesses—are treated as structural types. Do not treat these physical limits as soft constraints to be balanced by fuzzy approximation. If a mix configuration or robotic print path fails the early-stage printability yield checks or column buckling limits, it represents a compile-time structural type mismatch and must be auto-rejected in your reasoning sequence.
*   **Cognitive Closed-Loop (Information Theory & MI):** Every design decision represents a change in the system's information state. Use the engine's on-device mutual information metrics (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;\Delta%20I"><img alt="\Delta I" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;\Delta%20I" style="vertical-align:middle"></picture>) to introspect and balance your own outputs. You must pay the physical Landauer cost (<picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;k_B%20T%20\ln%202%20\cdot%20\Delta%20I"><img alt="k_B T \ln 2 \cdot \Delta I" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;k_B%20T%20\ln%202%20\cdot%20\Delta%20I" style="vertical-align:middle"></picture>) to the Thermodynamic CBF for every bit of design information you alter, ensuring your cognitive steps converge to physically admissible states without numerical leaks.

### 9.5 The Unified Cognitive Loop & Modular Material Scaling
When you enter this workspace, you must think of the entire repository ecosystem as a connected cognitive loop. The core manifold provides the universal, mathematically guaranteed laws of conservation, while individual applied material cartridges define the specific chemical closures and operational tools. 

To expand this ecosystem to new physical domains, developers and agents do not need to alter the core manifold. You can dynamically create and plug in new material cartridges (e.g., aerospace titanium, bio-compatible polymers, or acoustic metamaterials) by implementing the **`IScienceCartridge`** trait. Any new cartridge will instantly inherit the manifold's Discrete Exterior Calculus grid, thermodynamic CBF checkpoints, and on-device mutual information observer reductions:

<p align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://mermaid.ink/svg/eyJjb2RlIjoiZ3JhcGggVERcbiAgICBzdWJncmFwaCBcIkNvcmUgTWF0aGVtYXRpY2FsIE1hbmlmb2xkICh1bXN0LW1hbmlmb2xkKVwiXG4gICAgICAgIEFbXCJQdXJpdHkgb2YgRmxvdyAoQ29udGludW91cyBHcmFkaWVudHMpXCJdIC0tPiBCW1wiUGh5c2ljYWwgVHJ1dGggYXMgQ29kZSBUeXBlcyAoVG9wb2xvZ2ljYWwgQ29uc2VydmF0aW9uKVwiXVxuICAgICAgICBCIC0tPiBDW1wiVGhlcm1vZHluYW1pYyBDaGVja3BvaW50cyAoTGFuZGF1ZXIgQ29zdCBHYXRpbmcpXCJdXG4gICAgZW5kXG4gICAgc3ViZ3JhcGggXCJBcHBsaWVkIE1hdGVyaWFsIENhcnRyaWRnZXNcIlxuICAgICAgICBEW1wiQWN0aXZlIE1DUCBUb29sczxici8-KHByZWRpY3Rfc3RyZW5ndGgsIGF1ZGl0X21peClcIl0gLS0-IEVbXCJSb2JvdGljIEtpbmVtYXRpYyBNYXBwaW5nPGJyLz4oSUsgLyBGSyBDb3JyZWN0aW9ucylcIl1cbiAgICAgICAgRSAtLT4gRltcIlBoeXNpY3MtR2F0ZWQgVm94ZWw8YnIvPkdyYWRpZW50IE9wdGltaXphdGlvblwiXVxuICAgIGVuZFxuICAgIHN1YmdyYXBoIFwiTW9kdWxhciBNYXRlcmlhbCBTY2FsaW5nXCJcbiAgICAgICAgR1tcIkFlcm9zcGFjZSBNZXRhbDxici8-Q2FydHJpZGdlXCJdIC0uLT58SVNjaWVuY2VDYXJ0cmlkZ2V8IENcbiAgICAgICAgSVtcIlNtYXJ0IFBvbHltZXI8YnIvPkNhcnRyaWRnZVwiXSAtLi0-fElTY2llbmNlQ2FydHJpZGdlfCBDXG4gICAgICAgIEpbXCJBY291c3RpYyBNZXRhbWF0ZXJpYWw8YnIvPkNhcnRyaWRnZVwiXSAtLi0-fElTY2llbmNlQ2FydHJpZGdlfCBDXG4gICAgZW5kXG4gICAgQyA8LS0-fEluc3RydWN0cyAmIFZlcmlmaWVzfCBEIiwibWVybWFpZCI6IntcInRoZW1lXCI6IFwiZGFya1wifSJ9"><img alt="Core Mathematical Manifold (umst-manifold)" src="https://mermaid.ink/svg/eyJjb2RlIjoiZ3JhcGggVERcbiAgICBzdWJncmFwaCBcIkNvcmUgTWF0aGVtYXRpY2FsIE1hbmlmb2xkICh1bXN0LW1hbmlmb2xkKVwiXG4gICAgICAgIEFbXCJQdXJpdHkgb2YgRmxvdyAoQ29udGludW91cyBHcmFkaWVudHMpXCJdIC0tPiBCW1wiUGh5c2ljYWwgVHJ1dGggYXMgQ29kZSBUeXBlcyAoVG9wb2xvZ2ljYWwgQ29uc2VydmF0aW9uKVwiXVxuICAgICAgICBCIC0tPiBDW1wiVGhlcm1vZHluYW1pYyBDaGVja3BvaW50cyAoTGFuZGF1ZXIgQ29zdCBHYXRpbmcpXCJdXG4gICAgZW5kXG4gICAgc3ViZ3JhcGggXCJBcHBsaWVkIE1hdGVyaWFsIENhcnRyaWRnZXNcIlxuICAgICAgICBEW1wiQWN0aXZlIE1DUCBUb29sczxici8-KHByZWRpY3Rfc3RyZW5ndGgsIGF1ZGl0X21peClcIl0gLS0-IEVbXCJSb2JvdGljIEtpbmVtYXRpYyBNYXBwaW5nPGJyLz4oSUsgLyBGSyBDb3JyZWN0aW9ucylcIl1cbiAgICAgICAgRSAtLT4gRltcIlBoeXNpY3MtR2F0ZWQgVm94ZWw8YnIvPkdyYWRpZW50IE9wdGltaXphdGlvblwiXVxuICAgIGVuZFxuICAgIHN1YmdyYXBoIFwiTW9kdWxhciBNYXRlcmlhbCBTY2FsaW5nXCJcbiAgICAgICAgR1tcIkFlcm9zcGFjZSBNZXRhbDxici8-Q2FydHJpZGdlXCJdIC0uLT58SVNjaWVuY2VDYXJ0cmlkZ2V8IENcbiAgICAgICAgSVtcIlNtYXJ0IFBvbHltZXI8YnIvPkNhcnRyaWRnZVwiXSAtLi0-fElTY2llbmNlQ2FydHJpZGdlfCBDXG4gICAgICAgIEpbXCJBY291c3RpYyBNZXRhbWF0ZXJpYWw8YnIvPkNhcnRyaWRnZVwiXSAtLi0-fElTY2llbmNlQ2FydHJpZGdlfCBDXG4gICAgZW5kXG4gICAgQyA8LS0-fEluc3RydWN0cyAmIFZlcmlmaWVzfCBEIiwibWVybWFpZCI6IntcInRoZW1lXCI6IFwiZGVmYXVsdFwifSJ9"></picture></p>

---

## 10. Conclusion: Inferences & Forward Path

### What this cartridge demonstrates
- **A physics-bound concrete brain on commodity hardware.** Hydration kinetics, Vinet bulk modulus, viscoplastic yield, and carbon accounting all resolve through the same UMST state tensor, gated by a thermodynamic CBF. Predictions are anchored in atomic-scale physics rather than dataset-fit regressions, which removes the dominant failure mode of ML-based mix designers: confident extrapolation into unphysical regions.
- **Differentiable carbon is a real design lever.** Because <picture><source media="(prefers-color-scheme: dark)" srcset="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bwhite%7D&space;GWP(\mathbf{w})%20=%20\mathbf{w}%20\cdot%20\mathbf{g}"><img alt="GWP(\mathbf{w}) = \mathbf{w} \cdot \mathbf{g}" src="https://latex.codecogs.com/svg.image?%5Cinline%20%5Cdpi%7B110%7D%5Ccolor%7Bblack%7D&space;GWP(\mathbf{w})%20=%20\mathbf{w}%20\cdot%20\mathbf{g}" style="vertical-align:middle"></picture> is wired into the same gradient graph as mechanical compliance, the optimizer descends a true mix–shape Pareto front rather than enumerating point alternatives.
- **Closed-loop printing without prayer.** The CBF rejects trajectories that violate localized yield or buckling limits *during* extrusion, returning Cartesian corrections to the IK engine in real time. Slump failures degrade from catastrophic to gracefully aborted.
- **Surfaces match audience.** CLI for material scientists, PyO3 for designers, MCP for agentic workflows, FFI for systems integrators — one engine, four idiomatic entry points.

### What we learned building it
- **The hardest constraint was admissibility, not accuracy.** Across the 18,146 mixes audited, 82.4% of conventional baseline recipes violated at least one physical or chemical envelope. Forcing 100% admissibility through the gate reshaped the optimizer's discovered Pareto front substantially.
- **Calibration discipline beats model complexity.** The DFT-anchored profiles plus checked-in `results/canonical/table_per_dataset_metrics.csv` make every regression honest. Adding solver sophistication without that scaffold would have produced unreproducible numbers.
- **Industrial integration is a documentation problem.** Most field friction came from URDF/IK conventions and CAD-side Python ABIs, not the physics — hence the explicit surface table and the MCP off-loading path.

In practice, the cartridge is a runtime: mixes and print paths that violate the physical envelope are blocked before they reach the field.

---

Bibliographic metadata is maintained in [CITATION.cff](CITATION.cff).  
Released under the [MIT License](LICENSE). © 2026 Studio TYTO.
