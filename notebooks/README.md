<!--
SPDX-License-Identifier: MIT
-->

# Notebooks

- [`sustainable_mix_audit.ipynb`](sustainable_mix_audit.ipynb) — ten-row corpus (bundled UCI/Zenodo CSV slices plus literature-style transcribed mixes), **`umst_concrete_cartridge.audit`** Python binding, three matplotlib panels (regime counts, predicted-vs-observed with MAE band, provenance pie), and only approved claim language for envelopes.

Run headless execution:

```bash
./notebooks/run_all.sh
```

Requires **`jupyter nbconvert`**, **`matplotlib`**, **`pandas`**, and the editable extension (`cd crates/umst-py && maturin develop`). CI installs these before `--execute`. Optional `[notebook]` / `[pandas]` extras on the Python package overlap with those deps.
