# Agent MCP contract — Physical Reasoning Layer

**Audience:** Cursor agents, SDK integrations, `umst-mcp` stdio consumers  
**Schemas:** [`schemas/`](../schemas/) (CI-validated)  
**Examples:** [`examples/agent/`](../examples/agent/)  
**ADR:** [`outputs/.plans/archive/prl-shipped/ai-physical-reasoning-layer.md`](../../outputs/.plans/archive/prl-shipped/ai-physical-reasoning-layer.md)

---

## Quick Start (< 5 minutes)

```bash
cd umst-concrete-cartridge
cargo build -p umst-mcp --features agent-layer
export UMST_MEMORY_DB=.umst-memory/memory.db   # optional; in-memory if unset
cargo run -p umst-mcp --features agent-layer
```

Or run the example script (spawns MCP, exercises gate + query):

```bash
python3 examples/agent/01_gate_explore.py
```

**Minimal JSON-RPC exchange** (one line per message on stdin/stdout):

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"agent","version":"0.1"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"umst_gate_check","arguments":{"mix":{"w_c":"9/20","temperature_k":"29315/100","aggregate_volume_fraction":"7/10"}}}}
```

On **REJECT**, the tool returns `isError: true` with `gate_reject.v1` and `explain` (default `explain: true`):

```json
{
  "gate_summary": { "admissible": false, "verdict": "REJECT", "catalog_ids": ["umst.gate.cd_transition"] },
  "gate_reject": { "schema_version": "gate_reject.v1", "verdict": "REJECT" },
  "explain": {
    "regime_violations": ["mix_spec_rational_parse_fail"],
    "remediation": ["Use rational strings like \"3/4\" for all mix fields …"],
    "fields": [{ "path": "mix.w_c", "issue": "rational_parse_fail" }],
    "catalog_witnesses": ["umst.gate.cd_transition"]
  }
}
```

**Smoke test:** `python3 scripts/mcp_smoke.py --agent-layer`

---

## Core concepts

The Physical Reasoning Layer (PRL) is a **gate-validated, cartridge-local memory** for structured mix/outcome contributions. It is **not** RAG, not a paper store, and not a vector database.

| Concept | Meaning |
|---------|---------|
| **Gate** | Hard thermodynamic admissibility (`umst_gate_check`) before any memory write |
| **Memory** | Append-only research rows (`memory_record.v1`); query by regime / L1 / Morton locality |
| **Contribution** | Schema-valid `contribution.v1` ingest via `umst_contribute` (admissible only) |
| **Promotion** | Human-gated calibration update (`umst promote-contribution`) — **never** automatic from MCP |

Agent contract:

1. **Check** admissibility (`umst_gate_check`).
2. **Query** prior gate-passed cases (`umst_memory_query`).
3. **Contribute** only admissible rows (`umst_contribute`).
4. **Never** expect silent calibration updates.

```text
umst_gate_check → umst_predict (optional) → umst_contribute
       → umst_memory_query (similar cases)
       → human promotion_approval.v1 → umst promote-contribution
       → calibration/*.toml (never automatic from MCP)
```

---

## Build profiles

| Profile | Features | Use |
|---------|----------|-----|
| Default CI | *(none)* | `umst_predict`, `umst_audit`, `umst_profiles`, `umst_certify` |
| Agent layer | `agent-layer` on `umst-mcp` | + gate, contribute, memory query, schema resources |
| UCRS stamps | `ucrs-provenance` on cartridge | Live observation stamp on ingest (`observed_at`) |
| Manifest CD | `manifest-bridge` (via `agent-layer`) | Runtime Clausius–Duhem gate on mix |

Docker agent image should enable `agent-layer` + `manifest-bridge`.

**Transport:** hand-rolled stdio JSON-RPC (MCP 2024-11-05). See [Limitations](#limitations-deliberately-defer-v1).

---

## Tool reference

### Shipped (always)

| Tool | Purpose |
|------|---------|
| `umst_predict` | Constitutive prediction (`result.v2`) |
| `umst_audit` | Batch CSV audit (`audit.v1`) |
| `umst_profiles` | Bundled calibration profile list |
| `umst_certify` | Formal-anchor certify chain for a profile |

### Agent layer (`agent-layer`)

| Tool | Purpose |
|------|---------|
| `umst_gate_check` | Hard admissibility + `catalog_ids` + `mi_bits_est`; `explain` defaults **true** |
| `umst_contribute` | Ingest `contribution.v1` → research memory (admissible only); `idempotency_key` dedup |
| `umst_contribute_status` | Poll async contribute job (`async: true` on contribute) |
| `umst_memory_query` | Filter rows by regime / L1 / Morton locality |
| `umst_mi_estimate` | Advisory MI bits surrogate (**not** admissibility) |
| `umst_transition_propose` | Predict → gate → async contribute (`job_id` for status poll) |

### Tool ↔ schema cross-link

| MCP tool | JSON Schema resource | Example request | Example response |
|----------|---------------------|-----------------|------------------|
| `umst_gate_check` | inline + `gate_reject.v1` | `{"mix":{"w_c":"1/2","temperature_k":"29315/100"}}` | `gate_summary`, optional `gate_reject`, `explain` — `isError: true` on REJECT |
| `umst_contribute` | `umst://schemas/contribution.v1.json` | `{"contribution":{...}}` | `memory_id`, `content_id`, `observed_at` |
| `umst_memory_query` | `umst://schemas/memory_record.v1.json` | `{"near_mix_spec":{...},"max_mix_l1":0.05,"limit":10}` | `rows`, `next_cursor` |
| `umst_contribute_status` | — | `{"job_id":"..."}` | `ContributeJob` status |
| `umst_mi_estimate` | — | `{"mix":{...}}` | `mi_bits_est`, `advisory: true` |

Call `resources/list` then `resources/read` — do not guess field shapes.

All agent tool `inputSchema` objects declare `$schema: https://json-schema.org/draft/2020-12/schema` and MCP annotations (`readOnlyHint` / `destructiveHint`).

---

## Common workflows

### Safe exploration (read-only)

Use MCP prompt `safe-exploration` (`prompts/get`).

1. `umst_gate_check` on candidate `mix_spec` (`explain` defaults true).
2. Optional `umst_predict` for constitutive detail.
3. `umst_memory_query` for similar admissible cases.
4. **Never** call `umst_contribute` when `gate_summary.admissible` is false.

**Dry-run pattern:** gate check and predict are read-only. For inbox JSONL without writing memory, use the [Federated git inbox](#federated-git-inbox) validate + `ingest_contributions.py <file> --dry-run --skip-gate` path.

### Contribute admissible row

Use prompts `gate-before-contribute` + `contribute-admissible`.

1. `umst_gate_check` → must PASS.
2. Build `contribution.v1` with matching `gate_summary.admissible: true`.
3. `umst_contribute` → `memory_id`.
4. Optional `umst_memory_query` to verify row appears.

See [`examples/agent/02_contribute_admissible.py`](../examples/agent/02_contribute_admissible.py).

**No contribute preview flag (v1):** `umst_contribute` always writes on success — there is no `preview: true` or dry-run mode on the MCP tool. Treat `umst_gate_check` (with default `explain: true`) plus `resources/read` on `umst://schemas/contribution.v1.json` as the preflight path. A dedicated contribute preview arg is deferred; inbox JSONL uses `ingest_contributions.py --dry-run` instead (see below).

### Federated git inbox

Local `umst_contribute` writes to **your** `UMST_MEMORY_DB` only. To propose rows for the **shared corpus**, export JSONL and open a PR — federation latency is review time, not live MCP sync.

| Step | Who | Action |
|------|-----|--------|
| 1 | Lab | `umst_gate_check` → `umst_contribute` with `UMST_MEMORY_DB` set |
| 2 | Lab | Export admissible rows not already in `contributions/merged/` |
| 3 | Lab | Local validate + dry-run ingest (no SQLite write) |
| 4 | Lab | PR adding **one** file under `contributions/inbox/` |
| 5 | CI | Schema, admissible flag, duplicate scan vs `MANIFEST.jsonl`, dry-run ingest |
| 6 | Maintainer | Merge → move shard to `contributions/merged/YYYY-MM/` → append manifest |

**Naming:** `contributions/inbox/<lab-slug>-<YYYYMMDD>-<6char>.jsonl` (example: `tyto-20260619-a1b2c3.jsonl`). One JSONL per PR preferred.

**Contributor commands** (from repo root):

```bash
export UMST_MEMORY_DB=.umst-memory/memory.db

# Export: skips content_ids already in contributions/merged/MANIFEST.jsonl
python3 scripts/export_contributions_jsonl.py \
  --db "$UMST_MEMORY_DB" --lab <slug> \
  --out contributions/inbox/<slug>-<YYYYMMDD>-<id>.jsonl

# Validate: schema + gate_summary.admissible + duplicate scan (add --gate-check for local MCP re-check)
python3 scripts/validate_contribution_inbox.py contributions/inbox/<file>.jsonl

# Dry-run ingest: parse + structure check only; writes nothing (--skip-gate trusts embedded gate_summary; CI re-checks gate separately)
python3 scripts/ingest_contributions.py contributions/inbox/<file>.jsonl --dry-run --skip-gate
# Success prints {"counts":{"would_insert":N},...,"dry_run":true} and exits 0
```

**Dry-run vs live ingest:**

| Flag | Effect |
|------|--------|
| `--dry-run` | No SQLite or JSONL writes; rows that would insert report `would_insert` in `counts` |
| `--skip-gate` | Trust `gate_summary.admissible` on each line (bootstrap / inbox CI speed); **do not** use for untrusted input without `validate_contribution_inbox.py --gate-check` |
| *(neither)* | Full MCP `umst_gate_check` re-run per line before insert |

After maintainer merge, other labs refresh local memory:

```bash
cat contributions/merged/*/*.jsonl | python3 scripts/ingest_contributions.py --db .umst-memory/memory.db
```

MCP prompt `export_for_git_inbox` walks the same flow. See [`contributions/README.md`](../contributions/README.md). **Not** live git push from MCP — human PR merge required. Merge does **not** auto-update calibration profiles (`umst promote-contribution` stays human-only).

**Runnable example:**

```bash
UMST_MEMORY_DB=.umst-memory/memory.db bash examples/agent/03_export_inbox.sh
```

See [`examples/agent/03_export_inbox.sh`](../examples/agent/03_export_inbox.sh).

---

## Error handling

Recoverable failures use **two complementary transports**. Agents must distinguish them:

| Transport | JSON-RPC `error`? | `result.isError` | Payload shape | When |
|-----------|-------------------|------------------|---------------|------|
| **Fatal** | yes (`-32603` / `-32601`) | — | plain `message` string | Unknown tool/method, protocol bugs |
| **Recoverable tool** | no | `true` | `agent_error.v1` or domain schema | Missing args, profile load, contribute reject, gate REJECT |
| **Recoverable gate** | no | `true` | `gate_reject.v1` + `explain` | `umst_gate_check` thermodynamic REJECT |

**Rule:** If `result.isError` is `true`, parse the text `content[0].text` as JSON — never treat the call as a hard JSON-RPC failure.

### `agent_error.v1` (recoverable tool errors)

Structured errors for agent-correctable mistakes (bad profile, missing `mix`, contribute validation, etc.). Returned inside a normal `tools/call` **result** with `isError: true` — not a JSON-RPC `error` frame.

**MCP envelope:**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [{ "type": "text", "text": "{ … agent_error body … }" }],
    "isError": true
  }
}
```

**Body shape (`agent_error.v1`):**

```json
{
  "agent_error": {
    "schema_version": "agent_error.v1",
    "code": "mix_parse_fail",
    "message": "mix parse error: …",
    "remediation": "Use rational strings like \"9/20\" for w_c and temperature_k; see contribution.v1 schema."
  }
}
```

| Field | Type | Role |
|-------|------|------|
| `schema_version` | `"agent_error.v1"` | Wire version pin |
| `code` | string | Stable machine code (see table below) |
| `message` | string | Human-readable detail |
| `remediation` | string | Actionable fix for the agent |

**Agent remediation loop:**

1. Check `result.isError` on every `tools/call` response.
2. If `true`, `JSON.parse(content[0].text)`.
3. If `agent_error` is present → read `code` + `remediation`; fix wire and retry.
4. If `gate_reject` / `explain` is present → use prompt `interpret_gate_failure` or read `explain.remediation`.
5. Do **not** retry blindly on `contribute_gate_reject` — re-run `umst_gate_check` first.

**Stable `code` values (umst-mcp `agent-layer`):**

| `code` | Tool(s) | Agent action |
|--------|---------|--------------|
| `missing_argument` | gate, contribute, status, MI, transition | Supply required field (`mix`, `contribution`, `job_id`) |
| `profile_load_fail` | predict, gate, contribute, transition | Call `umst_profiles`; use bundled id (`default`, `uci_d1`) |
| `mix_parse_fail` | predict, transition | Rational strings for all mix fields |
| `predict_fail` | predict | Verify mix + profile calibration |
| `contribute_validation_fail` | contribute, transition | Fix wire against `contribution.v1.json` |
| `contribute_gate_reject` | contribute, transition | Run `umst_gate_check` first |
| `contribute_scope_fail` | contribute | Supply valid `scope_token` |
| `contribute_non_monotonic_stamp` | contribute | Use server-assigned `observed_at` |
| `contribute_store_fail` | contribute | Check `UMST_MEMORY_DB`; duplicate may be idempotent |
| `unknown_job_id` | contribute_status | Re-submit or check `contribute_jobs.json` beside DB |
| `transition_gate_reject` | transition_propose | Gate must PASS before async propose |
| `transition_propose_fail` | transition_propose | Verify mix/outcome/process wire |
| `resource_read_fail` | resources/read | Use `resources/list` URIs |
| `prompt_not_found` | prompts/get | Use `prompts/list` names |

Gate REJECT on `umst_gate_check` still returns `gate_reject.v1` + `explain` (not `agent_error.v1`) with `isError: true` — see Quick Start example above.

### Gate violation codes (`explain.regime_violations`)

| Code | Meaning | Agent action |
|------|---------|--------------|
| `mix_spec_rational_parse_fail` | Non-rational or missing required field | Use `"n/d"` strings; check `explain.fields` for paths |
| `mix_spec_wire_invalid` | Parsed wire failed `MixSpec` validation | Compare against `contribution.v1.json` |
| `thermodynamic_cd_fail` | Clausius–Duhem margin negative | Reduce `w_c`, adjust `temperature_k` / curing regime |
| `manifest_bridge_disabled` | Gate not compiled with manifest-bridge | Build with `agent-layer` + `manifest-bridge` |
| `thermodynamic_fail` | Generic CD fail | Read `explain.remediation`; iterate gate check |

Each code has a matching entry in `explain.remediation` and optional `explain.fields`.

**Golden vectors (SSOT):** adversarial gate + `query_page` expectations are pinned in [`tests/fixtures/phase8_adversarial.json`](../tests/fixtures/phase8_adversarial.json). See [`GOLDEN_VECTORS.md`](GOLDEN_VECTORS.md) for manifest layout, fixture paths, and `python3 scripts/validate_golden_vectors.py`.

### Contribute / transport errors

| Error / signal | When | Agent action |
|----------------|------|--------------|
| `AcceptError::Validation` | Schema / rational parse fail | Fix wire against `contribution.v1.json` |
| `AcceptError::GateReject` | Gate re-check fail on contribute | Run `umst_gate_check` first; never bypass |
| `AcceptError::Scope` | `UMST_AGENT_SCOPE_TOKENS` mismatch | Supply valid `scope_token` |
| `AcceptError::NonMonotonicStamp` | `observed_at` regresses session clock | Use server-assigned stamps |
| `AcceptError::Store` duplicate | Same `content_id` / idempotency key | Treat as success if idempotent retry |
| MCP `isError: true` on gate_check | REJECT verdict | Parse `gate_reject.v1` + `explain`; do not contribute |
| MCP `isError: true` + `agent_error.v1` | Recoverable tool mistake | Read `code` + `remediation`; fix wire and retry |
| `unknown job_id` | Stale async poll | Re-submit or check `contribute_jobs` beside DB |

**Fatal JSON-RPC (`error`, not `isError`):** unknown tool/method (`-32601`), unhandled internal faults (`-32603`). Most recoverable paths above now return `agent_error.v1` instead of bare `-32603` — if you still see a plain `message` string, treat it as fatal or report upstream.

Use prompt `interpret_gate_failure` when parsing REJECT payloads.

---

## MCP prompts

Call `prompts/list` then `prompts/get`.

| Prompt | Purpose |
|--------|---------|
| `safe-exploration` | Read-only gate → predict → query workflow |
| `gate-before-contribute` | Hard gate ordering before ingest |
| `contribute-admissible` | Safe contribute workflow |
| `interpret_gate_failure` | Read `gate_reject.v1` + `explain` remediation |
| `query-near-mix` | L1 locality query |
| `suggest_similar_mix` | Paginated `near_mix_spec` search |
| `audit_mix_csv` | Batch CSV audit via `umst_audit` |
| `export_for_git_inbox` | Export JSONL → validate → PR to `contributions/inbox/` |

---

## Best practices

- **Rationals:** all physical quantities as `"numerator/denominator"` strings, never JSON floats.
- **Gate first:** always `umst_gate_check` before `umst_contribute`; REJECT rows never enter memory.
- **Explain:** leave `explain: true` (default) on gate check; read `remediation` and `fields`.
- **Scope tokens:** when `UMST_AGENT_SCOPE_TOKENS` is set, include matching `scope_token` on contribute.
- **Idempotency:** use `idempotency_key` on contribute for safe retries.
- **MI advisory:** `mi_bits_est` and `umst_mi_estimate` are hints, not admissibility proofs.
- **Performance / fast path:** for batched or performance-sensitive work, prefer the in-process library / arena path over per-call Docker MCP round-trips. MCP (stdio + Docker) stays the stable default; the planned `umst-runtime-arena` is an opt-in fast path that parses inputs **once** and loops in-process (target ≥10× an MCP round-trip). See [`umst-manifold/docs/RUNTIME_TOPOLOGY.md`](../../umst-manifold/docs/RUNTIME_TOPOLOGY.md) for the hot/warm/cold boundary.

---

## Environment variables

| Variable | Values | Role |
|----------|--------|------|
| `UMST_UCRS_WITNESS` | `live` \| `synthetic` (default) | Session clock mode. `live` → live observation stamp on accept; `synthetic` → CI-safe deterministic stamps. |
| `UMST_MEMORY_DB` | SQLite file path | Durable `memory_records` (STRICT + WAL) + `contribute_jobs` dual-write. When unset, session is in-memory only. |
| `UMST_MEMORY_JSONL` | Optional path override | JSONL sidecar (default: `.umst-memory/memory.jcs.jsonl` beside DB). |
| `UMST_MEMORY_REGIME` | e.g. `standard_20C_water` | Default curing regime filter hint |
| `UMST_MEMORY_L1_RADIUS` | rational string | Default `max_mix_l1` hint for locality queries |
| `UMST_MEMORY_MORTON_DEPTH` | integer | Morton depth for geometry indexing |
| `UMST_AGENT_SCOPE_TOKENS` | comma-separated | Required scope tokens on contribute when set |

**Live witness + promotion:** When `UMST_UCRS_WITNESS=live`, human promotion requires a live observation stamp (`observed_at.stamp_tier` must not be synthetic-only). Synthetic stamps are rejected on the promotion path.

```bash
export UMST_UCRS_WITNESS=live
export UMST_MEMORY_DB=.umst-memory/memory.db
cargo run -p umst-mcp --features agent-layer,ucrs-provenance
```

---

## Durable memory (`UMST_MEMORY_DB`)

When set, `AgentSession` uses **STRICT + WAL** `memory_records` with immutability triggers. Accepted rows also append to JSONL sidecar.

Bulk bootstrap (honest limit: no 18k public corpus in-repo):

```bash
python3 scripts/bootstrap_memory_from_audit.py fixtures/bootstrap_audit_slice.csv \
  | python3 scripts/ingest_contributions.py --skip-gate --db .umst-memory/memory.db
```

**Async contribute:** `umst_contribute` with `async: true` runs predict+gate+accept in-process. Poll `umst_contribute_status`. Job state dual-writes to SQLite `contribute_jobs` table and `contribute_jobs.json` sidecar when `UMST_MEMORY_DB` is set.

---

## Operator runbook

```text
1. Bootstrap (optional)
   python3 scripts/bootstrap_memory_from_audit.py … | python3 scripts/ingest_contributions.py --db .umst-memory/memory.db

2. export UMST_MEMORY_DB=.umst-memory/memory.db

3. umst_gate_check(mix) → admissible before contribute

4. umst_contribute(contribution) → memory_id

5. umst_memory_query(near_mix_spec, max_mix_l1, cursor pagination)

6. umst memory export --db .umst-memory/memory.db --out exports/run-001/

7. umst promote-contribution --approval promotion_approval.v1.json  (human only)
```

---

## Troubleshooting

### MCP session

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `profile load error` | Unknown `profile` arg | Call `umst_profiles`; use bundled id (e.g. `default`) |
| `missing mix` | Gate check without `mix` object | Pass full rational `mix_spec` |
| Empty `remediation` on REJECT | `explain: false` | Omit `explain` or set `explain: true` (default) |
| `manifest_bridge_disabled` in violations | MCP built without manifest-bridge | `cargo build -p umst-mcp --features agent-layer` (includes bridge) |
| `unknown job_id` | Stale poll or wrong DB path | Check `contribute_jobs.json` beside `UMST_MEMORY_DB` |
| Memory query always empty | In-memory session or strict filters | Set `UMST_MEMORY_DB`; relax `max_mix_l1` / regime filters |
| Contribute fails after gate PASS | Contribution wire mismatch | Validate against `umst://schemas/contribution.v1.json` |
| Need to test contribute wire without writing | No `preview` on `umst_contribute` | Run `umst_gate_check` first; validate JSON against `contribution.v1` schema |

### Federated inbox export

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| Export writes 0 rows | All `content_id`s already in `MANIFEST.jsonl` / merged shards, or DB has no admissible rows | Contribute new rows locally first; use `--since-content-id` for incremental export |
| `error: database not found` | `UMST_MEMORY_DB` unset or empty | Run `02_contribute_admissible.py` or set `--db` explicitly |
| `validate_contribution_inbox: duplicate content_id` | Row already merged into shared corpus | Export skips known IDs; do not re-export merged rows |
| `gate_summary.admissible` not true | REJECT row in JSONL | Only export gate-passed rows; rejects stay local |
| PR CI fails on inbox | Schema, duplicate, or dry-run ingest | Run the three local commands in [Federated git inbox](#federated-git-inbox) before pushing |

### Dry-run ingest (`ingest_contributions.py --dry-run`)

| `counts` key | Meaning | Fix |
|--------------|---------|-----|
| `would_insert` | Line would insert on live ingest | Expected on success |
| `skip_schema` | Not `contribution.v1` | Fix line against `schemas/contribution.v1.json` |
| `skip_gate` | MCP gate re-check failed (without `--skip-gate`) | Re-run `umst_gate_check`; fix mix |
| `skip_not_admissible` | `gate_summary.admissible` is false | Remove line or re-gate locally |
| Exit code 1 with `dry_run: true` | Any skip_* count > 0 | Fix failing lines; inbox CI uses the same dry-run path |

For trusted inbox shards in CI, `--dry-run --skip-gate` checks structure only; `validate_contribution_inbox.py` (without `--gate-check` in the default workflow) still enforces schema + admissible flag + manifest duplicate scan. Add `--gate-check` locally when you want per-line MCP gate re-check before opening a PR.

---

## Resources

With `agent-layer`, MCP exposes JSON Schema resources:

- `umst://schemas/contribution.v1.json`
- `umst://schemas/memory_record.v1.json`
- `umst://schemas/gate_reject.v1.json`
- `umst://schemas/promotion_*.v1.json`

---

## UCRS sidecar (constitutional time)

For **live observation stamps** (`UMST_UCRS_WITNESS=live`), operators may run [`umst-ucrs`](https://github.com/tytolabs/umst-ucrs) as a sidecar alongside `umst-mcp`. See [`TemporalWitness`](https://github.com/tytolabs/umst-ucrs/blob/master/Rust/src/observation.rs), [`HLC_SIDECAR.md`](https://github.com/tytolabs/umst-ucrs/blob/master/Docs/HLC_SIDECAR.md).

---

## Limitations (deliberately defer v1)

| Temptation | Why wait |
|------------|----------|
| **`rmcp` 1.7 rewrite** | [`MCP_PROTOCOL_ROADMAP.md`](MCP_PROTOCOL_ROADMAP.md) |
| **Streamable HTTP + OAuth** | stdio + Docker suffices for cartridge-local agents |
| **Hosted multi-tenant MCP** | Product decision required |
| **RFC 3161 TSA / Sigstore per-contribute** | Human-gated promotion only — [`PROMOTION_TRUST.md`](PROMOTION_TRUST.md) |
| **Litestream S3 Object Lock** | [`MEMORY_REPLICATION.md`](MEMORY_REPLICATION.md) |
| **Background contribute worker** | Inline accept + job poll |

---

## MI labeling

`gate_summary.mi_bits_est` and manifold `info_gain` surrogates are **not** histogram `epistemic_mi` unless `epistemic-ppo` is enabled. Treat as frugal hints, not proved information gain.

---

## FP implementation note

Pure morphisms live in `src/research/` (`validation`, `gate_check_mix`, `accept`, `filter_records`). The MCP server holds an `AgentSession` at the **stdio boundary only**. `umst-py` exposes `gate_check`, `memory_query`, `contribute` when built with `--features agent-layer`.

---

## Related

- [`GOLDEN_VECTORS.md`](GOLDEN_VECTORS.md) — phase8 adversarial golden-vector SSOT + CI parity
- [`README.md`](../README.md) — agent callout + §9
- [`examples/agent/`](../examples/agent/) — runnable workflows
- [`PROMOTION_TRUST.md`](PROMOTION_TRUST.md) — TSA + Sigstore operator scripts
- [`MCP_PROTOCOL_ROADMAP.md`](MCP_PROTOCOL_ROADMAP.md) — rmcp / HTTP defer criteria
- [`docker/README.md`](../docker/README.md) — OCI agent image + `server.json`
- [`CARTRIDGE_PORT.md`](CARTRIDGE_PORT.md) — cross-cartridge port guide
- [`MEMORY_REPLICATION.md`](MEMORY_REPLICATION.md) — durability + Litestream defer
