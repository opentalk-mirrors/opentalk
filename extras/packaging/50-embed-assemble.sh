#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BUILD_DIR="$ROOT/build"
EMB_DIR="$BUILD_DIR/embedded"
SRC_VENDOR_DIR="$BUILD_DIR/vendor"
ALT_VENDOR_DIR="$BUILD_DIR/collections/vendor"

mkdir -p "$EMB_DIR"

if [[ -d "$SRC_VENDOR_DIR/collections/ansible_collections" ]]; then
  echo "[embed] Copy vendor from build/vendor"
  rm -rf "$EMB_DIR/vendor"
  cp -a "$SRC_VENDOR_DIR" "$EMB_DIR/vendor"
elif [[ -d "$ALT_VENDOR_DIR/collections/ansible_collections" ]]; then
  echo "[embed] Copy vendor from legacy build/collections/vendor"
  rm -rf "$EMB_DIR/vendor"
  cp -a "$ALT_VENDOR_DIR" "$EMB_DIR/vendor"
else
  echo "[embed] WARN: No vendor collections found (build/vendor)."
fi

echo "[embed] Embedded assets prepared at $EMB_DIR"