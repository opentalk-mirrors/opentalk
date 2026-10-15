#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

bash -n extras/packaging/*.sh extras/packaging/wrappers/* extras/packaging/lib/common.sh
shellcheck extras/packaging/*.sh extras/packaging/wrappers/* extras/packaging/lib/common.sh
BUILD_TARGET=both extras/packaging/install-build-deps.sh --check

auto_revision_help=$(extras/packaging/build-all.sh --help)
grep -q -- '--auto-revision' <<<"$auto_revision_help"

package_targets=$(extras/packaging/build-all.sh \
  --help 2>&1)
grep -q -- '--offline' <<<"$package_targets"
grep -q -- '--auto-revision' <<<"$package_targets"

echo "Packaging checks passed."