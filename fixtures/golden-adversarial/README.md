SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
# Golden adversarial fixtures (agent wire contract)

Agent-eval fixture pack for the Physical Reasoning Layer **MCP boundary** (`umst_gate_check` / `gate_check_mix`).

| File | Role |
|------|------|
| `admissible_mix_01.json` | `contribution.v1` instance — thermodynamically admissible OPC mix (w/c = 0.45) |
| `reject_mix_01.json` | `contribution.v1` instance — inadmissible high w/c mix (w/c = 0.75) |
| `expected_verdicts.json` | SSOT for `golden_gate_check`: admissible flag + verdict per fixture |

Fixtures use `stamp_tier: Synthetic` — isolated from production merge paths per UCRS logging policy.

## FNR / FPR honesty (what this pack does *not* claim)

The **hard safety witness** for gate admissibility is in **`umst-manifold`**, not here:

| Layer | Test | Contract |
|-------|------|----------|
| **Physics gate core** | `gate_adversarial` | **FNR = 0** and **FPR = 0** on **75** vendored cases (`tests/data/adversarial_gate_test.json`) |
| **Agent wire boundary** (this pack) | `golden_gate_check`, `phase8_adversarial` | Exact parity on **2** pinned `contribution.v1` mixes + reject explain / pagination wire |

This directory proves **agent JSON contracts** (schema, rational mix wire, `gate_reject.v1`, `explain` payloads). It does **not** replace manifold's 75-case adversarial golden or certify global FNR/FPR = 0. Run both layers when validating a full stack.

**Manifold SSOT:** [`umst-manifold/docs/GOLDEN_FIXTURES.md`](../../../umst-manifold/docs/GOLDEN_FIXTURES.md) (monorepo sibling) · [GitHub mirror](https://github.com/tytolabs/umst-manifold/blob/main/docs/GOLDEN_FIXTURES.md)

### Run `gate_adversarial` locally (manifold)

From a clone of `umst-manifold` (Rust **1.88+**, default features):

```bash
cd /path/to/umst-manifold
cargo test --test gate_adversarial
```

Full gate parity bundle (recommended before release):

```bash
cargo test --test gate_parity_fixture \
  --test gate_kleisli --test gate_cbf_parity \
  --test gate_dual_run_parity --test gate_reject_catalog_id \
  --test gate_adversarial
```

From a **multi-repo workspace** root:

```bash
cargo test -p umst-manifold --manifest-path umst-manifold/Cargo.toml --test gate_adversarial
```

See [`GOLDEN_FIXTURES.md` §2](../../../umst-manifold/docs/GOLDEN_FIXTURES.md) for case fields, pinned summary JSON, and optional E6 regeneration.

## CI (live)

[`.github/workflows/agent-layer.yml`](../../.github/workflows/agent-layer.yml) runs on push/PR when these fixtures or agent-layer code change:

```bash
cargo test -p umst-concrete-cartridge --features agent-layer \
  --test research_memory --test golden_gate_check --test phase8_adversarial
```

| Test binary | What it asserts |
|-------------|-----------------|
| `golden_gate_check` | `expected_verdicts.json` → `gate_check_mix` admissible/verdict parity for both JSON fixtures |
| `phase8_adversarial` | Full `gate_check_mix_result` wire: `gate_reject.v1` on reject, `explain` remediation/fields, memory `query_page` edge cases |
| `research_memory` | Research store ingest/query invariants (same workflow job) |

The same workflow also runs `mcp_smoke.py --agent-layer` and `examples/agent/01_gate_explore.py`; those require a built MCP server and Python 3.

[`.github/workflows/schema.yml`](../../.github/workflows/schema.yml) asserts `contribution.v1` schema conformance on both fixture JSON files.

## Run locally (cartridge)

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

**Honest scope:** these fixtures validate the in-process gate and research-memory API at the agent wire boundary. They do not substitute for manifold `gate_adversarial` (FNR/FPR = 0 on 75 cases), hosted MCP integration tests, or live contribution promotion.

## Explain samples (`explain: true` on REJECT)

Pinned violation codes agents should parse from `umst_gate_check` (`result.isError: true`). Full remediation loop: [`docs/AGENT_MCP.md`](../../docs/AGENT_MCP.md#error-handling). Runnable walkthrough: [`examples/agent/05_explain_violations.py`](../../examples/agent/05_explain_violations.py).

**SSOT cross-links:** [`docs/GOLDEN_VECTORS.md`](../../docs/GOLDEN_VECTORS.md) · [`tests/fixtures/phase8_adversarial.json`](../../tests/fixtures/phase8_adversarial.json) · [`umst-manifold/docs/GOLDEN_FIXTURES.md`](../../../umst-manifold/docs/GOLDEN_FIXTURES.md)

### 1. `mix_spec_rational_parse_fail` (inline vector)

```json
{
  "gate_summary": { "admissible": false, "verdict": "REJECT" },
  "gate_reject": { "schema_version": "gate_reject.v1", "verdict": "REJECT" },
  "explain": {
    "regime_violations": ["mix_spec_rational_parse_fail"],
    "remediation": ["Use rational strings like \"9/20\" for w_c and temperature_k; see contribution.v1 schema."],
    "fields": [{ "path": "mix.w_c", "issue": "rational_parse_fail" }],
    "catalog_witnesses": ["umst.gate.cd_transition"]
  }
}
```

Trigger: `mix.w_c: "not-rational"` (see manifest `inline_vectors.rational_parse_fail`).

### 2. `thermodynamic_cd_fail` (`reject_mix_01.json`)

High w/c (`3/4`) contribution fixture — gate re-check returns thermodynamic REJECT. Run:

```bash
cargo test -p umst-concrete-cartridge --features agent-layer --test phase8_adversarial -- --nocapture
```

Expect `explain.regime_violations` containing `thermodynamic_cd_fail` or `thermodynamic_fail`, paired `remediation` strings, and optional `explain.fields` pointing at mix coordinates.

### 3. `mix_spec_wire_invalid`

Parsed rationals that fail `MixSpec` validation (missing required field or out-of-envelope wire). Expect `explain.fields` listing paths; remediation references `contribution.v1.json`. Golden coverage: `phase8_adversarial` + schema CI on both JSON fixtures in this directory.
## 🔒 Confidentiality Notice

This repository contains proprietary information. Copyright (c) 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar. All rights reserved. Unauthorized copying, distribution, or use of these files, via any medium, is prohibited.
