#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
CHECK="$REPO_ROOT/extras/packaging/check-packaging.sh"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# check-packaging.sh orchestrates real checks against the repo tree (bash -n,
# build-all.sh --help). All external build tools it probes are provided as
# deterministic stubs so the test runs without any host packaging tooling.
FAKE_TOOLS="$TMP_DIR/tools"
mkdir -p "$FAKE_TOOLS"
ln -s "$(command -v bash)" "$FAKE_TOOLS/bash"

# Minimal real system tools the orchestration itself relies on. They must NOT
# be taken from the host PATH at run time: run_check uses a strict PATH so that
# removing a stub (e.g. uv) reliably makes the dependency missing instead of
# falling back to a real host binary.
for tool in env dirname grep sed date; do
  ln -s "$(command -v "$tool")" "$FAKE_TOOLS/$tool"
done

make_stub() {
  printf '#!/usr/bin/env bash\nexit 0\n' > "$FAKE_TOOLS/$1"
  chmod +x "$FAKE_TOOLS/$1"
}

# Every binary install-build-deps.sh --check probes with `command -v` for
# BUILD_TARGET=both, plus shellcheck for the lint pass.
for tool in shellcheck curl git jq rsync tar unzip sha256sum python3 \
    pip3 ansible-galaxy uv just gem file objdump readelf rpm rpm2cpio \
    cpio dpkg-deb rpmbuild; do
  make_stub "$tool"
done

# common.sh computes SOURCE_DATE_EPOCH from git and falls back to `date`.
# A git stub that exits 1 exercises that fallback deterministically.
printf '#!/usr/bin/env bash\nexit 1\n' > "$FAKE_TOOLS/git"
chmod +x "$FAKE_TOOLS/git"

run_check() {
  local expect_rc=$1
  shift
  set +e
  PATH="$FAKE_TOOLS" bash "$CHECK" "$@" >"$TMP_DIR/run.log" 2>&1
  local rc=$?
  set -e
  [[ $rc -eq $expect_rc ]] || { echo "check-packaging.sh expected rc $expect_rc, got $rc." >&2; cat "$TMP_DIR/run.log" >&2; exit 1; }
}

# Happy path: every staged check passes.
run_check 0
grep -q 'Packaging checks passed' "$TMP_DIR/run.log"

# A failing shellcheck must abort the chain.
printf '#!/usr/bin/env bash\nexit 1\n' > "$FAKE_TOOLS/shellcheck"
chmod +x "$FAKE_TOOLS/shellcheck"
run_check 1
set +e
PATH="$FAKE_TOOLS" bash "$CHECK" >"$TMP_DIR/run.log" 2>&1
grep -q 'Packaging checks passed' "$TMP_DIR/run.log"
grep_rc=$?
set -e
[[ $grep_rc -ne 0 ]] || { echo "check-packaging.sh must not pass with a failing shellcheck." >&2; exit 1; }

# A missing build dependency must be reported by install-build-deps.sh.
make_stub shellcheck
rm "$FAKE_TOOLS/uv"
run_check 1
grep -q 'Missing' "$TMP_DIR/run.log"

echo "Packaging check orchestration tests passed."