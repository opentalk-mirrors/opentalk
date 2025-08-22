#!/usr/bin/env bash
set -euo pipefail

# Cleanup for build artifacts.
# Default: removes embedded/, vendor/, collections_dist/, pkgroot/, debroot/, .lockenv, license report.
# Keeps: wheelhouse (cache), portable python runtime (build/python), package artifacts.
#
# Options:
#   --all             Remove everything under build/ (incl. wheelhouse, python, artifacts)
#   --rm-wheelhouse   Also remove build/wheelhouse
#   --rm-python       Also remove build/python
#   --rm-artifacts    Remove package artifacts (*.deb, *.rpm)
#   --dry-run         Show what would be removed
#   -y                No confirmation prompt
#   --verbose         Verbose output
#   -h|--help         Show this help

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BUILD="$ROOT/build"

ALL=false
DRY=false
YES=false
VERBOSE=false
KEEP_WHL=true
KEEP_ARTIFACTS=true
KEEP_PYTHON=true

while [[ $# -gt 0 ]]; do
  case "$1" in
    --all) ALL=true; KEEP_WHL=false; KEEP_ARTIFACTS=false; KEEP_PYTHON=false ;;
    --rm-wheelhouse) KEEP_WHL=false ;;
    --rm-python) KEEP_PYTHON=false ;;
    --rm-artifacts) KEEP_ARTIFACTS=false ;;
    --dry-run) DRY=true ;;
    -y) YES=true ;;
    --verbose) VERBOSE=true ;;
    -h|--help)
      grep -E '^#( |$)' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *) echo "Unknown option: $1" >&2; exit 2 ;;
  esac
  shift
done

[[ -d "$BUILD" ]] || { echo "Nothing to do: $BUILD does not exist."; exit 0; }
shopt -s nullglob

paths=(
  "$BUILD/embedded"
  "$BUILD/vendor"
  "$BUILD/collections_dist"
  "$BUILD/pkgroot"
  "$BUILD/debroot"
  "$BUILD/.lockenv"
  "$BUILD/THIRD_PARTY_LICENSES.txt"
)

# Artifacts
if ! $KEEP_ARTIFACTS; then
  for f in "$BUILD"/*.deb "$BUILD"/*.rpm; do
    paths+=("$f")
  done
fi

# Wheelhouse
$KEEP_WHL || paths+=("$BUILD/wheelhouse")
# Portable Python
$KEEP_PYTHON || paths+=("$BUILD/python")

to_delete=()
for p in "${paths[@]}"; do
  [[ -e "$p" ]] && to_delete+=("$p")
done

if [[ ${#to_delete[@]} -eq 0 ]]; then
  echo "No paths to delete (based on options)."
  exit 0
fi

echo "Will remove:"
for p in "${to_delete[@]}"; do
  echo "  - $p"
done

$DRY && { echo "(dry-run) Nothing removed."; exit 0; }

if ! $YES; then
  read -r -p "Proceed? [y/N] " ans
  [[ "${ans,,}" =~ ^y(es)?$ ]] || { echo "Aborted."; exit 1; }
fi

for p in "${to_delete[@]}"; do
  $VERBOSE && echo "Removing: $p"
  rm -rf --one-file-system -- "$p"
done

if $ALL; then
  $VERBOSE && echo "Removing remaining contents in $BUILD"
  find "$BUILD" -mindepth 1 -maxdepth 1 -exec rm -rf --one-file-system -- {} +
fi

echo "Cleanup completed."