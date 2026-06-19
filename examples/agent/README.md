# Agent example workflows

Runnable scripts for the Physical Reasoning Layer MCP server. See [`docs/AGENT_MCP.md`](../../docs/AGENT_MCP.md) for the full contract.

| Script | What it shows |
|--------|----------------|
| [`01_gate_explore.py`](01_gate_explore.py) | Safe exploration: REJECT with remediation, PASS gate, memory query (read-only contribute path) |
| [`02_contribute_admissible.py`](02_contribute_admissible.py) | Gate → contribute golden fixture → verify row in memory (temp SQLite) |
| [`03_export_inbox.sh`](03_export_inbox.sh) | Export → validate → dry-run ingest for federated git inbox PRs |

**Prerequisites:** Rust toolchain, repo root as cwd, `cargo build -p umst-mcp --features agent-layer` (scripts invoke `cargo run`).

```bash
python3 examples/agent/01_gate_explore.py
python3 examples/agent/02_contribute_admissible.py
UMST_MEMORY_DB=.umst-memory/memory.db bash examples/agent/03_export_inbox.sh
```
