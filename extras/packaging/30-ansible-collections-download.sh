#!/usr/bin/env bash
set -euo pipefail
# Download Ansible collection tarballs (offline cache).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REQ="${1:-$ROOT/automation/collections/requirements.yml}"
OUT="$ROOT/build/collections_dist"
STATE="$ROOT/build/.cache/collections.requirements.sha256"
FORCE=${FORCE_REFRESH:-0}

[[ -f "$REQ" ]] || { echo "[collections] NOTE: requirements file missing ($REQ) - skip."; exit 0; }
mkdir -p "$OUT" "$ROOT/build/.cache"

echo "[collections] Using requirements: $REQ"
grep -v '^[[:space:]]*#' "$REQ" | sed 's/^/  /' || true

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
ANSIBLE_GALAXY="${ANSIBLE_GALAXY_BIN:-ansible-galaxy}"
set -x
$ANSIBLE_GALAXY collection download -r "$REQ" -p "$OUT"
set +x
echo "$CUR_HASH" > "$STATE"

count=$(ls -1 "$OUT"/*.tar.gz 2>/dev/null | wc -l || true)
echo "[collections] Tarballs at $OUT (count=$count)"
[[ $count -gt 0 ]] || { echo "[collections] WARN: no tarballs downloaded."; }