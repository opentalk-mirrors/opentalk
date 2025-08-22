#!/usr/bin/env bash
set -euo pipefail
# Download wheels only if missing or forced.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REQ="$ROOT/extras/packaging/requirements-ansible.txt"
WHL="$ROOT/build/wheelhouse"
FORCE=${FORCE_REFRESH:-0}

[[ -f "$REQ" ]] || { echo "Missing $REQ (run 10-ansible-lock.sh)"; exit 1; }
mkdir -p "$WHL"

missing=0
# Check each requirement line with hashes; if corresponding wheel not present, mark missing
while IFS= read -r line; do
  [[ "$line" =~ ^#|^$ ]] && continue
  pkg=$(echo "$line" | awk '{print $1}')
  name="${pkg%%==*}"
  ver="${pkg##*==}"
  # simplistic match: look for any wheel with name-version
  if ! ls "$WHL"/"$name"-"$ver"*  >/dev/null 2>&1; then
    missing=1
    break
  fi
done < "$REQ"

if [[ $missing -eq 0 && $FORCE -eq 0 ]]; then
  echo "[wheelhouse] All wheels present; skip download."
  exit 0
fi

echo "[wheelhouse] Downloading (missing or forced)..."
rm -f "$WHL/.partial"
pip download --require-hashes -r "$REQ" --dest "$WHL"
echo "[wheelhouse] Ready at $WHL"