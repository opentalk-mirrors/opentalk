#!/usr/bin/env bash
set -euo pipefail
ARG="${1:-}"  # rpm passes $1: 0=final erase, 1=upgrade, 2=fail? etc
ACTION="${1:-remove}"  # legacy compatibility if called manually

PREFIX="/opt/opentalk-compose"
RUNTIME_DIR="$PREFIX/runtime"
VENV_DIR="$RUNTIME_DIR/venv"
PY_DIR="$RUNTIME_DIR/python"
SERVICES_DIR="$PREFIX/services"
COMPOSE_FILE="$PREFIX/compose.yml"
COMPOSE_ENV_FILE="$PREFIX/.env"
COMMON_FILE="$PREFIX/common.yml"
AUTOMATION_DIR="$PREFIX/automation"

log() { echo "[postrm] $*"; }

cleanup_ephemeral() {
  rm -rf "$VENV_DIR" "$PY_DIR" "$SERVICES_DIR" "$COMPOSE_FILE" "$COMPOSE_ENV_FILE" "$COMMON_FILE"
  if [ -d "$RUNTIME_DIR" ]; then
    rm -rf "$RUNTIME_DIR"
    log "Removed runtime dir: $RUNTIME_DIR"
  fi
  # Remove automation only on final purge (controlled outside here)
}

remove_prefix_if_empty() {
  if [ -d "$PREFIX" ]; then
    if [ -z "$(find "$PREFIX" -mindepth 1 -maxdepth 1 2>/dev/null)" ]; then
      rmdir "$PREFIX" 2>/dev/null && log "Removed empty prefix: $PREFIX" || true
    fi
  fi
}

if [[ -n "$ARG" && "$ARG" =~ ^[0-9]+$ ]]; then
  # rpm scriptlet context
  # $1 meanings (rpm):
  # 0 -> package is being erased (final removal)
  # 1 -> upgrade (old version being uninstalled while new installed)
  # >1 other states (treat as non-final removal)
  if [[ "$ARG" == "0" ]]; then
    cleanup_ephemeral
    # Remove automation + bin directories explicitly on final removal
    if [ -f "$COMMON_FILE" ]; then
      rm -f "$COMMON_FILE" && log "Purged common.yml: $COMMON_FILE" || true
    fi
    # Final sweep: if prefix still exists, attempt to remove harmless leftover files (compose/override/common yaml, env files)
    if [ -d "$PREFIX" ]; then
      # Gather remaining top-level entries
      rem_entries=$(find "$PREFIX" -mindepth 1 -maxdepth 1 -printf '%f\n' 2>/dev/null || true)
      if [ -n "$rem_entries" ]; then
        safe=1
        while IFS= read -r e; do
          case "$e" in
            common.yml|compose.yml|*.override.yml|*.yml|.env|README*|LICENSE*|CHANGELOG* ) ;; # allowed
            *) safe=0; break ;;
          esac
        done <<< "$rem_entries"
        if [ $safe -eq 1 ]; then
          while IFS= read -r e; do rm -f "$PREFIX/$e" 2>/dev/null || true; done <<< "$rem_entries"
          log "Removed leftover files in prefix (final purge)."
        else
          log "Leftover files not auto-purged (unexpected entries present)."
        fi
      fi
    fi
    if [ -d "$AUTOMATION_DIR" ]; then
      rm -rf "$AUTOMATION_DIR"
      log "Purged automation dir (arg=0): $AUTOMATION_DIR"
    fi
    if [ -d "$PREFIX/bin" ]; then
      rm -rf "$PREFIX/bin"
      log "Purged bin dir (arg=0): $PREFIX/bin"
    fi
    remove_prefix_if_empty
  else
    log "Upgrade or non-final removal (arg=$ARG) - skipping runtime purge."
  fi
else
  # Non-rpm numeric context (likely fpm --after-remove). Default: DO NOT purge unless override.
  if [[ "${FORCE_POSTRM_PURGE:-0}" == "1" ]]; then
    log "FORCE_POSTRM_PURGE=1 set - performing cleanup (non-numeric arg context)."
    cleanup_ephemeral
    # If package really gone, purge automation directory too
    if ! rpm -q opentalk-compose >/dev/null 2>&1; then
      if [ -d "$AUTOMATION_DIR" ]; then
        rm -rf "$AUTOMATION_DIR"
        log "Purged automation dir (final removal): $AUTOMATION_DIR"
      fi
      if [ -f "$COMMON_FILE" ]; then
        rm -f "$COMMON_FILE" && log "Purged common.yml: $COMMON_FILE" || true
      fi
      remove_prefix_if_empty
    fi
  else
    # Detect if package truly erased but script called without arg (fpm after-remove scenario following rpm erase)
    if ! rpm -q opentalk-compose >/dev/null 2>&1; then
      log "Detected package not installed; performing final prefix purge."
      cleanup_ephemeral
      if [ -d "$AUTOMATION_DIR" ]; then
        rm -rf "$AUTOMATION_DIR"
        log "Purged automation dir: $AUTOMATION_DIR"
      fi
      if [ -f "$COMMON_FILE" ]; then
        rm -f "$COMMON_FILE" && log "Purged common.yml: $COMMON_FILE" || true
      fi
      # Attempt final sweep for benign leftovers
      if [ -d "$PREFIX" ]; then
        rem_entries=$(find "$PREFIX" -mindepth 1 -maxdepth 1 -printf '%f\n' 2>/dev/null || true)
        if [ -n "$rem_entries" ]; then
          safe=1
          while IFS= read -r e; do
            case "$e" in
              common.yml|compose.yml|*.override.yml|*.yml|.env|README*|LICENSE*|CHANGELOG* ) ;;
              *) safe=0; break ;;
            esac
          done <<< "$rem_entries"
          if [ $safe -eq 1 ]; then
            while IFS= read -r e; do rm -f "$PREFIX/$e" 2>/dev/null || true; done <<< "$rem_entries"
            log "Removed leftover files in prefix (final purge)."
          else
            log "Leftover files not auto-purged (unexpected entries present)."
          fi
        fi
      fi
      remove_prefix_if_empty
    else
      log "Non-rpm postrm invocation without numeric arg - skipping purge (set FORCE_POSTRM_PURGE=1 to force)."
    fi
  fi
fi
exit 0