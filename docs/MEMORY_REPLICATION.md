SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
# Memory replication — operator guide

**Status:** v1 per-deployment SQLite + JSONL sidecar (shipped). External replication is opt-in.

---

## v1 durability (shipped)

| Artifact | Path | Role |
|----------|------|------|
| SQLite STRICT + WAL | `UMST_MEMORY_DB` (e.g. `.umst-memory/memory.db`) | Authoritative `memory_records` |
| JCS JSONL sidecar | `.umst-memory/memory.jcs.jsonl` | Append-only audit mirror |
| Reject stream | `.umst-memory/gate_reject.jcs.jsonl` | REJECT rows never in `admissible_only` queries |
| Async jobs sidecar | `contribute_jobs.json` beside DB | In-process async contribute map (not SQLite table in v1) |
| Row sidecars | `.umst-memory/rows/{memory_id}.json` | Human promotion lookup |

There is **no** central shared memory cloud. Lab handoff uses signed export bundles:

```bash
umst memory export --db .umst-memory/memory.db --out exports/run-001/
```

---

## Litestream → S3 Object Lock (deferred)

Continuous replication to object storage with WORM retention is **deferred** for v1. Rationale: storage research in [`contribution-stack-storage-research.md`](../../outputs/.plans/contribution-stack-storage-research.md); per-deployment SQLite + periodic export bundles suffice for agent MVP.

When enabling Litestream later:

1. Point Litestream at `UMST_MEMORY_DB` only (not JSONL sidecars — they are derived).
2. Use S3 Object Lock (Compliance mode) on the bucket for tamper-evident backups.
3. Keep promotion bundles human-gated; replication does not auto-promote calibration.

Example starter config (not wired in CI):

```yaml
# docs/examples/litestream.yml — operator template only
dbs:
  - path: /var/lib/umst/.umst-memory/memory.db
    replicas:
      - type: s3
        bucket: umst-memory-replicas
        path: concrete/prod
        region: eu-west-1
        retention: 168h
        sync-interval: 60s
```

Install: [Litestream](https://litestream.io/) v0.3.x+. Example systemd unit: [`scripts/litestream-systemd.example`](../scripts/litestream-systemd.example).

### Restore smoke (operator)

```bash
litestream restore -config docs/examples/litestream.yml -o /tmp/memory-restored.db /var/lib/umst/.umst-memory/memory.db
sqlite3 /tmp/memory-restored.db 'SELECT COUNT(*) FROM memory_records;'
```

Compare count to pre-failover export manifest before cutting traffic back.

---

## Merkle checkpoints

Hourly Merkle batch roots append to `checkpoints.jsonl` via `checkpoint.rs` helper. Use with export bundles for hash-chain verification across handoff.

---

## Related

- [`AGENT_MCP.md`](AGENT_MCP.md) — env vars, export runbook  
- [`CARTRIDGE_PORT.md`](CARTRIDGE_PORT.md) — new cartridge durability expectations
