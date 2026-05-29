<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Formal grounding audit — `umst-concrete-cartridge`

**Date:** 2026-05-21  
**Scope:** Map cartridge formal-documentation surfaces to manifold `catalog_id`s; verify the predict façade delegates Clausius–Duhem (CD) transition checks to `umst-manifold`; state a scaling pattern for what this repo should vs should not own.

**SSOT references**

| Artifact | Location |
|----------|----------|
| Manifold gate IDs | `umst-manifold/docs/GateUnificationSpec.md` |
| Lean ↔ Rust traceability | `umst-manifold/docs/claims-vs-proofs.md` |
| Cartridge anchor policy | `docs/FormalAnchors.md` |
| Generated symbol ledger | `docs/PROOF-STATUS.md` (from `tests/proof_status_doc.rs`) |
| Anchor CI guard | `tests/formal_anchors.rs` |
| Manifest catalog grounding (W8) | `tests/manifest_bridge_catalog_grounding.rs` (`required-features = ["manifest-bridge"]`) |

---

## Executive summary

| Surface | Manifold `catalog_id`? | Verdict |
|---------|------------------------|---------|
| **`formal_anchor` doc blocks** | No — parallel URI scheme (`lean://`, `empirical://`, `literature://`, `STRUCTURAL`, `NONE`) | Cartridge-local proof ledger; aligns *conceptually* with Lean modules cited in `claims-vs-proofs.md`, not with gate telemetry IDs |
| **`PROOF-STATUS.md`** | No — regenerated from Rust `/// formal_*` lines | Same as above; drift-checked by `proof_status_markdown_matches_committed_snapshot` |
| **`manifest-bridge` feature** | Indirect — wires `UmstManifest::default_transition_gate` → **`umst.gate.cd_transition`** | Predict path calls manifold SSOT; no duplicated `check_transition` implementation in cartridge |
| **`manifold-gate` feature** | Re-exports `GateEvaluator` / `TransitionGateEvaluator` traits only | Empty flag on manifold; enables typed imports, not separate math |
| **Default build (no `manifest-bridge`)** | CD gate **not** run on `predict` | Regime warnings only (`regime_check_scalars`); gate is opt-in via feature |

**Facade gate check:** `enforce_manifold_transition_gate` builds `ThermodynamicState` via manifold APIs and calls `gate.check_transition_host` on `UmstManifest::default().default_transition_gate` (`ThermodynamicTransitionEvaluator` → `umst.gate.cd_transition`). Rejection strings name that `catalog_id` explicitly. Cartridge `ThermoEngine` (Arrhenius heat rate on tensors) is a **different** channel and does not reimplement CD admissibility.

---

## Two parallel naming systems (do not conflate)

### 1. Cartridge `formal_anchor` (per-symbol doc grammar)

Enforced by `cargo test --test formal_anchors`. Every public Rust symbol carries `formal_status` + `formal_anchor` (+ status-specific fields). See `docs/FormalAnchors.md`.

| `formal_anchor` prefix | Role | Example |
|------------------------|------|---------|
| `lean://umst-formal/Lean/...` | Mechanised lemma pointer | `RegimeSoundness.lean#warnings_empty_iff_in_regime` |
| `empirical://datasets/...` | Calibrated closure + CSV envelope | `dataset_d1.csv` |
| `literature://...` | Published equation transcription | `wire-schema-result-v2` |
| `STRUCTURAL` | Type-system / orchestration guarantee | `predict`, `MixSpec` |
| `NONE` | Non-claim glue (IO, accessors, re-exports) | `predict_with_options` |

**Wire projection:** `profile_formal_anchor_uri` copies TOML `[provenance.formal].anchor` into `PredictBundle.formal_anchor` and certify JSON — still **not** a manifold `catalog_id`.

### Mechanised `lean://umst-formal` anchors (PROOF-STATUS / `formal_anchors`)

**Audit date:** 2026-05-21  
**Sources:** `docs/PROOF-STATUS.md` Mechanised bucket (26 symbols); `cargo test --test formal_anchors` guard on the same `/// formal_anchor:` lines; bundled TOML `[provenance.formal].anchor` / `[acceptance].formal_anchor` reuse three URIs below.  
**Catalog lock (post-update):** `umst-manifold/artifacts/catalog.lock.json` (v2 dual-pin) pins **`upstream_catalog_digest_hex`** `0697014fb5b90a3aca4db3e5cc226896ca198802c910d5395f254e4262aa6227` over the **119-module** unified export (`module_count` field). Runtime bundle digest **`UMST_CATALOG_LOCK_SHA256_HEX`** = `1d0d1ed62dfa6144d47cf45c7340ab03405da4f2f5773e2fb281a430c59e3958` (`catalog_lock_bundle_sha256_hex()` — SHA-256 of lock **file bytes**, not the upstream Lean digest). Fiber pins retain **69** + **62** module counts for provenance; SSOT for gates is the **composed** digest. Proposed cartridge `catalog_id` slugs remain traceability-only until `GateUnificationSpec.md` / `claims-vs-proofs.md` rows are added — upstream digest bump does **not** auto-register them.

**Unique mechanised URIs:** 10 (26 symbol rows).

| Anchor URI (`lean://umst-formal/…`) | Mechanised symbols | Digest attestation | Proposed `catalog_id` | Manifold gate path (Rust SSOT) | Notes |
|-------------------------------------|-------------------:|-----------------------|--------------------------------|-------|
| `Lean/Gate.lean#Admissible` | 1 (`FormalBlock`) | **Attested** — `Gate.lean` ∈ 119-export; lock digest verified (`manifest_bridge_catalog_grounding`) | **`umst.gate.cd_transition`** *(registered)* | `umst-manifold/src/gate/thermo_transition.rs`, `src/gate/evaluator.rs` | `umst-formal` parallel `Gate.lean` (ℚ) hand-aligned per `claims-vs-proofs.md`; predict path uses same ID when `manifest-bridge` → `facade::enforce_manifold_transition_gate`. |
| `Lean/Powers.lean#PowersState` | 6 | **Attested** — `Powers.lean` ∈ 119-export | **`thermodynamic_mix`** *(registered)*; secondary **`umst.gate.http_shim`** | `src/gate/mix_proposal.rs`, `src/gate/http_manifest.rs`, `src/gate/concrete_cartridge.rs` | Gel-space state + closure literals on HTTP mix gate; cartridge tensor path: `physics/chemo_water.rs`, `porosity.rs`, `transport.rs`, `homogeneous.rs`. |
| `Lean/Powers.lean#S_intrinsic` | 1 (`CalibrationMeta`) | **Attested** — `Powers.lean` ∈ 119-export | **`thermodynamic_mix`** *(registered)* | `src/gate/mix_proposal.rs` (Powers intrinsic constant in mix filter) | Calibration metadata only; no separate `GateEvaluator`. |
| `Lean/Powers.lean#powers_monotone` | 4 | **Attested** — `Powers.lean` ∈ 119-export | **`thermodynamic_mix`** *(registered)*; secondary **`umst.gate.http_shim`** | `src/gate/mix_proposal.rs`, `src/gate/http_manifest.rs` | Also in bundled profile TOML `anchor = "…#powers_monotone"`; cartridge: `homogeneous.rs`, `physics/strength.rs`. |
| `Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime` | 6 | **Attested** — `RegimeSoundness.lean` ∈ 119-export (no per-lemma `catalog_id`) | **`umst.cartridge.concrete.regime`** *(proposed)* | `—` (no `GateEvaluator`; adjacent policy marker **`umst.cartridge.concrete.policy`** → `src/gate/concrete_cartridge.rs`) | Cartridge SSOT: `calibration::regime_check_scalars`, `facade` `WaterCementRatio` / `TemperatureK`, `homogeneous::safety_margin`. Orthogonal to CD gate on default `predict`. |
| `Lean/OrderStatisticsBand.lean#order_statistic_concentration` | 1 (`RegimeBounds`) | **Attested** — `OrderStatisticsBand.lean` ∈ 119-export | **`umst.cartridge.concrete.acceptance_band`** *(proposed)* | `—` | Profile `[acceptance]` hyperbox metadata; **digest-attested** via 119-export pin; `catalog_id` still proposed. |
| `Lean/OrderStatisticsBand.lean#p25_p75_admissibility` | 1 (`AcceptanceBlock`) | **Attested** — `OrderStatisticsBand.lean` ∈ 119-export | **`umst.cartridge.concrete.acceptance_band`** *(proposed)* | `—` | Same module; bundled profiles set `formal_anchor` to this URI on `[acceptance]`; **digest-attested**. |
| `Lean/JenningsGelSpace.lean#jennings_strength_monotone` | 1 (`compute_hydration_degree`) | **Attested** — `JenningsGelSpace.lean` ∈ 119-export | **`umst.cartridge.concrete.jennings_gel`** *(proposed)* | `—` | Cartridge `physics/hydration.rs`; homogeneous Jennings branch still `JenningsNotImplemented` (see `FormalAnchors.md`). |
| `Lean/Helmholtz.lean#ψAntitoneHelmholtz` | 4 | **Attested** — `Helmholtz.lean` ∈ 119-export | **`umst.gate.cd_transition`** *(registered)* | `src/gate/thermo_transition.rs` (ψ antitone feeds CD admissibility via `umst-formal` `Gate.lean` / `HelmholtzState`) | Cartridge `physics/thermo.rs`, `set_time.rs` are **tensor** Arrhenius/set-time closures — not host transition gate; Lean witness is the CD input model, not a second evaluator. |
| `Lean/MeasurementCost.lean#zero_info_zero_energy` | 1 (`compute_chloride_diffusivity`) | **Attested** — `MeasurementCost.lean` ∈ 119-export | **`umst.gate.landauer_cbf`** *(registered)* | `src/ai/cbf.rs`, `src/gate/cbf_bridge.rs` | Transport diffusivity anchor; Landauer bit-energy cap (claims-rust, no `GateEvaluator` in `gate/`). |

**Registry vs proposed**

| Class | `catalog_id` count | In `catalog.lock.json` (119-export pin)? |
|-------|-------------------:|----------------------------------------|
| Registered (manifold `traceability.rs` / `GateUnificationSpec.md`) | 6 rows above | **Attested** — all cited `Lean/*.lean` modules appear in the **119-module** `umst-formal-double-slit` export pinned by `upstream_catalog_digest_hex` |
| Proposed cartridge slugs | 3 (`…regime`, `…acceptance_band`, `…jennings_gel`) | No — spec extension only; **does not** bump lock digest |
| Export pin (all 10 URIs) | All 10 URIs | **`umst.formal.catalog_lock`** → `catalog.lock.json` + `catalog_lock_bundle_sha256_hex()`; per-lemma `catalog_id` only where registered above |

**`manifest-bridge` predict path (registered IDs only)**

```
facade::predict → enforce_manifold_transition_gate
  → ThermodynamicState::from_mix_calibrated
  → UmstManifest::default().default_transition_gate   // umst.gate.cd_transition
       → umst-manifold/src/gate/thermo_transition.rs
```

Regime / acceptance / Jennings anchors do **not** participate in this chain unless a future evaluator registers the proposed cartridge slugs.

### 2. Manifold `catalog_id` (gate / witness registry)

Stable slugs for telemetry, orchestrator routing, ROS contracts, and Lean export rows. Canonical table: `umst-manifold/docs/GateUnificationSpec.md` + `claims-vs-proofs.md`.

---

## Feature → manifold mapping

| Cartridge feature | Depends on | Manifold modules used | Primary `catalog_id`(s) | Cartridge behaviour |
|-------------------|------------|----------------------|-------------------------|---------------------|
| *(default)* | `umst-manifold` git **`rev = fe22437`** | `core`, `pipeline`, physics via `ConcreteCartridge` | — (no gate on predict) | Tensor physics + regime hyperbox only |
| **`manifold-gate`** | `umst-manifold/manifold-gate` (empty flag) | `gate::evaluator`, `gate::thermo_transition` | **`umst.gate.cd_transition`** (via re-exported traits) | `lib.rs` re-exports `GateEvaluator`, `ThermodynamicTransitionEvaluator`, `TransitionGateEvaluator` — **no call site** unless caller uses them |
| **`manifest-bridge`** | `manifest-bridge` + **`manifold-gate`** (forced in `Cargo.toml`) | `manifest::UmstManifest`, `gate::{ThermodynamicState, TransitionGateEvaluator}` | **`umst.gate.cd_transition`** on reject | `facade::manifest` → `pub use umst_manifold::manifest::*`; `predict` / `predict_from_mix_row` call `enforce_manifold_transition_gate` |
| **`manifold-manifest`** | `umst-manifold/manifold-manifest` (empty flag) | `manifest::UmstManifest` | `umst.formal.catalog_lock` (digest via `UmstManifest::compiled_catalog_lock_bundle_sha256_hex`) | `lib.rs` re-export only; examples note future topology `manifest.json` hydration |
| **`ros2-contract`** | `umst-manifold/ros2-contract` | `ros::contract` (serde DTOs + `catalog_hash`) | Hand-aligned with gate telemetry (`claims-vs-proofs.md`) | `lib.rs` re-exports `umst_manifold::ros`; no cartridge ROS runtime |

**Related manifold IDs (not wired on cartridge `predict` today)**

| `catalog_id` | Manifold location | Cartridge touchpoint |
|--------------|-------------------|----------------------|
| `thermodynamic_mix` | `gate/mix_proposal.rs`, `mix_eval_registry.rs` | None on predict path; used in manifold HTTP / embodied orchestrator tests |
| `umst.gate.http_shim` | `gate/http_manifest.rs`, `bin/gate_server.rs` | Separate service; shares Powers closure literals via `ConcreteCartridge::default_gate_manifest` |
| `umst.cartridge.concrete.policy` | `gate/concrete_cartridge.rs` | Policy marker for HTTP defaults (`gate_family: concrete_powers_manifest_defaults`), not CD math |
| `umst.gate.landauer_cbf` | `ai/cbf.rs`, `gate/cbf_bridge.rs` | Topology / `ManifoldGateway` steps in manifold; cartridge `compute_topology` delegates to manifold Laplacian path |
| `umst.formal.catalog_lock` | `build.rs`, `runtime/catalog`, `artifacts/catalog.lock.json` | Advisory via manifest type; cartridge does not pin digest in default CI |
| `umst.gate.kleisli_unit` | `gate/kleisli.rs` (spec; no `GateEvaluator` impl yet) | Not used in cartridge |

---

## `PROOF-STATUS.md` ↔ manifold

| Aspect | Detail |
|--------|--------|
| **Generator** | `tests/proof_status_doc.rs` scans `src/**/*.rs`, `umst-cli`, `umst-mcp`, `umst-py` |
| **Buckets** | Mechanised (26), Structural (33), Empirical (27), Literature (38), NONE (96) — counts from committed snapshot |
| **`catalog_id` column** | On **Mechanised** Rust-doc blocks (`/// catalog_id:`) and `docs/PROOF-STATUS.md`; traceability to manifold registry per table § Mechanised anchors |
| **Cross-link to manifold** | Mechanised rows cite the same Lean modules listed in `claims-vs-proofs.md` (e.g. `Powers.lean`, `RegimeSoundness.lean`, `Helmholtz.lean`) |
| **Regenerate** | `cargo test -p umst-concrete-cartridge --test proof_status_doc proof_status_refresh_markdown_on_disk -- --ignored --nocapture` |

Gate-related symbols in PROOF-STATUS are classified **NONE** (re-exports, manifest shim) because the cartridge does not restate a proof on forwarded types — the witness lives in manifold.

---

## Facade gate delegation (verified)

### Call chain (`manifest-bridge` enabled)

```
predict / predict_from_mix_row
  → homog::mix_hydration_state (cartridge calibrated α, w/c, T)
  → ThermodynamicState::from_mix_calibrated (manifold)
  → UmstManifest::default().default_transition_gate
       .check_transition_host(&old, &new, dt_s)   // catalog_id: umst.gate.cd_transition
  → FacadeError::Tensor if !verdict.admissible
  → run_full_physics_pipeline (cartridge tensor engines)
```

### What is **not** duplicated

| Concern | SSOT crate | Cartridge role |
|---------|------------|----------------|
| CD inequality, mass, dissipation sign | `umst-manifold/src/gate/thermo_transition.rs` | Caller builds states; delegates check |
| `ThermodynamicState::from_mix_calibrated` closure | Manifold | Used as-is for gate inputs |
| Regime hyperbox warnings | Cartridge `Profile::regime_check_scalars` | Orthogonal to CD gate (Lean `RegimeSoundness` anchor) |
| Arrhenius **tensor** heat rate | Cartridge `physics/thermo.rs` | Differentiable kinetics, not host `f64` transition filter |
| Powers **tensor** strength pipeline | Cartridge `physics/strength.rs`, homogeneous routing | May disagree in detail with gate snapshot strength — gate uses manifold snapshot formula intentionally |

### Tests run (2026-05-29, G-02 parity)

```bash
cargo test -p umst-concrete-cartridge --features manifest-bridge   # git-pinned umst-manifold @ fe22437 (no workspace [patch])
cargo test -p umst-concrete-cartridge --features manifest-bridge --lib manifest_bridge
cargo test -p umst-concrete-cartridge --features manifest-bridge --test manifest_bridge_catalog_grounding
```

**Re-verified after 119-module catalog lock alignment (2026-05-29):**

| Command | Result |
|---------|--------|
| `cargo test -p umst-concrete-cartridge --test formal_anchors --features manifest-bridge` | **6/6 passed** |
| `cargo test -p umst-concrete-cartridge --features manifest-bridge --test manifest_bridge_catalog_grounding` | **1/1 passed** (`manifest_default_gate_catalog_ids_resolve_embedded_catalog_digest`) |
| `cargo test -p umst-concrete-cartridge --features manifest-bridge --lib manifest_bridge` | **1/1 passed** (`predict_runs_umst_manifest_transition_gate_for_in_regime_mix`) |

Lock JSON checked: `upstream_catalog_digest_hex` = `0697014fb5b90a3aca4db3e5cc226896ca198802c910d5395f254e4262aa6227`; `module_count` = **119**; bundle SHA-256 = `1d0d1ed62dfa6144d47cf45c7340ab03405da4f2f5773e2fb281a430c59e3958`. All **10** unique `lean://umst-formal/…` mechanised URIs map to modules in that export → **digest-attested** under `umst.formal.catalog_lock` (per-lemma `catalog_id` on Rust doc blocks where registered in the table above).


#### `manifest_default_gate_catalog_ids_resolve_embedded_catalog_digest`

Integration test (`tests/manifest_bridge_catalog_grounding.rs`) pins the **`manifest-bridge`** predict path to manifold catalog SSOT:

| Check | Assertion |
|-------|-----------|
| Default gate ID | `UmstManifest::default().default_transition_gate.catalog_id()` → **`umst.gate.cd_transition`** |
| Lock bundle digest | `UmstManifest::compiled_catalog_lock_bundle_sha256_hex()` equals `runtime::catalog::catalog_lock_bundle_sha256_hex()` (64-char hex) |
| Embedded lock JSON | `bundled_catalog_lock_json()` contains `"upstream_catalog_digest_hex"` (Lean export fingerprint in `artifacts/catalog.lock.json`) |
| Witness envelope | `witness_catalog_quickcheck_ok()` and `WitnessCatalog::from_embedded()` parse |
| Traceability | Each default `catalog_id` appears in sibling `umst-manifold/docs/claims-vs-proofs.md` and `GateUnificationSpec.md` |

**Skip rule (today):** if `../../../umst-manifold` is absent, the test **returns immediately** (cartridge-only GHA may not exercise grounding until refactored). **G-02 intent:** assert embedded lock digest + default `catalog_id` against the **git-pinned** `umst-manifold` crate with **no** sibling path — monorepo checkouts still run the full `claims-vs-proofs.md` / `GateUnificationSpec.md` cross-check when the sibling tree is present.

---

## Scaling pattern — ownership boundaries

Use this when adding domains (e.g. asphalt, geopolymers) or new cartridges.

### Cartridge **should** own

| Layer | Examples in this repo |
|-------|----------------------|
| **Constitutive closures** | Powers/Jennings gel-space, YODEL/DLVO, B4 creep, printability, optical paste profile |
| **Calibration profiles** | TOML bundles, regime hyperboxes, acceptance envelopes vs CSVs |
| **Mix layout → tensor** | `mix_layout`, `MixRow`, `run_full_physics_pipeline` stage graph |
| **Wire / transport** | `facade` serde DTOs, JSON schemas, MCP/CLI/Python boundaries |
| **Per-symbol proof ledger** | `formal_anchor` doc blocks + `PROOF-STATUS.md` |
| **Domain-specific empirical anchors** | `empirical://datasets/...` tied to bundled CSVs |

### Cartridge **should not** own (delegate to `umst-manifold`)

| Layer | Manifold `catalog_id` / module |
|-------|------------------------------|
| **Clausius–Duhem transition admissibility** | `umst.gate.cd_transition` — `gate/thermo_transition.rs` |
| **Mix-proposal thermodynamic filter (HTTP/registry)** | `thermodynamic_mix` |
| **Landauer / CBF on topology steps** | `umst.gate.landauer_cbf` |
| **DEC Laplacian / graph physics merge** | `physics::laplacian`, `apply_physics_to_umst` |
| **Lean catalog digest pin** | `umst.formal.catalog_lock` |
| **Gate server / embodied orchestrator** | `umst.gate.http_shim`, `manifest/orchestrator.rs` |
| **`GateEvaluator` registry routing** | `mix_eval_registry.rs` |

### Thin integration pattern (recommended)

1. **Feature flags:** `manifest-bridge` = typed manifest + predict-path CD gate; `manifold-gate` = trait re-exports for custom orchestration.
2. **One enforcement site:** keep `enforce_manifold_transition_gate` (or successor) in `facade` — do not copy `ThermodynamicGate::check_transition` into `physics/`.
3. **State construction:** map cartridge scalars → `ThermodynamicState::from_mix_calibrated` only; never fork the snapshot closure.
4. **Telemetry:** on reject/accept, log manifold `catalog_id` (`umst.gate.cd_transition`), not a cartridge-only slug.
5. **Proof docs:** cartridge `formal_anchor` may cite the same Lean lemmas as manifold (`Gate.lean`, `GateCompat.lean`) for **regime** and **Powers** symbols; gate **execution** remains NONE/Structural at the façade glue layer.

### CI / release note (W8 + G-02)

| Wave | Status | What shipped |
|------|--------|----------------|
| **W8** | **Done** on [`umst-manifold` @ `fe22437`](https://github.com/tytolabs/umst-manifold/commit/fe22437) | `manifest`, `manifest-bridge`, gate evaluators, **119-module** `catalog.lock.json` on upstream `main` |
| **G-02** | **In-repo** (cartridge) | `umst-manifold` git **`rev = fe22437`** in `crates/umst-concrete-cartridge/Cargo.toml`; **no** workspace `[patch]`; [`rust.yml`](../.github/workflows/rust.yml) **`manifest-bridge tests (pinned umst-manifold)`** step |

**Default-feature** workspace jobs still omit `manifest-bridge` on `predict` (tensor physics + regime hyperbox only). **G-02** exercises CD gate + catalog grounding on the pinned git dependency:

```bash
cargo test -p umst-concrete-cartridge --features manifest-bridge
```

`manifest_bridge_catalog_grounding` asserts default gate `catalog_id`s resolve against the embedded catalog digest from the pinned manifold crate; optional sibling `../umst-manifold` adds markdown traceability cross-checks when present.

---

## Gaps and follow-ups

| Gap | Risk | Suggested action |
|-----|------|----------------|
| Default `predict` skips CD gate | Inadmissible transitions possible in production default builds | Opt in `manifest-bridge` for predict-path CD gate; G-02 CI already tests the feature matrix on git `fe22437` |
| `formal_anchor` vs `catalog_id` | Operators may assume URI equality | Mapping table § Mechanised anchors; optional `gate_catalog_id` on certify/MCP when `manifest-bridge` is on |
| Proposed `umst.cartridge.concrete.{regime,acceptance_band,jennings_gel}` | Not in `traceability.rs` / gate registry | Add `GateUnificationSpec` + `claims-vs-proofs` rows (W8); Lean modules **digest-attested** via 119-export pin — promote slugs without another lock bump if export unchanged |
| `thermodynamic_mix` not on predict path | Mix filter only in HTTP/orchestrator | Wire `EmbodiedOrchestrator` if dual-run policy needed in cartridge |
| `umst.cartridge.concrete.policy` | Name suggests cartridge owns policy | Policy literals only; transition SSOT remains `umst.gate.cd_transition` |

---

## Quick reference — `catalog_id` lookup

| ID | Proved / aligned in Lean (per `claims-vs-proofs.md`) | Rust SSOT |
|----|------------------------------------------------------|-----------|
| `umst.gate.cd_transition` | `Gate`, `GateCompat`, `UMSTCore`, `Naturality` | `gate/thermo_transition.rs`, `gate/evaluator.rs` |
| `thermodynamic_mix` | `GateCompat` (prototype filter) | `gate/mix_proposal.rs` |
| `umst.gate.landauer_cbf` | `LandauerBound`, `EpistemicMI`, … | `ai/cbf.rs` |
| `umst.gate.http_shim` | TCB | `gate/http_manifest.rs` |
| `umst.formal.catalog_lock` | `FormalFoundations` (digest) | `artifacts/catalog.lock.json` |
| `umst.cartridge.concrete.policy` | — (engineering marker) | `gate/concrete_cartridge.rs` |

*End of audit.*
