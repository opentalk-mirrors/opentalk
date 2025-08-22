#!/usr/bin/env bash
set -euo pipefail
# Rebuild / repair runtime venv offline.

PREFIX="/opt/opentalk-compose"
RUNTIME_DIR="$PREFIX/runtime"
VENV_DIR="$RUNTIME_DIR/venv"
ASSETS_DIR="$RUNTIME_DIR/build_assets"
WHEELHOUSE="$ASSETS_DIR/wheelhouse"
REQ_TXT="$ASSETS_DIR/requirements-ansible.txt"
REQ_HASH="$ASSETS_DIR/requirements-ansible.sha256"
PYTHON_BIN="$RUNTIME_DIR/python/bin/python3"

log() { echo "[postinst] $*"; }

export PIP_NO_INDEX=1
export PIP_DISABLE_PIP_VERSION_CHECK=1
export PIP_RETRY=0

[ -f "$REQ_TXT" ] || { log "Requirements file missing ($REQ_TXT) - cannot build venv."; exit 0; }

need_rebuild=0
if [ ! -x "$VENV_DIR/bin/python3" ]; then
  log "Venv incomplete/missing - rebuild."
  need_rebuild=1
fi
if [ $need_rebuild -eq 0 ] && [ -f "$VENV_DIR/bin/ansible" ]; then
  first="$(head -1 "$VENV_DIR/bin/ansible" 2>/dev/null || true)"
  if ! grep -q '^#!' <<<"$first" || ! [ -x "${first#!}" ]; then
    log "Shebang interpreter invalid - rebuild."
    need_rebuild=1
  fi
fi
if [ $need_rebuild -eq 0 ] && [ -f "$VENV_DIR/pyvenv.cfg" ]; then
  if ! grep -q "$RUNTIME_DIR/python" "$VENV_DIR/pyvenv.cfg"; then
    log "pyvenv.cfg home mismatch - rebuild."
    need_rebuild=1
  fi
fi
if [ $need_rebuild -eq 0 ] && [ -f "$REQ_HASH" ]; then
  EXPECTED="$(cat "$REQ_HASH")"
  ACTUAL="$(sha256sum "$REQ_TXT" | awk '{print $1}')"
  [ "$EXPECTED" = "$ACTUAL" ] || { log "Requirements hash mismatch - rebuild."; need_rebuild=1; }
fi

[ $need_rebuild -eq 0 ] && { log "No rebuild required."; exit 0; }
[ -x "$PYTHON_BIN" ] || { log "Portable python missing ($PYTHON_BIN)"; exit 1; }

log "Creating virtualenv..."
rm -rf "$VENV_DIR"
"$PYTHON_BIN" -m venv "$VENV_DIR"
. "$VENV_DIR/bin/activate"

log "Installing wheels (offline)..."
if [ -d "$WHEELHOUSE" ]; then
  pip install --no-index --find-links "$WHEELHOUSE" -r "$REQ_TXT"
else
  log "Wheelhouse missing - cannot install dependencies."
  deactivate
  exit 1
fi

deactivate
log "Venv rebuild complete."
exit 0