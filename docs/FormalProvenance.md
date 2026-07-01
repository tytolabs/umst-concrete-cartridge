# SPDX-License-Identifier: MIT

# Formal provenance (`formal_anchor` → Lean)

Every **`result.v2`** object includes **`formal_anchor`**, **`calibration_profile`**, and **`axioms`**. **`formal_anchor`** is a **`lean://umst-formal/Lean/...#symbol`** URI naming the primary mechanised hook for the active calibration profile.

## Resolve the URI

1. Clone [`umst-formal`](https://github.com/tytolabs/umst-formal) next to this repository (matching the commit pinned in **`docs/PROOF-STATUS.md`** when reproducibility matters).

2. Open the path embedded in the URI. Example:
   **`lean://umst-formal/Lean/Concrete/Powers.lean#PowersState`** points at definition **`PowersState`** in **`Lean/Concrete/Powers.lean`** relative to that repo root.

3. Read the **`formal_anchor_rationale`** and bucket lines on the cartridge symbol in **`docs/PROOF-STATUS.md`** to see how that Lean symbol connects to emitted scalars (`Mechanised`, **`Structural`**, **`Empirical`**, **`Literature`**, or **`NONE`**).

## Axiom identifiers

Bundled profiles list axiom identifiers from the Lean prelude closure (for example **`physicalSecondLaw`**) in profile TOML and on the wire (**`result.v2` **`axioms`**, certify JSON). The **`schema/*.json`** allowlist constrains stray tokens from entering JSON without a schema bump.

### Example

Given:

```json
"formal_anchor": "lean://umst-formal/Lean/Concrete/Powers.lean#powers_monotone",
"axioms": ["physicalSecondLaw"]
```

inspect **`Lean/Concrete/Powers.lean`** in **`umst-formal`** at **`powers_monotone`**, cross-check that **`physicalSecondLaw`** appears in **`[provenance.formal]`** on the calibrated profile TOML, and correlate regime warnings documented under **`Lean/RegimeSoundness.lean#warnings_empty_iff_in_regime`** when reading CLI stderr.

For scripted checks inside **`umst-formal`**, use Lake (`lake exe`, `lake env lean`) with the project's own tooling—this cartridge does not vendor Lean sources.
