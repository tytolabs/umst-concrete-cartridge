<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Proof status (Rust cartridge sources)

Rolling counts of documented `/// formal_status:` buckets under `src/`. This Markdown file is checked
against on-disk generators in **`tests/proof_status_doc.rs`**.

Formal verification artefacts themselves live alongside the proofs in **`umst-formal`**
(which publishes its own `PROOF-STATUS.md` ledger for Lean / attendant proof languages).

## Buckets found in Rust doc comments (`src/**/*.rs`)

| formal_status bucket | Approximate occurrences |
|----------------------|------------------------|
| `Boundary` | 2 |
| `Library` | 59 |
| `Mechanised` | 16 |
| `Structural` | 22 |

Total doc-comment occurrences: **`99`**.

## Refresh

```bash
cargo test -p umst-concrete-cartridge --test proof_status_doc \
proof_status_refresh_markdown_on_disk -- --ignored --nocapture
```
