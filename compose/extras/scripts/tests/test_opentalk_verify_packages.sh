#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
VERIFY="$REPO_ROOT/extras/packaging/verify-packages.sh"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# Argument validation must not require any package tooling.
set +e
usage_err=$(bash "$VERIFY" 2>&1)
usage_rc=$?
set -e
[[ $usage_rc -eq 2 ]] || { echo "verify-packages.sh without arguments must exit 2 (got $usage_rc)." >&2; exit 1; }
grep -q 'Usage:' <<<"$usage_err"

# Metadata/function checks below need the package inspection tools.
for tool in dpkg-deb rpm rpm2cpio cpio; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "Package smoke-check metadata tests skipped (missing tool: $tool)." >&2
    exit 0
  fi
done

set +e
bash "$VERIFY" "$TMP_DIR/missing.deb" >/dev/null 2>&1
missing_rc=$?
set -e
[[ $missing_rc -eq 1 ]] || { echo "verify-packages.sh must exit 1 for missing package files (got $missing_rc)." >&2; exit 1; }

printf 'not-a-real-package' > "$TMP_DIR/not-a-package"
set +e
unsupported_err=$(bash "$VERIFY" "$TMP_DIR/not-a-package" 2>&1)
unsupported_rc=$?
set -e
[[ $unsupported_rc -eq 1 ]] || { echo "verify-packages.sh must exit 1 for unsupported package types (got $unsupported_rc)." >&2; exit 1; }
grep -q 'Unsupported package type' <<<"$unsupported_err"

# ---------------------------------------------------------------------------
# Functional checks: build minimal fake .deb / .rpm packages in $TMP_DIR and
# exercise verify_tree success and error paths.
# ---------------------------------------------------------------------------

run_verify() {
  local expect_rc=$1
  shift
  set +e
  bash "$VERIFY" "$@" >"$TMP_DIR/run.log" 2>&1
  local rc=$?
  set -e
  [[ $rc -eq $expect_rc ]] || { echo "verify-packages.sh expected rc $expect_rc, got $rc." >&2; cat "$TMP_DIR/run.log" >&2; exit 1; }
}

# Scaffolds opt/opentalk-compose/runtime inside $1. with_python/with_ansible
# control the embedded runtime binaries; leak adds a packaging-source file.
make_fake_tree() {
  local tree=$1
  local with_python=$2
  local with_ansible=$3
  local leak=$4
  mkdir -p "$tree/opt/opentalk-compose/runtime/python/bin"
  mkdir -p "$tree/opt/opentalk-compose/runtime/venv/bin"
  if [[ $with_python == 1 ]]; then
    ln -s "$(command -v python3)" "$tree/opt/opentalk-compose/runtime/python/bin/python3"
  fi
  if [[ $with_ansible == 1 ]]; then
    printf '#!/usr/bin/env bash\nexit 0\n' > "$tree/opt/opentalk-compose/runtime/venv/bin/ansible"
    chmod +x "$tree/opt/opentalk-compose/runtime/venv/bin/ansible"
  fi
  if [[ $leak == 1 ]]; then
    mkdir -p "$tree/opt/opentalk-compose/extras/packaging"
    : > "$tree/opt/opentalk-compose/extras/packaging/leaked.txt"
  fi
}

if command -v dpkg-deb >/dev/null 2>&1; then
  # Successful deb verification.
  make_fake_tree "$TMP_DIR/good-deb" 1 1 0
  mkdir -p "$TMP_DIR/good-deb/DEBIAN"
  cat > "$TMP_DIR/good-deb/DEBIAN/control" <<CTRL
Package: opentalk-verify-test
Version: 1.0
Section: utils
Priority: optional
Architecture: all
Maintainer: opentalk-test <test@example.org>
Description: fake package for verify-packages functional tests
CTRL
  dpkg-deb -b --root-owner-group "$TMP_DIR/good-deb" "$TMP_DIR/good.deb" >/dev/null
  run_verify 0 "$TMP_DIR/good.deb"
  grep -q 'Package smoke checks passed' "$TMP_DIR/run.log"

  # Embedded Python missing.
  make_fake_tree "$TMP_DIR/no-python-deb" 0 1 0
  mkdir -p "$TMP_DIR/no-python-deb/DEBIAN"
  cp "$TMP_DIR/good-deb/DEBIAN/control" "$TMP_DIR/no-python-deb/DEBIAN/control"
  dpkg-deb -b --root-owner-group "$TMP_DIR/no-python-deb" "$TMP_DIR/no-python.deb" >/dev/null
  run_verify 1 "$TMP_DIR/no-python.deb"
  grep -q 'Embedded Python missing' "$TMP_DIR/run.log"

  # Embedded Ansible missing.
  make_fake_tree "$TMP_DIR/no-ansible-deb" 1 0 0
  mkdir -p "$TMP_DIR/no-ansible-deb/DEBIAN"
  cp "$TMP_DIR/good-deb/DEBIAN/control" "$TMP_DIR/no-ansible-deb/DEBIAN/control"
  dpkg-deb -b --root-owner-group "$TMP_DIR/no-ansible-deb" "$TMP_DIR/no-ansible.deb" >/dev/null
  run_verify 1 "$TMP_DIR/no-ansible.deb"
  grep -q 'Embedded Ansible missing' "$TMP_DIR/run.log"

  # Packaging source file leaked into the package.
  make_fake_tree "$TMP_DIR/leak-deb" 1 1 1
  mkdir -p "$TMP_DIR/leak-deb/DEBIAN"
  cp "$TMP_DIR/good-deb/DEBIAN/control" "$TMP_DIR/leak-deb/DEBIAN/control"
  dpkg-deb -b --root-owner-group "$TMP_DIR/leak-deb" "$TMP_DIR/leak.deb" >/dev/null
  run_verify 1 "$TMP_DIR/leak.deb"
  grep -q 'Packaging source leaked' "$TMP_DIR/run.log"
else
  echo "dpkg-deb is unavailable; skipping deb functional tests." >&2
fi

# The rpm functional path needs rpmbuild to produce a valid archive. It is
# optional: when missing only the deb path is exercised.
if command -v rpmbuild >/dev/null 2>&1; then
  rpmbuild_dir="$TMP_DIR/rpmbuild"
  mkdir -p "$rpmbuild_dir"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}
  make_fake_tree "$TMP_DIR/good-rpm-root" 1 1 0
  cat > "$TMP_DIR/verify-test.spec" <<SPEC
Name: opentalk-verify-test
Version: 1.0
Release: 1
Summary: fake package for verify-packages functional tests
License: MIT
BuildArch: noarch

%description
Fake package for verify-packages functional tests.

%install
rm -rf %{buildroot}
install -d %{buildroot}/opt/opentalk-compose/runtime/python/bin
install -d %{buildroot}/opt/opentalk-compose/runtime/venv/bin
ln -s /usr/bin/python3 %{buildroot}/opt/opentalk-compose/runtime/python/bin/python3
printf '#!/usr/bin/env bash\\nexit 0\\n' > %{buildroot}/opt/opentalk-compose/runtime/venv/bin/ansible
chmod +x %{buildroot}/opt/opentalk-compose/runtime/venv/bin/ansible

%files
/opt/opentalk-compose/runtime/python/bin/python3
/opt/opentalk-compose/runtime/venv/bin/ansible
SPEC
  set +e
  rpmbuild -bb --define "_topdir $rpmbuild_dir" "$TMP_DIR/verify-test.spec" >/dev/null 2>&1
  rpmbuild_rc=$?
  set -e
  if [[ $rpmbuild_rc -eq 0 ]]; then
    rpm_pkg=$(find "$rpmbuild_dir/RPMS" -name '*.rpm' -print -quit)
    [[ -n $rpm_pkg ]] || { echo "rpmbuild produced no package." >&2; exit 1; }
    run_verify 0 "$rpm_pkg"
    grep -q 'Package smoke checks passed' "$TMP_DIR/run.log"
  else
    echo "rpmbuild could not build the fake package; skipping the rpm functional test." >&2
  fi
else
  echo "rpmbuild is unavailable; skipping the rpm functional test." >&2
fi

echo "Package smoke-check argument and metadata tests passed."