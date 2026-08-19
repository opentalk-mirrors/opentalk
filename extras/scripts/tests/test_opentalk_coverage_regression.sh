#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
MEASURE="$REPO_ROOT/extras/scripts/coverage/measure_shell_coverage.py"
PY_MEASURE="$REPO_ROOT/extras/scripts/coverage/measure_python_coverage.py"
PY_GENERATOR="$REPO_ROOT/extras/scripts/generate-config-reference.py"

[[ -f "$MEASURE" ]] || { echo "measure_shell_coverage.py not found." >&2; exit 1; }
[[ -f "$PY_MEASURE" ]] || { echo "measure_python_coverage.py not found." >&2; exit 1; }

# Each entry is "test_script.sh:threshold": the fast test traces the target
# script and overall coverage must stay at or above the threshold percentage.
# Thresholds are baselines (growth over time, not hard first targets).
FAILURES=0
check() {
  local test_name=$1
  local threshold=$2
  local test_script="$REPO_ROOT/extras/scripts/tests/$test_name"
  local out
  local rc
  out=$("$MEASURE" --threshold "$threshold" --verbose "$test_script" 2>&1)
  rc=$?
  if [[ $rc -ne 0 ]]; then
    printf '%s\n' "$out"
    echo "FAIL: $test_name coverage below threshold ${threshold}%." >&2
    FAILURES=$((FAILURES + 1))
  else
    overall=$(printf '%s\n' "$out" | grep '^overall:' || true)
    echo "ok: $test_name -> $overall"
  fi
}

check test_opentalk_otctl_smoke.sh 55
check test_opentalk_packaging_common_lib.sh 60
check test_opentalk_verify_packages.sh 55

# Python coverage: trace the config-reference generator run on the real repo
# structure; the generated file goes to a temp dir.
if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required for Python coverage checks." >&2
  exit 1
fi
if ! python3 -c 'import yaml' >/dev/null 2>&1; then
  echo "PyYAML is required for Python coverage checks." >&2
  exit 1
fi
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT
py_out=$("$PY_MEASURE" --threshold 55 --verbose "$PY_GENERATOR" --output "$TMP_DIR/reference.md" 2>&1)
py_rc=$?
if [[ $py_rc -ne 0 ]]; then
  printf '%s\n' "$py_out"
  echo "FAIL: Python coverage below threshold 55%." >&2
  FAILURES=$((FAILURES + 1))
else
  overall=$(printf '%s\n' "$py_out" | grep '^overall:' || true)
  echo "ok: generate-config-reference.py -> $overall"
fi

if [[ $FAILURES -ne 0 ]]; then
  echo "$FAILURES coverage regression check(s) failed." >&2
  exit 1
fi

echo "Coverage regression checks passed."
