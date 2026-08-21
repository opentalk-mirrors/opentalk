#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
BUILD_ALL="$REPO_ROOT/extras/packaging/build-all.sh"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# A fake PACKAGING_ROOT in which every stage script is replaced by a tiny
# logging stub. build-all.sh is the real file: it resolves PACKAGING_ROOT
# via lib/common.sh (env override) and must not touch the real repo tree.
FAKE_ROOT="$TMP_DIR/fakeroot"
FAKE_PKG="$FAKE_ROOT/extras/packaging"
FAKE_TOOLS="$TMP_DIR/tools"
ONLY_BASH_DIR="$TMP_DIR/only-bash"
FAKE_LOG="$TMP_DIR/stages.log"
RUN_LOG="$TMP_DIR/run.log"
mkdir -p "$FAKE_PKG" "$FAKE_TOOLS" "$ONLY_BASH_DIR" "$FAKE_ROOT/automation/collections"
ln -s "$(command -v bash)" "$ONLY_BASH_DIR/bash"
ln -s "$(command -v dirname)" "$ONLY_BASH_DIR/dirname"
ln -s "$(command -v awk)" "$ONLY_BASH_DIR/awk"

export SOURCE_DATE_EPOCH=1

# build-all.sh reads the default Python version from .versions.yml via
# versions_get, so the fake root needs one.
printf 'python_version: 3.14.6\n' > "$FAKE_ROOT/.versions.yml"

make_stage() {
  local name=$1
  local base=${name%.sh}
  local body=${2:-}
  body=${body//\$FAKE_ROOT/$FAKE_ROOT}
  cat > "$FAKE_PKG/$name" <<STUB
#!/usr/bin/env bash
printf '${base}|%s|DEB=%s|RPM=%s|TARGET=%s\n' "\$*" "\${DEB_REVISION:-}" "\${RPM_RELEASE:-}" "\${BUILD_TARGET:-}" >> "$FAKE_LOG"
$body
STUB
  chmod +x "$FAKE_PKG/$name"
}

# shellcheck disable=SC2016 # Stub bodies intentionally defer $FAKE_ROOT expansion.
make_stage 10-ansible-lock.sh
make_stage 11-uv-check.sh
make_stage 12-python-runtime-fetch.sh
make_stage 13-tools-fetch.sh
make_stage 14-runtime-compatibility-check.sh
make_stage 20-ansible-wheelhouse.sh
make_stage 30-ansible-collections-download.sh
# shellcheck disable=SC2016 # Stub body defers $FAKE_ROOT expansion.
make_stage 31-ansible-collections-vendor.sh 'if [[ "${NO_VENDOR:-}" != "1" ]]; then mkdir -p "$FAKE_ROOT/build/vendor/collections/dummy/ansible_collections/x"; fi'
make_stage 40-ansible-venv-build.sh
make_stage 50-embed-assemble.sh
# shellcheck disable=SC2016 # Stub body defers $FAKE_ROOT expansion.
make_stage 60-pkg-stage.sh 'if [[ "${NO_PKGROOT_VENDOR:-}" != "1" ]]; then mkdir -p "$FAKE_ROOT/build/pkgroot/opt/opentalk-compose/runtime/vendor/collections/ansible_collections/x"; fi'
make_stage 70-pkg-build-deb.sh
# shellcheck disable=SC2016 # Stub body uses the $FAIL_RPM env at runtime.
make_stage 80-pkg-build-rpm.sh 'if [[ "${FAIL_RPM:-}" == "1" ]]; then exit 1; fi'
make_stage 90-cleanup.sh
make_stage install-build-deps.sh

# Deterministically provide `ansible-galaxy` (build-all only probes existence).
printf '#!/usr/bin/env bash\nexit 0\n' > "$FAKE_TOOLS/ansible-galaxy"
chmod +x "$FAKE_TOOLS/ansible-galaxy"

run_build() {
  local expect_rc=$1
  shift
  : > "$FAKE_LOG"
  : > "$RUN_LOG"
  set +e
  PACKAGING_ROOT="$FAKE_ROOT" PATH="$FAKE_TOOLS:$PATH" \
    bash "$BUILD_ALL" "$@" >"$RUN_LOG" 2>&1
  local rc=$?
  set -e
  if [[ $rc -ne $expect_rc ]]; then
    echo "build-all.sh expected rc $expect_rc, got $rc." >&2
    cat "$RUN_LOG" >&2
    exit 1
  fi
}

takes() {
  grep -Fq -- "$1" "$FAKE_LOG" || {
    echo "missing stage/arg in run: $1" >&2
    cat "$FAKE_LOG" >&2
    exit 1
  }
}

skip() {
  if grep -Fq -- "$1" "$FAKE_LOG"; then
    echo "unexpected stage in run: $1" >&2
    cat "$FAKE_LOG" >&2
    exit 1
  fi
  return 0
}

# ---------------------------------------------------------------------------
# Argument parsing / help
# ---------------------------------------------------------------------------

set +e
help_out=$(bash "$BUILD_ALL" --help 2>&1)
help_rc=$?
set -e
[[ $help_rc -eq 0 ]] || { echo "--help must exit 0." >&2; exit 1; }
grep -q -- '--auto-revision' <<<"$help_out"

set +e
unknown_out=$(bash "$BUILD_ALL" --bogus-flag 2>&1 < /dev/null)
unknown_rc=$?
set -e
[[ $unknown_rc -eq 2 ]] || { echo "unknown option must exit 2." >&2; exit 1; }
grep -q 'Unknown option' <<<"$unknown_out"

# --version without a value fails under set -euo pipefail.
set +e
bash "$BUILD_ALL" --version >/dev/null 2>&1
noarg_rc=$?
set -e
[[ $noarg_rc -ne 0 ]] || { echo "--version without a value must fail." >&2; exit 1; }

# ---------------------------------------------------------------------------
# Default online run: full chain, deb+rpm, vendor echoed, no clean.
# ---------------------------------------------------------------------------

printf 'collections: []\n' > "$FAKE_ROOT/automation/collections/requirements.yml"
run_build 0 --no-package
takes '11-uv-check|'
takes '12-python-runtime-fetch|--version 3.14.6'
takes '13-tools-fetch|'
takes '10-ansible-lock|'
takes '20-ansible-wheelhouse|'
takes '30-ansible-collections-download|'"$FAKE_ROOT/automation/collections/requirements.yml"
takes '31-ansible-collections-vendor|'
takes '50-embed-assemble|'
takes '60-pkg-stage|1.0.0'
takes '40-ansible-venv-build|'
takes '14-runtime-compatibility-check|'
skip '90-cleanup'
skip '70-pkg-build-deb'
skip '80-pkg-build-rpm'
grep -Fq '[result] Packaging skipped (--no-package)' "$RUN_LOG"

# --clean triggers the cleanup stage first; custom VERSION flows through.
run_build 0 --clean --no-package --version 2.1.0
takes '90-cleanup|'
takes '60-pkg-stage|2.1.0'

# CLEAN=1 environment variable is equivalent to --clean.
run_build 0 --no-package
skip '90-cleanup'
export CLEAN=1
run_build 0 --no-package
unset CLEAN
takes '90-cleanup|'

# ---------------------------------------------------------------------------
# Offline mode: skips the fetch/lock/wheelhouse stages entirely.
# ---------------------------------------------------------------------------

run_build 0 --offline --no-package
skip '12-python-runtime-fetch'
skip '13-tools-fetch'
skip '10-ansible-lock'
skip '20-ansible-wheelhouse'
takes '11-uv-check|'
takes '50-embed-assemble|'
takes '60-pkg-stage|1.0.0'
takes '40-ansible-venv-build|'
takes '14-runtime-compatibility-check|'

# --no-python keeps the offline-independent stages but skips python fetch.
run_build 0 --no-python --no-package
skip '12-python-runtime-fetch'
takes '13-tools-fetch|'
takes '10-ansible-lock|'
takes '20-ansible-wheelhouse|'

# --skip-collections skips download/vendor but keeps the rest.
run_build 0 --skip-collections --no-package
skip '30-ansible-collections-download'
skip '31-ansible-collections-vendor'
takes '50-embed-assemble|'
takes '60-pkg-stage|'

# Missing requirements.yml: collections are skipped with a NOTE.
rm -f "$FAKE_ROOT/automation/collections/requirements.yml"
run_build 0 --no-package
skip '30-ansible-collections-download'
skip '31-ansible-collections-vendor'
grep -q 'requirements file missing.*skipping' "$RUN_LOG"

# ---------------------------------------------------------------------------
# Release semantics: --release is applied to both, explicit values win.
# ---------------------------------------------------------------------------

printf 'collections: []\n' > "$FAKE_ROOT/automation/collections/requirements.yml"
run_build 0 --release 3
takes '70-pkg-build-deb|1.0.0|DEB=3'
takes '80-pkg-build-rpm|1.0.0|DEB=3|RPM=3'

run_build 0 --release 3 --deb-revision 2
takes '70-pkg-build-deb|1.0.0|DEB=2'
takes '80-pkg-build-rpm|1.0.0|DEB=2|RPM=3'

# Package toggles: --deb only, --rpm only, then both explicitly.
run_build 0 --deb --release 1
takes '70-pkg-build-deb|1.0.0|DEB=1'
skip '80-pkg-build-rpm'

run_build 0 --rpm --release 2
takes '80-pkg-build-rpm|1.0.0|DEB=|RPM=2'
skip '70-pkg-build-deb'

run_build 0 --deb --rpm --release 1
takes '70-pkg-build-deb|1.0.0|DEB=1'
takes '80-pkg-build-rpm|1.0.0|DEB=1|RPM=1'

# ---------------------------------------------------------------------------
# CHECK_BUILD_DEPS: install-build-deps.sh --check runs with a BUILD_TARGET.
# ---------------------------------------------------------------------------

run_build 0 --deb --release 1
skip 'install-build-deps'
export CHECK_BUILD_DEPS=1
run_build 0 --deb --release 1
unset CHECK_BUILD_DEPS
takes 'install-build-deps|--check|DEB=|RPM=|TARGET=deb'

export CHECK_BUILD_DEPS=1
run_build 0 --rpm --release 1
unset CHECK_BUILD_DEPS
takes 'install-build-deps|--check|DEB=|RPM=|TARGET=rpm'

export CHECK_BUILD_DEPS=1
run_build 0 --release 1
unset CHECK_BUILD_DEPS
takes 'install-build-deps|--check|DEB=|RPM=|TARGET=both'

# --no-package skips the check as well.
export CHECK_BUILD_DEPS=1
run_build 0 --no-package --release 1
unset CHECK_BUILD_DEPS
skip 'install-build-deps'

# ---------------------------------------------------------------------------
# Remaining option flags and summary echo branches.
# ---------------------------------------------------------------------------

# --rpm-release and the rpm echo branch.
run_build 0 --rpm --rpm-release 5
takes '80-pkg-build-rpm|'
grep -q '\[rpm\] Explicit rpm release: 5' "$RUN_LOG"

# --auto-revision / --force / --verbose are accepted and often combined.
run_build 0 --deb --auto-revision --force --verbose
takes '70-pkg-build-deb|'

# --deb alone prints the single-target package summary.
run_build 0 --deb
grep -q 'Package targets: deb$' "$RUN_LOG" || { cat "$RUN_LOG" >&2; exit 1; }

# ---------------------------------------------------------------------------
# Error paths
# ---------------------------------------------------------------------------

# Vendor layout missing after the build is an error.
rm -rf "$FAKE_ROOT/build"
export NO_VENDOR=1
run_build 1 --no-package
unset NO_VENDOR
grep -Fq '[collections] ERROR: vendor layout missing after build' "$RUN_LOG"

# ansible-galaxy not in PATH with collections requested is an error.
set +e
PACKAGING_ROOT="$FAKE_ROOT" PATH="$ONLY_BASH_DIR" bash "$BUILD_ALL" --no-package >"$RUN_LOG" 2>&1
galaxy_rc=$?
set -e
[[ $galaxy_rc -eq 1 ]] || { echo "missing ansible-galaxy must exit 1 (got $galaxy_rc)." >&2; exit 1; }
grep -q 'ansible-galaxy not in PATH' "$RUN_LOG"

# Failed RPM build is an error (packaging must be enabled for this path).
export FAIL_RPM=1
run_build 1 --release 1
unset FAIL_RPM
grep -q "RPM build failed" "$RUN_LOG"

# Vendor exists pre-staging but was not copied into pkgroot: error.
rm -rf "$FAKE_ROOT/build"
export NO_PKGROOT_VENDOR=1
run_build 1 --no-package
unset NO_PKGROOT_VENDOR
grep -q 'vendor missing in pkgroot' "$RUN_LOG"

# No vendor collections built at all (skip-collections): informational notice.
rm -rf "$FAKE_ROOT/build"
run_build 0 --skip-collections --no-package
grep -Fq '[stage] INFO: No vendor collections built (none expected)' "$RUN_LOG"

echo "build-all orchestration tests passed."