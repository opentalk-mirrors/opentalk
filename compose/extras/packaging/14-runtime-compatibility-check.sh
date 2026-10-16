#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail
# shellcheck disable=SC1091
. "$(dirname "${BASH_SOURCE[0]}")/lib/common.sh"

ROOT="$PACKAGING_ROOT"
STAGE="${1:-$ROOT/build/pkgroot}"
MAX_GLIBC="${MAX_GLIBC:-2.34}"

[[ -d "$STAGE" ]] || { echo "[compat] Missing staging directory: $STAGE" >&2; exit 1; }
command -v file >/dev/null 2>&1 || { echo "[compat] 'file' is required." >&2; exit 1; }
command -v objdump >/dev/null 2>&1 || { echo "[compat] 'objdump' is required." >&2; exit 1; }
command -v readelf >/dev/null 2>&1 || { echo "[compat] 'readelf' is required." >&2; exit 1; }

elf_count=0
highest_glibc=""
while IFS= read -r -d '' candidate; do
  description=$(file -b "$candidate")
  [[ "$description" == *ELF* ]] || continue
  ((elf_count += 1))
  [[ "$description" == *"x86-64"* ]] || {
    echo "[compat] Unsupported ELF architecture: $candidate ($description)" >&2
    exit 1
  }
  if readelf -l "$candidate" 2>/dev/null | grep -q '/ld-musl-'; then
    echo "[compat] musl ELF is not supported: $candidate" >&2
    exit 1
  fi
  versions=$(objdump -p "$candidate" 2>/dev/null | grep -oE 'GLIBC_[0-9.]+' | sort -Vu || true)
  candidate_glibc=$(printf '%s\n' "$versions" | tail -n 1)
  if [[ -n "$candidate_glibc" ]]; then
    candidate_version=${candidate_glibc#GLIBC_}
    if [[ "$(printf '%s\n' "$MAX_GLIBC" "$candidate_version" | sort -V | tail -n 1)" != "$MAX_GLIBC" ]]; then
      echo "[compat] $candidate requires glibc $candidate_version (maximum: $MAX_GLIBC): $candidate" >&2
      exit 1
    fi
    [[ -n "$highest_glibc" ]] || highest_glibc="$candidate_glibc"
    if [[ "$(printf '%s\n' "${highest_glibc#GLIBC_}" "$candidate_version" | sort -V | tail -n 1)" == "$candidate_version" ]]; then
      highest_glibc="$candidate_glibc"
    fi
  fi
done < <(
  find "$STAGE" -type f \
    \( -perm /111 -o -name '*.so' -o -name '*.so.*' \) \
    -print0
)

PYTHON_BIN="$STAGE/opt/opentalk-compose/runtime/python/bin/python3"
[[ -x "$PYTHON_BIN" ]] || { echo "[compat] Embedded Python missing: $PYTHON_BIN" >&2; exit 1; }
"$PYTHON_BIN" -c 'import ssl, hashlib, sqlite3, ctypes'

echo "[compat] ELF files checked: $elf_count"
echo "[compat] Highest required glibc: ${highest_glibc:-none}"
echo "[compat] Runtime compatibility check passed."