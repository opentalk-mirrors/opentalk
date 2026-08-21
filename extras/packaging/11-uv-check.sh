#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail
# shellcheck disable=SC1091
. "$(dirname "${BASH_SOURCE[0]}")/lib/common.sh"
# shellcheck disable=SC1091
. "$(dirname "${BASH_SOURCE[0]}")/lib/versions.sh"

UV_VERSION="$(versions_get uv_version)"

UV_BIN="${UV_BIN:-$(command -v uv || true)}"
[[ -x "$UV_BIN" ]] || {
  echo "[uv] ERROR: uv $UV_VERSION is required for the build." >&2
  exit 1
}

INSTALLED_VERSION="$($UV_BIN --version | awk '{print $2}')"
[[ "$INSTALLED_VERSION" == "$UV_VERSION" ]] || {
  echo "[uv] ERROR: expected uv $UV_VERSION, found $INSTALLED_VERSION." >&2
  exit 1
}

echo "[uv] Using uv $INSTALLED_VERSION from $UV_BIN"