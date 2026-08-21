#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

if [[ -z "${PACKAGING_ROOT:-}" ]]; then
  PACKAGING_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
fi

if [[ -z "${SOURCE_DATE_EPOCH:-}" ]]; then
  SOURCE_DATE_EPOCH="$(git -C "$PACKAGING_ROOT" log -1 --format=%ct 2>/dev/null || date +%s)"
fi
export SOURCE_DATE_EPOCH

log() {
  local scope="$1"
  shift
  printf '[%s] %s\n' "$scope" "$*"
}

die() {
  local scope="$1"
  shift
  printf '[%s] ERROR: %s\n' "$scope" "$*" >&2
  exit 1
}

run() {
  printf '>>> %s\n' "$*"
  "$@"
}

sha256_file() {
  sha256sum "$1" | awk '{print $1}'
}

require_commands() {
  local command_name
  for command_name in "$@"; do
    command -v "$command_name" >/dev/null 2>&1 || die deps "required command not found: $command_name"
  done
}

# Readers for .versions.yml live in lib/versions.sh. Source that file in the
# stage scripts that actually need a version pin.
