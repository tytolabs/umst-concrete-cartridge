# Solver status (cartridge mirror, v0.4)

**Authoritative table:** [`umst-manifold/docs/Solver-Status.md`](../../umst-manifold/docs/Solver-Status.md) in a sibling checkout. This file mirrors manifold docs for cartridge-side deep links and the Striatus pipeline; edit manifold first, then refresh here.

**Verification index (manifold):** [`VERIFICATION_SCOPE_INDEX.md`](../../umst-manifold/docs/VERIFICATION_SCOPE_INDEX.md).

**PROOF-STATUS (different stub contracts):** manifold [`PROOF-STATUS.md`](../../umst-manifold/docs/PROOF-STATUS.md) is a short solver→lane index (Track J3); this repo's [`PROOF-STATUS.md`](PROOF-STATUS.md) is generated from Rust `#[doc]` formal blocks (`proof_status_doc`).

## Pending registry (canonical sources)

**YAML close-out rows (authoritative):** [`PENDING_GAPS_PLAIN.md`](../../umst-manifold/docs/PENDING_GAPS_PLAIN.md) — update this section when that file’s `id` / `status` fields change.

**Swarm dispatch:** waves **A–E**, task IDs, and operator hygiene are tracked in [`PENDING_GAPS_PLAIN.md`](../../umst-manifold/docs/PENDING_GAPS_PLAIN.md); avoid duplicating long bodies in `README.md`.

### P0 runbook — `shell_topology_rib_pattern_full_v04` (B6, honest)

**Purpose:** one Striatus-scale **B6** proof attempt (**40×40×4**, Burn seed **42**); default CI remains **`shell_topology_rib_pattern_quick`**.

**Environment — 200 Adam outers:** **`UMST_SHELL_RIB_PATTERN=1`** (required). Use **`UMST_SHELL_RIB_FULL_ITERS=200`** or omit it — the harness **defaults to 200** (clamped **1…200**). Values **< 200** are **smoke only**: the test **skips** greyness / planar-variance / compliance-ratio gates and only checks finite metrics + a loose VF band.

**Command (`--release` must sit on the `cargo test` argv, before `--`):**

```bash
cd umst-concrete-cartridge
export UMST_SHELL_RIB_PATTERN=1
export UMST_SHELL_RIB_FULL_ITERS=200   # optional; default 200 when unset
cargo test -p umst-concrete-cartridge --test shell_topology_rib_pattern \
  --features solver-experimental shell_topology_rib_pattern_full_v04 --release -- --ignored
```

Uniform roof (match **`optimize_shell_3d`** with ramp off): prefix with **`UMST_SHELL_ROOF_RAMP=0`**. Default when unset: x-ramp on at **`UMST_SHELL_ROOF_RAMP_STRENGTH`** (alias **`UMST_SHELL_ROOF_RAMP_F`**, default **0.2**). **Self-weight:** default **on** (`UMST_SHELL_SELF_WEIGHT` unset); set **`UMST_SHELL_SELF_WEIGHT=0`** to disable gravity. **Discretization audit (before 200-outer):** `cargo test -p umst-manifold --features mechanics-voigt-cauchy --test mechanics_analytic uniform_rho_q1_hex_compliance_vs_kirchhoff_ssss_audit --release -- --nocapture` — read **`stiff_bias_pct`** in the VERIFY line. Final **`acceptance diag`** / pre-gate line logs ramp flag, **F**, **`vf_final`**, **`vf_err`**, and **`z_rho_mean=[…]`** (mean ρ per z-layer).

**200-outer verdict (pre-registered):** if **greyness + c1 pass** but **xy_var fails** with **sandwich z-profile** → insufficient load asymmetry, not optimizer bug (tune ramp **F** before `λ_xy`). If **xy_var fails** with **greyness/c1 also failing** → report as optimizer/volume path issue. **Greyness ≪1 at β≲2** implies DensityNet saturation, not Heaviside; always check **`vf_err`** after η-bisection.

**Greyness target:** B6 asserts **volume**-mean **`mean(4ρ(1−ρ)) < 0.15`** on the final **post–volume-projection** nodal **ρ** (`crates/umst-concrete-cartridge/tests/shell_topology_rib_pattern.rs`, **`greyness_mean`** on **`last_rho`**).

**B6 attempt log (honest rows):**

| Date | Env (highlights) | `greyness_pre_vol` @ outer 40 | `greyness_post_vol` @ outer 40 | Gate greyness | Result |
|------|------------------|-------------------------------|--------------------------------|---------------|--------|
| 2026-05-11 | 200 outers, λ-shift `VolumeProjection` | — | ~0.51 | 0.51 | **FAIL** (historical) |
| 2026-06-10 | smoke 40, `UMST_SHELL_VOL_BISECT=0`, `UMST_SHELL_HELM=0`, `UMST_SHELL_METRICS=1` | 0.993 | 0.510 | 0.510 | smoke — **H1 REFUTED**: pre_vol≈1 ⇒ uniform ρ≈0.5 **before** projection; post 0.510=`4·vf·(1−vf)` degenerate constant field |
| 2026-06-10 | smoke 40, `UMST_SHELL_VOL_BISECT=1`, `UMST_SHELL_HELM=0`, `UMST_SHELL_METRICS=1` | 0.993 | 0.510 | 0.510 | smoke — identical post_vol ⇒ projector choice irrelevant on constant field |
| 2026-06-10 | **aborted** @ outer 20, 200 planned, `UMST_SHELL_HELM=0`, η-bisect on | 1.000 | 0.844 | — | **ABORT** — pre_vol≥0.95, Adam NaN skips; would land ~0.51 / `xy_var≈0` (same as 2026-05-11) |
| 2026-06-10 | **smoke** 1-outer H4 diag pre-fix, `UMST_SHELL_H4_DIAG=1`, `b6-h4-diagnosis`, `--release` | 0.954 | — | — | **FAIL** VF band; Step C **green**; **`grad_l2=0`** (`INIT_SCALE=0.05` + uniform sens cancellation) |
| 2026-06-10 | **smoke** 1-outer H4 diag post-fix, `INIT_SCALE=1.0`, `--release` | 0.940 | — | **0.934** | Step C **green** (`eq_rel≈9.7×10⁻⁵`); **`grad_l2≈0.934`**; VF band still fails (1-outer smoke) |
| 2026-06-10 | **smoke 5-outer** post-volume-fix + **self-weight ON**, in-loop AL, terminal η @ β=32, `a5fe90c`, `--release` | 0.510 | 0.293 (terminal) | 0.293 | **PASS** smoke gate — `vf=0.1501`, `eq_rel≈9.8×10⁻⁵` all outers, `max_grad_l2=6.28`, `beta_last=1.087` (200-outer schedule), `xy_var=0.054`; Kirchhoff **`stiff_bias_pct≈34%`** @ 16²×4 (discretization caveat for §9 compliance) |
| 2026-06-11 | **A′ smoke 20-outer** run 2, `SELF_WEIGHT=0`, AL `μ=64` uncapped, `b6-h4-diagnosis` WIP | — | — | 0.622 | **FAIL** smoke — vf overshoot 0.099→0.193 (`μ` escalated to 8M); `max_grad_l2=363k`; greyness↓ OK; `xy_var@18+≈0.001` OK; c1 gate |
| 2026-06-11 | **A′ smoke 20-outer** run 3γ=1.5, `SELF_WEIGHT=0`, AL `μ=32` cap 4096 | — | — | 0.771 | **FAIL** smoke vf band — vf ring 0.091→0.261 (`err=+0.111`); `max_grad_l2=189` (bounded); greyness↓ then ↑; `xy_var@18+≈0.00018` |
| 2026-06-11 | **A′ smoke 20-outer** run 4, `SELF_WEIGHT=0`, AL `μ=32` `γ=1.2` | — | — | 0.768 | **FAIL** vf band — vf 0.098→0.260 (`err=+0.110`); AL too weak post-outer-10 |
| 2026-06-11 | **A′ smoke 20-outer** run 5 (accept), `SELF_WEIGHT=0`, AL `μ=96` `γ=1.2` `τ=0.85` cap 4096, `c789b5d`, `--release` | 0.625 | — | 167 | **PASS** AL-shaped health — vf damped ring 0.484→0.055@12→0.194@20 (`err=+0.044` in band); `max_grad_l2=167` bounded (no run-2 363k); greyness 0.999→0.625↓; `xy_var@18+=0.000198`; `c1` 26.7→6.8 below peak; `eq_rel≈9.9×10⁻⁵`. Log: `/tmp/b6-aprime-20outer.log` |
| 2026-06-11 | **Suspect 2** FD adjoint 9×8×2, `adjoint_q1_hex_self_weight_fd`, `umst-manifold` `140483d` | — | — | — | **PASS** — central FD ε=2×10⁻³, 10 nodes; **documented bound rel ≤2.5%** ON/OFF (worst ON **2.01%** @ nid=186; f32 FD floor, not analytic exact); void deciles all sens negative (no spurious “add mass” sign) |
| 2026-06-11 | **Suspect 2 smoke 20-outer**, `SELF_WEIGHT=1`, AL knobs as A′, `96c537a` + `140483d`, `--release` | 0.625 | — | 163 | **PASS** smoke A′ — same AL health as run 5; `xy_var@18+=0.000198`; `c1` peak→6.7 (non-monotone OK under design-dependent load) |

**A′ in-loop volume AL knobs (`AugmentedLagrangianVolume`, `shell_topology_rib_pattern.rs`):**

| Knob | Value | Grounding |
|------|-------|-----------|
| `μ` (initial) | **96** | Bertsekas AL initial penalty scale ([Bertsekas 1996](https://doi.org/10.1887/978-1-881529-07-6), Ch. 2). **Fit during A′**, admissible **32–128**: run 2 `μ=64` uncapped → `grad_l2` 363k; run 4 `μ=32` `γ=1.2` → vf rebound 0.098→0.260 (under-penalized); run 5 `μ=96` `γ=1.2` → bounded grads + vf settles in band. |
| `γ` (μ growth) | **1.2** | Bertsekas adaptive penalty increase when `\|g_k\| > τ·\|g_{k-1}\|` ([Bertsekas 1996](https://doi.org/10.1887/978-1-881529-07-6), Ch. 2). **Fit during A′**, admissible **1.1–1.5**: damp **before** touching `μ` — run 2 `γ=2` underdamped vf 0.099→0.193; run 3 `γ=1.5` ring 0.091→0.261; **1.2** first stable damping (run 5). |
| `τ` (sufficient-decrease) | **0.85** | Bertsekas “retain `μ` if constraint residual decreases adequately” ([Bertsekas 1996](https://doi.org/10.1887/978-1-881529-07-6), Ch. 2). **Fit during A′**, admissible **0.75–0.9**: `topology.rs` default **0.5** doubles `μ` too often at B6 N (run 2 evidence). Held at 0.85 for runs 3–5. |
| `μ` cap | **4096** | Engineering ceiling (not in Bertsekas). **Fit during A′**, admissible **2048–8192**: prevents f32 blow-up when `update_period=1` and early `vf` residual is large (0.48→0.15). Run 2 without cap hit 8M. |
| `update_period` | **1** | B6 smoke: update λ every outer. Default **10** in `topology.rs` leaves `λ=0` for first smoke pass (run 1 false-negative guard). |

**Pending gate definition (B6 compliance baseline):**

| id | status | proposal |
|----|--------|----------|
| `b6-c0-uniform-at-target-vf` | **open** | 200-outer gate `c1 < 0.6·c0` uses **c0 = outer-1 compliance at vf≈0.48** (init DensityNet field). Meaningless once AL drives vf→0.15. **Proposed:** `c0 := compliance(uniform ρ = target_vf)` — same material budget, no design; gate = “optimized layout ≥40% stiffer than smeared solid at target vf” ([Bendsøe & Sigmund 2003](https://doi.org/10.1007/978-3-662-05086-6), Ch. 1 compliance minimization). Smoke A′ uses `c1 < c1_peak` instead; full gate unchanged until this row closes. |

**H4 diagnosis (2026-06-10, `UMST_SHELL_H4_DIAG=1`):** primary hypothesis — compliance adjoint / sensitivity pathology at Striatus N. Instrumentation logs per-outer `sens_l2`, `sens_var`, `pcg_iter`, `pcg_rel_res`, `eq_rel_res`, `xy_var`, `adam_skipped` (see harness). **H1 REFUTED:** `greyness_pre_vol≈0.99` is uniform ρ≈0.5 **before** projection; post 0.510=`4·vf·(1−vf)` is degenerate constant-field signature (identical under λ-shift and η-bisect). **λ_g forbidden** for this failure mode.

| Scale | outer | `rho_raw` | `pcg_rel_res` | `eq_rel_res` | `xy_var` | `adam_skip` |
|-------|-------|-----------|---------------|--------------|----------|-------------|
| quick 9×8×2 (pre-H4; solver convergence **never asserted** before 2026-06-10) | 24/24 | [0.489,0.493] | **1.1e9** | **1.1e9** | 3.1e-5 | 0 |
| full 40×40×4 | 20/20 | [0.501,0.501] | **7.1e9** | **7.1e9** | 0 | 1/20 |

First divergent metric at Striatus scale: **forward PCG never meets tolerance** (`pcg_iter` pinned at cap; `pcg_rel_res` ≫ 1). Discrete `sens_l2` explodes accordingly; field stays uniform → no rib signal.

**H4 fix sequence (2026-06-10, in progress on `b6-h4-diagnosis`):**

| Step | Status | Notes |
|------|--------|-------|
| A — operator probes | **verified** | `bar_network_operator_step_a`: symmetry, PSD, 2-node rel_res metric, 9-node axial manufactured |
| B — scale / solve lane | **resolved (Q1 hex)** | bar-network mechanism retired; harness uses `AdjointComplianceQ1Hex` |
| C — permanent gate | **green (quick + 1-outer full)** | quick CI + full **40×40×4** 1-outer H4 diag: `eq_rel≈9.7×10⁻⁵`; **`grad_l2≈0.93`** after init-scale fix (`UMST_SHELL_INIT_SCALE` default **1.0**, not **0.05**) |
| D — Suspect 2 self-weight adjoint | **verified** | `2uᵀ(∂f/∂ρ)` Bruyneel–Duysinx in `adjoint_q1_hex.rs` (`140483d`); FD property test; 20-outer `SELF_WEIGHT=1` smoke pass (`96c537a`) |
| E — 200-outer B6 acceptance | **pending** | Requires manifold pin bump to `140483d` on remote + `METRICS=1` overnight run; §9 verdict + `b6-c0-uniform-at-target-vf` caveat |

**Roof-traction mechanism probes (2026-06-10, `bar_network_roof_mechanism_probe`, 9×8×2 harness):**

| Probe | Result | Key metrics |
|-------|--------|-------------|
| PROBE 1 — mechanism modes + floor | **mechanism confirmed** | interior column z-slide κ=0; CGLS min-residual floor ρ≈1.0; bar PCG rel_res **0.937** @ 2000 iters (`use_preconditioner=false`) |
| PROBE 1b — perimeter column point load | **well-posed** | pcg_rel≈0, 2 iters |
| PROBE 2 — f32/f64 matvec | **pass** | max rel err **5.1×10⁻⁸** |
| PROBE 3 — dense K_ff Cholesky | **singular** | n_free=708; Cholesky fails (min pivot **2.3×10⁻¹⁰**); ρ_cgls≈1.0 vs pcg_obs=0.937 |

**Q1-hex spike (2026-06-10, `q1_hex_harness_roof_spike`, same 9×8×2 / pins / 50 Pa roof):** **GO** — `c0≈3.24×10⁻⁴`, `pcg_rel≈8.4×10⁻⁵`, `eq_rel≈9.6×10⁻⁵` (tol **`HEX_PCG_REL_TOL_F32`** = 10⁻⁴).

**PCG 2×2 bisection (2026-06-10, `q1_hex_pcg_bisect_probe`, 9×8×2, tol 10⁻⁴, Jacobi):**

| Arm | loop | nondim | iters | r_recursive | r_true | verdict |
|-----|------|--------|-------|-------------|--------|---------|
| A | original | OFF | 58 | 8.4×10⁻⁵ | 9.6×10⁻⁵ | baseline — recursive self-report drifts from true |
| B | original | ON | 58 | 9.6×10⁻⁵ | 1.04×10⁻⁴ | **trajectory-identical to A** (`|Δu|∞≈1.6×10⁻¹⁰`) — nondim is exact no-op |
| C | refresh+masked p | OFF | 59 | 8.1×10⁻⁵ | 8.1×10⁻⁵ | rewrite OK at quick scale; r_rec = r_true |
| D | refresh+masked p | ON | 59 | 6.9×10⁻⁵ | 6.9×10⁻⁵ | bundled state; r_rec = r_true |

**PCG iteration budget (`HEX_PCG_MAX_ITER_DEFAULT_STRIATUS=4000`, 2026-06-10):** derived from measured **~1213** iters @ outer 1 on **40×40×4**; **2×** headroom because κ grows as the design develops contrast (outers **11–18** previously hit the **2000** cap with `eq_rel` creeping to **8.9×10⁻⁴**). Override with **`UMST_SHELL_MAX_CG`**.

**Tolerance policy (`q1_hex_elasticity`, 2026-06-10 re-ground):** both lanes **`HEX_PCG_REL_TOL_F32=HEX_PCG_REL_TOL_F64=1e-4`**. Rationale: (i) **sensitivity fidelity** — adjoint compliance gradients are dominated by equilibrium solve error; tightening below the measured κ·ε floor at 40×40×4 buys no gradient signal ([Bendsøe & Sigmund 2003](https://doi.org/10.1007/978-3-662-05086-6), Ch. 1 inexact-solve TO practice); (ii) **attainable floor** — full f64 PCG lane (`eq_rel` binding) still reports true **`rel≈1e-4`** at Striatus N after ~2k iters while recursive self-report can lie by orders of magnitude (arm-A table below); **`1e-6`** overshoots that floor and caused false **`pcg_rel` pass / `eq_rel` fail** on smoke. Step C gates assert **`eq_rel`** against the **lane** tol (f64 harness uses **`HEX_PCG_REL_TOL_F64`**). Production solver: **arm A** (original recursive loop + nondim).

**f64 descent curve (`q1_hex_pcg_descent_probe`, 40×40×4, Jacobi, run-to-10k, tol=1e-4, 2026-06-10):**

| iter | r_recursive | r_true | r_rec/r_true |
|------|-------------|--------|--------------|
| 2000 | 7.94×10⁻⁷ | 7.94×10⁻⁷ | 1.0 |
| 4000 | 5.00×10⁻¹¹ | 5.00×10⁻¹¹ | 1.0 |
| 6000 | 4.96×10⁻¹¹ | 4.96×10⁻¹¹ | 1.0 |
| 8000 | 4.99×10⁻¹¹ | 4.99×10⁻¹¹ | 1.0 |
| 10000 | 5.01×10⁻¹¹ | 5.01×10⁻¹¹ | 1.0 |

**Verdict:** full f64 PCG lane **closes Signal 1** — recursive self-report matches true `eq_rel` at every milestone (contrast arm-A table: recursive 9×10⁻⁵ vs true 0.24). True residual **plateaus below tol by 2k**; flat tail is convergence, not preconditioner stall. **Block-Jacobi (3×3 nodal) deferred** — not the binding bottleneck after f64 lane.

**Unit sanity @ 40×40×4 (`q1_hex_unit_sanity_striatus_n`, 2026-06-10):** `rel = ‖Pr‖/‖Pf‖` with both in **N** (masked nodal force components). **Not** a mixed-unit absolute dressed as relative.

| Arm | tol | iters | ‖Pf‖ [N] | ‖Pr‖ [N] | rel | r_recursive |
|-----|-----|-------|----------|----------|-----|-------------|
| A (original, no nondim) | 1e-4 | 1972 | 19.75 | 4.70 | **2.38×10⁻¹** | 9.3×10⁻⁵ |
| D (refresh+masked, nondim) | 1e-6 | 10000 | 19.75 | 8.37×10⁵ | **4.24×10⁴** | 4.24×10⁴ |

**4.2×10⁴ verdict:** true dimensionless divergence — iterate residual force ~**836 kN** against ~**20 N** RHS (≈4×10⁴× worse than equilibrium). **Incomplete nondim refuted** for that number (`‖Pf‖` matches arm A). **Arm A @ Striatus:** recursive self-report **lies** (9×10⁻⁵ vs true **0.24**) — explains smoke `pcg_rel≈1e-6` / `eq_rel≈0.24` split.

**Harness swap (2026-06-10):** `shell_topology_rib_pattern_{quick,full_v04}` forward+adjoint → **`AdjointComplianceQ1Hex`**; Step C **green** on quick CI. **Perf:** Striatus **40×40×4** only with `--release` (`#[cfg(debug_assertions)]` guard on full harness); mechanism probes stay **9×8×2**; `pre-gate metrics` logs **`pcg_iter_final`**. **Stale debug smoke (discarded):** outer 1/20 hit **`pcg_iter=2000`** cap with **`eq_rel_res≈0.21`** while **`pcg_rel_res≈5×10⁻⁵`** — slow-converging equilibrium residual, not the bar-network **0.94** incompatible-RHS floor. 
**Release smoke 1-outer H4 diag (2026-06-10, `INIT_SCALE=1.0`, cartridge `b7b6782`, manifold `25ed588`):** Step C **pass** (`eq_rel≈9.4×10⁻⁵`); **`grad_l2≈0.861`**; five-criteria smoke **pass** (VF band deferred).
**Release smoke 20-outer (2026-06-10, `UMST_SHELL_RIB_FULL_ITERS=20`, `--release`, Q1-hex lane tol **1e-4**):** five-criteria smoke **PASS** — `last_grad_l2=0.012986`, `max_grad_l2=0.934`, `adam_skipped=0`, `rho_raw=[0.996,0.999]`, `xy_var≈1×10⁻⁶`, `eq_rel≈9.53×10⁻⁵`, `pcg_iter_final≈2285`. **200-outer still blocked:** `vf≈0.996` vs target **0.15**, `xy_var` collapsed, `c1/c0≈0.49` (needs **0.6×** drop). Committed harness @ `b7b6782` without WIP `smoke_subset` logged **`last_grad_l2=0`** on outer 20 (re-run after landing harness fix). **λ_g forbidden.**
**Stale (pre–Q1-hex / wrong tol):** 20-outer @ `MAX_CG=10000` with **`eq_rel≈0.24`** — superseded by f64 lane @ **1e-4**.

**`gates_track_b8` path (Track L / B8 rollup):** boolean **`gates_track_b8_all_pass`** lives in **`notebooks/_artifacts/striatus_shell_v0.4.print_ready.json`** (repo root **`umst-concrete-cartridge/`**). It is emitted by **`notebooks/export_print_ready.py`**; **`notebooks/tests/test_print_ready.py`** (or **`python notebooks/test_print_ready.py`**) reads the same field — **`test_print_ready_track_b8_topology_gates`** **skips** when false unless **`UMST_REQUIRE_B8=1`**. **Profiling:** peak GPU VRAM or unified-memory figures are **not** part of default CI or this status table — cite them only from an explicit profiling task or hardware note. Optional log template: [`VRAM.md`](VRAM.md).

**`gates_track_b8_all_pass` semantics:** in **`notebooks/export_print_ready.py`**, the rollup is exactly **`gate_topo_complexity_b7` ∧ `gate_volume_fraction_mesh_b7` ∧ `gate_density_xy_variance_b8`**. It is **`true` only when all three are `true`**; do **not** hand-edit the rollup (or the three gate booleans) out of sync with the numeric fields — re-run the exporter on a **`final.npy`** / STL that actually meets the thresholds. **`gate_volume_fraction_mesh_b7`** uses **nodal** **`mean(ρ)`** on the optimisation lattice (same band **[0.10, 0.25]** as the brief’s design VF); **`mesh_volume_fraction_in_bbox`** remains a **diagnostic** (marching-cubes watertight volume ÷ mesh AABB — can read **≈1** on thin shells whose AABB is mostly solid).

**CI lint (workspace root, mirror of manifold `solver-status` job):** `python3 umst-manifold/scripts/check_solver_status.py --status-md umst-concrete-cartridge/docs/Solver-Status.md --root umst-manifold --check-paths --check-memo-links --check-statmech-verification-set`
