#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
MEASURE="$REPO_ROOT/extras/scripts/coverage/measure_shell_coverage.py"

[[ -f "$MEASURE" ]] || { echo "measure_shell_coverage.py not found." >&2; exit 1; }

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

check test_opentalk_otctl_smoke.sh 45
check test_opentalk_packaging_common_lib.sh 60
check test_opentalk_verify_packages.sh 55

if [[ $FAILURES -ne 0 ]]; then
  echo "$FAILURES coverage regression check(s) failed." >&2
  exit 1
fi

echo "Coverage regression checks passed."
