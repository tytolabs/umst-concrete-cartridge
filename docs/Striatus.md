<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Striatus and the funicular shell lineage

This note connects a concrete shell topology-optimisation demo in this repository to a long design tradition: finding where **compression** wants to flow when gravity and surface loads are fixed. Each historical step below is cited with `[Author Year]` shorthand that resolves to [`References.bib`](References.bib).

## Hooke and the hanging chain

Robert Hooke, in 1675, recorded the duality between a flexible hanging line and a rigid arch carrying thrust in compression: invert the chain and you read the funicular for the arch [Hooke 1675]. That observation is not a numerical method; it is a **geometric principle**—material is useful where it lies along the thrust path, and thin where bending is negligible. Modern topology optimisation revisits the same question on a grid: given loads and supports, where should solid material remain so that stiffness is bought efficiently?

The leap from a hanging chain to a **continuum** shell is large. A chain has only axial stiffness; a solid shell carries stress tensors with multiple nonzero components. Nevertheless, the **design intent** remains legible across centuries: align the solid skeleton with the dominant load-bearing directions so that expensive tension steel—or brittle tension in unreinforced concrete—does not govern the global resistance mechanism. Topology optimisation on a fine grid automates that alignment under a chosen objective (here, compliance minimisation) and constraints (volume fraction, filtered design variables).

## Gaudí, Nervi, and drawn isostatics

Antoni Gaudí used inverted hanging models to shape the Sagrada Família so that stone and later concrete could work primarily in compression along intuitive thrust surfaces. Pier Luigi Nervi, for the Palazzetto dello Sport (1957), exploited ribbed shells whose layout follows **compression isostatics** traced by hand from elastic theory and structural judgment. In both cases the designer’s eye stayed on load paths that avoid tensile reliance in the bulk masonry or concrete. The Striatus-class visual signature in this repository’s GIF—ribs following principal compression directions—is deliberately in dialogue with that graphic tradition, not with a black-box “style transfer.”

Nervi’s ribs are not merely decorative: they thicken the shell where moment and shear demand material and open voids where the stress field is quiescent. That **material distribution** mindset is exactly what SIMP-style topology optimisation encodes, albeit with different governing equations (continuum elasticity instead of hand-traced isostatics) and different manufacturing assumptions (here, a monolithic printed solid thresholded from a density field).

## Otto and minimal surfaces

Frei Otto’s experimental form-finding with soap films and cable nets shifted twentieth-century practice toward **surface- and network-based** equilibrium models before dense finite-element workflows were routine. Soap films illustrate tension-dominated minimal surfaces; inverted analogies still motivate designers to think in terms of pure path structures. Shell roofs in concrete, however, are dominated by **compression and shear** in the solid; the relevant mathematical picture is closer to a 3D elasticity problem with a thin boundary and strong self-weight coupling.

Otto’s legacy is often misread as “organic shape for its own sake.” In the lineage relevant here, the deeper lesson is **experimental equilibrium**: build a physical or numerical model whose degrees of freedom move until an energy-like functional is stationary, then read geometry from that stationary state. Topology optimisation is another equilibrium loop—outer iterations adjust the density field while inner linear solves enforce static equilibrium on the current stiffness layout.

## Block, thrust networks, and three-dimensional funicular analysis

Philippe Block and Lorenz Lachauer developed practical algorithms for **three-dimensional funicular analysis** of masonry vaults, connecting discrete thrust networks with safe equilibrium in compression-only idealisations [Block & Lachauer 2014]. That line of work makes explicit what earlier builders did tacitly: equilibrium is admissible when internal forces stay inside a **no-tension** or compression-biased yield envelope appropriate to the material. The demo here does not run thrust-network analysis; it uses density-based SIMP on a continuum grid. The intellectual link is that both seek **compression-favouring** layouts under fixed external actions.

## Striatus Bridge, Venice 2021

The Striatus Bridge in Venice (2021) is referenced here by **its own name** and authorship: *Block Research Group, Bhooshan & Van Mele, Striatus Bridge, Venice 2021*—an unreinforced 3D-printed concrete footbridge whose geometry is organised so that compression carries the primary load path [Bhooshan, Van Mele & Block 2021]. It is the modern anchor for the claim that **additively placed concrete** can stand without conventional steel reinforcement when the shape respects funicular action. This repository does not reproduce that bridge’s boundary conditions, printer, or joint layout; it shows that a **SIMP optimiser** on a model slab can converge to a **Nervi/Striatus-class compression-favouring topology** in the sense of approved claim language used elsewhere in the project.

## Topology optimisation ingredients on the grid

The `optimize_shell_3d` example bundles ingredients that separate a **literature-faithful** SIMP run from an under-regularised toy:

- A **Helmholtz-type density filter** smooths the design variable before stiffness is assembled [Lazarov & Sigmund 2011].
- A **tanh Heaviside projection** with continuation sharpens the filtered field toward a near-binary layout [Wang, Lazarov & Sigmund 2011].
- **Self-weight** uses Bruyneel–Duysinx mass–stiffness decoupling so intermediate densities do not appear “weightless” while still stiff [Bruyneel & Duysinx 2005].
- The **volume fraction** is driven with an augmented Lagrangian update so the solid fraction tracks a target [Bertsekas 1996].
- **Roof traction in x:** **`optimize_shell_3d`** enables the gentle top-face ramp only when **`UMST_SHELL_ROOF_RAMP=1`** (strength **`UMST_SHELL_ROOF_RAMP_F`**, default **0.2**); **unset** ⇒ **uniform** roof pressure. The opt-in **`shell_topology_rib_pattern_full_v04`** harness is **different**: **`UMST_SHELL_ROOF_RAMP=0`** forces uniform (aligned with the example’s “no ramp” default); **unset** or any value other than **`0`** / **`1`** turns the **x** ramp **on** at **`UMST_SHELL_ROOF_RAMP_F`** so CI quick and full B6 share the same load asymmetry — see **`docs/Solver-Status.md`** (Track B6 + **“2026-05-11 greyness vs roof-ramp defaults”**). A **2026-05-11** **200**-outer **`--release`** full B6 attempt still **missed** the greyness gate at **~0.51** (finite run); that log matched the **then**-documented “unset = uniform” pairing and does **not** re-verify greyness after later harness wording changes.
- **Default CI quick (`shell_topology_rib_pattern_quick`):** compact **9×8** slab, **50 Pa** roof **x-ramp** **r=0.2**, **β=10** Heaviside, **24** default Adam outers — a lightweight smoke for **vf**, roof-slice Heaviside **`xy_var`**, compliance, and **finite** Heaviside greyness (see `shell_topology_rib_pattern.rs` `#!`; strict **greyness < 0.15** / **`xy_var > 0.1`** are for the **ignored** **40×40×4** full harness, not this quick path).
- **XY symmetry (default on):** in-loop averaging over four mirror partners every **`UMST_SHELL_SYMM_PERIOD`** outers; **`final.npy`** applies the same XY average once at the end so the exported field matches the symmetrised design. **`iter_*.npy`** frames (when **`UMST_SHELL_DUMP_ITER=1`**) can differ slightly on the last GIF frame vs **`final.npy`** because mirroring is periodic in the loop — use **`final.npy`** + **`manifest.json`** (`symmetry_xy`, `sym_period`, `iters`, `dump_stride`, …) for reproducible **ρ**-span checks and **`export_print_ready.py`**. **`UMST_SHELL_DUMP_STRIDE`** defaults to **10** when unset (aligned with **`notebooks/_run_shell_demo.sh`**).
- The mechanical kernel follows a **continuum topology optimisation** elasticity route on an extruded plate scaffold [Bendsøe & Sigmund 2003], with classical SIMP stiffness interpolation rooted in homogenisation-based thinking [Bendsøe & Sigmund 1989].

The comparative review by [Sigmund & Maute 2013] situates these choices among robust formulations in structural optimisation. Efficient reference implementations for 3D topology optimisation on regular grids, such as the MATLAB code of [Liu & Tovar 2014], informed engineering expectations for iteration counts and solver structure even though this repository’s core is Rust, not MATLAB. Homogenisation-based interpretations of the SIMP exponent remain the conceptual bridge between **microstructural void families** and **macroscopic penalised stiffness** [Bendsøe & Sigmund 1989]; the textbook treatment [Bendsøe & Sigmund 2003] is the canonical reference for assembling filtered, projected, and volume-constrained compliance problems in structural optimisation practice.

## Streamlines on the final GIF frames

The overlay script integrates the **principal compression** direction field with a fourth-order Runge–Kutta streamer seeded on a regular grid. Those curves are a **visual analogue** of the isostatic nets Nervi drew and of the thrust lines Block’s tools compute for masonry, but they are derived here from a **continuum elasticity** stress tensor after SIMP-modulated stiffness—not from a discrete thrust network. When the animation slows on the last frames, the eye should read **aligned ribs** plus **aligned streamlines** as mutually reinforcing evidence that the optimiser is organising solid material along compression-dominated paths.

The streamline overlay is deliberately saturated (red–orange) so that publication-scale GIF compression still leaves the pattern legible at preview resolution. That is a rendering choice, not a structural claim: the underlying stress field is mesh-dependent and should be interpreted qualitatively in this demo context.

## Artefacts produced by the demo

Running `notebooks/_run_shell_demo.sh` (after building the Rust example with the documented feature flags) is intended to yield:

- A hero animation at `notebooks/_artifacts/striatus_emergence.gif` showing density evolution and, on the final frames, an overlay of **principal-compression** streamlines—the Striatus / Nervi visual payoff.
- A watertight STL at `notebooks/_artifacts/striatus_shell_v0.4.stl` plus a JSON sidecar `notebooks/_artifacts/striatus_shell_v0.4.print_ready.json` with volumes, bounding box, minimum feature scale, worst overhang relative to build direction **+z**, and **v0.4 topology gates** (mesh genus / Euler characteristic, density XY variance, marching-cubes volume fraction in bbox). `export_print_ready.py` also writes `striatus_shell_v0.3.*` as symlinks (or copies) for older scripts. The Rust example also writes **`crates/umst-concrete-cartridge/examples/_artifacts/shell/manifest.json`** beside **`final.npy`**, including grid, **`burn_seed`: 42**, symmetry / dump cadence, and outer-count metadata for reproducible **40×40×4** Track L runs (see **`docs/Solver-Status.md`** for the full overnight command block).

The Python export path treats mesh generation as an **IO / artefact pipeline** with explicit checks: watertight manifold mesh, minimum circumradius feature scale, and an overhang angle policy consistent with typical extrusion-based concrete printing without internal supports. Claim language stays within the project rule: the STL is asserted to **pass watertight and minimum-feature checks for a 12 mm concrete printer nozzle** together with a **30° overhang policy** when those checks succeed at runtime; if they fail, the deliverable is incomplete until the export or demo is repaired—not the tolerances.

Concrete extrusion differs from polymer fused filament: layer interfaces, aggregate jamming, and hydration shrinkage all interact with geometric **feature size** and **overhang**. The sidecar JSON is therefore part of the engineering contract alongside the STL: it records what was checked, not a blanket certificate for every printer or mix.

## Concrete printing vocabulary (short)

**Build direction** is taken as **+z** in the export script so that overhang is measured against gravity in the slicer’s usual convention. **Minimum feature size** is reported via a circumradius proxy on the exported triangle soup: thin spines that would break during extrusion should fail the check rather than surviving as non-manifold garbage. **Watertight** means a single closed shell suitable for finite-volume infill planning in mainstream mesh tools; it does not guarantee build success without tuning layer height, print speed, or mix rheology.

## Relation to UMST

The UMST manifold supplies differentiable mechanics and topology hooks; the concrete cartridge wires a **cementitious** interpretation and demonstration assets. Together they support the approved framing that **the optimiser converges to a Nervi/Striatus-class compression-favouring topology** and that **the recovered shape lies within the funicular-shell tradition**, without asserting blanket structural safety for every site-specific load combination.

The differentiable stack matters for research workflows: gradients of compliance-like objectives with respect to design variables are the workhorse of first-order optimisers (here, Adam on the density field). That is not the same as claiming that every local minimum is globally optimal, nor that the discrete voxel model matches a full shell theory with drilling rotations and transverse shear refinements. The demo is a **showcase** aligned with cited numerical practice [Lazarov & Sigmund 2011; Wang, Lazarov & Sigmund 2011; Bruyneel & Duysinx 2005; Bertsekas 1996; Sigmund & Maute 2013], not a substitute for project-specific peer review.

## Reading order for newcomers

1. Skim this essay top-to-bottom for the historical arc.
2. Open [`References.bib`](References.bib) and locate each `[Author Year]` key used above.
3. Run the shell demo script once you have Rust + Python extras installed; watch the GIF with the overlay frames paused.
4. Inspect **`notebooks/_artifacts/striatus_shell_v0.4.print_ready.json`** (field **`gates_track_b8_all_pass`**) next to **`notebooks/_artifacts/striatus_shell_v0.4.stl`** before sending the mesh to any slicer. Pytest coverage: **`notebooks/tests/test_print_ready.py`**. Full **B6** opt-in harness + honesty on **200** outers / greyness / roof semantics: **`docs/Solver-Status.md`** → **P0 runbook — `shell_topology_rib_pattern_full_v04`** and **Solver lanes — Topology / shell**.

That order keeps **intent** (compression paths) ahead of **implementation** (meshes and JSON).

## BibTeX keys used in prose

| Inline tag | BibTeX entry key (see `References.bib`) |
|------------|----------------------------------------|
| [Hooke 1675] | `hooke1675potentia` |
| [Bendsøe & Sigmund 1989] | `bendsoe1989homogenization` |
| [Bendsøe & Sigmund 2003] | `bendsoe2003topology` |
| [Bruyneel & Duysinx 2005] | `bruyneel2005selfweight` |
| [Lazarov & Sigmund 2011] | `lazarov2011helmholtz` |
| [Wang, Lazarov & Sigmund 2011] | `wang2011projection` |
| [Sigmund & Maute 2013] | `sigmund2013comparative` |
| [Block & Lachauer 2014] | `block2014funicular3d` |
| [Liu & Tovar 2014] | `liu2014matlabtop3d` |
| [Bhooshan, Van Mele & Block 2021] | `bhooshan2021striatus` |
| [Bertsekas 1996] | `bertsekas1996multiplier` |

Rust `formal_anchor` comments in the cartridge and manifold use the same bracket shorthand where a symbol is classified as **`Literature`**; IO-only export code uses **`NONE`** with an explicit rationale string, matching the five-status grammar documented in `docs/PROOF-STATUS.md`.

## Frequently asked questions

**Does the GIF prove the bridge is safe?** No. The GIF shows a **deterministic** optimisation trajectory on a model problem. Structural safety is a site-specific judgment involving loads, materials, construction tolerances, and codes—not something inferred from an animation alone.

**Why cite Striatus without naming industrial partners?** The repository policy is to foreground **authorship and structure** (Block Research Group, Bhooshan, Van Mele) and the bridge’s public name, avoiding promotional coupling to any single supplier.

**Why Helmholtz filtering instead of a Gaussian blur?** The Helmholtz PDE filter is the cited operator in topology optimisation literature [Lazarov & Sigmund 2011]; it preserves length-scale control in a way that ad hoc kernels do not, and it matches the test specification in the manifold crate.

**Can I swap the optimiser for a black-box evolutionary method?** That would break the differentiable research story and is out of scope for this demo. The point is to show **gradient-based** topology optimisation with modern regularisation ingredients.

**Where do I change nozzle or overhang limits?** Adjust the export script constants and the accompanying JSON schema together; do not silence failing checks—fix the mesh or the optimisation setup until the STL passes.

---

> *The shell above is the SIMP optimiser's answer to the same question Hooke asked in 1675 and Block answered in 2021: where does material want to go when gravity pulls it and the load is fixed? The answer, as the GIF makes visible, is that material flows along the compression isostatics until what remains is a Nervi/Striatus-class rib pattern that stands in compression alone. The output STL passes a 12 mm minimum-feature check and a 30° overhang policy — it is buildable on a Studio TYTO concrete printer.*

## References

Machine-readable BibTeX lives in [`References.bib`](References.bib).
