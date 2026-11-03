#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# Regression: compose-python created its venv with whatever Python uv found
# first. With Python 3.15 available, ansible-core failed on import, so the
# venv must use the minor release of the pinned python_version.

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/bin"
cat > "$TMP_DIR/bin/uv" <<'SH'
#!/usr/bin/env bash
echo "uv $*" >> "$UV_TEST_LOG"
SH
chmod +x "$TMP_DIR/bin/uv"

pinned="$(sed -n -E 's/^python_version:[[:space:]]*([^[:space:]#]+).*$/\1/p' "$REPO_ROOT/.versions.yml")"
[[ -n "$pinned" ]] || { echo "python_version not found in .versions.yml" >&2; exit 1; }
expected_minor="${pinned%.*}"

PATH="$TMP_DIR/bin:$PATH" \
UV_TEST_LOG="$TMP_DIR/uv.log" \
OPENTALK_DEV_DIR="$TMP_DIR/dev" \
  just --justfile "$REPO_ROOT/justfile" --working-directory "$REPO_ROOT" compose-python >/dev/null

venv_call="$(grep '^uv venv ' "$TMP_DIR/uv.log" || true)"
[[ "$venv_call" == *"--python $expected_minor "* ]] \
  || { cat "$TMP_DIR/uv.log" >&2; echo "compose-python must create the venv with Python $expected_minor" >&2; exit 1; }
[[ "$venv_call" == *"$TMP_DIR/dev/.venv"* ]] \
  || { cat "$TMP_DIR/uv.log" >&2; echo "compose-python must create the venv in OPENTALK_DEV_DIR" >&2; exit 1; }

echo "compose-python Python version tests passed."
