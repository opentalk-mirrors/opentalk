#!/usr/bin/env bash
set -euo pipefail
# Called with: action (remove|upgrade|deconfigure...)
PREFIX="/opt/opentalk-compose"

case "${1:-}" in
  remove|upgrade|deconfigure)
    if [ -d "$PREFIX" ]; then
      find "$PREFIX" -type f -name '*.py[co]' -delete 2>/dev/null || true
      find "$PREFIX" -type d -name '__pycache__' -exec rm -rf {} + 2>/dev/null || true
    fi
    ;;
esac
exit 0