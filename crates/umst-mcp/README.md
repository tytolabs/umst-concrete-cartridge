<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# `umst-mcp` — stdio MCP server

Model Context Protocol (**JSON-RPC 2.0** over newline-delimited **stdin → stdout**) exposing:

- **`umst_predict`** — mix JSON (`w_c`, `temperature_k`, …) → `result.v1`/`result.v2` via `facade::predict_with_options`
- **`umst_audit`** — CSV corpus (`datasets/dataset_d1.csv`-style headers) → `audit.v1`
- **`umst_profiles`** — bundled calibration ids (+ descriptions), sorted deterministically
- **`umst_certify`** — `CertifyChain` JSON mirror of **`umst certify`**

Optional `canonical: true` tool argument routes output through **`umst_cli::canonical::canonical_json_bytes`** (sorted keys, Ryū float literals).

```bash
cargo run -p umst-mcp
```

Docker builds from the repo **`Dockerfile`** (distroless runtime). Compose keeps **`stdin_open`** / **`tty`** for MCP stdio ergonomics:

```bash
docker compose build
docker compose run --rm umst-mcp
```
