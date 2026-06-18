# Golden adversarial fixtures

Agent-eval fixture pack for the Physical Reasoning Layer MCP boundary (`umst_gate_check`).

| File | Role |
|------|------|
| `admissible_mix_01.json` | `contribution.v1` instance — thermodynamically admissible OPC mix (w/c = 0.45) |
| `reject_mix_01.json` | `contribution.v1` instance — inadmissible high w/c mix (w/c = 0.75) |
| `expected_verdicts.json` | Expected MCP gate outcomes (not schema-validated in CI) |

**Usage (Phase 4+):** CI should assert `umst_gate_check(admissible)` → `admissible: true` and `umst_gate_check(reject)` → `admissible: false`. Phase 1 validates schema conformance only.

Fixtures use `stamp_tier: Synthetic` — isolated from production merge paths per UCRS logging policy.
