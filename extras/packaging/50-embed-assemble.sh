#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
EMB="$ROOT/build/embedded"
TPL_CFG="$ROOT/extras/packaging/templates/ansible.cfg"
VEN_COL="$ROOT/build/vendor/collections"
[ -d "$EMB/.venv" ] || { echo "Missing venv in $EMB. Run 40-ansible-venv-build.sh"; exit 1; }
cp -a "$TPL_CFG" "$EMB/ansible.cfg"
mkdir -p "$EMB/vendor"
[ -d "$VEN_COL" ] && { rm -rf "$EMB/vendor/collections"; cp -a "$VEN_COL" "$EMB/vendor/"; } || true
mkdir -p "$EMB/wrappers"
cp -a "$ROOT/extras/packaging/wrappers/." "$EMB/wrappers/" 2>/dev/null || true
chmod +x "$EMB/wrappers/"* 2>/dev/null || true
echo "Embedded assembled at $EMB"