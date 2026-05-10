<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Validation

This document is the public record of which constitutive modules have been
validated against published experimental data, on what dataset, and to
what tolerance. Every row is backed by an automated regression test in
[`tests/`](../tests). Where validation is still in progress, we say so
explicitly rather than implying coverage we have not yet achieved.

Each constitutive module also carries a Rust-doc **`formal_status`** bucket (**Mechanised**, **Structural**, **Empirical**, **Literature**, or **NONE**) lint-tested in CI; see [`docs/PROOF-STATUS.md`](PROOF-STATUS.md) for the authoritative per-symbol breakdown (sorted file paths). Empirical rows below cross-link the headline **[acceptance]** gate when a Contract profile owns one.

## Validation envelope

| Module | formal_status | Dataset | Quantity | Tolerance | Test | Status |
|--------|---------------|---------|----------|-----------|------|--------|
| `hydration` | Mechanised | Powers (1948) isothermal calorimetry, OPC | DoH α(t), $w/c \in [0.30, 0.60]$ | $\le 5\%$ MAE | [`tests/hydration.rs::powers_doh_envelope`](../tests/hydration.rs) | passing |
| `chemo_water` | Mechanised | Powers–Brownyard (1946) | $w_n / c$, $w_g / c$ | $\le 3\%$ MAE | [`tests/chemo_water.rs::powers_brownyard`](../tests/chemo_water.rs) | passing |
| `set_time` | Mechanised | ASTM C191 round-robin | initial / final set | $\le 30$ min | [`tests/set_time.rs::astm_c191`](../tests/set_time.rs) | passing |
| `rheology` | Empirical | Roussel (2006) slump-test corpus | $\tau_y$, $\eta_p$ | $\le 10\%$ relative | [`tests/rheology.rs::roussel_slump`](../tests/rheology.rs) | passing |
| `printability` | Empirical | Roussel (2018) buildability bench | $\tau_y$ at failure | $\le 15\%$ relative | [`tests/printability.rs::roussel_buildability`](../tests/printability.rs) | passing |
| `strength` | Mechanised | Jennings (2008) CM-II validation | $f_c$ at 28 d | $\le 3$ MPa | [`tests/strength.rs::cm_ii_28d`](../tests/strength.rs) | passing |
| `fracture` | Literature | Ulm & Coussy (2003) | $K_{Ic}$ at 28 d | $\le 0.2\ \mathrm{MPa\sqrt{m}}$ | [`tests/fracture.rs::ulm_28d`](../tests/fracture.rs) | passing |
| `creep` | Empirical | RILEM B4 calibration set | $J(t,t')$ | $\le 15\%$ relative | [`tests/creep.rs::rilem_b4`](../tests/creep.rs) | passing |
| `shrinkage` | Empirical | Bažant–Baweja calibration set | $\varepsilon_{\mathrm{sh}}^{\infty}$ | $\le 15\%$ relative | [`tests/shrinkage.rs::bazant_baweja`](../tests/shrinkage.rs) | passing |
| `freeze_thaw` | Empirical | ASTM C666 round-robin | mass loss trend | qualitative | [`tests/freeze_thaw.rs::astm_c666_trend`](../tests/freeze_thaw.rs) | trend reproduced; absolute calibration in progress |
| `transport` | Mechanised (chloride diffusivity) | Tang–Nilsson migration | $D_{\mathrm{Cl}}$ | $\le 25\%$ relative | [`tests/transport.rs::tang_nilsson`](../tests/transport.rs) | passing |
| `nano`, `colloidal`, `porosity`, `itz`, `packing`, `fiber`, `polymer`, `self_heal`, `thermo`, `sustainability`, `cost` | mixed (see PROOF-STATUS) | — | — | — | — | implemented; published-dataset validation in progress |
| Homogeneous Powers lift (`src/homogeneous.rs`) | mixed | Bundled CSV mirrors under `datasets/` for **`[contract].verification_status = "Contract"`** profiles only | compressive strength $f_c^\prime$ vs observed | Per-profile **`[acceptance]`** in TOML (MAE / RMSE / $R^2$). Example: [`calibration/profiles/uci_d1.v1.toml`](../calibration/profiles/uci_d1.v1.toml) **`[acceptance]`**. **Boundary** profiles (`uhpc`, `selfheal`, …) omit `[acceptance]` — no widened bounds | [`tests/calibration/dataset_metrics.rs`](../tests/calibration/dataset_metrics.rs) | passing with `cargo test --features cli` |
| Single source of truth row counts | NONE | `datasets/*.csv` vs `docs/SSOT.json` | line counts (data rows) | exact equality on sum and per file | [`tests/calibration/ssot_row_counts.rs`](../tests/calibration/ssot_row_counts.rs) | passing |
| Tensor engine adversarial guards | NONE | `burn` engines in `src/physics/` | finite outputs; order-of-magnitude bands for DLVO, YODEL, thermo proxy, chloride diffusivity, printability, ITZ | regression | [`tests/realism/adversarial_physics.rs`](../tests/realism/adversarial_physics.rs) | passing |
| `formal_status` documentation ledger | NONE | `src/**/*.rs` doc lines | bucket counts | snapshot file | [`tests/proof_status_doc.rs`](../tests/proof_status_doc.rs) + [`docs/PROOF-STATUS.md`](../docs/PROOF-STATUS.md) | passing |
| Live `umst` wire + certify | mixed | N/A (binary) | `result.v2` fields; regime `warnings`; certify JSON incl. `formal_status` | See [`tests/cli/public_contract.rs`](../tests/cli/public_contract.rs) (acceptance checks 7–10) | `cli_public_contract` | passing with `cargo test --features cli` |

## Empirical modules vs acceptance gates

Tensor engines marked **`formal_status: Empirical`** carry `empirical://datasets/*.csv` anchors and an explicit **`formal_envelope`** line in Rust doc comments. That envelope must either:

1. Quote the headline strength bounds from the paired **Contract** profile’s **`[acceptance]`** block (for example **`uci_d1.v1.toml`** for OPC-row modules tied to `dataset_d1.csv`), or
2. State that the bundled profile is **Boundary** (no `[acceptance]`) and point at [`tests/realism/adversarial_physics.rs`](../tests/realism/adversarial_physics.rs) (and any module-specific regression cited in the validation table).

Cross-check: [`tests/calibration/dataset_metrics.rs`](../tests/calibration/dataset_metrics.rs) enforces Contract CSV metrics; adversarial coverage is described in the realism test module.

The validation envelope is deliberately narrow. We choose a small number
of canonical datasets per module, document the tolerance, and refuse to
claim coverage outside it.

## How to reproduce

```bash
git clone https://github.com/tytolabs/umst-manifold
git clone https://github.com/tytolabs/umst-concrete-cartridge
cd umst-concrete-cartridge
cargo test --no-default-features --release
cargo test --features cli --release
```

The deep calibration / CLI integration tests (dataset metrics, regime warnings, JSON schema round-trips) require **`--features cli`**. Formal-anchor documentation is enforced by **`cargo test --test formal_anchors`** (no extra feature flags). Row-count SSOT and tensor realism harnesses run under the same default integration suite; use **`cargo test --test proof_status_doc`** for the `formal_status` Markdown snapshot.

CI runs the full suite on every push and pull request — see
[`.github/workflows/rust.yml`](../.github/workflows/rust.yml).

## Worked example: Powers degree-of-hydration curve

[`examples/hydration_simulation.rs`](../examples/hydration_simulation.rs)
reproduces the Powers (1948) degree-of-hydration curve for $w/c = 0.40$
ordinary portland cement at $T = 293\ \mathrm{K}$, isothermal. The
example prints DoH at 1 d, 7 d, 28 d, and 90 d, and the test
`tests/hydration.rs::powers_doh_envelope` asserts that the four values
agree with the Powers curve to within $5\%$ MAE.

## Out-of-envelope behaviour

Inputs outside the validation envelope (extreme $w/c$, exotic
SCM ratios, sub-zero temperatures outside `freeze_thaw`'s window) may
return values, but those values are not validated and must not be cited
as predictive without further calibration. The
`PhysicalResult::safety_margin` field is the appropriate signal that
the cartridge is being driven into untested territory; downstream
agents should treat a small or negative `safety_margin` as a request
for human review.

## Contributing a new validation row

When a contributor adds a new constitutive module or refines an
existing one, they must:

1. Cite the experimental dataset (DOI or full reference).
2. Add a regression test that loads the dataset and asserts a stated
   tolerance.
3. Add a row to the table above with module, dataset, quantity,
   tolerance, test path, and status.
4. Mention the row in the pull-request description.

See [CONTRIBUTING.md](../CONTRIBUTING.md#physics-changes).
