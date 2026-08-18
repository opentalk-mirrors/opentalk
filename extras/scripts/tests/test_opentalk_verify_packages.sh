#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
VERIFY="$REPO_ROOT/extras/packaging/verify-packages.sh"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# Argument validation must not require any package tooling.
set +e
usage_err=$(bash "$VERIFY" 2>&1)
usage_rc=$?
set -e
[[ $usage_rc -eq 2 ]] || { echo "verify-packages.sh without arguments must exit 2 (got $usage_rc)." >&2; exit 1; }
grep -q 'Usage:' <<<"$usage_err"

# Metadata/function checks below need the package inspection tools.
for tool in dpkg-deb rpm rpm2cpio cpio; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "Package smoke-check metadata tests skipped (missing tool: $tool)." >&2
    exit 0
  fi
done

set +e
bash "$VERIFY" "$TMP_DIR/missing.deb" >/dev/null 2>&1
missing_rc=$?
set -e
[[ $missing_rc -eq 1 ]] || { echo "verify-packages.sh must exit 1 for missing package files (got $missing_rc)." >&2; exit 1; }

printf 'not-a-real-package' > "$TMP_DIR/not-a-package"
set +e
unsupported_err=$(bash "$VERIFY" "$TMP_DIR/not-a-package" 2>&1)
unsupported_rc=$?
set -e
[[ $unsupported_rc -eq 1 ]] || { echo "verify-packages.sh must exit 1 for unsupported package types (got $unsupported_rc)." >&2; exit 1; }
grep -q 'Unsupported package type' <<<"$unsupported_err"

echo "Package smoke-check argument and metadata tests passed."