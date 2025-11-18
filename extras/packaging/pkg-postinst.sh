#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# shellcheck disable=SC1091  # Optional venv activation happens conditionally; suppress info.
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

CFG_DIR="/etc/opentalk-compose"
EXAMPLES_DIR="/usr/share/opentalk-compose/examples"

# Ensure main config dir exists (normally packaged)
if [ ! -d "$CFG_DIR" ]; then
  log "Creating missing config directory $CFG_DIR"
  install -d -m 0755 "$CFG_DIR"
fi

# Ensure secrets directory (not a conffile)
if [ ! -d "$CFG_DIR/secrets" ]; then
  install -d -m 0700 "$CFG_DIR/secrets"
else
  chmod 0700 "$CFG_DIR/secrets" || true
fi

# Ensure ssl directory (cert/key drop-in) – not a conffile, restrictive perms
if [ ! -d "$CFG_DIR/ssl" ]; then
  install -d -m 0700 "$CFG_DIR/ssl"
else
  chmod 0700 "$CFG_DIR/ssl" || true
fi

# Recovery: create settings.yml from example ONLY if absent (should exist as conffile)
if [ ! -f "$CFG_DIR/settings.yml" ]; then
  if [ -f "$EXAMPLES_DIR/settings.yml.example" ]; then
    cp "$EXAMPLES_DIR/settings.yml.example" "$CFG_DIR/settings.yml"
    log "Recovered missing settings.yml from example template (unexpected)."
  else
    log "WARNING: settings.yml missing and no example template found."
  fi
fi

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
# Legacy cleanup: remove obsolete /opt/opentalk-compose/config symlink or directory if it exists.
if [ -L "/opt/opentalk-compose/config" ] || [ -d "/opt/opentalk-compose/config" ]; then
  log "Removing obsolete /opt/opentalk-compose/config entry (symlink or dir)"
  rm -rf /opt/opentalk-compose/config || true
fi

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