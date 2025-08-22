#!/usr/bin/env bash
set -euo pipefail
# Install vendored collections only if tarballs newer or vendor missing.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SRC="$ROOT/build/collections_dist"
DST="$ROOT/build/vendor/collections"
FORCE=${FORCE_REFRESH:-0}

shopt -s nullglob
TARS=("$SRC"/*.tar.gz)
[[ ${#TARS[@]} -eq 0 ]] && { echo "[vendor] No tarballs. Skip."; exit 0; }

if [[ -d "$DST" && $FORCE -eq 0 ]]; then
  # crude freshness: compare latest mtime tarball vs marker
  MARKER="$DST/.installed"
  latest_tar=$(ls -t "$SRC"/*.tar.gz | head -1)
  if [[ -f "$MARKER" && "$MARKER" -nt "$latest_tar" ]]; then
    echo "[vendor] Up-to-date; skip."
    exit 0
  fi
fi

echo "[vendor] Installing collections..."
rm -rf "$DST"
mkdir -p "$DST"
ansible-galaxy collection install "${TARS[@]}" -p "$DST"
date -u > "$DST/.installed"
echo "[vendor] Installed to $DST"