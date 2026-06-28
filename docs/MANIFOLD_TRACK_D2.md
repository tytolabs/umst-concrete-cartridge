# Manifold track D2 — compliance-aware volume projection

**Date:** 2026-06-28  
**Harness:** `shell_topology_rib_pattern_full_v04` (`#[ignore]` Striatus 40×40×4)

## Problem (D1 verdict)

Real-adjoint gradient is FD-correct. Compliance rises **downstream**:

1. **Logit `b`-bisect volume projection** — Δab ≈ +26 per outer (dominant).
2. **Heaviside β-continuation** — Δbc grows as β ramps (outer-to-outer rise in `c1_fixed_p3`).

## Fix — hybrid volume mode

| Env | Effect |
|-----|--------|
| `UMST_SHELL_VOL_MODE=eta` | In-loop: OC η-bisection on `ρ̃` ([`VolumeEtaProjection`]) — preserves layout, enables descent. |
| `UMST_SHELL_VOL_MODE=lambda` | In-loop: OC λ-shift on `ρ̃` (experimental; **not** recommended — Δab still large). |
| *(default)* `logit` | Legacy logit-`b` in-loop (pre-D2 behaviour). |
| `UMST_SHELL_FIXED_BETA=1` | Optional: freeze Heaviside β during smoke experiments. |

**Hybrid export (automatic for `eta` / `lambda`):** terminal **logit-`b` finisher** @ `β_max` runs even on smoke subsets (`UMST_SHELL_RIB_FULL_ITERS` < 200). Acceptance VF gates read `vf_export`; in-loop `c1_fixed_p3` logs track descent on η path.

## Run commands

```bash
# D1 diagnostic (default logit path)
UMST_SHELL_RIB_PATTERN=1 UMST_SHELL_FIXED_P_ACT=3 UMST_SHELL_RIB_FULL_ITERS=10 \
UMST_SHELL_D1=1 UMST_SHELL_METRICS=1 \
cargo test -p umst-concrete-cartridge --test shell_topology_rib_pattern \
  --features solver-experimental shell_topology_rib_pattern_full_v04 --release -- --ignored --nocapture

# D2 hybrid (η in-loop + logit-b export)
UMST_SHELL_VOL_MODE=eta UMST_SHELL_FIXED_P_ACT=3 UMST_SHELL_RIB_FULL_ITERS=10 \
UMST_SHELL_FIXED_BETA=1 UMST_SHELL_D1=1 UMST_SHELL_METRICS=1 \
cargo test ... (same as above)
```

## Evidence logs (MaOS workspace)

| Run | Log | Result |
|-----|-----|--------|
| D1 logit 10-outer | `/tmp/d1-three-point-N10.log` | `c1_fixed_p3` 33.99→40.23 RISING |
| D2 η+fixed-β 10-outer | `/tmp/d2-eta-fixedbeta-N10.log` | in-loop `c1` 2.78→0.26 DESCENDING; VF fail pre-hybrid |
| C2 η+β-ramp 10-outer | `/tmp/c2-eta-beta-N10.log` | in-loop `c1` 2.78→0.27 DESCENDING |
| D2 λ (rejected) | `/tmp/d2-lambda-fixedbeta-N10.log` | `c1` 42.9→44.2 RISING |

## D3 solver

`UMST_SHELL_PRECOND=mg` → WARN + Jacobi fallback at Striatus scale. MG N=60: `rel_residual=1.0` (`/tmp/g3-converge-at-p3-mg-N60.log`).
