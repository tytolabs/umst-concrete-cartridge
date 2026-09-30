# Git contribution inbox — federated memory

Labs contribute **validated memory rows** to the shared corpus via **pull request**, not live MCP to a central server.

**Full spec:** [MaOS-Workspace `git-contribution-inbox` plan](https://github.com/tytolabs/MaOS-Workspace/blob/prime-spectral-research/outputs/.plans/git-contribution-inbox.md)

## Quick flow

```text
1. Local: umst_gate_check → umst_contribute (UMST_MEMORY_DB set)
2. Export: python3 scripts/export_contributions_jsonl.py --db .umst-memory/memory.db --lab <slug> --out contributions/inbox/<slug>-<YYYYMMDD>-<id>.jsonl
3. PR: add only files under contributions/inbox/ (one JSONL per PR preferred)
4. CI: schema + gate re-check + duplicate scan
5. Maintainer merge → move file to `contributions/merged/YYYY-MM/`
6. Append manifest: `python3 scripts/update_contribution_manifest.py --append contributions/merged/YYYY-MM/<file>.jsonl`
7. Periodic: `cat contributions/merged/*/*.jsonl | python3 scripts/ingest_contributions.py --db .umst-memory/memory.db`

CI verifies `contributions/merged/MANIFEST.jsonl` matches on-disk merged shards (`validate_contribution_inbox.py --check-manifest`).
```

## File naming

```
contributions/inbox/<lab-slug>-<YYYYMMDD>-<6char>.jsonl
```

Example: `contributions/inbox/tyto-20260619-a1b2c3.jsonl`

## Line format

One **`contribution.v1`** JSON object per line (same wire as MCP `umst_contribute`). See [`schemas/contribution.v1.json`](../schemas/contribution.v1.json).

Required: `gate_summary.admissible` must be `true`. CI re-runs gate check — do not rely on local bypass.

## Git-only contributors

No local MCP? Author `contribution.v1` JSONL by hand or tool, open PR to `contributions/inbox/`. CI validates schema and admissibility.

## What merge does **not** do

- Does **not** auto-update `calibration/profiles/` — use human `umst promote-contribution`.
- Does **not** include `gate_reject` rows — rejects stay local.

## Maintainer

After merge, relocate inbox file to `contributions/merged/<YYYY-MM>/` and run:

```bash
python3 scripts/update_contribution_manifest.py --append contributions/merged/<YYYY-MM>/<file>.jsonl
```

Manifest schema (one JSON object per line): `path`, `sha256`, `merged_at`, `rows`, `lab`, optional `content_ids` for duplicate scanning.
