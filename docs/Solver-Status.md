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

Uniform roof (match **`optimize_shell_3d`** with ramp off): prefix the same command with **`UMST_SHELL_ROOF_RAMP=0`**. Harness default when **`UMST_SHELL_ROOF_RAMP` is unset** is the gentle **x** ramp at **`UMST_SHELL_ROOF_RAMP_F`** (default **0.2**).

**Greyness target:** B6 asserts **volume**-mean **`mean(4ρ(1−ρ)) < 0.15`** on the final **post–volume-projection** nodal **ρ** (`crates/umst-concrete-cartridge/tests/shell_topology_rib_pattern.rs`, **`greyness_mean`** on **`last_rho`**).

**`gates_track_b8` path (Track L / B8 rollup):** boolean **`gates_track_b8_all_pass`** lives in **`notebooks/_artifacts/striatus_shell_v0.4.print_ready.json`** (repo root **`umst-concrete-cartridge/`**). It is emitted by **`notebooks/export_print_ready.py`**; **`notebooks/tests/test_print_ready.py`** (or **`python notebooks/test_print_ready.py`**) reads the same field — **`test_print_ready_track_b8_topology_gates`** **skips** when false unless **`UMST_REQUIRE_B8=1`**. **Profiling:** peak GPU VRAM or unified-memory figures are **not** part of default CI or this status table — cite them only from an explicit profiling task or hardware note. Optional log template: [`VRAM.md`](VRAM.md).

**`gates_track_b8_all_pass` semantics:** in **`notebooks/export_print_ready.py`**, the rollup is exactly **`gate_topo_complexity_b7` ∧ `gate_volume_fraction_mesh_b7` ∧ `gate_density_xy_variance_b8`**. It is **`true` only when all three are `true`**; do **not** hand-edit the rollup (or the three gate booleans) out of sync with the numeric fields — re-run the exporter on a **`final.npy`** / STL that actually meets the thresholds. **`gate_volume_fraction_mesh_b7`** uses **nodal** **`mean(ρ)`** on the optimisation lattice (same band **[0.10, 0.25]** as the brief’s design VF); **`mesh_volume_fraction_in_bbox`** remains a **diagnostic** (marching-cubes watertight volume ÷ mesh AABB — can read **≈1** on thin shells whose AABB is mostly solid).

**CI lint (workspace root, mirror of manifold `solver-status` job):** `python3 umst-manifold/scripts/check_solver_status.py --status-md umst-concrete-cartridge/docs/Solver-Status.md --root umst-manifold --check-paths --check-memo-links --check-statmech-verification-set`
