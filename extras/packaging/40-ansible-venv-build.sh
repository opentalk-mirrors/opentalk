#!/usr/bin/env bash
set -euo pipefail
# Build or reuse venv if requirements hash unchanged (unless forced).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WHL="$ROOT/build/wheelhouse"
REQ="$ROOT/extras/packaging/requirements-ansible.txt"
EMB="$ROOT/build/embedded"
RTPY="$ROOT/build/python/bin/python3"
PYBIN="${PYBIN:-$RTPY}"
[[ -x "$PYBIN" ]] || PYBIN="$(command -v python3 || true)"
FORCE=${FORCE_REFRESH:-0}
MARKER_HASH="$EMB/.venv/.reqs.sha256"

ls "$WHL"/*.whl >/dev/null 2>&1 || { echo "Wheelhouse empty: $WHL"; exit 1; }
[[ -f "$REQ" ]] || { echo "Missing $REQ"; exit 1; }
[[ -x "$PYBIN" ]] || { echo "No python runtime."; exit 1; }

CUR_HASH="$(sha256sum "$REQ" | awk '{print $1}')"

if [[ -f "$MARKER_HASH" && $FORCE -eq 0 ]]; then
  OLD_HASH="$(cat "$MARKER_HASH")"
  if [[ "$CUR_HASH" == "$OLD_HASH" && -x "$EMB/.venv/bin/ansible" ]]; then
    echo "[venv] Up-to-date; skip rebuild."
    exit 0
  fi
fi

echo "[venv] (Re)building..."
rm -rf "$EMB" && mkdir -p "$EMB"
"$PYBIN" -m venv "$EMB/.venv"
. "$EMB/.venv/bin/activate"
pip install --upgrade pip
pip install --no-index --find-links "$WHL" --require-hashes -r "$REQ"
deactivate
find "$EMB/.venv" -name "__pycache__" -type d -exec rm -rf {} + || true
find "$EMB/.venv" -name "*.pyc" -delete || true
echo "$CUR_HASH" > "$MARKER_HASH"
echo "[venv] Ready."