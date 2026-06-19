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

### P1 (shipped)

| Tool | Purpose |
|------|---------|
| `umst_transition_propose` | Predict → gate → async contribute (`job_id` for `umst_contribute_status`) |

**Transport:** shipped as **hand-rolled stdio JSON-RPC** (MCP 2024-11-05). See [Deliberately defer](#deliberately-defer-v1) for protocol upgrades.

### MCP prompts (`agent-layer`)

| Prompt | Purpose |
|--------|---------|
| `contribute-admissible` | Safe contribute workflow |
| `query-near-mix` | L1 locality query |
| `gate-before-contribute` | Hard gate ordering |
| `interpret_gate_failure` | Read `gate_reject.v1` + `explain.regime_violations` |
| `suggest_similar_mix` | Paginated `near_mix_spec` search |
| `audit_mix_csv` | Batch CSV audit workflow via `umst_audit` |

Call `prompts/list` then `prompts/get` with the prompt name.

---

## Tool ↔ schema cross-link

| MCP tool | JSON Schema resource | Example request | Example response shape |
|----------|---------------------|-----------------|------------------------|
| `umst_gate_check` | *(inline `GateSummary` + optional `gate_reject.v1`)* | `{"mix":{"w_c":"1/2","temperature_k":"29315/100"},"explain":true}` | `{"gate_summary":{...},"gate_reject":{...},"explain":{"regime_violations":[...],"catalog_witnesses":[...]}}` — `isError: true` on REJECT |
| `umst_contribute` | `umst://schemas/contribution.v1.json` | `{"contribution":{...}}` | `AcceptResult` (`memory_id`, `content_id`, `observed_at`) |
| `umst_memory_query` | `umst://schemas/memory_record.v1.json` | `{"near_mix_spec":{...},"max_mix_l1":0.05,"limit":10}` | `{"rows":[...],"next_cursor":"..."}` |
| `umst_contribute_status` | — | `{"job_id":"..."}` | `ContributeJob` status enum |
| `umst_mi_estimate` | — | `{"mix":{...}}` | `{"mi_bits_est":"...","advisory":true}` |

All agent tool `inputSchema` objects declare `$schema: https://json-schema.org/draft/2020-12/schema` and MCP annotations (`readOnlyHint` / `destructiveHint`).

---

## Error catalog

| Error / signal | When | Agent action |
|----------------|------|--------------|
| `AcceptError::Validation` | `contribution.v1` schema / rational parse fail | Fix wire against `contribution.v1.json`; re-validate |
| `AcceptError::GateReject` | `gate_summary.admissible=false` or gate re-check fail | Run `umst_gate_check`; never bypass |
| `AcceptError::Scope` | `UMST_AGENT_SCOPE_TOKENS` mismatch | Supply valid `scope_token` |
| `AcceptError::NonMonotonicStamp` | `observed_at` regresses session clock | Use server-assigned stamps on accept |
| `AcceptError::Store` duplicate | Same `content_id` / idempotency key | Treat as success if idempotent retry |
| `gate_summary.verdict: REJECT` | Thermodynamic CD fail | Read `explain.regime_violations`; adjust mix |
| MCP `isError: true` on gate_check | Same as REJECT | Parse embedded `gate_reject.v1`; do not contribute |
| `unknown job_id` | Stale async poll | Re-submit contribute or check `contribute_jobs.json` beside DB |

---

## Operator runbook (one page)

```text
1. Bootstrap (optional)
   python3 scripts/bootstrap_memory_from_audit.py … | python3 scripts/ingest_contributions.py --db .umst-memory/memory.db

2. Export UMST_MEMORY_DB + UMST_UCRS_WITNESS (synthetic for CI, live for promotion path)
   export UMST_MEMORY_DB=.umst-memory/memory.db

3. Gate every proposal
   umst_gate_check(mix, explain:true) → admissible before contribute

4. Contribute admissible rows
   umst_contribute(contribution) → memory_id

5. Query similar cases
   umst_memory_query(near_mix_spec, max_mix_l1, cursor pagination)

6. Export signed checkpoint (CLI, not MCP)
   umst memory export --db .umst-memory/memory.db --out exports/run-001/
   → memory_export_bundle.v1.json + memory.jcs.jsonl + hash_chain

7. Human promotion (never automatic)
   umst promote-contribution --approval promotion_approval.v1.json
```

**Async contribute (v1 close-out):** `umst_contribute` with `async: true` runs predict+gate+accept **in-process** on the same stdio session and persists job state to `contribute_jobs.json` beside `UMST_MEMORY_DB`. There is no separate worker daemon in v1 — inline accept after gate is acceptable for agent timeouts; poll `umst_contribute_status` for the result. A dedicated background worker is deferred.

**`contribute_jobs` SSOT:** JSON sidecar (`contribute_jobs.json`), not a SQLite table in v1. Jobs are ephemeral operator state; durable truth remains `memory_records` + JSONL sidecars.

Async contribute jobs persist in `contribute_jobs.json` next to `UMST_MEMORY_DB` when set.

---

## Environment variables (extended)

| Variable | Values | Role |
|----------|--------|------|
| `UMST_UCRS_WITNESS` | `live` \| `synthetic` (default) | Session clock mode |
| `UMST_MEMORY_DB` | SQLite file path | Durable `memory_records` + `contribute_jobs.json` sidecar |
| `UMST_MEMORY_JSONL` | Optional path override | JSONL sidecar destination |
| `UMST_MEMORY_REGIME` | e.g. `standard_20C_water` | Default curing regime filter hint (manifold registry) |
| `UMST_MEMORY_L1_RADIUS` | rational string | Default `max_mix_l1` hint for locality queries |
| `UMST_MEMORY_MORTON_DEPTH` | integer | Morton depth for geometry indexing (concrete cartridge) |
| `UMST_AGENT_SCOPE_TOKENS` | comma-separated | Required scope tokens on contribute when set |

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

**Litestream → S3 Object Lock:** deferred (storage research). See [`MEMORY_REPLICATION.md`](MEMORY_REPLICATION.md) + [`docs/examples/litestream.yml`](examples/litestream.yml). Per-deployment SQLite + JSONL sidecar is v1; replicate via signed export bundles or external backup tooling.

**Sigstore / in-toto on promotion bundle:** deferred for v1 CI; document human `promote-contribution` path only. **RFC 3161 TSA** countersign on promotion bundles is likewise deferred — not required per MCP tool call. See [`governance/promotion_policy.yaml`](../governance/promotion_policy.yaml) comments.

---

## Environment variables

| Variable | Values | Role |
|----------|--------|------|
| `UMST_UCRS_WITNESS` | `live` \| `synthetic` (default) | Session clock mode. `live` → `TemporalWitness::stamp()` emits `stamp_tier: UcrsTier2` on accept; `synthetic` → CI-safe deterministic stamps. |
| `UMST_MEMORY_DB` | SQLite file path | Enables durable `memory_records` (STRICT + WAL). When unset, session is in-memory only. |
| `UMST_MEMORY_JSONL` | Optional path override | JSONL sidecar destination (default: `.umst-memory/memory.jcs.jsonl` beside the DB). |

**Live witness + promotion (Track A):** When `UMST_UCRS_WITNESS=live`, human promotion (`umst promote-contribution`) requires the memory row `observed_at.stamp_tier` to be **`UcrsTier2`**. Synthetic stamps are rejected on the promotion path. Hold-out metrics and human `promotion_approval.v1` are still required; RFC 3161 / Sigstore on the promotion bundle remain deferred.

```bash
export UMST_UCRS_WITNESS=live
export UMST_MEMORY_DB=.umst-memory/memory.db
cargo run -p umst-mcp --features agent-layer,ucrs-provenance
```

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

## UCRS sidecar (constitutional time)

For **live Tier-2 observation stamps** (`UMST_UCRS_WITNESS=live`), operators may run the [`umst-ucrs`](https://github.com/tytolabs/umst-ucrs) daemon as a **sidecar process** alongside `umst-mcp`: the MCP agent stays material-memory-only while the sidecar owns the thermodynamic clock, credit ledger, and Prometheus metrics (`:9090/metrics`). Wire `ProvenanceClock` to the sidecar via env (`UMST_UCRS_WITNESS=live`) or in-process `umst_ucrs` when embedding the library; P2P gossip is optional (`p2p` feature / `umst-ucrs-p2p` binary). See [`TemporalWitness`](https://github.com/tytolabs/umst-ucrs/blob/master/Rust/src/observation.rs), [`Docs/HLC_SIDECAR.md`](https://github.com/tytolabs/umst-ucrs/blob/master/Docs/HLC_SIDECAR.md), `scripts/umst-ucrs.service`, and the umst-ucrs `Dockerfile` for production layout.

---

## Deliberately defer (v1)

| Temptation | Why wait |
|------------|----------|
| **`rmcp` 1.7 rewrite** | Hand-rolled stdio JSON-RPC (MCP 2024-11-05) is shipped and Cursor-stable; migrate when upstream prompts/resources stabilize |
| **MCP 2025-11-25 protocol** | No multi-tenant hosted MCP requirement yet |
| **Streamable HTTP + OAuth** | stdio + Docker suffices for cartridge-local agents |
| **ghcr OCI MCP distribution** | Docker `ghcr.io/tytolabs/umst-concrete-cartridge` documented; dedicated MCP OCI layer deferred |
| **RFC 3161 TSA on promotion bundle** | Human-gated promotion only; external anchor deferred |
| **Sigstore / in-toto per-contribute** | Bundle-release Sigstore only (also deferred for v1 CI) |
| **Litestream S3 Object Lock** | See [`MEMORY_REPLICATION.md`](MEMORY_REPLICATION.md) |
| **SQLite `contribute_jobs` table** | JSON sidecar SSOT in v1 |
| **Background contribute worker** | Inline in-process accept + `contribute_jobs.json` poll |

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
- [`umst-ucrs`](https://github.com/tytolabs/umst-ucrs) — observation stamps, [`TemporalWitness`](https://github.com/tytolabs/umst-ucrs/blob/master/Rust/src/observation.rs), [`HLC_SIDECAR.md`](https://github.com/tytolabs/umst-ucrs/blob/master/Docs/HLC_SIDECAR.md)
- [`CARTRIDGE_PORT.md`](CARTRIDGE_PORT.md) — cross-cartridge port guide
- [`MEMORY_REPLICATION.md`](MEMORY_REPLICATION.md) — durability + Litestream defer
