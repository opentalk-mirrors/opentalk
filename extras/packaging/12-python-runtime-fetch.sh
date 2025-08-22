#!/usr/bin/env bash
set -euo pipefail
# Fetch portable Python unless already matching same URL (marker).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUTDIR="$ROOT/build/python"
TMP="$ROOT/build/.tmp-python"
URL=""
FILE=""
SHA="${PY_SHA256:-}"
FORCE=${FORCE_REFRESH:-0}
MARKER="$OUTDIR/.source-url"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --url) URL="${2:?}"; shift;;
    --file) FILE="${2:?}"; shift;;
    -h|--help) echo "Usage: $0 --url <tar> | --file <tar> (env: PY_SHA256, FORCE_REFRESH)"; exit 0;;
    *) echo "Unknown option: $1" >&2; exit 2;;
  esac; shift
done

if [[ -d "$OUTDIR/bin" && $FORCE -eq 0 ]]; then
  if [[ -n "$URL" && -f "$MARKER" && "$(cat "$MARKER")" == "$URL" ]]; then
    echo "[python] Cached runtime matches URL. Skip."
    exit 0
  fi
  if [[ -n "$FILE" && -f "$MARKER" && "$(cat "$MARKER")" == "file:$FILE" ]]; then
    echo "[python] Cached runtime matches file. Skip."
    exit 0
  fi
fi

mkdir -p "$ROOT/build"
rm -rf "$OUTDIR" "$TMP"; mkdir -p "$TMP"

if [[ -n "$FILE" ]]; then
  [[ -f "$FILE" ]] || { echo "File not found: $FILE" >&2; exit 1; }
  TARPATH="$FILE"
  SRC_ID="file:$FILE"
elif [[ -n "$URL" ]]; then
  fname="$TMP/$(basename "$URL")"
  echo "[python] Download $URL"
  curl -fL "$URL" -o "$fname"
  TARPATH="$fname"
  SRC_ID="$URL"
else
  echo "No URL or file provided." >&2
  exit 1
fi

if [[ -n "$SHA" ]]; then
  echo "$SHA  $TARPATH" | sha256sum -c - || { echo "SHA256 mismatch" >&2; exit 1; }
else
  echo "[python] WARN: no checksum provided."
fi

echo "[python] Extracting..."
mkdir -p "$OUTDIR"
tar -xf "$TARPATH" -C "$OUTDIR" --strip-components=1
[[ -x "$OUTDIR/bin/python3" ]] || { echo "Extraction failed" >&2; exit 1; }
"$OUTDIR/bin/python3" -V
echo "$SRC_ID" > "$MARKER"
echo "[python] Ready at $OUTDIR"