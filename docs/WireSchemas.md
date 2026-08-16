SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT

# Wire schemas

This cartridge emits several JSON artefacts. Schemas ship under [`schema/`](../schema/) and mirror `include_str!` bundles in **`umst_concrete_cartridge::facade`** for MCP/CLI/Python.

## Registry

| Schema | File | Produced by | Status |
|--------|------|---------------|--------|
| **mix.v1** | [`schema/mix.v1.json`](../schema/mix.v1.json) | `umst predict` stdin / Python `MixSpecWire` parsing | Stable |
| **result.v1** | [`schema/result.v1.json`](../schema/result.v1.json) | `--schema-version v1` legacy scalars only | Deprecated; remove one minor cycle after callers migrate |
| **result.v2** | [`schema/result.v2.json`](../schema/result.v2.json) | Default `predict` output (tensor summary + **`physics_pipeline`**) | Stable |
| **audit.v1** | [`schema/audit.v1.json`](../schema/audit.v1.json) | `umst audit`; Python `audit` / `audit_rows` / `audit_dataframe` | Stable |
| **physics_pipeline.v1** | Inline **`schema_version`** under `physics_pipeline` in **result.v2** | Nested tensor staged report | Stable |

## Deprecation rule

Breaking shape changes bump the **`schema_version`** literal (`audit.v1`, **`result.v2`**, **`mix.v1`**, pipeline tag). Older tags remain parseable until the next **minor** release after a migration note appears in **`CHANGELOG.md`**. **`result.v1`** is retained temporarily for scripted clients; **`--schema-version v1`** is the explicit compatibility switch.

## `schema_version` field convention

Top-level payloads carry **`schema_version`** with values **`audit.v1`**, **`result.v2`**, or **`result.v1`**. Nested capsules (for example **`physics_pipeline`**) declare their own string tag internally so mixed JSON trees remain self-describing.
