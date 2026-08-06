<!-- SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Formal anchors (`src/`)

Every **`pub`** function, struct, enum, trait, type alias, constant, and selected re-exports in `src/**/*.rs` carries a formal documentation block enforced by **`cargo test --test formal_anchors`**.

## Bucket-count summary

Precise per-symbol counts and tables are regenerated into [`docs/PROOF-STATUS.md`](PROOF-STATUS.md) from `tests/proof_status_doc.rs`. Consult that file for the current ledger.

## Five-status taxonomy

The cartridge classifies each public Rust symbol into exactly one of five buckets:

- **Mechanised** — a cited lemma in `umst-formal/Lean/` (`lean://umst-formal/Lean/<File>.lean#<lemma>`) witnesses the documented property; `formal_axioms` is **`NONE`** or **`physicalSecondLaw`** only.

- **Structural** — the guarantee is carried by Rust’s type system (`formal_anchor: STRUCTURAL`): type-state, exhaustive pattern matching, newtype invariants, serde routing, or functor-shaped CLI orchestration (`formal_anchor_rationale` names which feature).

- **Empirical** — curve fits or calibrated closures tied to bundled CSV / profile data (`empirical://datasets/<csv>.csv`); requires **`formal_dataset`**, **`formal_citation`**, and **`formal_envelope`** (profile **`[acceptance]`** path or an explicit regression test).

- **Literature** — direct transcription of a published equation or convention (`literature://<author-year-shortform>`); requires **`formal_citation`** and **`formal_form`** (one-line LaTeX-like statement).

- **NONE** — true non-claims: IO boundaries, glue, trivial accessors, auxiliary objectives (`formal_anchor: NONE` + **`formal_anchor_rationale`**). Rationales must **not** contain the boilerplate substrings “Differentiable training” or “training pathway”.

**Retired:** **`Library`** (over-broad bucket) and using **`Boundary`** as a Rust `formal_status` — **`Boundary`** remains a **`verification_status`** field inside calibration profile TOML only.

## Required doc grammar (by status)

| Status | `formal_anchor` | Required lines |
|--------|-----------------|----------------|
| Mechanised | `lean://umst-formal/Lean/...#lemma` | `catalog_id` (manifold slug per `docs/FORMAL_GROUNDING_AUDIT.md`); `formal_axioms` ∈ {`NONE`, `physicalSecondLaw`} |
| Structural | `STRUCTURAL` | `formal_anchor_rationale` |
| Empirical | `empirical://datasets/<file>.csv` | `formal_dataset`, `formal_citation`, `formal_envelope` |
| Literature | `literature://<slug>` | `formal_citation`, `formal_form` |
| NONE | `NONE` | `formal_anchor_rationale` (forbidden boilerplate substrings above) |

## Representative classified symbols

| Symbol | Kind | Location | Status | Anchor / note |
|--------|------|----------|--------|----------------|
| `ChemoWaterEngine` | struct | `physics/chemo_water.rs` | Mechanised | `Powers.lean#PowersState` |
| `ThermoEngine::compute_heat_rate` | fn | `physics/thermo.rs` | Mechanised | `Helmholtz.lean#ψAntitoneHelmholtz` |
| `TransportEngine::compute_chloride_diffusivity` | fn | `physics/transport.rs` | Mechanised | `MeasurementCost.lean#zero_info_zero_energy` |
| `any_bundled_profile_covers_scalars` | fn | `calibration.rs` | Mechanised | `RegimeSoundness.lean#warnings_empty_iff_in_regime` |
| `predict` | fn | `cli/mod.rs` | Structural | Natural transformation φ ∘ F ∘ ψ |
| `ColloidalEngine` | struct | `physics/colloidal.rs` | Empirical | DLVO envelope + dataset citation |
| `FiberEngine` | struct | `physics/fiber.rs` | Literature | Naaman (2006) pullout / bridging form |
| `compute_packing_density` | fn | `physics/packing.rs` | Literature | Andreasen & Andersen (1930) grading form |
| `yield_stress_pa` | fn | `homogeneous.rs` | Empirical | Roussel + Château–Ovarlez; `tests/printability.rs` |
| `compute_cost` | fn | `physics/cost.rs` | NONE | Auxiliary objective rationale |

Full inventory: [`docs/PROOF-STATUS.md`](PROOF-STATUS.md) (sorted, regenerated).

### Future formal links (cross-repo / open)

- **Adjoint / terminal gradient:** `lean://umst-formal/Lean/Adjoint.lean#adjoint_recovers_gradient` — belongs with **`umst-manifold`** adjoint sensitivities, not TOML serde structs.
- **DEC / graph Laplacian:** `lean://umst-formal/Lean/DEC.lean#laplacian_row_sum_zero` — target manifold **`physics::laplacian`** once anchor-audited.
- **Jennings homogeneous branch:** `lean://umst-formal/Lean/JenningsGelSpace.lean#jennings_strength_monotone` — tensor **`physics/hydration.rs`** already cites this lemma; homogeneous dispatch still returns `JenningsNotImplemented` until CC-P-JENNINGS (board: `archived/residuals/misc-outputs-tmp/JENNINGS_RESIDUAL_2252.md` TODO-M3-002). Tensor CM-II `compute_strength_jennings` is a separate micromechanics surface and does not close this item.

### Empirical modules with highest-leverage Mechanised candidates (v0.2)

| Empirical module / surface | Rationale for a future flip |
|----------------------------|----------------------------|
| **DLVO / `colloidal`** | Interaction-potential admissibility lemmas on the gate layer (spec TBD). |
| **Bažant B4 creep / `creep`** | Dedicated viscoelastic compliance bounds once a lemma corpus exists. |
| **Roussel printability / `printability`** | Tie yield/buildability constraints to measurement-band or regime-soundness lemmas. |

*Policy + highlights only — `PROOF-STATUS.md` and `src/**/*.rs` comments remain the SSOT for counts and per-symbol lines.*
