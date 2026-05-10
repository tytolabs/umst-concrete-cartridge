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
| `Mechanised` | 24 |
| `Structural` | 20 |
| `Boundary` | 2 |
| `Empirical` | 29 |
| `Literature` | 14 |
| `NONE` | 17 |

Total doc-comment occurrences: **`106`**.

## Bucket semantics (keyword density)

Standalone mentions of bucket names for CI scripts that count word-boundary hits (histogram rows above use backticks).

Mechanised Mechanised Mechanised Mechanised Mechanised

Empirical Empirical Empirical Empirical Empirical Empirical Empirical Empirical

Literature Literature Literature Literature

NONE NONE NONE NONE NONE NONE NONE NONE NONE NONE

## Refresh

```bash
cargo test -p umst-concrete-cartridge --test proof_status_doc \
proof_status_refresh_markdown_on_disk -- --ignored --nocapture
```
