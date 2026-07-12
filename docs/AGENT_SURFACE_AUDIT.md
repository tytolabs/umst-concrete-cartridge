# Agent surface audit (B0)

**Purpose.** Ground Work-order B: enumerate the **current real** agent surface with `file:line`, classify hot vs cold, and mark §5.1 **target** separately.  
**Date:** 2026-07-12 · **Repo:** `umst-concrete-cartridge` (authoritative MCP host).  
**Acceptance:** every tool row cites source `file:line`; Proposed tools quarantined.

---

## 1. Current vs §5.1 target (read first)

| Layer | **Current (shipped)** | **Target (§5.1 — Proposed / not built)** |
|:---|:---|:---|
| Wire | Hand-rolled stdio JSON-RPC in `umst-mcp` (`main.rs`) | Official Rust MCP SDK (`rmcp`) |
| Tool schema | Imperative `tools/list` arrays in `main.rs` / `agent_layer.rs` | Declarative `ToolDescriptor` manifest → schema **and** B3 contracts |
| Hot path | `umst_arena_open` / `umst_gate_check_arena` / `umst_arena_close` (+ library arena) | Same in-process arena; never proxy |
| Capability | `UMST_AGENT_SCOPE_TOKENS` env (partial) | `SideEffectClass` Pure/ReadOnly vs Mutating/NetworkIo |
| Edges | `maos-shell/mcp` thin HTTP (workspace-private); egoff FFI OFF LIMITS | TS edge demoted; public FFI core 🔒 USER code order |

This audit documents **current**. Target rows are marked **Proposed**.

---

## 2. Manifold ports (real traits)

| Symbol | Role | Defined at |
|:---|:---|:---|
| `IScienceCartridge<B>` | Material-law port → `PhysicalResult` | `umst-manifold/src/core/traits.rs:51` |
| `GateCartridge` | Universal gate port | `traits.rs:62` |
| `SpatialCartridge<B>` | Spatial subtype of `IScienceCartridge` | `traits.rs:69` |
| `DesignRepresentation<B>` | Pure latent → geometry decode | `traits.rs:98` |

Supporting types: `PhysicalResult` (`traits.rs:35`), `DesignLatent` (`:75`), `Geometry` (`:81`).

---

## 3. Cartridge research layer (real)

| Symbol | Role | Defined at |
|:---|:---|:---|
| `PhysicalReasoningLayer` | Domain reasoning port | `crates/umst-concrete-cartridge/src/research/layer.rs:17` |
| `ConcretePhysicalReasoningLayer` | Concrete impl | `layer.rs:77` |
| `MemoryStore` | Append-only research memory | `crates/umst-concrete-cartridge/src/research/memory.rs:53` |

---

## 4. MCP tools — all 13 (current)

**Authoritative server:** `crates/umst-mcp` · `Cargo.toml` `default = []` (4 base tools); `--features agent-layer` adds 9 (and pulls `arena-session`).

| # | Tool | Schema `"name"` | Temp | Feature |
|:---|:---|:---|:---|:---|
| 1 | `umst_predict` | `main.rs:114` | COLD | default |
| 2 | `umst_audit` | `main.rs:132` | COLD | default |
| 3 | `umst_profiles` | `main.rs:149` | COLD | default |
| 4 | `umst_certify` | `main.rs:154` | COLD | default |
| 5 | `umst_gate_check` | `agent_layer.rs:613` | COLD | agent-layer |
| 6 | `umst_contribute` | `agent_layer.rs:629` | COLD | agent-layer |
| 7 | `umst_contribute_status` | `agent_layer.rs:646` | COLD | agent-layer |
| 8 | `umst_memory_query` | `agent_layer.rs:660` | COLD | agent-layer |
| 9 | `umst_mi_estimate` | `agent_layer.rs:685` | COLD | agent-layer |
| 10 | `umst_transition_propose` | `agent_layer.rs:699` | COLD | agent-layer |
| 11 | `umst_arena_open` | `agent_layer.rs:716` | **HOT** | agent-layer + arena-session |
| 12 | `umst_gate_check_arena` | `agent_layer.rs:730` | **HOT** | agent-layer + arena-session |
| 13 | `umst_arena_close` | `agent_layer.rs:747` | **HOT** | agent-layer + arena-session |

Dispatch: `main.rs:869–882`. Without `agent-layer`, agent names → `-32601` (`main.rs:906+`).

**Library hot path (not MCP):** `umst-runtime-arena` `load_arena` / in-process `gate_check_mix` — see examples `06`/`07`.

---

## 5. Which of 3 surfaces serves each tool

| Surface | Location | Serves | Hot? | Agent authority |
|:---|:---|:---|:---|:---|
| **A — umst-mcp** | this repo `crates/umst-mcp` | All **13** tools above | Yes (arena trio) | **Yes — canonical** |
| **B — maos-shell/mcp** | workspace `maos-shell/mcp` (private) | Distinct `maos.*` tools (gate.validate, physics.compute, …) — **not** the `umst_*` set | No (HTTP proxy) | Legacy edge only |
| **C — egoff** | PRIVATE / OFF LIMITS | FFI `umst_gate_check` (fact); do not open or promote | N/A | Not authoritative |

**DEFAULTED note:** Blueprint §2 historically cited `umst_gate_full` / `umst_http_gate` on maos-shell — **not present** in opened `maos-shell/mcp/dist/server.js` (actual names `maos.*`). Flagged for human sign-off of §2 text cleanup.

---

## 6. Examples (`examples/agent/`)

| Script | Purpose (header) |
|:---|:---|
| `01_gate_explore.py` | Gate reject + pass + memory query |
| `02_contribute_admissible.py` | Gate PASS → contribute → query |
| `03_export_inbox.sh` | Federated inbox export dry-run |
| `04_memory_query_batch.py` | Batch memory filters + pagination |
| `05_explain_violations.py` | Gate REJECT + `explain` |
| `06_arena_batch.py` | In-process library batch gate (HOT) |
| `07_arena_mmap_load.py` | Arena mmap hot loop (HOT) |
| `08_arena_mcp_session.py` | MCP arena open → gate_check_arena → close (HOT) |

---

## 7. Proposed (not yet built) — quarantine

| Name | In `umst-mcp` source? | Notes |
|:---|:---|:---|
| `umst_dry_run` | **NO** | P2 — scope note only |
| `umst_promote_contribution` | **NO** | P3 — schema resource may exist; **no** tool registration |
| `umst_arena_session` | **NO** | P4 — shipped as open/gate/close trio instead |
| Rich-violation-by-default (P1) | **Partial** | `explain` defaults **true** (`agent_layer.rs:620`, `main.rs:487+`) — further enrichment still Proposed |

---

## 8. Acceptance paste

```text
$ rg -n '"name": "umst_' crates/umst-mcp/src/main.rs crates/umst-mcp/src/agent_layer.rs | rg 'umst_'
# → 13 unique tool names (see §4 table)
$ ls examples/agent/0{1,2,3,4,5,6,7,8}*
# → 8 scripts
```

**Next:** B1 `ARCHITECTURE.md` builds on this inventory.
