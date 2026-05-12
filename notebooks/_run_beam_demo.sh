#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Run RC beam topology example with NPY dumps, then render the Strut-and-Tie GIF.
# If `optimize_rc_beam` fails (backward has regressed with Burn NdArray scatter on some graphs),
# fall back to `beam_demo_synthetic_npys.py` so the GIF path still verifies end-to-end.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV="${BEAM_DEMO_VENV:-$ROOT/.venv_beam_gif}"
if [[ ! -x "$VENV/bin/python" ]]; then
  python3 -m venv "$VENV"
  "$VENV/bin/pip" install -q numpy pillow
fi
PY="$VENV/bin/python"

cd "$ROOT/crates/umst-concrete-cartridge"

mkdir -p examples/_artifacts/beam
rm -f examples/_artifacts/beam/iter_*.npy examples/_artifacts/beam/manifest.json

export UMST_BEAM_DUMP="${UMST_BEAM_DUMP:-1}"
export UMST_BEAM_ITERS="${UMST_BEAM_ITERS:-15}"
export UMST_BEAM_DUMP_STRIDE="${UMST_BEAM_DUMP_STRIDE:-3}"

if ! cargo run --release -p umst-concrete-cartridge \
  --example optimize_rc_beam \
  --features solver-experimental; then
  echo "WARN: optimize_rc_beam failed — writing synthetic NPY artefacts for GIF smoke." >&2
  "$PY" "$ROOT/notebooks/beam_demo_synthetic_npys.py"
fi

"$PY" "$ROOT/notebooks/render_beam_gif.py"
