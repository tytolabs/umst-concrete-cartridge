<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# The Multi-Physics Constitutive Engine

The `umst-concrete-cartridge` maps the abstract topological flows of the UMST Manifold into highly specific, real-world equations governing cementitious materials. It solves 14 domains simultaneously.

## 1. Early-Age Hydration & Thermodynamics
Based on the **Jennings CM-II** model, the engine tracks the phase transition from free water and clinker into C-S-H gel. The exothermic reaction releases heat flux $\mathbf{q}$, which the manifold routes via the `$B_1$` matrix.

## 2. Rheology & Extrudability
Before setting, the material is modeled as a non-Newtonian yield-stress fluid. We utilize a combination of the **Chateau-Ovarlez** suspension model and the **YODEL** framework to compute apparent viscosity $\eta$ and static yield stress $\tau_0$.

## 3. Printability Constraints
To ensure the concrete can be 3D printed without collapsing, the engine maps the rheological states against **Roussel's structural build-up constraints**. If the vertical stress $\sigma_z$ exceeds the time-dependent yield stress $\tau_0(t)$, the manifold flags a topological failure.

## 4. Fracture Mechanics
Once hardened, the framework switches to **Micromechanics (Ulm et al.)**. Tensile stresses are integrated across the edges. When the critical stress intensity factor $K_{Ic}$ is exceeded, the continuous damage scalar $d$ on the edge approaches 1.0, cleanly fracturing the Cellular Sheaf.
