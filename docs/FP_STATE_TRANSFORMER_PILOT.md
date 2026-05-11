# FP state-transformer pilot (`fp-v04-2-state-pilot`)

## Module

**`crates/umst-concrete-cartridge/src/physics/self_heal.rs`** — smallest single-entry physics head in the cartridge: one public tensor map (no hidden configuration, no `&mut self`).

## Pattern

1. **`pub(crate)` observable bundle** — `HealingObservableState` in `crates/umst-concrete-cartridge/src/physics/self_heal.rs` groups the differentiable field tensors that drive the head. It is crate-private so the external API surface is unchanged.
2. **Pure free function** — `transform_healing_observable_state` in the same file takes that bundle by value and returns the output tensor. All work is expressed as functional `burn` ops on owned tensors.
3. **Thin public wrapper** — `SelfHealEngine::compute_healing_potential` only packs arguments into the bundle and delegates to the transformer, preserving call sites (e.g. pipeline orchestrator, formal anchor paths).

## When to extend

Use the same split for other `PhantomData` “engine” structs: keep the `impl` as a namespaced entrypoint; move tensor logic into `pub(crate) fn transform_*` plus a small input state struct if multiple tensors (or config) need to stay aligned.

## Verification

From the workspace root of this repo:

- `cargo test -p umst-concrete-cartridge`
- `cargo clippy -p umst-concrete-cartridge -- -D warnings` (Rust flags after `--`)
