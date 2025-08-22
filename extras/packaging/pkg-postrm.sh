#!/usr/bin/env bash
set -euo pipefail
# Called with: action (purge|remove|upgrade|abort-install|...)
PREFIX="/opt/opentalk-compose"
ETCDIR="/etc/opentalk-compose"

if [ "${1:-}" = "purge" ]; then
  if [ -d "$PREFIX" ]; then
    find "$PREFIX" -type f -name '*.py[co]' -delete 2>/dev/null || true
    find "$PREFIX" -type d -name '__pycache__' -exec rm -rf {} + 2>/dev/null || true
    rm -rf "$PREFIX" 2>/dev/null || true
  fi
  [ -d "$ETCDIR" ] && rm -rf "$ETCDIR" || true
fi
exit 0