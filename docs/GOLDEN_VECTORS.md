# Golden vectors — gate adversarial SSOT

**Audience:** Agent authors, CI maintainers, external researchers validating `umst_gate_check`  
**SSOT manifest:** [`tests/fixtures/phase8_adversarial.json`](../tests/fixtures/phase8_adversarial.json)  
**Fixture payloads:** [`fixtures/golden-adversarial/`](../fixtures/golden-adversarial/)  
**Rust integration test:** `crates/umst-concrete-cartridge/tests/phase8_adversarial.rs`

---

## Purpose

The Physical Reasoning Layer gate boundary (`gate_check_mix` / `umst_gate_check`) has adversarial end-conditions: rational parse failures, thermodynamic REJECT wire (`gate_reject.v1` + `explain`), and research-memory `query_page` filter edges.

`tests/fixtures/phase8_adversarial.json` is the **single source of truth** for which vectors those tests assert. The JSON files under `fixtures/golden-adversarial/` hold the wire payloads; the manifest records expected outcomes so agents and CI can validate structure without reading Rust.

| Artifact | Role |
|----------|------|
| `tests/fixtures/phase8_adversarial.json` | Manifest: fixture refs, inline vectors, query-page cases |
| `fixtures/golden-adversarial/admissible_mix_01.json` | Admissible `contribution.v1` (w/c = 9/20) |
| `fixtures/golden-adversarial/reject_mix_01.json` | Inadmissible `contribution.v1` (w/c = 3/4) |
| `fixtures/golden-adversarial/expected_verdicts.json` | Verdict parity SSOT for `golden_gate_check` |
| `scripts/validate_golden_vectors.py` | Offline manifest + fixture presence check |

All golden fixtures use `stamp_tier: Synthetic` — isolated from production merge paths per UCRS logging policy.

---

## Validate locally

```bash
# Manifest + fixture presence (no Rust build required)
python3 scripts/validate_golden_vectors.py

# Full adversarial integration suite (agent-layer feature)
cargo test -p umst-concrete-cartridge --features agent-layer \
  --test phase8_adversarial
```

CI runs the same test binary in [`.github/workflows/agent-layer.yml`](../.github/workflows/agent-layer.yml) alongside `golden_gate_check` and `research_memory`.

---

## Manifest sections

### `fixtures`

Each entry names a file under `fixtures/golden-adversarial/` and documents `gate_expect` fields the Rust test asserts (admissible/verdict, `gate_reject` presence, `explain` shape).

### `inline_vectors`

Wire-level mixes not stored as standalone JSON files — e.g. `rational_parse_fail` with `w_c: "not-rational"` — used to pin `mix_spec_rational_parse_fail`, remediation text, and `explain.fields`.

### `query_page_cases`

Memory query pagination and filter boundaries after ingesting an admissible fixture via `accept`. Documents empty filters, cursor semantics, `admissible_only`, `outcome_source`, and `wall_ms` windows.

---

## Agent contract mapping

| Manifest expectation | MCP wire (`umst_gate_check`) |
|----------------------|------------------------------|
| `gate_reject_schema: gate_reject.v1` | `result.isError: true` + `gate_reject` body |
| `regime_violations` | `explain.regime_violations` |
| `remediation_contains` | `explain.remediation` (default `explain: true`) |
| `fields` | `explain.fields` with `path` + `issue` |

See [`AGENT_MCP.md`](AGENT_MCP.md#error-handling) for the full gate REJECT remediation loop.

---

## Related

- [`fixtures/golden-adversarial/README.md`](../fixtures/golden-adversarial/README.md) — fixture pack overview + CI commands
- [`AGENT_MCP.md`](AGENT_MCP.md) — MCP gate tool contract
- [`schemas/gate_reject.v1.json`](../schemas/gate_reject.v1.json) — REJECT payload schema
