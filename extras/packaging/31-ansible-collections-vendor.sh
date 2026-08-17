#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail
# shellcheck disable=SC1091
. "$(dirname "${BASH_SOURCE[0]}")/lib/common.sh"
# Install downloaded collection tarballs into build/vendor (offline layout).
ROOT="$PACKAGING_ROOT"
REQ="${1:-$ROOT/automation/collections/requirements.yml}"
DIST="$ROOT/build/collections_dist"
VENDOR="$ROOT/build/vendor"
STATE_HASH="$ROOT/build/.cache/collections.vendor.sha256"
FORCE=${FORCE_REFRESH:-0}

[[ -f "$REQ" ]] || { echo "[vendor] NOTE: requirements file missing ($REQ) - skip."; exit 0; }
mkdir -p "$ROOT/build/.cache"

CUR_HASH="$(sha256sum "$REQ" | awk '{print $1}')"
if [[ -d "$VENDOR/collections/ansible_collections" && -f "$STATE_HASH" && $FORCE -eq 0 ]]; then
  OLD_HASH="$(cat "$STATE_HASH")"
  if [[ "$CUR_HASH" == "$OLD_HASH" ]]; then
    echo "[vendor] Up-to-date; skip."
    exit 0
  fi
fi

[[ -d "$DIST" ]] || { echo "[vendor] ERROR: dist dir missing ($DIST) - run 30-script first."; exit 1; }
shopt -s nullglob
TARS=("$DIST"/*.tar.gz)
if (( ${#TARS[@]} == 0 )); then
  echo "[vendor] ERROR: no tarballs in $DIST - 30-script failed?"
  exit 1
fi

echo "[vendor] Rebuilding vendor (tarballs: ${#TARS[@]})"
rm -rf "$VENDOR"
mkdir -p "$VENDOR/collections"
ANSIBLE_GALAXY="${ANSIBLE_GALAXY_BIN:-ansible-galaxy}"

set -x
FORCE_FLAG="--force"
if [[ "${DISABLE_FORCE_VENDOR_INSTALL:-0}" == "1" ]]; then
  FORCE_FLAG=""
fi

for f in "${TARS[@]}"; do
  # Always force installation into vendor to avoid early exit when the same
  # version is already installed in a different (global/user) collections path.
  $ANSIBLE_GALAXY collection install "$f" -p "$VENDOR/collections" ${FORCE_FLAG}
done
set +x

find "$VENDOR/collections" -maxdepth 4 -type d -name ansible_collections -print | sed 's/^/[vendor] + /' || true
if ! find "$VENDOR/collections" -maxdepth 3 -type d -name ansible_collections -quit; then
  echo "[vendor] ERROR: ansible_collections directory not created."
  exit 1
fi

# Mandatory collections validation (fail fast if something went missing)
for coll in community/general community/docker; do
  if [[ ! -d "$VENDOR/collections/ansible_collections/$coll" ]]; then
    echo "[vendor] ERROR: required collection '$coll' missing in vendor path." >&2
    echo "           Expected directory: $VENDOR/collections/ansible_collections/$coll" >&2
    echo "           Available collections:" >&2
    find "$VENDOR/collections/ansible_collections" -maxdepth 2 -mindepth 2 -type d -printf '             - %P\n' 2>/dev/null || true
    exit 2
  fi
done

echo "[vendor] Verified required collections: community.general, community.docker"

echo "$CUR_HASH" > "$STATE_HASH"
echo "[vendor] Ready at $VENDOR"