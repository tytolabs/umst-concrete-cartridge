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
