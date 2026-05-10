<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Calibration schema `calibration.v1` (TOML)

This directory holds versioned calibration profiles consumed by `umst-concrete-cartridge` via `include_str!` and optional `--profile-file` overrides.

## Sections

| Section | Purpose |
|---------|---------|
| `[meta]` | `name`, `schema` (`calibration.v1`), `material` (e.g. `concrete`), optional `model`, `authors`, `date_fit`, `source_repo` |
| `[provenance]` | `dataset_lift_from`, `prototype_3_sha256` (source JSON), `primary_reference`, `secondary_references[]` |
| `[provenance.formal]` | `anchor` (Lean URI), `status` (`Mechanised` / `Structural` / `Boundary` / `NONE`), `axioms[]`, optional `rationale` |
| `[regime]` | `w_c_min`/`max`, `temperature_k_min`/`max`, `age_hours_min`/`max`, optional `fly_ash_pct_max`, `silica_fume_pct_max`, `slag_pct_max`, `scm_sum_min_pct`, `silica_fume_pct_max_special` |
| `[model]` | `kind` = `powers_gel_space` or `jennings_gel_space` |
| `[parameters.powers_gel_space]` | `s_intrinsic`, `k_slag`, `k_fly_ash`, `k_ref`, `early_boost` (four decimal places; lifted from prototype-3 JSON) |
| `[acceptance]` | Optional `strength_mae_max`, `strength_rmse_max`, `strength_r2_min`, `strength_max_err_max`, formal anchor fields tying metrics to lemmas |
| `[contract]` | `verification_status` = `Contract` or `Boundary` (see prototype `PROTOTYPE3_PLAN.md` and `umst-formal/PROOF-STATUS.md`) |

Formal status vocabulary is aligned with [`umst-formal/PROOF-STATUS.md`](../../umst-formal/PROOF-STATUS.md): **Mechanised** (proved in Lean/Coq/Agda), **Structural** (Rust type/state discipline mirroring Kleisli/gate scaffolding), **Boundary** (limited reference corpus; honest scope), **NONE** (no mechanised claim).

## Prototype JSON fingerprint

Lift constants from `umst-prototype-3/data/calibration/concrete_calibration.json`.

SHA-256 (authoritative at lift time):

`6ca1128a251d66ae3e70782779f8c46f2f29fadf04e379134ad1dff257a704e1`

Cross-references:

- Symbol cross-walk: [`docs/FormalAnchors.md`](../docs/FormalAnchors.md)
- Contract / boundary doctrine: `umst-prototype-3/PROTOTYPE3_PLAN.md` (read-only sibling checkout)
