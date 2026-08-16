SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
# V2 — tyto_mortar physical specimen protocol

1. Mix per [`../profiles/tyto_mortar.v1.toml`](../profiles/tyto_mortar.v1.toml) regime bounds.
2. Cast three 40×40×160 mm prisms; cure 20 °C RH ≥ 95% to 24 h, then 20 °C dry to 48 h.
3. Measure compressive strength at 24 h and 48 h (EN 196-1 style, n=3).
4. Record printability window via proxy extrusion (θ, τ₀) per in-house S1 rheology session (τ₀ band only; no compressive strength).
5. Commit raw CSV to `measurements.json` in this directory.

**Candidates (biochar / GBFS / RCA):** defer to Term 3 lab queue — not in this bundle.
