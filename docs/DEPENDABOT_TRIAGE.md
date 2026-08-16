SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
# Dependabot triage — concrete cartridge

**Status (2026-06-10):** Major bumps deferred via `.github/dependabot.yml` ignore rules.

| Dependency | Reason |
|------------|--------|
| `burn` / `burn-ndarray` 0.21 | pinned `=0.13.2` for manifold gate parity |
| `bincode` 3.x | workspace pinned `=2.0.0-rc.3` |
| `rand` 0.10 | API drift |
| `jsonschema` 0.46 | major API churn |
| `docker/build-push-action` 7.x | major; Dockerfile workflow unchanged |
| `docker/login-action` 4.x | major |
| `actions/setup-python` 6.x | major; CI pins SHA + stable toolchain |
| `actions/checkout` 6.x | major; workflows SHA-pinned to v4 commit |
| `sha2` 0.11 | same hybrid-array finalize break as manifold |
| `toml` 1.x | major API churn |
| `pyo3` / `pyo3-build-config` 0.28 | major; wheel ABI wave deferred |

Revisit after `umst-manifold` burn/bincode upgrade wave completes.

## O6 triage (2026-06-24)

**Known alerts:** ~10 open Dependabot alerts on `umst-concrete-cartridge` (open
`dependabot/*` branches include `bincode-eq-3.0.0`, `burn-0.20.1`, `burn-ndarray-0.21.0`,
`jsonschema-0.46`, `rand-0.10`, `actions/setup-python-6`, `docker/build-push-action-7`,
`docker/login-action-4`).

**Automation state:** `.github/dependabot.yml` enables `cargo` (weekly) + `github-actions`
(monthly) ecosystem updates. Non-major updates are now **grouped** (`cargo-minor-patch`,
`actions-minor-patch`) into a single PR each to cut review noise; known-incompatible
majors stay in the `ignore` list above.

**Version bumps deferred:** Actual `Cargo.toml` version bumps are **not** performed in this
pass. Applying and validating a bump requires building with the pinned `rustc 1.88`
toolchain, which is unavailable in the Ops/cold environment. Bumps are deferred to a
**build-capable worker** who can run `cargo build && cargo test` + `mcp_smoke` before
merging each (grouped) Dependabot PR. The major bumps above remain intentionally pinned per
the table; revisit after the `umst-manifold` burn/bincode upgrade wave.
