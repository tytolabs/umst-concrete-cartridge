# ARCHITECTURE.md — categorical agent surface (B1)

**SSOT for Objects / Morphisms / Functors / Monads on the agent path.**  
Companion inventory: [`AGENT_SURFACE_AUDIT.md`](AGENT_SURFACE_AUDIT.md). Tool index: [`AGENT_MCP.md`](AGENT_MCP.md).  
**Current** = shipped symbols. **Proposed** = §5.1 target (not built).

---

## Objects (real)

| Object | Meaning | Symbol | At |
|:---|:---|:---|:---|
| Material / mix state | Rational mix + thermo fields | `MixSpec` | `facade/mod.rs:151` |
| Contribution | Durable research row | `Contribution` | `research/types.rs:79` |
| Gate result | Pass/reject + explain | `GateCheckResult` | `research/contribution.rs:161` |
| Memory | Append-only store | `MemoryStore` | `research/memory.rs:53` |
| Observation stamp | Temporal provenance | `UcrsObservedAt` | `umst-ucrs` `observation.rs:42` |
| Clock gate state | Sync admissibility object | `ClockThermState` / `GateVerdict` | `umst-ucrs` `gate.rs:15` / `:28` |
| Cartridge result | Free energy / dissipation / margin | `PhysicalResult` | `umst-manifold` `traits.rs:35` |

**Proposed (not yet built):** standalone `Observation` / `Transition` ADTs beyond `Contribution` + `GateCheckResult` — do not invent; use the rows above.

---

## Morphisms (real)

| Morphism | Meaning | Symbol | At |
|:---|:---|:---|:---|
| `gateCheck` | Hard admissibility | `gate_check_mix` / `gate_check_mix_result` | `contribution.rs:108` / `:175` |
| `contribute` / `accept` | Memory write iff PASS | `accept` | `contribution.rs:516` |
| `queryMemory` | Filter/paginate | `MemoryStore` + MCP `umst_memory_query` | `memory.rs:53`; `agent_layer.rs:660` |
| `predict` | Constitutive envelope | MCP `umst_predict` | `main.rs:114` |
| `propose` | Predict→gate→async contribute | `umst_transition_propose` | `agent_layer.rs:699` |
| `arenaOpen` / `arenaGate` / `arenaClose` | Warm session morphisms | `umst_arena_*` | `agent_layer.rs:716–747` |

**Proposed:** `promote` (human-gated corpus promotion) — P3; not an MCP tool today (inbox scripts only).

---

## Functors

| Functor | Meaning | Real symbol |
|:---|:---|:---|
| Cartridge | Domain law → manifold carrier | `IScienceCartridge` (`traits.rs:51`) |
| Design decode | Latent → geometry | `DesignRepresentation` (`traits.rs:98`) |
| Physical reasoning | Cartridge→schemas/memory geometry | `PhysicalReasoningLayer` (`layer.rs:17`) |

---

## Monads / effect boundary

| Effect | Current | Proposed (§5.1) |
|:---|:---|:---|
| Agent session | `AgentSession` in `umst-mcp` (stdio boundary) | Same; + `SideEffectClass` capability |
| Async contribute | In-process job + `umst_contribute_status` | Unchanged semantics |
| Wire protocol | Hand-rolled JSON-RPC | **Proposed:** `rmcp` SDK |

Pure research morphisms live under `src/research/`; MCP holds effects at the edge ([`AGENT_MCP.md`](AGENT_MCP.md) FP note).

---

## Hot / Cold boundary

| Path | Morphisms | Rule |
|:---|:---|:---|
| **HOT** | `umst_arena_open`, `umst_gate_check_arena`, `umst_arena_close`; library `load_arena` / batch gate | Parse-once; no Docker; no HTTP hop |
| **COLD** | remaining 10 MCP tools | Stdio JSON-RPC; memory I/O; explain |

Detail: [`FAST_ARENA.md`](FAST_ARENA.md).

---

## Acceptance

Every Object/Morphism row cites a real type or is marked Proposed. Spot-check: `GateCheckResult` @ `contribution.rs:161`.
