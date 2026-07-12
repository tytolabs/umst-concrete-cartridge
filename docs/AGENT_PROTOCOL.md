# AGENT_PROTOCOL.md — guarantees & workflows (B2)

**What the code actually enforces** (not aspirational policy). Inventory: [`AGENT_SURFACE_AUDIT.md`](AGENT_SURFACE_AUDIT.md).

---

## Agent guarantees (enforced)

| # | Guarantee | Enforcement | Remediation |
|:---|:---|:---|:---|
| G1 | Never contribute without gate PASS | `accept` / MCP contribute reject if not admissible — `main.rs:71–72`, `agent_layer.rs:310–311` | Run `umst_gate_check`; read `explain.remediation` |
| G2 | Gate REJECT surfaces `isError` | `main.rs:502–508` (`is_error = !admissible`) | Parse `gate_reject.v1` + `explain` |
| G3 | Explain defaults on for MCP gate | `agent_layer.rs:620` schema default; `main.rs:487+` | Set `explain:false` only if intentional |
| G4 | Admissible-only memory queries stay clean | `umst_memory_query` + `admissible_only` filter | Widen filters; seed via `02_contribute_admissible.py` |
| G5 | Arena bytes fail closed on bad header | `load_arena` in arena open path `agent_layer.rs:359+` | Use trusted arena files only |
| G6 | Default build does not expose agent tools | `Cargo.toml` `default=[]`; missing feature → `-32601` `main.rs:906+` | Build `--features agent-layer` |
| G7 | Scope tokens when configured | `UMST_AGENT_SCOPE_TOKENS` (see AGENT_MCP env) | Supply `scope_token` or unset env |

---

## Workflows (composed morphisms)

### Safe exploration (read-only / COLD)
1. `umst_gate_check` (`explain` default true)  
2. Optional `umst_predict`  
3. `umst_memory_query`  
4. **Never** `umst_contribute` on REJECT  

Example: `examples/agent/01_gate_explore.py`, `05_explain_violations.py`.

### Contribute admissible (COLD, mutating)
1. `umst_gate_check` → PASS  
2. Build `contribution.v1`  
3. `umst_contribute` → `memory_id`  
Example: `02_contribute_admissible.py`.

### Hot batch gate (HOT)
1. Library: `06_arena_batch.py` / `07_arena_mmap_load.py`  
2. Or MCP: `08_arena_mcp_session.py` (`arena_open` → `gate_check_arena` → `arena_close`)

### Federated inbox (not MCP promote)
Export JSONL + PR — scripts under `scripts/`; **no** `umst_promote_contribution` tool (Proposed P3).

---

## Acceptance

Every guarantee row has an enforcing `file:line`.
