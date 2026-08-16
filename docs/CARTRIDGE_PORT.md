SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
# Cartridge port guide — Physical Reasoning Layer

**Audience:** Authors of new `IScienceCartridge` repos (geomaterial, structure, alloy contexts)  
**Reference impl:** [`umst-concrete-cartridge`](../)  
**Manifold SSOT:** [`umst-manifold` gate + catalog](https://github.com/tytolabs/umst-manifold)

---

## What you are porting

A **cartridge** is not a commodity bucket (“metals”, “polymers”). It is a named law + memory geometry:

| Axis | Concrete (shipped) | Your cartridge |
|------|-------------------|----------------|
| `cartridge_slug` | `umst-concrete-cartridge` | `umst-<domain>-cartridge` |
| `state_carrier` | Bulk `StatePoint` mix tensor | Bulk, topology `UMST`, or hybrid |
| `design_coordinates` | `w_c`, `T`, φ_agg (Morton on mix grid) | Domain-specific rational JSON |
| `constitutive_closure` | Hydration CD, rheology, maturity | Your physics functor |
| `gate_catalog` | `gate.clausius_duhem.v1` + extensions | Pinned `catalog_id` witnesses |

Implement **`PhysicalReasoningLayer`** (`src/research/layer.rs` pattern) for memory locality — do not copy concrete Morton keys blindly.

---

## Minimum file checklist

```text
schemas/contribution.v1.json          # extend mix_spec / outcome fields only via version bump
schemas/memory_record.v1.json
schemas/gate_reject.v1.json
src/research/                         # pure validation, accept, query_page, export
crates/umst-mcp/                      # stdio JSON-RPC; agent-layer feature
docs/AGENT_MCP.md                     # operator contract for your namespace
docker/Dockerfile.agent               # --features agent-layer,manifest-bridge
.github/workflows/agent-layer.yml     # research_memory + mcp_smoke
```

---

## Cargo pins

```toml
umst-manifold = { git = "https://github.com/tytolabs/umst-manifold.git", rev = "<pin>" }
umst-ucrs = { git = "https://github.com/tytolabs/umst-ucrs.git", rev = "<pin>" }  # optional ucrs-provenance
```

Match manifold `artifacts/catalog.lock.json` digest in CI. Never run Lean at agent runtime — catalog export pin only.

---

## MCP surface (copy concrete, rename namespace if needed)

| Tool | Required |
|------|----------|
| `umst_gate_check` | Yes — hard admissibility + catalog witnesses |
| `umst_memory_query` | Yes — regime + geometry filters, pagination |
| `umst_contribute` | Yes — gate-validated ingest |
| `umst_predict` / `umst_audit` | Physics lane — as needed |
| `umst_contribute_status` | When heavy physics exceeds stdio timeout |

**Never on MCP:** `propose-promotion`, `apply-promotion`, bundle release.

---

## Memory bootstrap

1. Ship honest fixture CSV + `provenance.v1.json` (SHA-256 of corpus).
2. `scripts/bootstrap_memory_from_audit.py` → `ingest_contributions.py`.
3. Document row count in CI (`agent-layer.yml` pattern ~16k for concrete).

---

## UCRS stamps

Enable `ucrs-provenance` feature. Set `UMST_UCRS_WITNESS=live` only when embedding `umst_ucrs::TemporalWitness` or running the sidecar daemon. Promotion requires `UcrsTier2` when live — see [`AGENT_MCP.md`](AGENT_MCP.md).

---

## Verification

```bash
cargo test -p <your-cartridge> --features agent-layer,ucrs-provenance
python3 scripts/mcp_smoke.py --agent-layer
```

---

## Related

- [`AGENT_MCP.md`](AGENT_MCP.md) — operator runbook  
- [`MEMORY_REPLICATION.md`](MEMORY_REPLICATION.md) — durability + Litestream defer  
- [`physical-reasoning-layer-master.md`](../../outputs/.plans/physical-reasoning-layer-master.md) — phase map
