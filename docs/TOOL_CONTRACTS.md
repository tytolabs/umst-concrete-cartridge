# MCP tool contracts (B3)

**Hand-derived from `umst-mcp` source in ToolDescriptor shape** (manifest-ready; §5.1).  
**Not** generated from a shipped manifest yet — **Proposed target:** one manifest → `list_tools` + this file.  
Index: [`AGENT_MCP.md`](AGENT_MCP.md). Inventory: [`AGENT_SURFACE_AUDIT.md`](AGENT_SURFACE_AUDIT.md).

Each contract has **seven fields** (table aliases → ToolDescriptor):

| Alias | Field |
|:---|:---|
| Pre | preconditions |
| Post | postconditions |
| Errors | error taxonomy |
| Idempotent | idempotency |
| SideEffectClass | SideEffectClass |
| Cost | cost |
| Provenance | provenance |

**SideEffectClass (current mapping / target enum):** `ReadOnly` | `EpistemicMutating` | `NetworkIo` | `Pure` (Proposed names from §5.1).

---

## Shared error shapes

| Shape | When | Remediation |
|:---|:---|:---|
| `gate_reject.v1` + `explain` | Gate REJECT | Adjust mix; re-gate |
| `agent_error.v1` | Tool/transport failure | See `code` / AGENT_MCP error table |
| `-32601` | Unknown tool / missing feature | Enable `agent-layer` |

---

## `umst_gate_check` — COLD · ReadOnly

| Field | Contract |
|:---|:---|
| Pre | Valid rational `mix` (+ optional `profile`) |
| Post | `gate_summary.admissible` bool; on fail `isError` + reject + explain (default) |
| Errors | parse fail, CD fail, profile_load_fail |
| Idempotent | Yes (read-only) |
| SideEffectClass | ReadOnly |
| Cost | Landauer/MI advisory fields may appear; not a billable sync |
| Provenance | `catalog_ids` witnesses |

Source: `agent_layer.rs:613`, `main.rs:502–508`.

---

## `umst_gate_check_arena` — HOT · ReadOnly

| Field | Contract |
|:---|:---|
| Pre | Live `arena_session_id` from `umst_arena_open`; valid mix |
| Post | Same physics as gate_check + arena catalog witnesses |
| Errors | unknown session; arena-session feature missing |
| Idempotent | Yes |
| SideEffectClass | ReadOnly |
| Cost | Avoids re-parse; still Landauer-advisory optional |
| Provenance | Arena + gate catalog ids |

Source: `agent_layer.rs:730`.

---

## `umst_arena_open` — HOT · EpistemicMutating (session map)

| Field | Contract |
|:---|:---|
| Pre | Path/bytes to trusted arena file |
| Post | `arena_session_id` |
| Errors | parse/header fail |
| Idempotent | No (new session each call) |
| SideEffectClass | EpistemicMutating (session state) |
| Cost | One-time parse |
| Provenance | Arena digest/header |

Source: `agent_layer.rs:716`.

---

## `umst_arena_close` — HOT · EpistemicMutating

| Field | Contract |
|:---|:---|
| Pre | `arena_session_id` |
| Post | Session dropped |
| Errors | unknown id |
| Idempotent | Closing twice → error or no-op (treat as non-idempotent) |
| SideEffectClass | EpistemicMutating |
| Cost | Negligible |
| Provenance | N/A |

Source: `agent_layer.rs:747`.

---

## `umst_contribute` — COLD · EpistemicMutating

| Field | Contract |
|:---|:---|
| Pre | `contribution.v1` with `gate_summary.admissible=true`; prior PASS |
| Post | `memory_id`, `content_id`, `observed_at` (UCRS stamp fields) |
| Errors | `contribute_gate_reject`, schema fail, scope_token |
| Idempotent | Yes with `idempotency_key` |
| SideEffectClass | EpistemicMutating |
| Cost | Memory write; Landauer not charged as sync |
| Provenance | `observed_at` / UCRS stamp tiers |

Source: `agent_layer.rs:629`. **No dry-run** — Proposed P2.

---

## `umst_contribute_status` — COLD · ReadOnly

| Field | Contract |
|:---|:---|
| Pre | `job_id` from async contribute / transition |
| Post | Job status `Running` / `Succeeded` / `Failed` (`ContributeJobStatus` @ `agent_layer.rs:91`) |
| Errors | `missing_argument`, `unknown_job_id` (`main.rs:575–594`) |
| Idempotent | Yes |
| SideEffectClass | ReadOnly |
| Cost | Negligible poll |
| Provenance | Job record only (no new stamp) |

Source: `agent_layer.rs:646`.

---

## `umst_memory_query` — COLD · ReadOnly

| Field | Contract |
|:---|:---|
| Pre | Optional filters (`near_mix_spec`, `max_mix_l1`, cursor) |
| Post | `rows`, `next_cursor` |
| Errors | `agent_error.v1` on DB/filter parse fail (no silent empty-as-success for hard errors) |
| Idempotent | Yes |
| SideEffectClass | ReadOnly |
| Cost | Local SQLite read |
| Provenance | Returned rows carry stamps if stored |

Source: `agent_layer.rs:660`.

---

## `umst_mi_estimate` — COLD · ReadOnly

| Field | Contract |
|:---|:---|
| Pre | Mix |
| Post | `mi_bits_est`, `advisory: true` — **not** admissibility |
| Errors | `missing_argument` (`main.rs:603–608`); mix parse via `agent_error.v1` |
| Idempotent | Yes |
| SideEffectClass | ReadOnly |
| Cost | Advisory Landauer surrogate only |
| Provenance | Advisory estimate — no catalog witness claim |

Source: `agent_layer.rs:685`.

---

## `umst_transition_propose` — COLD · EpistemicMutating

| Field | Contract |
|:---|:---|
| Pre | Mix (+ profile); will gate before contribute |
| Post | `job_id` for status poll |
| Errors | gate reject (`isError`); `profile_load_fail`; transition enqueue fail |
| Idempotent | No (new job) |
| SideEffectClass | EpistemicMutating |
| Cost | Predict + gate + async contribute enqueue |
| Provenance | Job → contribute path carries `observed_at` on success |

Source: `agent_layer.rs:699`.

---

## `umst_predict` — COLD · ReadOnly

| Field | Contract |
|:---|:---|
| Pre | Mix + profile |
| Post | `result.v2` constitutive envelope |
| Errors | `profile_load_fail`, `mix_parse_fail`, `predict_fail`, `serialize_fail`, `canonical_json_fail` (`main.rs:204–267`) |
| Idempotent | Yes |
| SideEffectClass | ReadOnly |
| Cost | Constitutive eval (cold stdio) |
| Provenance | `formal_anchor` / profile id in envelope when present |

Source: `main.rs:114`.

---

## `umst_audit` — COLD · ReadOnly

| Field | Contract |
|:---|:---|
| Pre | CSV text + profile |
| Post | `audit.v1` rows / warnings |
| Errors | `profile_load_fail`; CSV/parse `agent_error.v1` (`main.rs:305+`) |
| Idempotent | Yes |
| SideEffectClass | ReadOnly |
| Cost | Batch over CSV rows |
| Provenance | Profile id; per-row warnings — not gate PASS |

Source: `main.rs:132`.

---

## `umst_profiles` — COLD · Pure/ReadOnly

| Field | Contract |
|:---|:---|
| Pre | None |
| Post | Bundled profile ids |
| Errors | None expected (empty args); transport-level only |
| Idempotent | Yes |
| SideEffectClass | Pure |
| Cost | Negligible list |
| Provenance | Bundled profile set (crate bake) |

Source: `main.rs:149`.

---

## `umst_certify` — COLD · ReadOnly

| Field | Contract |
|:---|:---|
| Pre | Profile id |
| Post | Formal-anchor certify chain JSON |
| Errors | `certify_missing_profile`, `profile_load_fail` (`main.rs:378+`) |
| Idempotent | Yes |
| SideEffectClass | ReadOnly |
| Cost | Negligible chain emit |
| Provenance | Formal-anchor URIs in chain |

Source: `main.rs:154`.

---

## Acceptance

All 13 tools have the 7 fields; no silent failure; Proposed tools absent from this list.
