#!/usr/bin/env bash
set -euo pipefail
ACTION="${1:-remove}"

PREFIX="/opt/opentalk-compose"
RUNTIME_DIR="$PREFIX/runtime"
VENV_DIR="$RUNTIME_DIR/venv"
PY_DIR="$RUNTIME_DIR/python"
SERVICES_DIR="$PREFIX/services"
COMPOSE_FILE="$PREFIX/compose.yml"
COMPOSE_ENV_FILE="$PREFIX/.env"

log() { echo "[postrm] $*"; }

cleanup_ephemeral() {
  rm -rf "$VENV_DIR" "$PY_DIR" "$SERVICES_DIR" "$COMPOSE_FILE" "$COMPOSE_ENV_FILE"
  if [ -d "$RUNTIME_DIR" ]; then
    rm -rf "$RUNTIME_DIR"
    log "Removed runtime dir: $RUNTIME_DIR"
  fi
}

remove_prefix_if_empty() {
  if [ -d "$PREFIX" ]; then
    if [ -z "$(find "$PREFIX" -mindepth 1 -maxdepth 1 2>/dev/null)" ]; then
      rmdir "$PREFIX" 2>/dev/null && log "Removed empty prefix: $PREFIX" || true
    fi
  fi
}

case "$ACTION" in
  remove|purge)
    cleanup_ephemeral
    remove_prefix_if_empty
    ;;
  upgrade|failed-upgrade|abort-install|abort-upgrade|disappear)
    :
    ;;
  *)
    cleanup_ephemeral
    remove_prefix_if_empty
    ;;
esac
exit 0