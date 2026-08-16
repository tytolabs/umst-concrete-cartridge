SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
# MENTAL_MODEL.md — how to think about the agent surface (B6)

Secondary to [`ARCHITECTURE.md`](ARCHITECTURE.md). No claim here may contradict it.

---

## One sentence

Agents propose mixes; the **gate** either admits them into research memory or rejects them with explainable thermodynamic reasons; **arena** makes repeated checks cheap; **MCP** is the cold coordination edge.

---

## Picture

```text
  proposal (mix)
       │
       ▼
  gateCheck  ──REJECT──► explain / remediate ──► revise
       │ PASS
       ▼
  contribute ──► MemoryStore (+ UCRS stamp)
       │
       ▼
  memory_query / federated inbox (human merge)
```

Hot loops skip MCP round-trips: reuse arena bytes, then contribute only on PASS.

---

## What it is not

- Not a soft RL reward that “usually” works — REJECT is structural.  
- Not a lab certificate of moral/legal truth.  
- Not the formal Lean kernel at runtime — catalog witnesses pin proofs; solvers run in Rust.

---

## Where to go next

| Need | Doc |
|:---|:---|
| Symbols | ARCHITECTURE / REFERENCE |
| Guarantees | AGENT_PROTOCOL |
| Speed | FAST_ARENA |
| Tool fields | TOOL_CONTRACTS / AGENT_MCP |
| Epistemic loops | EPISTEMIC_PRIMITIVES |
