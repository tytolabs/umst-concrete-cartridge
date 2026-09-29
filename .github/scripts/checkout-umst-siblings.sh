#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# Legacy path checkout for monorepo dev only. Public CI uses git-pinned workspace.dependencies (W-62).
set -euo pipefail
if [ -z "${UMST_PRIVATE_CHECKOUT:-}" ]; then
  echo "::error::UMST_PRIVATE_CHECKOUT secret missing. Public CI does not use sibling path checkout — use git SHAs in Cargo.toml. For local monorepo path overrides, copy .cargo/config.toml.example → .cargo/config.toml"
  exit 1
fi
PARENT="$(dirname "${GITHUB_WORKSPACE:?GITHUB_WORKSPACE required}")"
clone_private() {
  local name="$1"
  local sha="${2:-}"
  local dest="${PARENT}/${name}"
  local url="https://x-access-token:${UMST_PRIVATE_CHECKOUT}@github.com/tytolabs/${name}.git"
  if [ -d "${dest}/.git" ]; then
    git -C "${dest}" fetch --depth 1 origin "${sha:-HEAD}"
    git -C "${dest}" checkout "${sha:-FETCH_HEAD}"
  else
    if [ -n "${sha}" ]; then
      git clone --depth 1 "${url}" "${dest}"
      git -C "${dest}" fetch --depth 1 origin "${sha}"
      git -C "${dest}" checkout "${sha}"
    else
      git clone --depth 1 "${url}" "${dest}"
    fi
  fi
  echo "cloned ${name} @ $(git -C "${dest}" rev-parse --short HEAD)"
}
if [ "$#" -eq 0 ]; then
  mapfile -t specs < <(python3 - <<'PY'
import tomllib
from pathlib import Path
pins = tomllib.loads(Path(".umst-pins.toml").read_bytes())
for name, cfg in pins.items():
    print(f"{name}@{cfg['sha']}")
PY
)
  set -- "${specs[@]}"
fi
for spec in "$@"; do
  name="${spec%%@*}"
  sha=""
  if [[ "${spec}" == *"@"* ]]; then sha="${spec#*@}"; fi
  clone_private "${name}" "${sha}"
done
