<!-- SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Formal anchors (`src/`)

Every **`pub`** function, struct, enum, trait, and associated `pub fn` in `src/**/*.rs` carries a formal documentation block enforced by **`cargo test --test formal_anchors`**.

Rolling **`formal_status`** histogram: see [`docs/PROOF-STATUS.md`](PROOF-STATUS.md) (regenerated from `tests/proof_status_doc.rs`).

## Status vocabulary

| Status | `formal_anchor` scheme | Required companion lines |
|--------|------------------------|---------------------------|
| **Mechanised** | `lean://umst-formal/Lean/...#lemma` | — |
| **Structural** | `lean://...` | wire/schema witnesses (schema versions, Kleisli arrows, gate naturality, …) |
| **Boundary** | `lean://...` | explicit scope limits on serde structs lifted from TOML (`ProvenanceFormal`) |
| **Empirical** | `empirical://datasets/<file>.csv` | `formal_dataset`, `formal_citation`, **`formal_envelope`** quoting existing **`[acceptance]`** bounds or an explicit Boundary/adversarial test path — never invented tolerances |
| **Literature** | `literature://<slug>` | `formal_citation`, **`formal_form`** (closed-form or tensor-graph summary) |
| **NONE** | `NONE` | **`formal_anchor_rationale`** (IO boundary, trivial accessor, auxiliary objective, placeholder dispatcher, …) |

Retired: **`Library`** as a catch-all — replaced by **NONE**, **Empirical**, or **Literature** per symbol.

## Representative classified symbols

| Symbol | Kind | Location | Status | Anchor | Dataset / citation / envelope (abbrev.) |
|--------|------|----------|--------|--------|-------------------------------------------|
| `ChemoWaterEngine` | struct | `physics/chemo_water.rs` | Mechanised | `Powers.lean#PowersState` | — |
| `ThermoEngine::compute_heat_rate` | fn | `physics/thermo.rs` | Mechanised | `Helmholtz.lean#ψAntitoneHelmholtz` | — |
| `TransportEngine::compute_chloride_diffusivity` | fn | `physics/transport.rs` | Mechanised | `MeasurementCost.lean#zero_info_zero_energy` | — |
| `any_bundled_profile_covers_scalars` | fn | `calibration.rs` | Mechanised | `RegimeSoundness.lean#warnings_empty_iff_in_regime` | — |
| `ColloidalEngine` | struct | `physics/colloidal.rs` | Empirical | `empirical://datasets/dataset_d1.csv` | `uci_concrete_yeh_1998`; envelope cites **`uci_d1`** `[acceptance]` + adversarial harness |
| `FiberEngine` | struct | `physics/fiber.rs` | Empirical | `dataset_uhpc.csv` | Boundary **`uhpc`** envelope + adversarial / CSV pairing |
| `compute_packing_density` | fn | `physics/packing.rs` | Literature | `literature://Andreasen-Andersen-1930-Fuller-curve` | Andreasen & Andersen (1930); parabolic CPM proxy form |
| `yield_stress_pa` | fn | `homogeneous.rs` | Literature | Roussel / Château–Ovarlez rheology closure | explicit tensor-free formula line |
| `compute_cost` | fn | `physics/cost.rs` | NONE | — | auxiliary economic dot-product rationale |

Full inventory: browse `src/**/*.rs` doc comments; CI fails on any missing or malformed block.

### Future formal links (cross-repo / deferred)

- **Adjoint / terminal gradient:** `lean://umst-formal/Lean/Adjoint.lean#adjoint_recovers_gradient` — belongs with **`umst-manifold`** adjoint sensitivities, not TOML serde structs.
- **DEC / graph Laplacian:** `lean://umst-formal/Lean/DEC.lean#laplacian_row_sum_zero` — target manifold **`physics::laplacian`** once anchor-audited.
- **Jennings monotone strength:** `lean://umst-formal/Lean/JenningsGelSpace.lean#jennings_strength_monotone` — pending **`JenningsGelSpace`** homogeneous branch (**TODO_FORMAL** on `powers_compressive_strength_mpa`).

### Empirical modules with plausible Mechanised successors

| Empirical surface | Candidate Lean witness (future `umst-formal` work) |
|-------------------|-----------------------------------------------------|
| DLVO / colloidal stability | Gate-layer admissibility lemmas on interaction potentials (spec TBD) |
| Rheology / printability (Roussel) | Order-statistics or regime envelopes tying yield stress to measurement bands |
| Transport porosity / chloride | `MeasurementCost` / mass-balance portfolio already partially linked |
| Creep / shrinkage / fracture empirical closures | Stress-path **Structural** interfaces — dedicated lemmas not yet exported |

*Total **`pub`** symbols remain lint-covered; this document is policy + highlights — not a second SSOT.*
