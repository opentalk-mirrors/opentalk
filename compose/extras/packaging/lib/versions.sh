#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# Readers for .versions.yml, the single source of truth for component versions.
#
# This lives apart from lib/common.sh so that only the stage scripts that
# actually read versions pull it in. Sourcing it everywhere would count these
# lines against the coverage of every unrelated stage script.
#
# Requires lib/common.sh to be sourced first: PACKAGING_ROOT and die() come
# from there.

VERSIONS_FILE="${VERSIONS_FILE:-$PACKAGING_ROOT/.versions.yml}"

# Print one scalar key from .versions.yml. The file is a flat "key: value"
# mapping, so this reads it with bash builtins only: no yq dependency, and it
# still works in the tests that run stage scripts under a stripped PATH. Only a
# key in the first column matches, so comments and nested mappings are ignored.
#
# Returns 1 if the key is absent and 2 if the file is missing. It never exits,
# so callers that tolerate an absent pin can use "|| true"; an exiting helper
# could not be caught that way inside a command substitution.
versions_lookup() {
  local key="$1"
  local line value=""
  [[ -f "$VERSIONS_FILE" ]] || return 2
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ "$line" == "$key:"* ]] || continue
    value="${line#"$key":}"
    value="${value%%#*}"
    value="${value#"${value%%[![:space:]]*}"}"
    value="${value%"${value##*[![:space:]]}"}"
    value="${value%\"}"
    value="${value#\"}"
    break
  done < "$VERSIONS_FILE"
  [[ -n "$value" ]] || return 1
  printf '%s\n' "$value"
}

# Strict read for pins the build cannot proceed without: abort with a message
# naming the file and the key instead of continuing with an empty version.
versions_get() {
  local value rc=0
  value="$(versions_lookup "$1")" || rc=$?
  case $rc in
    0) printf '%s\n' "$value" ;;
    2) die versions "versions file not found: $VERSIONS_FILE" ;;
    *) die versions "key not found in $VERSIONS_FILE: $1" ;;
  esac
}
