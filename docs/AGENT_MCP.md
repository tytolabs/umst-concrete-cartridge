# Agent MCP contract — Physical Reasoning Layer

**Audience:** Cursor agents, SDK integrations, `umst-mcp` stdio consumers  
**Schemas:** [`schemas/`](../schemas/) (CI-validated)  
**ADR:** [`outputs/.plans/ai-physical-reasoning-layer.md`](../../outputs/.plans/ai-physical-reasoning-layer.md)

---

## What this is (and is not)

The Physical Reasoning Layer (PRL) is a **gate-validated, cartridge-local memory** for structured mix/outcome contributions. It is **not** RAG, not a paper store, and not a vector database.

Agents should:

1. **Check** admissibility before proposing material changes (`umst_gate_check`).
2. **Query** prior gate-passed cases (`umst_memory_query`).
3. **Contribute** only admissible, schema-valid rows (`umst_contribute`).
4. **Never** expect silent calibration updates — promotion is human-gated (`umst promote-contribution`).

---

## Build profiles

| Profile | Features | Use |
|---------|----------|-----|
| Default CI | *(none)* | `umst_predict`, `umst_audit`, `umst_profiles`, `umst_certify` |
| Agent layer | `agent-layer` on `umst-mcp` | + gate check, contribute, memory query, schema resources |
| UCRS stamps | `ucrs-provenance` on cartridge | Tier-2 `observed_at` fields on ingest |
| Manifest CD | `manifest-bridge` (via `agent-layer`) | Runtime Clausius–Duhem gate on mix |

```bash
cargo build -p umst-mcp --features agent-layer
cargo run -p umst-mcp --features agent-layer
```

Docker agent image should enable `agent-layer` + `manifest-bridge`.

---

## Tools

### Shipped (always)

| Tool | Purpose |
|------|---------|
| `umst_predict` | Constitutive prediction (`result.v2`) |
| `umst_audit` | Batch CSV audit (`audit.v1`) |
| `umst_profiles` | Bundled calibration profile list |
| `umst_certify` | Formal-anchor certify chain for a profile |

### P0 (`agent-layer`)

| Tool | Purpose |
|------|---------|
| `umst_gate_check` | Hard admissibility + `catalog_ids` + optional `mi_bits_est` for a `mix_spec` |
| `umst_contribute` | Ingest `contribution.v1` → research memory (admissible only); `idempotency_key` dedup |
| `umst_contribute_status` | Poll async contribute job (`async: true` on contribute) |
| `umst_memory_query` | Filter rows by regime / L1 / Morton locality |
| `umst_mi_estimate` | Advisory MI bits surrogate (not admissibility) |

### P1+ (roadmap)

`umst_transition_propose` — async heavy predict uses `umst_contribute` + `umst_contribute_status`.

**Transport:** shipped as **hand-rolled stdio JSON-RPC** (MCP 2024-11-05). `rmcp` 1.7 migration deferred — no behavior change required for agents today.

---

## Durable memory (`UMST_MEMORY_DB`)

When `UMST_MEMORY_DB` points at a SQLite path, `AgentSession` uses **STRICT + WAL** `memory_records` with immutability triggers. Accepted rows also append to `.umst-memory/memory.jcs.jsonl` alongside per-row JSON sidecars.

```bash
export UMST_MEMORY_DB=.umst-memory/memory.db
cargo run -p umst-mcp --features agent-layer
```

Bulk bootstrap (honest limit: no 18k public corpus in-repo):

```bash
python3 scripts/bootstrap_memory_from_audit.py fixtures/bootstrap_audit_slice.csv \\
  | python3 scripts/ingest_contributions.py --skip-gate --db .umst-memory/memory.db
```

**Litestream → S3 Object Lock:** deferred (storage research). Per-deployment SQLite + JSONL sidecar is v1; replicate via signed export bundles or external backup tooling.

**Sigstore / in-toto on promotion bundle:** deferred for v1 CI; document human `promote-contribution` path only.

---

## Resources

With `agent-layer`, MCP exposes JSON Schema resources:

- `umst://schemas/contribution.v1.json`
- `umst://schemas/memory_record.v1.json`
- `umst://schemas/gate_reject.v1.json`
- `umst://schemas/promotion_*.v1.json`

Call `resources/list` then `resources/read` — do not guess field shapes.

---

## Feedback loop

```text
umst_gate_check → umst_predict (optional detail) → umst_contribute
       → umst_memory_query (similar cases)
       → human promotion_approval.v1 → umst promote-contribution
       → calibration/*.toml (never automatic from MCP)
```

---

## FP implementation note

Pure morphisms live in `src/research/` (`validation`, `gate_check_mix`, `accept`, `filter_records`). The MCP server holds an `AgentSession` at the **stdio boundary only**; each `umst_contribute` returns an updated session value internally. With `UMST_MEMORY_DB`, the session backs onto SQLite; otherwise in-memory.

`umst-py` exposes `gate_check`, `memory_query`, `contribute` when built with `--features agent-layer`.

---

## MI labeling

`gate_summary.mi_bits_est` and manifold `info_gain` surrogates are **not** histogram `epistemic_mi` unless `epistemic-ppo` is enabled. Treat as frugal hints, not proved information gain.

---

## Smoke test

```bash
python3 scripts/mcp_smoke.py
python3 scripts/mcp_smoke.py --agent-layer
```

---

## Related

- [`README.md`](../README.md) §9 — agent protocol
- [`umst-ucrs`](https://github.com/tytolabs/umst-ucrs) — observation stamps (not material memory)
- [`umst-manifold` gate docs](https://github.com/tytolabs/umst-manifold/blob/main/docs/CARTRIDGE_PORT.md)
