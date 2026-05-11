<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Notebooks

Jupyter notebooks for exploratory audits against the bundled UCI and Zenodo CSV slices using the [`umst_concrete_cartridge`](https://github.com/tytolabs/umst-concrete-cartridge) Python bindings.

| Notebook | Content |
|----------|--------|
| [`sustainable_mix_audit.ipynb`](sustainable_mix_audit.ipynb) | Small fixed corpus, `audit` binding, matplotlib panels (regime counts, predicted vs. observed, provenance summary). |
| Striatus shell | [`_run_shell_demo.sh`](_run_shell_demo.sh): `optimize_shell_3d` → PNG frames → GIF + STL (`pip install './crates/umst-py[render]'`). After a run: `pytest notebooks/tests/test_print_ready.py -v` with `./crates/umst-py[render,tests]`. Optional: [`check_shell_artifact_budgets.sh`](check_shell_artifact_budgets.sh), [`check_shell_determinism.sh`](check_shell_determinism.sh). |

## Run headless (repository root)

```bash
cd umst-concrete-cartridge
./notebooks/run_all.sh
```

Strict CI-style runs expect `jupyter` / `nbconvert` on `PATH` (see workflow). Dependencies: `matplotlib`, `pandas`, and an editable build of the extension (`pip install './crates/umst-py[notebook]'` or `maturin develop --extras notebook` from `crates/umst-py`).

GIF/STL tooling (`render_shell_gif.py`, `overlay_final_isostatics.py`): sets `pv.OFF_SCREEN = True` and `Plotter(off_screen=True)` — no GUI required. Prefer **PyVista ≥ 0.43 / VTK wheels** on Apple Silicon (arm64 VTK).
