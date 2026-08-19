#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail
# Validate the embedded runtime. Runtime creation belongs to the build, not installation.

PREFIX="${OT_PREFIX:-/opt/opentalk-compose}"
RUNTIME_DIR="$PREFIX/runtime"
VENV_DIR="$RUNTIME_DIR/venv"
ASSETS_DIR="$RUNTIME_DIR/build_assets"
REQ_TXT="$ASSETS_DIR/requirements-ansible.txt"
REQ_HASH="$ASSETS_DIR/requirements-ansible.sha256"
PYTHON_BIN="$RUNTIME_DIR/python/bin/python3"

log() { echo "[postinst] $*"; }

CFG_DIR="${OT_CFG_DIR:-/etc/opentalk-compose}"
EXAMPLES_DIR="${OT_EXAMPLES_DIR:-/usr/share/opentalk-compose/examples}"

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

[[ -x "$PYTHON_BIN" ]] || { log "Portable python missing ($PYTHON_BIN)"; exit 1; }
[[ -x "$VENV_DIR/bin/python3" ]] || { log "Embedded venv missing ($VENV_DIR)"; exit 1; }
[[ -x "$VENV_DIR/bin/ansible" ]] || { log "Embedded ansible missing ($VENV_DIR/bin/ansible)"; exit 1; }
[[ -f "$REQ_TXT" ]] || { log "Requirements file missing ($REQ_TXT)"; exit 1; }
[[ -f "$REQ_HASH" ]] || { log "Requirements hash missing ($REQ_HASH)"; exit 1; }

EXPECTED="$(cat "$REQ_HASH")"
ACTUAL="$(sha256sum "$REQ_TXT" | awk '{print $1}')"
[[ "$EXPECTED" == "$ACTUAL" ]] || { log "Requirements hash mismatch."; exit 1; }

log "Embedded Python runtime verified; no target-side package installation required."