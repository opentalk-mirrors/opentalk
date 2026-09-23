#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
GENERATOR="$REPO_ROOT/extras/scripts/generate-config-reference.py"
REFERENCE="$REPO_ROOT/docs/5-configuration-options-reference.md"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required to run the configuration reference regression test." >&2
  exit 1
fi

if ! python3 -c 'import yaml' >/dev/null 2>&1; then
  echo "PyYAML is required to run the configuration reference regression test." >&2
  exit 1
fi

GENERATED="$TMP_DIR/reference.md"
python3 "$GENERATOR" --output "$GENERATED" >/dev/null

normalize() {
  sed 's/^Last updated: .*$/Last updated: <normalized>/' "$1"
}

if ! diff -u <(normalize "$REFERENCE") <(normalize "$GENERATED") >"$TMP_DIR/diff.txt"; then
  echo "Generated configuration reference does not match the checked-in file." >&2
  echo "Run 'python3 extras/scripts/generate-config-reference.py' and commit the result." >&2
  cat "$TMP_DIR/diff.txt" >&2
  exit 1
fi

grep -q '^# Configuration Reference' "$GENERATED"
grep -q '^## opentalk_compose_' "$GENERATED"
grep -q '^| Variable | Type | Default / Example Value |' "$GENERATED"

echo "Configuration reference generator regression tests passed."