#!/usr/bin/env bash
set -euo pipefail
# Post-install (and self-heal) script.
# Rebuilds the venv if missing or if requirements hash changed.

PREFIX="/opt/opentalk-compose"
PY="$PREFIX/.python/bin/python3"
REQ="$PREFIX/extras/packaging/requirements-ansible.txt"
WHL="$PREFIX/build_assets/wheelhouse"
MARKER="$PREFIX/.venv/.reqs.sha256"

have_python() { [[ -x "$PY" ]]; }

compute_hash() { sha256sum "$REQ" | awk '{print $1}'; }

need_rebuild() {
  # Rebuild if venv missing or marker mismatch
  [[ ! -x "$PREFIX/.venv/bin/python" ]] && return 0
  [[ -f "$REQ" ]] || return 1
  NEW_HASH="$(compute_hash)"
  [[ -f "$MARKER" ]] || return 0
  OLD_HASH="$(cat "$MARKER")"
  [[ "$NEW_HASH" != "$OLD_HASH" ]]
}

if have_python; then
  if need_rebuild; then
    echo "[postinst] Rebuilding embedded venv..."
    rm -rf "$PREFIX/.venv"
    "$PY" -m venv "$PREFIX/.venv"
    "$PREFIX/.venv/bin/pip" install --upgrade pip >/dev/null
    if [[ -d "$WHL" && -f "$REQ" ]]; then
      "$PREFIX/.venv/bin/pip" install --no-index --find-links "$WHL" --require-hashes -r "$REQ"
      compute_hash > "$MARKER"
    else
      echo "[postinst] WARNING: wheelhouse or requirements missing – partial venv." >&2
    fi
  else
    echo "[postinst] Existing venv OK."
  fi
else
  echo "[postinst] ERROR: Python runtime missing at $PY" >&2
fi

# Standard runtime dirs
mkdir -p /var/opt/opentalk-compose/{data,backups}
chmod 0755 /var/opt/opentalk-compose || true

exit 0