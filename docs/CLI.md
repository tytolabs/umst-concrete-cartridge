<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# UMST concrete cartridge CLI

Build the synchronous **`umst`** binary with the **`cli`** Cargo feature (for example `cargo install umst-concrete-cartridge --features cli`).

Add **`calibration`** when you want the deterministic Markdown generator:

```bash
cargo install umst-concrete-cartridge --features "cli,calibration"
```

Place **`--profile`** / **`--profile-file` immediately after `umst`** (they are root-level flags), e.g. `umst --profile uci_d1 predict`, not `umst predict --profile …`.

## Global flags

- **`--profile NAME`** — bundled calibration bundle id (`default`, `uci_d1`, `zenodo_ndt`, `zenodo_sonreb`, `zenodo_rh`, …). Default: `default`.
- **`--profile-file PATH`** — external TOML profile; **`--profile`** still supplies the bundle id label.

When neither flag pins a profile beyond the implicit default, the CLI prints one **stderr info line** stating that `default` is in use (delivery **C6** in the calibration formal-anchors plan).

## Subcommands

```bash
echo '{"w_c": 0.40, "temperature_k": 293.15}' | umst predict
echo '{"w_c": 0.40, "temperature_k": 293.15}' | umst predict --schema-version v1
echo '{"w_c": 0.40, "temperature_k": 293.15}' | umst --profile uci_d1 predict

umst optimize --target compressive_strength_mpa=45 --steps 32 --input mix.json

umst schema mix
umst schema result
umst schema result-v2

umst profiles list
umst profiles describe uci_d1
umst profiles regime highscm

umst certify uci_d1

cargo run -q --bin calibration_report --features "cli,calibration" > docs/Calibration.md
# also writes results/canonical/table_per_dataset_metrics.csv + results/canonical/README.md
```

Wire contracts live under **`schema/`**:

- **`mix.v1.json`** — mix JSON unchanged; profile stays on the CLI / library, **not** in the JSON.
- **`result.v2.json`** — default **`umst predict`** output (warnings + calibration provenance fields).
- **`result.v1.json`** — **`--schema-version v1`** only; documented as deprecated for one minor release cycle.

## `umst certify` JSON

`umst certify <profile>` prints a single JSON object including a wire **`formal_status`** string: one of **`Mechanised`**, **`Structural`**, **`Empirical`**, **`Literature`**, or **`NONE`**. Profile TOML may carry **`verification_status = "Boundary"`** or legacy tokens; those are **not** emitted as Rust `formal_status` — they map to **`NONE`** on the wire unless provenance/acceptance explicitly sets one of the five buckets.

Example (truncated):

```json
{"profile":"uci_d1","formal_status":"Mechanised","model_kind":"powers_gel_space", "...":"..."}
```
