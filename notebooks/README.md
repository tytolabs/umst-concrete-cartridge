<!--
SPDX-License-Identifier: MIT
-->

# Notebooks

- [`sustainable_mix_audit.ipynb`](sustainable_mix_audit.ipynb) — ten-row corpus (bundled UCI and Zenodo CSV slices plus curated literature-style rows), the **`umst_concrete_cartridge.audit`** Python binding, three matplotlib panels (regime counts, predicted-vs-observed with MAE band, provenance summary), and approved envelope claim wording only.

Headless execution from the repository root:

```bash
./notebooks/run_all.sh
```

Requires **`jupyter`** / **`nbconvert`**, **`matplotlib`**, **`pandas`**, and an editable build of the extension (`cd crates/umst-py && maturin develop`, or **`pip install ./crates/umst-py[notebook]`**). CI installs these dependencies before **`jupyter nbconvert --execute`**. The **`[notebook]`** optional extra matches that dependency set.
