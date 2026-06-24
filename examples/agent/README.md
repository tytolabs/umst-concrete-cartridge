# Agent example workflows

Runnable scripts for the Physical Reasoning Layer. See [`docs/AGENT_MCP.md`](../../docs/AGENT_MCP.md) for the full contract.

> **Performance:** Prefer **arena / in-process** examples (`06`–`07`) for batch gate loops and optimization sweeps (≥5× stdio MCP, CI-pinned). Use **MCP stdio** examples (`01`–`05`) for prototyping, discovery, and cross-language agents.

## Fast path — arena / in-process (recommended for heavy use)

| Script | What it shows | CI |
|--------|----------------|-----|
| [`06_arena_batch.py`](06_arena_batch.py) | In-process `gate_check_mix` batch loop — no JSON-RPC overhead | yes |
| [`07_arena_mmap_load.py`](07_arena_mmap_load.py) | `load_arena` → `UmstArenaView` hot loop (mmap proxy via CI test) | yes |
| [`07_arena_mcp_session.py`](07_arena_mcp_session.py) | MCP arena session: `umst_arena_open` → `umst_gate_check_arena` → close | yes |

Benchmarks: [`umst-manifold/docs/benchmarks/arena_vs_mcp.md`](../../../umst-manifold/docs/benchmarks/arena_vs_mcp.md).

## MCP stdio — prototyping & discovery

| Script | What it shows | CI |
|--------|----------------|-----|
| [`01_gate_explore.py`](01_gate_explore.py) | Safe exploration: REJECT with remediation, PASS gate, memory query (read-only contribute path) | yes |
| [`02_contribute_admissible.py`](02_contribute_admissible.py) | Gate → contribute golden fixture → verify row in memory (temp SQLite) | yes |
| [`03_export_inbox.sh`](03_export_inbox.sh) | Export → validate → dry-run ingest for federated git inbox PRs | optional |
| [`04_memory_query_batch.py`](04_memory_query_batch.py) | Seed one row, then batch `umst_memory_query` filters (L1, regime, admissible_only) | yes |
| [`05_explain_violations.py`](05_explain_violations.py) | `explain: true` on rational parse fail + thermodynamic REJECT golden mix | yes |

**Prerequisites:** Rust toolchain, repo root as cwd. CI prebuilds `umst-mcp --features agent-layer`; local runs may invoke `cargo run` on first launch.

```bash
cargo build -p umst-mcp --features agent-layer
# Fast path (batch / mmap)
python3 examples/agent/06_arena_batch.py
python3 examples/agent/07_arena_mmap_load.py
python3 examples/agent/07_arena_mcp_session.py
# MCP prototyping
python3 examples/agent/01_gate_explore.py
python3 examples/agent/02_contribute_admissible.py
python3 examples/agent/04_memory_query_batch.py
python3 examples/agent/05_explain_violations.py
UMST_MEMORY_DB=.umst-memory/memory.db bash examples/agent/03_export_inbox.sh
```

**Golden fixtures:** adversarial mixes live under [`fixtures/golden-adversarial/`](../../fixtures/golden-adversarial/) — see [`docs/GOLDEN_VECTORS.md`](../../docs/GOLDEN_VECTORS.md).
