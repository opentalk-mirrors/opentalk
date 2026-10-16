#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail
# shellcheck disable=SC1091
. "$(dirname "${BASH_SOURCE[0]}")/lib/common.sh"
# Fetch portable Python with uv unless already matching the requested version.
ROOT="$PACKAGING_ROOT"
OUTDIR="$ROOT/build/python"
TMP="$ROOT/build/.tmp-python"
PYTHON_VERSION=""
FORCE=${FORCE_REFRESH:-0}
MARKER="$OUTDIR/.source-url"
UV_BIN="${UV_BIN:-$(command -v uv || true)}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version) PYTHON_VERSION="${2:?}"; shift;;
    -h|--help) echo "Usage: $0 --version <python-version>"; exit 0;;
    *) echo "Unknown option: $1" >&2; exit 2;;
  esac; shift
done

[[ -n "$PYTHON_VERSION" ]] || { echo "Python version is required." >&2; exit 1; }
[[ -x "$UV_BIN" ]] || { echo "uv is required to download Python." >&2; exit 1; }

mkdir -p "$ROOT/build"
REQUEST_ID="uv-python:$PYTHON_VERSION"

if [[ -d "$OUTDIR/bin" && $FORCE -eq 0 && -f "$MARKER" && "$(cat "$MARKER")" == "$REQUEST_ID" ]]; then
  echo "[python] Cached runtime matches source and checksum. Skip."
  exit 0
fi

rm -rf "$TMP"; mkdir -p "$TMP"
rm -rf "$OUTDIR"
mkdir -p "$OUTDIR"
echo "[python] Installing Python $PYTHON_VERSION with uv"
"$UV_BIN" python install "$PYTHON_VERSION" --install-dir "$TMP/python" --no-bin
installed_dir=$(find "$TMP/python" -mindepth 1 -maxdepth 1 -type d -name 'cpython-*' -print -quit)
[[ -n "$installed_dir" ]] || { echo "uv did not create a Python installation." >&2; exit 1; }
cp -a "$installed_dir/." "$OUTDIR/"
[[ -x "$OUTDIR/bin/python3" ]] || {
  echo "uv Python installation has no executable." >&2
  exit 1
}
"$OUTDIR/bin/python3" -V
echo "$REQUEST_ID" > "$MARKER"
echo "[python] Ready at $OUTDIR"