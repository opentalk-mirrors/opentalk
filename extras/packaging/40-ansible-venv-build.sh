#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail
# shellcheck disable=SC1091
. "$(dirname "${BASH_SOURCE[0]}")/lib/common.sh"
# Build or reuse the embedded venv if requirements hash unchanged (unless forced).
ROOT="$PACKAGING_ROOT"
WHL="$ROOT/build/wheelhouse"
REQ="$ROOT/extras/packaging/requirements-ansible.txt"
VENV_DEST="$ROOT/build/pkgroot/opt/opentalk-compose/runtime/venv"
RTPY="$ROOT/build/python/bin/python3"
PYBIN="${PYBIN:-$RTPY}"
[[ -x "$PYBIN" ]] || PYBIN="$(command -v python3 || true)"
FORCE=${FORCE_REFRESH:-0}
UV_BIN="${UV_BIN:-$(command -v uv || true)}"
MARKER_HASH="$VENV_DEST/.reqs.sha256"

[[ -x "$UV_BIN" ]] || { echo "uv is required to build the embedded runtime."; exit 1; }
compgen -G "$WHL/*.whl" >/dev/null || { echo "Wheelhouse empty: $WHL"; exit 1; }
[[ -f "$REQ" ]] || { echo "Missing $REQ"; exit 1; }
[[ -x "$PYBIN" ]] || { echo "No python runtime."; exit 1; }
[[ -d "$(dirname "$VENV_DEST")" ]] || { echo "Package staging missing: $VENV_DEST"; exit 1; }

CUR_HASH="$(sha256sum "$REQ" | awk '{print $1}')"

if [[ -f "$MARKER_HASH" && $FORCE -eq 0 ]]; then
  OLD_HASH="$(cat "$MARKER_HASH")"
  if [[ "$CUR_HASH" == "$OLD_HASH" && -x "$VENV_DEST/bin/ansible" ]]; then
    echo "[venv] Up-to-date; skip rebuild."
    exit 0
  fi
fi

echo "[venv] (Re)building..."
rm -rf "$VENV_DEST"
"$UV_BIN" venv --no-project --no-python-downloads --python "$PYBIN" "$VENV_DEST"
ln -sfn ../../python/bin/python3 "$VENV_DEST/bin/python"
ln -sfn python "$VENV_DEST/bin/python3"
PYTHON_ABI="$($PYBIN -c 'import sys; print(f"python{sys.version_info.major}.{sys.version_info.minor}")' 2>/dev/null || true)"
if [[ -n "$PYTHON_ABI" ]]; then
  ln -sfn python "$VENV_DEST/bin/$PYTHON_ABI"
fi
"$UV_BIN" pip install --python "$VENV_DEST/bin/python" --offline --no-index --find-links "$WHL" --require-hashes -r "$REQ"
echo "$CUR_HASH" > "$MARKER_HASH"
echo "[venv] Ready."
