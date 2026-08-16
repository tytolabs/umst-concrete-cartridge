SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
<!--
-->

# `umst-mcp`

Stdio [Model Context Protocol]() server (JSON-RPC 2.0, newline-delimited **stdin → stdout**) exposing the UMST concrete façade to agents and IDE integrations.

**Repository:** `github.com/tytolabs/umst-concrete-cartridge` (workspace member)

## Build and run

```bash
cd umst-concrete-cartridge
cargo run -p umst-mcp
```

## Tools

Default build (`default = ["agent-layer"]`) exposes **13** tools. Base build (`--no-default-features`) exposes the historical 4-tool surface (`umst_predict`, `umst_audit`, `umst_profiles`, `umst_certify`).

| Tool | HOT/COLD | Behaviour |
|------|----------|-----------|
| `umst_gate_check` | HOT | Hard thermodynamic admissibility gate — run before writes. |
| `umst_predict` | HOT | Constitutive prediction envelope (`result.v1` / `result.v2`). |
| `umst_contribute` | COLD | Gate-validated memory ingest. |
| `umst_memory_query` | COLD | Paginated memory filter. |
| `umst_profiles` | COLD | Bundled calibration profile ids (deterministic order). |
| `umst_audit` | COLD | Batch CSV audit (`audit.v1`). |
| `umst_certify` | COLD | Certification chain JSON (CLI `certify` mirror). |
| `umst_mi_estimate` | COLD | Advisory MI bits estimate (Landauer surrogate). |
| `umst_contribute_status` | COLD | Poll async contribute job state. |
| `umst_transition_propose` | COLD | Predict + gate + async contribute. |
| `umst_arena_open` | COLD | Warm arena session. |
| `umst_gate_check_arena` | HOT | Gate check via warm arena session. |
| `umst_arena_close` | COLD | Release arena session. |

### Optional — HCOM semantic (`tool-semantic-hcom` / `tool-propose-communicative-act`)

**Schema stub** (`tool-semantic-hcom`): additive `propose_communicative_act` schema + fixture mock (HCOM-029 / IDEA-004).

**Full hybrid orchestration** (`tool-propose-communicative-act`, HCOM-021): mock frontier LLM proposes surface → chair cartridge maps → real `gate<SemanticResponse>` via `umst-semantics`.

```bash
# HCOM-021 — full hybrid loop
cargo build -p umst-mcp --features tool-propose-communicative-act
cargo test -p umst-mcp --test hcom_021_hybrid_orchestration --features tool-propose-communicative-act

# HCOM-029 / IDEA-004 — schema mock only
cargo test -p umst-mcp --test hcom_propose_communicative_act --features tool-semantic-hcom
```

Input: `intent`, `context.lang` (`en` | `ta`), optional `mock_llm` (`surface` override | `no_back_injection`). Output: `gated_communicative_act.v1` with `audit_digest` and three-step orchestration (`frontier_propose` → `cartridge_map` → `local_gate`).

Schemas: `src/schemas/propose_communicative_act_v0.json`, `src/schemas/gated_communicative_response_v0.json`.

Optional tool argument `canonical: true` routes output through `umst_cli::canonical::canonical_json_bytes` (sorted keys, Ryū float literals).

## Stdio smoke (FLEET-COMPOSER-H H08 · X05 retick)

Reproducible native stdio JSON-RPC battery:

```bash
cargo test -p umst-mcp stdio_smoke
```

Probe: `umst_mcp::stdio_smoke::native_stdio_smoke_h08_probe()`. Receipt: `outputs/.tmp/COMPOSER_H08_2242.md`.

WEB-009 retick (FLEET-COMPOSER-X X05): `umst_mcp::web_009::web_009_stdio_smoke_x05_probe()`. Receipt: `outputs/.tmp/COMPOSER_X05_0734.md`.

## Docker

```bash
cd umst-concrete-cartridge
docker compose build
docker compose run --rm umst-mcp
```

The compose file keeps `stdin_open` and `tty` for interactive stdio use.

## License

Released under the [MIT License](../../LICENSE).

## Workspace layout dependency (honest)

`umst-mcp` resolves `umst-manifold`, `umst-semantics`, `umst-trust`, and `umst-runtime-arena`
via relative `path = "../../../…"` entries. A clone of **only** this repository will not
build those optional/path edges. Build from the  workspace (or replace path deps
with published crates) — the crates are package-local for MCP sources, not fully standalone
for the full dependency graph.
