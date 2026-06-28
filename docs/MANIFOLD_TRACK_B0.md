# Manifold track B0 — OC binarization gate

**Date:** 2026-06-28  
**Harness:** `shell_topology_rib_pattern_full_v04` (`#[ignore]` Striatus 40×40×4)  
**Depends on:** [MANIFOLD_TRACK_D2.md](MANIFOLD_TRACK_D2.md) (η in-loop descent, merged `ff6c567`)

## Problem

D2 won in-loop η descent (`c1_fixed_p3` monotone @ N=200) but **terminal binarization** failed:

- `UMST_SHELL_FIXED_BETA=1` held β=1 → field drifted to vf≈1.0
- Terminal **logit-b @ β_fin=32** could not reach vf=0.15 without destroying layout
- N=200 panic: `striatus_vol_finisher_unreachable`

## Fix — OC λ export (B0)

| Env | Effect |
|-----|--------|
| `UMST_SHELL_VOL_MODE=eta` | In-loop η-bisection on `ρ̃` (unchanged from D2). |
| `UMST_SHELL_EXPORT_VOL=oc` | **Default** when `VOL_MODE=eta`. Terminal: η@β_fin + [`VolumeProjection`] OC λ-shift. **No logit-b fallback.** |
| `UMST_SHELL_EXPORT_VOL=logit` | Regression: legacy logit-b finisher @ β_max. |
| `UMST_SHELL_HEAVISIDE_BETA_MAX=32` | β continuation 1→32 (do **not** set `UMST_SHELL_FIXED_BETA=1` on B0 runs). |
| `UMST_SHELL_BINARIZE_OUTERS=N` | Optional: last N outers force β ramp toward β_max while η stays in-loop. |

**Do not use** `UMST_SHELL_FIXED_BETA=1` on the B0 ship path — it is ignored when `EXPORT_VOL=oc`.

## B0 gate command (authoritative)

```bash
UMST_SHELL_RIB_PATTERN=1 UMST_SHELL_VOL_MODE=eta UMST_SHELL_EXPORT_VOL=oc \
UMST_SHELL_FIXED_P_ACT=3 UMST_SHELL_RIB_FULL_ITERS=200 \
UMST_SHELL_HEAVISIDE_BETA_MAX=32 UMST_SHELL_D1=1 UMST_SHELL_METRICS=1 \
cargo test -p umst-concrete-cartridge --test shell_topology_rib_pattern \
  --features solver-experimental shell_topology_rib_pattern_full_v04 \
  --release -- --ignored --nocapture 2>&1 | tee /tmp/b0-oc-eta-N200.log
```

## Accept criteria

| Check | Threshold |
|-------|-----------|
| Exit | `HYBRID_DONE` / `test result: ok` |
| `vf_export` | 0.15 ± 0.02 |
| `greyness` | < 0.15 |
| `c1_fixed_p3` (acceptance) | ≤ gate (~20.7) |
| Panic | No `striatus_vol_finisher_unreachable` |

## Smoke (N < 200)

When `EXPORT_VOL=oc` and β reached β_max, smoke asserts `greyness < 0.15`.
