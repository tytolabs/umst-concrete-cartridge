<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Notebooks

Jupyter notebooks for exploratory audits against the bundled UCI and Zenodo CSV slices using the [`umst_concrete_cartridge`](https://github.com/tytolabs/umst-concrete-cartridge) Python bindings.

| Notebook | Content |
|----------|--------|
| [`sustainable_mix_audit.ipynb`](sustainable_mix_audit.ipynb) | Small fixed corpus, `audit` binding, matplotlib panels (regime counts, predicted vs. observed, provenance summary). |

## Run headless (repository root)

```bash
cd umst-concrete-cartridge
./notebooks/run_all.sh
```

Strict CI-style runs expect `jupyter` / `nbconvert` on `PATH` (see workflow). Dependencies: `matplotlib`, `pandas`, and an editable build of the extension (`pip install './crates/umst-py[notebook]'` or `maturin develop --extras notebook` from `crates/umst-py`).
