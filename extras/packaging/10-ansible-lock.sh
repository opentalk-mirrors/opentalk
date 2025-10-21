#!/usr/bin/env bash
set -euo pipefail
# Lock requirements only if source changed or forced.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
IN="$ROOT/extras/packaging/requirements-ansible.in"
OUT="$ROOT/extras/packaging/requirements-ansible.txt"
STATE="$ROOT/build/.cache/requirements-ansible.in.sha256"
FORCE=${FORCE_REFRESH:-0}

mkdir -p "$ROOT/build/.cache"
[[ -f "$IN" ]] || { echo "Missing $IN"; exit 1; }

CUR_HASH="$(sha256sum "$IN" | awk '{print $1}')"
if [[ -f "$OUT" && -f "$STATE" && $FORCE -eq 0 ]]; then
  OLD_HASH="$(cat "$STATE")"
  if [[ "$CUR_HASH" == "$OLD_HASH" ]]; then
    echo "[lock] Up-to-date (no changes in $IN)"
    exit 0
  fi
fi

echo "[lock] (Re)generating lock file..."
# Use ephemeral venv to avoid PATH issues
LOCKENV="$ROOT/build/.lockenv"
rm -rf "$LOCKENV"

# Prefer embedded portable python if fetched
PY_BIN="python3"
if [[ -x "$ROOT/build/python/bin/python3" ]]; then
  PY_BIN="$ROOT/build/python/bin/python3"
fi

"$PY_BIN" -m venv "$LOCKENV"
. "$LOCKENV/bin/activate"
pip install -q --upgrade pip pip-tools || { echo "[lock] ERROR: pip-tools install failed" >&2; deactivate || true; rm -rf "$LOCKENV"; exit 1; }
python -m piptools compile --generate-hashes "$IN" -o "$OUT" || { echo "[lock] ERROR: piptools compile failed" >&2; deactivate || true; rm -rf "$LOCKENV"; exit 1; }
deactivate || true
rm -rf "$LOCKENV"
echo "$CUR_HASH" > "$STATE"
echo "[lock] Wrote $OUT"