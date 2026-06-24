# Golden adversarial fixtures

Agent-eval fixture pack for the Physical Reasoning Layer MCP boundary (`umst_gate_check`).

| File | Role |
|------|------|
| `admissible_mix_01.json` | `contribution.v1` instance — thermodynamically admissible OPC mix (w/c = 0.45) |
| `reject_mix_01.json` | `contribution.v1` instance — inadmissible high w/c mix (w/c = 0.75) |
| `expected_verdicts.json` | Expected gate outcomes keyed by `mix_input` floats → rational wire at test time |

**CI (live):**

| Workflow | What it asserts |
|----------|-----------------|
| [`.github/workflows/schema.yml`](../../.github/workflows/schema.yml) | `contribution.v1` schema conformance on both fixture JSON files |
| [`.github/workflows/agent-layer.yml`](../../.github/workflows/agent-layer.yml) | `golden_gate_check` test: `gate_check_mix` verdicts match `expected_verdicts.json`; `phase8_adversarial` explain/reject rows; `01_gate_explore.py` stdio smoke |

**Local:**

```bash
cargo test -p umst-concrete-cartridge --features agent-layer \
  --test golden_gate_check --test phase8_adversarial
python3 examples/agent/01_gate_explore.py   # uses reject + admissible fixtures via MCP
```

Fixtures use `stamp_tier: Synthetic` — isolated from production merge paths per UCRS logging policy.
