# MCP protocol roadmap (v1 close-out)

**Status:** Hand-rolled stdio JSON-RPC (MCP 2024-11-05) is **shipped**. No hosted MCP requirement.

## What we ship today

| Surface | Implementation |
|---------|----------------|
| Transport | stdio line-delimited JSON-RPC |
| Tools | 9 with `--features agent-layer` |
| Prompts | 6 agent prompts |
| Resources | `umst://schemas/*` JSON Schema bytes |
| Distribution | Docker [`Dockerfile.agent`](Dockerfile.agent) + [`server.json`](server.json) |

## Deferred upgrades

| Item | Migrate when |
|------|----------------|
| **`rmcp` 1.7** | Cursor/SDK regression on hand-rolled server **or** rmcp parity spike is green on all `mcp_smoke.py` checks |
| **MCP 2025-11-25** | Upstream client requires `notifications/initialized` and breaks current handshake |
| **Streamable HTTP** | We productize **hosted MCP** (multi-tenant, remote agents) |
| **OAuth** | Same as HTTP — public registry with authenticated tenants |

## rmcp spike checklist (Pass 1, time-boxed)

1. Branch: replace stdio loop with `rmcp` Server listing same tool schemas.
2. Run `python3 scripts/mcp_smoke.py --agent-layer` — all green.
3. Compare prompt/resource list counts.
4. Document regressions in this file; merge only if zero regressions.

## Honest boundary

Federation today is **git JSONL inbox** ([`contributions/README.md`](../contributions/README.md)), not protocol upgrade.
