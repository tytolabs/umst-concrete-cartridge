SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
# EPISTEMIC_PRIMITIVES.md — composed agent workflows (B7)

Maps blueprint §7 primitives to **real** tools/examples, or marks **Proposed**.

---

## Critique / counter-critique

| Step | Real surface |
|:---|:---|
| Propose mix | Agent / `umst_predict` |
| Critique | `umst_gate_check` + `explain` |
| Counter | Adjust fields per `explain.remediation` / `regime_violations` |
| Commit | `umst_contribute` only on PASS |

Examples: `01_gate_explore.py`, `05_explain_violations.py`, `02_contribute_admissible.py`.

---

## Energy awareness

| Signal | Real? |
|:---|:---|
| `mi_bits_est` / `umst_mi_estimate` | Yes — **advisory** only (`agent_layer.rs:685`) |
| Landauer sync billing | UCRS clock path — not every MCP call |
| Soft-gate training penalties | Library `soft_gate` (not MCP admissibility) |

---

## Provenance / time

| Mechanism | Real? |
|:---|:---|
| `observed_at` / UCRS stamp on contribute | Yes — `UcrsObservedAt` / env `UMST_UCRS_WITNESS` |
| HLC sidecar | Docs in `umst-ucrs` (public) |

---

## Safe exploration

| Mechanism | Real? |
|:---|:---|
| Gate-before-contribute | Yes — G1 in AGENT_PROTOCOL |
| Scope tokens | Yes when `UMST_AGENT_SCOPE_TOKENS` set |
| `umst_dry_run` | **Proposed (P2)** — use gate + schema validate instead |
| Inbox `--dry-run` | Yes — `scripts/ingest_contributions.py` |

---

## Human-gated promotion

| Mechanism | Real? |
|:---|:---|
| Federated JSONL PR inbox | Yes — `03_export_inbox.sh` + scripts |
| `umst_promote_contribution` MCP tool | **Proposed (P3)** |

---

## Acceptance

Each primitive maps to real tools/examples or is marked Proposed.
