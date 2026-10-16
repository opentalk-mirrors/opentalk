#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)

# 90-cleanup.sh derives ROOT from its own location. CLEANUP_ROOT overrides it
# so the original script runs against the temp dir instead of the real
# repository (same pattern as PACKAGING_ROOT in lib/common.sh). Nothing outside
# the temp dir is ever touched.
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT
CLEANUP="$REPO_ROOT/extras/packaging/90-cleanup.sh"
ROOT="$TMP_DIR"

failed=0
run_cleanup() {
  local expect_rc=$1
  shift
  set +e
  (cd "$ROOT" && CLEANUP_ROOT="$ROOT" bash "$CLEANUP" "$@") >"$TMP_DIR/run.log" 2>&1
  local rc=$?
  set -e
  [[ $rc -eq $expect_rc ]] || { echo "90-cleanup.sh expected rc $expect_rc, got $rc." >&2; cat "$TMP_DIR/run.log" >&2; exit 1; }
}

check_gone() {
  local path=$1
  [[ ! -e "$path" ]] || { echo "expected $path to be removed" >&2; failed=1; }
}

check_kept() {
  local path=$1
  [[ -e "$path" ]] || { echo "expected $path to be kept" >&2; failed=1; }
}

# --help exits 0 and prints usage.
run_cleanup 0 --help
grep -q '^Usage:' "$TMP_DIR/run.log" || { echo "--help must print usage." >&2; failed=1; }

# Unknown options are rejected.
run_cleanup 1 --bogus
grep -q 'Unknown option' "$TMP_DIR/run.log" || { echo "unknown option must be reported." >&2; failed=1; }

# --all -y removes build/, artifacts/, and loose packages without prompting.
mkdir -p "$ROOT/build/sub" "$ROOT/artifacts"
touch "$ROOT/build/sub/x.bin" "$ROOT/artifacts/pkg.deb" "$ROOT/a.deb" "$ROOT/b.rpm" "$ROOT/c.tar.gz"
run_cleanup 0 --all -y
check_gone "$ROOT/build"
check_gone "$ROOT/artifacts"
check_gone "$ROOT/a.deb"
check_gone "$ROOT/b.rpm"
check_gone "$ROOT/c.tar.gz"

# --artifacts -y removes only artifacts and loose packages, keeping build/.
# Regression: loose files must actually be deleted (glob expansion bug).
mkdir -p "$ROOT/build" "$ROOT/artifacts"
touch "$ROOT/artifacts/pkg.deb" "$ROOT/loose.deb" "$ROOT/loose.rpm" "$ROOT/loose.tar.xz"
run_cleanup 0 --artifacts -y
check_gone "$ROOT/artifacts"
check_gone "$ROOT/loose.deb"
check_gone "$ROOT/loose.rpm"
check_gone "$ROOT/loose.tar.xz"
check_kept "$ROOT/build"

# --build-only -y removes only build/, keeping artifacts/ and loose files.
mkdir -p "$ROOT/build" "$ROOT/artifacts"
touch "$ROOT/build/x.bin" "$ROOT/artifacts/pkg.deb" "$ROOT/loose.deb"
run_cleanup 0 --build-only -y
check_gone "$ROOT/build"
check_kept "$ROOT/artifacts"
check_kept "$ROOT/loose.deb"

# Default mode (no arguments) cleans build/ only.
mkdir -p "$ROOT/build" "$ROOT/artifacts"
touch "$ROOT/build/x.bin" "$ROOT/artifacts/pkg.deb"
run_cleanup 0 -y
check_gone "$ROOT/build"
check_kept "$ROOT/artifacts"

# A negative confirmation aborts the cleanup (exit 0, nothing removed).
mkdir -p "$ROOT/build"
touch "$ROOT/build/x.bin"
set +e
printf 'n\n' | (cd "$ROOT" && CLEANUP_ROOT="$ROOT" bash "$CLEANUP" --all) >"$TMP_DIR/run.log" 2>&1
rc=$?
set -e
[[ $rc -eq 0 ]] || { echo "negative confirmation must exit 0." >&2; exit 1; }
check_kept "$ROOT/build"

# A positive confirmation (yes) proceeds.
printf 'yes\n' | (cd "$ROOT" && CLEANUP_ROOT="$ROOT" bash "$CLEANUP" --all) >"$TMP_DIR/run.log" 2>&1
check_gone "$ROOT/build"

# Missing build/ directory is reported, not fatal.
run_cleanup 0 -y
grep -q 'No build directory found' "$TMP_DIR/run.log" || { echo "missing build/ must be reported." >&2; failed=1; }

# Running from inside the removed tree relocates to ROOT and succeeds.
mkdir -p "$ROOT/build/sub"
set +e
(cd "$ROOT/build/sub" && CLEANUP_ROOT="$ROOT" bash "$CLEANUP" --build-only -y) >"$TMP_DIR/run.log" 2>&1
rc=$?
set -e
[[ $rc -eq 0 ]] || { echo "relocation run must succeed." >&2; exit 1; }
check_gone "$ROOT/build"
grep -q 'relocated' "$TMP_DIR/run.log" || { echo "relocation must be announced." >&2; failed=1; }

# Relocation in the --all mode when the working directory is inside artifacts/.
mkdir -p "$ROOT/artifacts" "$ROOT/build"
touch "$ROOT/artifacts/pkg.deb"
set +e
(cd "$ROOT/artifacts" && CLEANUP_ROOT="$ROOT" bash "$CLEANUP" --all -y) >"$TMP_DIR/run.log" 2>&1
rc=$?
set -e
[[ $rc -eq 0 ]] || { echo "--all relocation run must succeed." >&2; exit 1; }
check_gone "$ROOT/artifacts"
grep -q 'relocated' "$TMP_DIR/run.log" || { echo "--all relocation must be announced." >&2; failed=1; }

# Relocation in the --artifacts mode when the working directory is inside artifacts/.
mkdir -p "$ROOT/artifacts" "$ROOT/build"
touch "$ROOT/artifacts/pkg.deb"
set +e
(cd "$ROOT/artifacts" && CLEANUP_ROOT="$ROOT" bash "$CLEANUP" --artifacts -y) >"$TMP_DIR/run.log" 2>&1
rc=$?
set -e
[[ $rc -eq 0 ]] || { echo "--artifacts relocation run must succeed." >&2; exit 1; }
check_gone "$ROOT/artifacts"
check_kept "$ROOT/build"
grep -q 'relocated' "$TMP_DIR/run.log" || { echo "--artifacts relocation must be announced." >&2; failed=1; }

# Relocation in the default mode when the working directory is inside build/.
mkdir -p "$ROOT/build"
touch "$ROOT/build/x.bin"
set +e
(cd "$ROOT/build" && CLEANUP_ROOT="$ROOT" bash "$CLEANUP" -y) >"$TMP_DIR/run.log" 2>&1
rc=$?
set -e
[[ $rc -eq 0 ]] || { echo "default relocation run must succeed." >&2; exit 1; }
check_gone "$ROOT/build"
grep -q 'relocated' "$TMP_DIR/run.log" || { echo "default relocation must be announced." >&2; failed=1; }

[[ $failed -eq 0 ]] || exit 1

echo "Packaging cleanup script tests passed."