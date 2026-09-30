<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# `umst-mcp`

Stdio [Model Context Protocol](https://modelcontextprotocol.io/) server (JSON-RPC 2.0, newline-delimited **stdin → stdout**) exposing the UMST concrete façade to agents and IDE integrations.

**Repository:** [github.com/tytolabs/umst-concrete-cartridge](https://github.com/tytolabs/umst-concrete-cartridge) (workspace member)

## Build and run

```bash
cd umst-concrete-cartridge
cargo run -p umst-mcp
```

## Tools

| Tool | Behaviour |
|------|-----------|
| `umst_predict` | Mix JSON → `result.v1` / `result.v2` via `facade::predict_with_options`. |
| `umst_audit` | CSV rows → `audit.v1`. |
| `umst_profiles` | Bundled calibration profile ids (deterministic order). |
| `umst_certify` | Certification chain JSON (CLI `certify` mirror). |

Optional tool argument `canonical: true` routes output through `umst_cli::canonical::canonical_json_bytes` (sorted keys, Ryū float literals).

## Docker

```bash
cd umst-concrete-cartridge
docker compose build
docker compose run --rm umst-mcp
```

The compose file keeps `stdin_open` and `tty` for interactive stdio use.

## License

Released under the [MIT License](../../LICENSE).
