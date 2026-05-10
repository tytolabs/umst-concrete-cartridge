<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# UMST concrete cartridge CLI

Build and install the synchronous `umst` binary by enabling the optional `cli` Cargo feature (for example `cargo install umst-concrete-cartridge --features cli` from this repository).

Example invocations:

```bash
echo '{"w_c": 0.40, "temperature_k": 293.15}' | umst predict
umst optimize --target compressive_strength_mpa=45 --steps 32 --input mix.json
umst schema mix
```

The JSON payloads accepted by `predict` / `optimize` and emitted by `predict` are described by versioned schemas under `schema/` (`mix.v1.json`, `result.v1.json`). That wire contract is the canonical surface for any future front-end (for example an MCP tool server, a REST adapter, or language bindings): each adapter should validate against these schemas so all callers share one typed contract around the same cartridge functor.
