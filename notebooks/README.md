SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
<!--
-->

# Notebooks

Jupyter notebooks for exploratory audits against the bundled UCI and Zenodo CSV slices using the ``umst_concrete_cartridge`` Python bindings.

| Notebook | Content |
|----------|--------|
| [`sustainable_mix_audit.ipynb`](sustainable_mix_audit.ipynb) | Small fixed corpus, `audit` binding, matplotlib panels (regime counts, predicted vs. observed, provenance summary). |
| Striatus shell | [`_run_shell_demo.sh`](_run_shell_demo.sh): `optimize_shell_3d` → PNG frames → GIF + STL (`pip install './crates/umst-py[render]'`). After a run: `pytest notebooks/tests/test_print_ready.py -v` with `./crates/umst-py[render,tests]`. Optional: [`check_shell_artifact_budgets.sh`](check_shell_artifact_budgets.sh), [`check_shell_determinism.sh`](check_shell_determinism.sh). **Commit gate:** do not `git add` multi‑MB `notebooks/_artifacts/*.gif` / `*.stl` / `*.obj` without artefact budgets + print-ready checks; `iter_*.npy` under `examples/_artifacts/shell/` stays gitignored. Gate semantics and pending close-out: [`../docs/Solver-Status.md`](../docs/Solver-Status.md). |
| RC beam strut-and-tie | [`_run_beam_demo.sh`](_run_beam_demo.sh): [`optimize_rc_beam`](../crates/umst-concrete-cartridge/examples/optimize_rc_beam.rs) (default **32×8** grid, `UMST_BEAM_ITERS=90`, `UMST_BEAM_DUMP_STRIDE=3`, `UMST_BEAM_DUMP` on by default — set `0`/`false`/`off` to skip NPY dumps; NPY under `examples/_artifacts/beam/`) → [`render_beam_gif.py`](render_beam_gif.py) → `notebooks/_artifacts/beam_strut_and_tie.gif`. Optional renderer env: `UMST_BEAM_GIF_MAX_SIDE`, `UMST_BEAM_GIF_SUPERSAMPLE`, `UMST_BEAM_GIF_FRAME_MS`, `UMST_BEAM_GIF_HOLD_MS`, `UMST_BEAM_GIF_HOLD_FRAMES` (see script header / `render_beam_gif.py` module doc). Uses `numpy` + `pillow` + `svgwrite` + `cairosvg`; bootstraps `BEAM_DEMO_VENV` or `.venv_beam_gif`. If `cargo run … optimize_rc_beam` fails, falls back to [`beam_demo_synthetic_npys.py`](beam_demo_synthetic_npys.py). **Commit gate:** same `_artifacts` GIF hygiene as the Striatus shell row. |

## Run headless (repository root)

```bash
cd umst-concrete-cartridge
./notebooks/run_all.sh
```

Strict CI-style runs expect `jupyter` / `nbconvert` on `PATH` (see workflow). Dependencies: `matplotlib`, `pandas`, and an editable build of the extension (`pip install './crates/umst-py[notebook]'` or `maturin develop --extras notebook` from `crates/umst-py`).

## Python / NumPy (shell demo)

Use one virtual environment for **NumPy + PyVista + VTK** and the `umst-py` extension so ABI and wheel tags stay aligned (avoid mixing system Python NumPy with a different interpreter used for `maturin develop`).

**uv (recommended):**

```bash
cd umst-concrete-cartridge
uv venv .venv
source .venv/bin/activate
uv pip install -r notebooks/requirements-shell-demo.txt
uv pip install './crates/umst-py[render]'
```

**venv + pip:**

```bash
cd umst-concrete-cartridge
python3 -m venv .venv
source .venv/bin/activate
pip install -r notebooks/requirements-shell-demo.txt
pip install './crates/umst-py[render]'
```

After `source .venv/bin/activate`, confirm `which python3` matches the interpreter used for `pip` / `uv pip` (and for `maturin develop` if you build the extension that way).

GIF/STL tooling (`render_shell_gif.py`, `overlay_final_isostatics.py`): sets `pv.OFF_SCREEN = True` and `Plotter(off_screen=True)` — no GUI required. Prefer **PyVista ≥ 0.43 / VTK wheels** on Apple Silicon (arm64 VTK).

**Defaults, `UMST_SHELL_*` knobs, Track L regeneration, B8 / `UMST_REQUIRE_B8`, and swarm close-out registry:** [`../docs/Solver-Status.md`](../docs/Solver-Status.md). With **`umst-manifold`** as a sibling directory (same parent as this repository): [`../../umst-manifold/docs/PENDING_GAPS_PLAIN.md`](../../umst-manifold/docs/PENDING_GAPS_PLAIN.md).

**CI-fast smoke (small grid):** `optimize_shell_3d` clamps `UMST_SHELL_NX` / `UMST_SHELL_NY` to **[6, 40]** and `UMST_SHELL_NZ` to **[2, 8]** — e.g.

```bash
cd umst-concrete-cartridge
UMST_SHELL_NX=6 UMST_SHELL_NY=6 UMST_SHELL_NZ=2 UMST_SHELL_ITERS=2 UMST_SHELL_DUMP_ITER=0 UMST_SHELL_MAX_CG=200 \
  cargo run --release -p umst-concrete-cartridge --example optimize_shell_3d --features 'solver-experimental render'
```

This path is CPU-oriented (Burn default backend in typical checkouts); **no GPU is required** for the smoke above. If other workspace builds link CPU BLAS, cap threads with **`VECLIB_MAXIMUM_THREADS`** (macOS Accelerate) or **`OPENBLAS_NUM_THREADS`** (OpenBLAS) — see the repo root [`README.md`](../README.md) “CPU matmul” note.
