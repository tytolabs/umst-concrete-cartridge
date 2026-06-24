# Golden adversarial fixtures

Agent-eval fixture pack for the Physical Reasoning Layer gate boundary (`umst_gate_check` / `gate_check_mix`).

| File | Role |
|------|------|
| `admissible_mix_01.json` | `contribution.v1` instance — thermodynamically admissible OPC mix (w/c = 0.45) |
| `reject_mix_01.json` | `contribution.v1` instance — inadmissible high w/c mix (w/c = 0.75) |
| `expected_verdicts.json` | Expected gate outcomes for `golden_gate_check` parity (admissible + verdict per fixture) |

Fixtures use `stamp_tier: Synthetic` — isolated from production merge paths per UCRS logging policy.

## CI (live)

The [`.github/workflows/agent-layer.yml`](../../.github/workflows/agent-layer.yml) workflow runs on push/PR when these fixtures or agent-layer code change. The gate parity step is:

```bash
cargo test -p umst-concrete-cartridge --features agent-layer \
  --test research_memory --test golden_gate_check --test phase8_adversarial
```

| Test binary | What it asserts |
|-------------|-----------------|
| `golden_gate_check` | `expected_verdicts.json` → `gate_check_mix` admissible/verdict parity for both JSON fixtures |
| `phase8_adversarial` | Full `gate_check_mix_result` wire: `gate_reject.v1` on reject, `explain` remediation/fields, memory `query_page` edge cases (uses `admissible_mix_01.json` / `reject_mix_01.json`) |
| `research_memory` | Research store ingest/query invariants (same workflow job) |

The same workflow also runs `mcp_smoke.py --agent-layer` and `examples/agent/01_gate_explore.py`; those require a built MCP server and Python 3.

[`.github/workflows/schema.yml`](../../.github/workflows/schema.yml) asserts `contribution.v1` schema conformance on both fixture JSON files.

## Run locally (external researchers)

From a full clone of this repository (fixtures are not published as a standalone crate):

```bash
# Rust 1.88+ recommended (matches CI)
cargo test -p umst-concrete-cartridge --features agent-layer \
  --test golden_gate_check --test phase8_adversarial
```

**What works without extra setup**

- `golden_gate_check` and `phase8_adversarial` — pure Rust integration tests; no database file, Docker, or MCP process required.
- `expected_verdicts.json` is the SSOT for admissible/verdict expectations in `golden_gate_check`; `phase8_adversarial` additionally checks reject explain payloads (`regime_violations`, `remediation`, `fields`) and pagination filters.

**What needs more than `cargo test`**

- `agent-layer` feature (enabled above) pulls `manifest-bridge`, SQLite, and pinned `umst-manifold` — first compile may take several minutes.
- MCP smoke / `examples/agent/*` — need `cargo build` of `umst-mcp` and stdio JSON-RPC wiring; see [`docs/AGENT_MCP.md`](../../docs/AGENT_MCP.md).
- Bootstrap corpus row-count assertion in CI is optional locally (`fixtures/corpus/audit_corpus.v1.csv` may be absent in shallow checkouts).

**Honest scope:** these fixtures validate the in-process gate and research-memory API. They do not substitute for hosted MCP integration tests or live contribution promotion.
