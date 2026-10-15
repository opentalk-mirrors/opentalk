#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
COMMON_LIB="$REPO_ROOT/extras/packaging/lib/common.sh"
VERSIONS_LIB="$REPO_ROOT/extras/packaging/lib/versions.sh"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

export PACKAGING_ROOT="$TMP_DIR"
# shellcheck disable=SC1090 # sourced bash library, path built at runtime
source "$COMMON_LIB"
# versions.sh relies on PACKAGING_ROOT and die() from common.sh, so it is
# sourced second, exactly as the stage scripts do it.
# shellcheck disable=SC1090 # sourced bash library, path built at runtime
source "$VERSIONS_LIB"

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

# versions_get() reads .versions.yml, the single source of truth for component
# versions. VERSIONS_FILE derives from PACKAGING_ROOT, which points at TMP_DIR.
cat > "$TMP_DIR/.versions.yml" <<'VERSIONS'
---
# a comment line that must never match
opentalk_product_version: v26.2.0-beta.1
uv_version: 0.12.3
python_version: 3.14.6
quoted_version: "1.2.3"
trailing_comment_version: 9.9.9  # inline comment
hyphen-named_version: v0.1.2
opaque_version: RELEASE.2025-10-15T17-29-55Z
nested:
  indented_version: must-not-match
VERSIONS

versions_expect() {
  local key="$1" want="$2" got
  got="$(versions_get "$key")"
  [[ "$got" == "$want" ]] || { echo "versions_get($key) returned '$got', expected '$want'." >&2; exit 1; }
}

versions_expect opentalk_product_version v26.2.0-beta.1
versions_expect uv_version 0.12.3
versions_expect python_version 3.14.6
versions_expect quoted_version 1.2.3
versions_expect trailing_comment_version 9.9.9
versions_expect hyphen-named_version v0.1.2
versions_expect opaque_version RELEASE.2025-10-15T17-29-55Z

# An indented key is nested YAML and must not be matched as a top-level key.
if ( versions_get indented_version ) 2>/dev/null; then
  echo "versions_get() must not match an indented key." >&2
  exit 1
fi

# A missing key is fatal, so a typo in a stage script fails the build loudly.
missing_out=$( ( versions_get absent_version ) 2>&1 || true )
[[ "$missing_out" == *"key not found"* ]] \
  || { echo "versions_get() missing-key message mismatch: $missing_out" >&2; exit 1; }

# versions_lookup() returns instead of exiting, so "|| true" can catch it.
# A helper that called exit would kill the command substitution instead.
optional_value="$( versions_lookup absent_version || true )"
[[ -z "$optional_value" ]] || { echo "versions_lookup() must print nothing for an absent key." >&2; exit 1; }

lookup_rc=0
versions_lookup absent_version >/dev/null || lookup_rc=$?
[[ $lookup_rc -eq 1 ]] || { echo "versions_lookup() must return 1 for an absent key, got $lookup_rc." >&2; exit 1; }

lookup_rc=0
VERSIONS_FILE="$TMP_DIR/nope.yml" versions_lookup uv_version >/dev/null || lookup_rc=$?
[[ $lookup_rc -eq 2 ]] || { echo "versions_lookup() must return 2 for a missing file, got $lookup_rc." >&2; exit 1; }

# A missing versions file is fatal for the strict read and names the path.
absent_file_out=$( ( VERSIONS_FILE="$TMP_DIR/nope.yml" versions_get uv_version ) 2>&1 || true )
[[ "$absent_file_out" == *"versions file not found"* ]] \
  || { echo "versions_get() missing-file message mismatch: $absent_file_out" >&2; exit 1; }

echo "Packaging common library tests passed."