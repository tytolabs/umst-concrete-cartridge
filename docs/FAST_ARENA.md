# FAST_ARENA.md — hot vs cold (B4)

**Performance honesty for agents.** Inventory: [`AGENT_SURFACE_AUDIT.md`](AGENT_SURFACE_AUDIT.md) §4.  
§5.1 Layer 1 = native Rust MCP + arena (current + target). Layer 2 = remote SDK edge (**never** for hot loops).

---

## Rules

1. **HOT path never crosses Docker MCP or HTTP** for gate loops.  
2. **Parse arena once**; reuse bytes (`umst_arena_open` or library `load_arena`).  
3. **COLD path** owns contribute, memory, explain, audit, certify.  
4. Benchmark claim (aspirational 10×; CI pins ≥5×): sibling manifold `docs/benchmarks/arena_vs_mcp.md` when present.

---

## Per-tool labels (all 13)

| Tool | Label | Notes |
|:---|:---|:---|
| `umst_arena_open` | **HOT** | Warm boundary |
| `umst_gate_check_arena` | **HOT** | Same physics as cold gate; arena witnesses |
| `umst_arena_close` | **HOT** | Drop session |
| `umst_gate_check` | COLD | Stdio round-trip |
| `umst_contribute` | COLD | Mutating memory |
| `umst_contribute_status` | COLD | Poll |
| `umst_memory_query` | COLD | Read memory |
| `umst_mi_estimate` | COLD | Advisory only |
| `umst_transition_propose` | COLD | Compose predict+gate+async contribute |
| `umst_predict` | COLD | Read-only constitutive |
| `umst_audit` | COLD | Batch CSV |
| `umst_profiles` | COLD | List profiles |
| `umst_certify` | COLD | Anchor chain |

---

## Examples labeled

| Example | Path |
|:---|:---|
| `01`–`05` | COLD (stdio MCP) |
| `06`, `07` | **HOT** (library arena) |
| `08` | **HOT** (MCP arena session) |

---

## Invariants

- Arena open without `arena-session` feature → explicit error string (`agent_layer.rs` cfg).  
- Malformed arena header → fail closed at `load_arena`.  
- Arena PASS ≠ automatic contribute — still need cold `umst_contribute` with PASS summary.

---

## Acceptance

All 13 tools labeled; arena invariants stated.
