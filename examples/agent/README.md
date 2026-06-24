# Agent example workflows

Runnable scripts for the Physical Reasoning Layer MCP server. See [`docs/AGENT_MCP.md`](../../docs/AGENT_MCP.md) for the full contract.

| Script | What it shows | CI |
|--------|----------------|-----|
| [`01_gate_explore.py`](01_gate_explore.py) | Safe exploration: REJECT with remediation, PASS gate, memory query (read-only contribute path) | yes |
| [`02_contribute_admissible.py`](02_contribute_admissible.py) | Gate → contribute golden fixture → verify row in memory (temp SQLite) | yes |
| [`03_export_inbox.sh`](03_export_inbox.sh) | Export → validate → dry-run ingest for federated git inbox PRs | optional |
| [`04_memory_query_batch.py`](04_memory_query_batch.py) | Seed one row, then batch `umst_memory_query` filters (L1, regime, admissible_only) | yes |
| [`05_explain_violations.py`](05_explain_violations.py) | `explain: true` on rational parse fail + thermodynamic REJECT golden mix | yes |
| [`06_arena_batch.py`](06_arena_batch.py) | In-process batch gate loop (no MCP round-trips) | yes |

**Prerequisites:** Rust toolchain, repo root as cwd. CI prebuilds `umst-mcp --features agent-layer`; local runs may invoke `cargo run` on first launch.

```bash
cargo build -p umst-mcp --features agent-layer
python3 examples/agent/01_gate_explore.py
python3 examples/agent/02_contribute_admissible.py
python3 examples/agent/04_memory_query_batch.py
python3 examples/agent/05_explain_violations.py
UMST_MEMORY_DB=.umst-memory/memory.db bash examples/agent/03_export_inbox.sh
```

**Golden fixtures:** adversarial mixes live under [`fixtures/golden-adversarial/`](../../fixtures/golden-adversarial/) — see [`docs/GOLDEN_VECTORS.md`](../../docs/GOLDEN_VECTORS.md).
