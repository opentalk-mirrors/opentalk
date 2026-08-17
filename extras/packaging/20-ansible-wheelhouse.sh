#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail
# shellcheck disable=SC1091
. "$(dirname "${BASH_SOURCE[0]}")/lib/common.sh"
# Download wheels only if missing or forced.

# Suppress pip version availability notices for deterministic non-noisy build logs
export PIP_DISABLE_PIP_VERSION_CHECK=1
export PIP_NO_PYTHON_VERSION_WARNING=1

ROOT="$PACKAGING_ROOT"
REQ="$ROOT/extras/packaging/requirements-ansible.txt"
WHL="$ROOT/build/wheelhouse"
FORCE=${FORCE_REFRESH:-0}

[[ -f "$REQ" ]] || { echo "Missing $REQ (run 10-ansible-lock.sh)"; exit 1; }
mkdir -p "$WHL"

missing=0
# Check each requirement line with hashes; if corresponding wheel not present, mark missing
while IFS= read -r line; do
  [[ "$line" =~ ^[[:space:]]*#|^[[:space:]]*$ ]] && continue
  [[ "$line" =~ ^[[:space:]]*--hash ]] && continue
  pkg=$(echo "$line" | awk '{print $1}' | tr '_' '-')
  name="${pkg%%==*}"
  ver="${pkg##*==}"
  found=0
  for stem in "${name}-${ver}" "${name//-/_}-${ver}"; do
    if compgen -G "$WHL/$stem*" >/dev/null; then
      found=1
      break
    fi
  done
  if [[ $found -eq 0 ]]; then
    missing=1
    break
  fi
done < "$REQ"

if [[ $missing -eq 0 && $FORCE -eq 0 ]]; then
  echo "[wheelhouse] All wheels present; skip download."
  exit 0
fi

echo "[wheelhouse] Downloading (missing or forced)..."
rm -f "$WHL/.partial"
# Prefer the embedded portable python (ensures wheel tags match runtime) if present.
PY_DL="$ROOT/build/python/bin/python3"
if [[ -x "$PY_DL" ]]; then
  echo "[wheelhouse] Using portable python: $PY_DL"
  PIP_DISABLE_PIP_VERSION_CHECK=1 "$PY_DL" -m pip download --require-hashes -r "$REQ" --dest "$WHL"
else
  echo "[wheelhouse] Portable python missing, falling back to system pip"
  PIP_DISABLE_PIP_VERSION_CHECK=1 pip download --require-hashes -r "$REQ" --dest "$WHL"
fi
echo "[wheelhouse] Ready at $WHL"

# Verify that each pinned requirement has at least one wheel; account for dash/underscore differences and trailing backslashes.
missing_list=""
# Build a lowercase map of existing wheel basenames for comparison
declare -A _wheel_lc
while IFS= read -r f; do
  base="${f##*/}"
  _wheel_lc["${base,,}"]=1
done < <(find "$WHL" -maxdepth 1 -type f -printf '%f\n')

while IFS= read -r line; do
  [[ $line =~ ^[[:space:]]*#|^[[:space:]]*$ ]] && continue
  first_token=$(echo "$line" | awk '{print $1}')
  [[ $first_token == --hash* ]] && continue
  pkg=${first_token%\\}
  [[ $pkg == *==* ]] || continue
  name="${pkg%%==*}"
  ver="${pkg##*==}"
  ver=${ver%\\}
  # candidate stems (possible normalization variants)
  for stem in "${name}-${ver}" "${name//-/_}-${ver}" "${name//_/-}-${ver}"; do
    # match any wheel file starting with stem (case-insensitive)
    found=0
    for w in "${!_wheel_lc[@]}"; do
      if [[ $w == ${stem,,}* ]]; then
        found=1; break
      fi
    done
    [[ $found -eq 1 ]] && break
  done
  [[ $found -eq 1 ]] || missing_list+="$name==$ver\n"
done < "$REQ"

if [[ -n $missing_list ]]; then
  echo "[wheelhouse] ERROR: Missing wheel files for the following pinned packages:" >&2
  echo -e "$missing_list" >&2
  echo "[wheelhouse] HINT: Likely conditional markers or platform tag mismatch. Retry with FORCE_REFRESH=1 or adjust pins." >&2
  exit 1
fi