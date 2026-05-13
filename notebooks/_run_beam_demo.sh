#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Run RC beam topology example with NPY dumps, then render the Strut-and-Tie GIF.
# If `optimize_rc_beam` fails (e.g. OOM or future regression), fall back to
# `beam_demo_synthetic_npys.py` so the GIF path still verifies end-to-end.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NB="$ROOT/notebooks"
CRATE_ROOT="$ROOT/crates/umst-concrete-cartridge"
ART_DIR="$CRATE_ROOT/examples/_artifacts/beam"

log() { printf '%s\n' "[beam-demo] $*"; }
warn() { printf '%s\n' "[beam-demo] WARN: $*" >&2; }
die() { printf '%s\n' "[beam-demo] ERROR: $*" >&2; exit 1; }

# Prefer explicit BEAM_DEMO_VENV; else reuse a repo-local venv; else create .venv_beam_gif.
if [[ -n "${BEAM_DEMO_VENV:-}" ]]; then
  VENV="$BEAM_DEMO_VENV"
elif [[ -x "$ROOT/.venv_beam_gif/bin/python" ]]; then
  VENV="$ROOT/.venv_beam_gif"
elif [[ -x "$ROOT/.venv/bin/python" ]]; then
  VENV="$ROOT/.venv"
elif [[ -x "$ROOT/venv/bin/python" ]]; then
  VENV="$ROOT/venv"
else
  VENV="$ROOT/.venv_beam_gif"
fi

if [[ ! -x "$VENV/bin/python" ]]; then
  log "creating venv at $VENV"
  python3 -m venv "$VENV"
  "$VENV/bin/pip" install -q --upgrade pip
  "$VENV/bin/pip" install -q numpy pillow
elif ! "$VENV/bin/python" -c "import numpy, PIL" 2>/dev/null; then
  log "installing numpy + Pillow into $VENV"
  "$VENV/bin/pip" install -q numpy pillow
else
  log "using venv: $VENV"
fi
PY="$VENV/bin/python"

# Bounded defaults aligned with optimize_rc_beam + render_beam_gif.py (override via env).
# 32×8 grid, 90/3 → ~31 density dumps + final hold (GIF timing defaults in the renderer).
export UMST_BEAM_DUMP="${UMST_BEAM_DUMP:-1}"
export UMST_BEAM_ITERS="${UMST_BEAM_ITERS:-90}"
export UMST_BEAM_DUMP_STRIDE="${UMST_BEAM_DUMP_STRIDE:-3}"
#
# Optional GIF chrome (see notebooks/render_beam_gif.py docstring):
#   export UMST_BEAM_GIF_MAX_SIDE=1200
#   export UMST_BEAM_GIF_SUPERSAMPLE=2
#   export UMST_BEAM_GIF_FRAME_MS=220
#   export UMST_BEAM_GIF_HOLD_MS=200
#   export UMST_BEAM_GIF_HOLD_FRAMES=3

mkdir -p "$ART_DIR"
mkdir -p "$ROOT/notebooks/_artifacts"

log "UMST_BEAM_DUMP=$UMST_BEAM_DUMP UMST_BEAM_ITERS=$UMST_BEAM_ITERS UMST_BEAM_DUMP_STRIDE=$UMST_BEAM_DUMP_STRIDE"
log "cleaning previous beam iter dumps + manifest under $ART_DIR"
rm -f "$ART_DIR"/iter_*.npy "$ART_DIR/manifest.json"

cd "$CRATE_ROOT"

if ! cargo run --release -p umst-concrete-cartridge \
  --example optimize_rc_beam \
  --features solver-experimental; then
  warn "optimize_rc_beam failed — writing synthetic NPY artefacts for GIF smoke test"
  if ! "$PY" "$NB/beam_demo_synthetic_npys.py"; then
    die "synthetic NPY fallback failed"
  fi
else
  log "optimize_rc_beam finished OK"
fi

if ! "$PY" "$NB/render_beam_gif.py"; then
  die "render_beam_gif.py failed"
fi

log "done — output: $ROOT/notebooks/_artifacts/beam_strut_and_tie.gif"
exit 0
