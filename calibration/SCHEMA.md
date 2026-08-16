SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
<!--
-->

# Calibration schema `calibration.v1` (TOML)

This directory holds versioned calibration profiles consumed by `umst-concrete-cartridge` via `include_str!` and optional `--profile-file` overrides.

## Sections

| Section | Purpose |
|---------|---------|
| `[meta]` | `name`, `schema` (`calibration.v1`), `material` (e.g. `concrete`), optional `model`, `authors`, `date_fit`, `source_repo` |
| `[provenance]` | `dataset_lift_from`, `provenance_sha256` (source JSON), `primary_reference`, `secondary_references[]`, optional Zenodo fields (`zenodo_record`, `zenodo_doi`, `zenodo_url`, `license`, `subset`) for CC-BY corpora |
| `[provenance.formal]` | `anchor` (Lean URI), `status` (`Mechanised` / `Structural` / `Boundary` / `NONE`), `axioms[]`, optional `rationale` |
| `[regime]` | `w_c_min`/`max`, `temperature_k_min`/`max`, `age_hours_min`/`max`, optional `fly_ash_pct_max`, `silica_fume_pct_max`, `slag_pct_max`, `scm_sum_min_pct`, `silica_fume_pct_max_special` |
| `[model]` | `kind` = `powers_gel_space` or `jennings_gel_space` |
| `[parameters.powers_gel_space]` | `s_intrinsic`, `k_slag`, `k_fly_ash`, `k_ref`, `early_boost` (four decimal places; lifted from umst-prototype-2a (Zenodo 18940933) JSON) |
| `[acceptance]` | **Contract** profiles only: `strength_mae_max`, `strength_rmse_max`, `strength_r2_min`, optional `strength_max_err_max`, formal anchor fields. **Omit this section** for **`[contract].verification_status = "Boundary"`** bundles — headline CSV metrics do not apply; regime / CLI behaviour may still use the profile. |
| `[contract]` | `verification_status` = `Contract` (headline dataset acceptance applies) or `Boundary` (out-of-model-scope; no widened metric hacks) |

Formal status vocabulary is aligned with [`umst-formal/PROOF-STATUS.md`](../../umst-formal/PROOF-STATUS.md): **Mechanised** (proved in Lean/Coq/Agda), **Structural** (Rust type/state discipline mirroring Kleisli/gate scaffolding), **Boundary** (limited reference corpus; honest scope), **NONE** (no mechanised claim).

## Prototype JSON fingerprint

Lift constants from `umst-prototype-2a (Zenodo 18940933) — see calibration/SCHEMA.md`.

SHA-256 (authoritative at lift time):

`6ca1128a251d66ae3e70782779f8c46f2f29fadf04e379134ad1dff257a704e1`

Cross-references:

- Symbol cross-walk: [`docs/FormalAnchors.md`](../docs/FormalAnchors.md)
- Contract / boundary doctrine documented in `umst-prototype-2a` (Zenodo 18940933)
