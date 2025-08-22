#!/usr/bin/env bash
set -euo pipefail
# Download collection tarballs if requirements changed.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REQ="${1:-$ROOT/services/cforge/ansible/collections/requirements.yml}"
OUT="$ROOT/build/collections_dist"
STATE="$ROOT/build/.cache/collections.requirements.sha256"
FORCE=${FORCE_REFRESH:-0}

[[ -f "$REQ" ]] || { echo "NOTE: no requirements file ($REQ). Skip."; exit 0; }
mkdir -p "$OUT" "$ROOT/build/.cache"

CUR_HASH="$(sha256sum "$REQ" | awk '{print $1}')"
if [[ -f "$STATE" && $FORCE -eq 0 ]]; then
  OLD_HASH="$(cat "$STATE")"
  if [[ "$CUR_HASH" == "$OLD_HASH" && -n "$(ls -1 "$OUT"/*.tar.gz 2>/dev/null | head -1)" ]]; then
    echo "[collections] Up-to-date; skip download."
    exit 0
  fi
fi

echo "[collections] Downloading (changed or forced)..."
rm -f "$OUT"/*.tar.gz 2>/dev/null || true
ansible-galaxy collection download -r "$REQ" -p "$OUT"
echo "$CUR_HASH" > "$STATE"
echo "[collections] Tarballs at $OUT"