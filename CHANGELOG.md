<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — 2026-05-07

### Added
- Initial public release of the UMST Concrete Cartridge.
- `core::ConcreteCartridge` — implements
  `umst_manifold::core::traits::IScienceCartridge` over the manifold's
  cellular sheaf.
- 22 constitutive modules under `src/physics/`:
  - **Hydration & chemistry:** `hydration`, `chemo_water`, `set_time`, `thermo`.
  - **Microstructure:** `nano`, `colloidal`, `porosity`, `itz`, `packing`.
  - **Rheology & printing:** `rheology` (Chateau–Ovarlez / YODEL),
    `printability` (Roussel constraints).
  - **Mechanics:** `strength` (Jennings CM-II), `fracture` (Ulm
    micromechanics), `creep` (Bažant B4), `shrinkage`, `fiber`, `polymer`.
  - **Durability:** `freeze_thaw`, `transport`, `self_heal`.
  - **Sustainability and economics:** `sustainability` (embodied CO₂),
    `cost`.
- Documentation: README, `docs/Constitutive-Equations.md`, `docs/Validation.md`.
- Worked example: `examples/hydration_simulation.rs`.
- CI: `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo doc -D warnings`
  with sibling-checkout of `umst-manifold`.
- Governance: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1),
  `SECURITY.md`, `CITATION.cff`, issue and pull-request templates, dependabot.

[Unreleased]: https://github.com/tytolabs/umst-concrete-cartridge/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/tytolabs/umst-concrete-cartridge/releases/tag/v0.1.0
