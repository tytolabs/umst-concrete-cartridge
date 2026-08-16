SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
# REFERENCE.md — agent surface signatures (B5)

**Signatures and schemas agents bind to.** Spot-check against source; full prose in [`AGENT_MCP.md`](AGENT_MCP.md).

---

## Library (Rust)

| Symbol | Signature (abbrev) | At |
|:---|:---|:---|
| `MixSpec` | struct of rational mix fields | `facade/mod.rs:151` |
| `gate_check_mix` | `(Profile, &Value) -> GateSummary` | `contribution.rs:108` |
| `gate_check_mix_result` | `(…) -> GateCheckResult` | `contribution.rs:175` |
| `GateCheckResult` | `{ gate_summary, gate_reject?, explain? }` | `contribution.rs:161` |
| `accept` | contribute admissible row → memory | `contribution.rs:516` |
| `Contribution` | `contribution.v1` domain type | `research/types.rs:79` |
| `MemoryStore` | `append` / `rows` | `memory.rs:53` |
| `PhysicalReasoningLayer` | trait | `layer.rs:17` |

## Manifold ports

| Symbol | At |
|:---|:---|
| `IScienceCartridge` | `umst-manifold/.../traits.rs:51` |
| `GateCartridge` | `traits.rs:62` |
| `DesignRepresentation` | `traits.rs:98` |

## Wire schemas (resources)

Served under `umst://schemas/…` when `agent-layer` enabled (see `agent_layer.rs` resources list):  
`contribution.v1`, `gate_reject.v1`, `memory_record.v1`, …

## MCP tools

See [`TOOL_CONTRACTS.md`](TOOL_CONTRACTS.md) — do not duplicate schemas here.

## Spot-check paste (acceptance)

```text
$ rg -n 'pub struct GateCheckResult' crates/umst-concrete-cartridge/src/research/contribution.rs
161:pub struct GateCheckResult {
$ rg -n 'pub struct MixSpec' crates/umst-concrete-cartridge/src/facade/mod.rs
151:pub struct MixSpec {
```
