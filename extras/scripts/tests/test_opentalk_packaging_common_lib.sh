#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
COMMON_LIB="$REPO_ROOT/extras/packaging/lib/common.sh"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

export PACKAGING_ROOT="$TMP_DIR"
# shellcheck disable=SC1090 # sourced bash library, path built at runtime
source "$COMMON_LIB"

# SOURCE_DATE_EPOCH must be set (fallback: current time in non-git tree).
[[ -n "${SOURCE_DATE_EPOCH:-}" ]] || { echo "SOURCE_DATE_EPOCH is not set." >&2; exit 1; }

log_out=$(
  log build "hello world"
)
# shellcheck disable=SC2016
[[ "$log_out" == '[build] hello world' ]] || { echo "log() output mismatch: $log_out" >&2; exit 1; }

run_out=$(run printf 'cmd-out')
[[ "$run_out" == $'>>> printf cmd-out\ncmd-out' ]] || { echo "run() output mismatch: $run_out" >&2; exit 1; }

if (run false) >/dev/null 2>&1; then
  echo "run() must propagate a failing command." >&2
  exit 1
fi

die_out=$( ( die scope "boom") 2>&1 || true )
[[ "$die_out" == '[scope] ERROR: boom' ]] || { echo "die() output mismatch: $die_out" >&2; exit 1; }
( die scope "boom" ) 2>/dev/null || [[ $? -eq 1 ]] || { echo "die() must exit 1." >&2; exit 1; }

printf 'expected-hash-input' > "$TMP_DIR/hashfile"
[[ "$(sha256_file "$TMP_DIR/hashfile")" == "$(sha256sum "$TMP_DIR/hashfile" | awk '{print $1}')" ]] \
  || { echo "sha256_file() mismatch." >&2; exit 1; }

require_commands bash 2>/dev/null
if ( require_commands definitely-not-a-real-command-xyz ) 2>/dev/null; then
  echo "require_commands() must fail for missing commands." >&2
  exit 1
fi

echo "Packaging common library tests passed."