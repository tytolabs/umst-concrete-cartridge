# Proof / solver status (cartridge mirror)

Short index for agent-facing honesty. Full manifold solver table: [`umst-manifold/docs/Solver-Status.md`](https://github.com/tytolabs/umst-manifold/blob/main/docs/Solver-Status.md).

| Claim | Status | Evidence |
|-------|--------|----------|
| MCP gate + memory agent layer | **Shipped** | `agent-layer.yml`, `docs/AGENT_MCP.md` |
| Golden adversarial wire (2 mixes) | **Shipped** | `fixtures/golden-adversarial/`, `phase8_adversarial` |
| Manifold 75-case FNR/FPR gate | **Shipped** (sibling repo) | `umst-manifold` `gate_adversarial` |
| Concrete `GateCartridge::transition_evidence` | **Partial** | `gate_evidence.rs` (manifold-gate feature) |
| On-robot extrusion end-to-end | **Deferred** | README scope |
| Formal Lean discharge on MCP hot path | **Not claimed** | Runtime uses Rust witnesses only |

**Evidence SSOT:** [IMPLEMENTATION_EVIDENCE.md](https://github.com/tytolabs/MaOS-Workspace/blob/main/outputs/IMPLEMENTATION_EVIDENCE.md)
