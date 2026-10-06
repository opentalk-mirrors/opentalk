#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
POSTRM="$REPO_ROOT/extras/packaging/pkg-postrm.sh"
POSTINST="$REPO_ROOT/extras/packaging/pkg-postinst.sh"
PRERM="$REPO_ROOT/extras/packaging/pkg-prerm.sh"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT
FAKE_TOOLS="$TMP_DIR/tools"
PREFIX="$TMP_DIR/prefix"
CFG_DIR="$TMP_DIR/etc"
EXAMPLES_DIR="$TMP_DIR/examples"

log_has() {
  grep -q "$1" "$TMP_DIR/log" || { echo "log missing: $1" >&2; cat "$TMP_DIR/log" >&2; exit 1; }
}

# --- shared helpers ---------------------------------------------------------

build_fake_tools() {
  # with_dpkg=1 -> dpkg-query present (IS_DEB=1), 0 -> absent (IS_DEB=0)
  local with_dpkg=$1
  rm -rf "$FAKE_TOOLS"
  mkdir -p "$FAKE_TOOLS"
  for tool in bash rm rmdir find env; do
    ln -s "$(command -v "$tool")" "$FAKE_TOOLS/$tool"
  done
  if [[ $with_dpkg -eq 1 ]]; then
    printf '#!/usr/bin/env bash\nexit 0\n' > "$FAKE_TOOLS/dpkg-query"
    chmod +x "$FAKE_TOOLS/dpkg-query"
  fi
}

make_rpm_stub() {
  printf '#!/usr/bin/env bash\nexit %s\n' "$1" > "$FAKE_TOOLS/rpm"
  chmod +x "$FAKE_TOOLS/rpm"
}

make_runtime_only() {
  local root=$1
  rm -rf "$root"
  mkdir -p "$root/runtime/venv"
}

make_valid_runtime() {
  local root=$1
  mkdir -p "$root/runtime/python/bin" "$root/runtime/venv/bin" "$root/runtime/build_assets"
  printf '#!/bin/sh\ntrue\n' > "$root/runtime/python/bin/python3"
  chmod +x "$root/runtime/python/bin/python3"
  for bin in venv/bin/python3 venv/bin/ansible venv/bin/ansible-playbook; do
    printf '#!/bin/sh\ntrue\n' > "$root/runtime/$bin"
    chmod +x "$root/runtime/$bin"
  done
  printf 'home = /builds/opentalk/opentalk-devops/opentalk-compose/build/python/bin\n' > "$root/runtime/venv/pyvenv.cfg"
  printf 'ansible==9.0.0\n' > "$root/runtime/build_assets/requirements-ansible.txt"
  sha256sum "$root/runtime/build_assets/requirements-ansible.txt" \
    | awk '{print $1}' > "$root/runtime/build_assets/requirements-ansible.sha256"
}

# --- pkg-postrm.sh ----------------------------------------------------------

run_postrm() {
  local expect_rc=$1
  local with_dpkg=$2
  shift 2
  set +e
  PATH="$FAKE_TOOLS" OT_PREFIX="$PREFIX" bash "$POSTRM" "$@" >"$TMP_DIR/log" 2>&1
  local rc=$?
  set -e
  [[ $rc -eq $expect_rc ]] || { echo "pkg-postrm.sh expected rc $expect_rc, got $rc." >&2; cat "$TMP_DIR/log" >&2; exit 1; }
}

# rpm numeric context: arg=0 performs a final purge; the leftover sweep runs
# BEFORE automation/bin are removed, so those dirs must be absent for the
# benign-sweep path.
make_runtime_only "$PREFIX"
touch "$PREFIX/README.md"
build_fake_tools 0
run_postrm 0 0 0
log_has 'Removed runtime dir'
log_has 'Removed leftover files in prefix (final purge).'
log_has 'Removed empty prefix'
[[ ! -e "$PREFIX" ]] || { echo "arg=0 must remove the whole prefix." >&2; exit 1; }

# arg=0 with automation/bin/unexpected entries: sweep is skipped, dirs purged.
make_runtime_only "$PREFIX"
mkdir -p "$PREFIX/automation" "$PREFIX/bin"
touch "$PREFIX/data.db"
run_postrm 0 0 0
log_has 'Leftover files not auto-purged (unexpected entries present).'
log_has 'Purged automation dir (arg=0)'
log_has 'Purged bin dir (arg=0)'
[[ -e "$PREFIX/data.db" ]] || { echo "unexpected entry must survive the sweep." >&2; exit 1; }

# arg=1 and arg>1 skip the runtime purge (upgrade context).
make_runtime_only "$PREFIX"
run_postrm 0 0 1
log_has 'Upgrade or non-final removal (arg=1) - skipping runtime purge.'
[[ -d "$PREFIX/runtime" ]] || { echo "arg=1 must keep the runtime dir." >&2; exit 1; }
run_postrm 0 0 2
log_has 'Upgrade or non-final removal (arg=2) - skipping runtime purge.'

# deb purge action performs a final cleanup (automation purged before sweep).
make_runtime_only "$PREFIX"
mkdir -p "$PREFIX/automation"
touch "$PREFIX/CHANGELOG.md"
build_fake_tools 1
run_postrm 0 1 purge
log_has 'Debian purge action - performing final cleanup.'
log_has 'Purged automation dir (purge)'
log_has 'Removed leftover files in prefix (purge).'
log_has 'Removed empty prefix'
[[ ! -e "$PREFIX" ]] || { echo "purge must remove the whole prefix." >&2; exit 1; }

# deb purge with unexpected entries keeps the prefix.
make_runtime_only "$PREFIX"
mkdir -p "$PREFIX/automation"
touch "$PREFIX/extra.bin"
run_postrm 0 1 purge
log_has 'Leftover files not auto-purged (unexpected entries present).'
[[ -e "$PREFIX/extra.bin" ]] || { echo "unexpected entry must survive purge." >&2; exit 1; }

# deb remove action cleans runtime but keeps unexpected user data.
make_runtime_only "$PREFIX"
mkdir -p "$PREFIX/automation" "$PREFIX/bin"
touch "$PREFIX/README.md"
run_postrm 0 1 remove
log_has 'Debian remove action - performing runtime cleanup'
log_has 'Removed automation dir (remove)'
log_has 'Removed bin dir (remove)'
log_has 'Removed benign leftover files (remove).'
[[ ! -e "$PREFIX" ]] || { echo "remove must remove an empty prefix." >&2; exit 1; }

# deb remove with unexpected entries keeps them.
make_runtime_only "$PREFIX"
mkdir -p "$PREFIX/automation" "$PREFIX/bin"
touch "$PREFIX/data.db"
run_postrm 0 1 remove
log_has 'Leftover files not auto-removed (unexpected entries present).'
[[ -e "$PREFIX/data.db" ]] || { echo "unexpected entry must survive remove." >&2; exit 1; }

# deb upgrade/failed-upgrade skip cleanup.
make_runtime_only "$PREFIX"
run_postrm 0 1 upgrade
log_has 'Debian upgrade action - skipping cleanup (upgrade context).'
[[ -d "$PREFIX/runtime" ]] || { echo "upgrade must keep the runtime dir." >&2; exit 1; }
run_postrm 0 1 failed-upgrade
log_has 'Debian failed-upgrade action - skipping cleanup (upgrade context).'

# empty arg defaults to no purge unless FORCE_POSTRM_PURGE=1.
make_runtime_only "$PREFIX"
run_postrm 0 1 ""
log_has 'Empty arg context - no purge (set FORCE_POSTRM_PURGE=1 to force).'
[[ -d "$PREFIX/runtime" ]] || { echo "empty arg must keep the runtime dir." >&2; exit 1; }
make_runtime_only "$PREFIX"
set +e
PATH="$FAKE_TOOLS" OT_PREFIX="$PREFIX" FORCE_POSTRM_PURGE=1 bash "$POSTRM" "" >"$TMP_DIR/log" 2>&1
local_rc=$?
set -e
[[ $local_rc -eq 0 ]] || { echo "forced empty-arg purge must exit 0." >&2; exit 1; }
log_has 'FORCE_POSTRM_PURGE=1 (empty arg) - performing cleanup.'
[[ ! -e "$PREFIX" ]] || { echo "forced purge must remove the prefix." >&2; exit 1; }

# non-deb heuristic: unknown arg purges when the package is absent (rpm rc 1).
make_runtime_only "$PREFIX"
build_fake_tools 0
make_rpm_stub 1
run_postrm 0 0 legacy-action
log_has 'Heuristic: package absent (non-deb env) - performing final purge.'
[[ ! -e "$PREFIX" ]] || { echo "heuristic purge must remove the prefix." >&2; exit 1; }

# non-deb heuristic: unknown arg skips when the package is still installed.
make_runtime_only "$PREFIX"
make_rpm_stub 0
run_postrm 0 0 legacy-action
log_has "Unrecognized arg 'legacy-action' - skipping purge."
[[ -d "$PREFIX/runtime" ]] || { echo "installed package must skip the purge." >&2; exit 1; }

# deb system with an unknown arg skips the purge.
make_runtime_only "$PREFIX"
build_fake_tools 1
run_postrm 0 1 legacy-action
log_has "Unrecognized Debian arg 'legacy-action' - skipping purge."
[[ -d "$PREFIX/runtime" ]] || { echo "unknown deb arg must keep the runtime dir." >&2; exit 1; }

# --- pkg-postinst.sh --------------------------------------------------------

run_postinst() {
  local expect_rc=$1
  shift
  set +e
  OT_PREFIX="$PREFIX" OT_CFG_DIR="$CFG_DIR" OT_EXAMPLES_DIR="$EXAMPLES_DIR" \
    bash "$POSTINST" "$@" >"$TMP_DIR/log" 2>&1
  local rc=$?
  set -e
  [[ $rc -eq $expect_rc ]] || { echo "pkg-postinst.sh expected rc $expect_rc, got $rc." >&2; cat "$TMP_DIR/log" >&2; exit 1; }
}

# Happy path: full runtime present, config dir complete, hash matches.
rm -rf "$CFG_DIR" "$EXAMPLES_DIR"
mkdir -p "$CFG_DIR/secrets" "$CFG_DIR/ssl" "$EXAMPLES_DIR"
printf 'settings: true\n' > "$CFG_DIR/settings.yml"
printf 'settings: true\n' > "$EXAMPLES_DIR/settings.yml.example"
make_valid_runtime "$PREFIX"
run_postinst 0
log_has 'Embedded Python runtime verified; no target-side package installation required.'
cmp -s "$CFG_DIR/settings.yml" <(printf 'settings: true\n') \
  || { echo "happy path must not overwrite settings.yml." >&2; exit 1; }
[[ $(stat -c '%a' "$CFG_DIR/secrets") == "700" ]] || { echo "secrets dir must be 0700." >&2; exit 1; }
[[ $(stat -c '%a' "$CFG_DIR/ssl") == "700" ]] || { echo "ssl dir must be 0700." >&2; exit 1; }
! grep -q 'otctl migrate status' "$TMP_DIR/log" \
  || { echo "a fresh install must not point at migrations." >&2; exit 1; }

# An upgrade of a deployed host points at 'otctl migrate status'.
printf 'X: y\n' > "$CFG_DIR/secrets/secrets.yml"
run_postinst 0
log_has "Run 'otctl migrate status' before 'otctl deploy'"
rm -f "$CFG_DIR/secrets/secrets.yml"

# Build-time shebangs in embedded venv console scripts are rebased at install time.
printf '#!/builds/opentalk/opentalk-devops/opentalk-compose/build/pkgroot/opt/opentalk-compose/runtime/venv/bin/python\nprint("stub")\n' \
  > "$PREFIX/runtime/venv/bin/ansible-playbook"
run_postinst 0
head -n 1 "$PREFIX/runtime/venv/bin/ansible-playbook" \
  | grep -q "^#!$PREFIX/runtime/venv/bin/python3$" \
  || { echo "postinst must rebase venv script shebangs." >&2; exit 1; }
grep -q "^home = $PREFIX/runtime/python/bin$" "$PREFIX/runtime/venv/pyvenv.cfg" \
  || { echo "postinst must rebase pyvenv.cfg home." >&2; exit 1; }

# Missing config dirs are created with the right permissions.
rm -rf "$CFG_DIR"
make_valid_runtime "$PREFIX"
run_postinst 0
log_has 'Creating missing config directory'
[[ -d "$CFG_DIR" ]] || { echo "config dir must be created." >&2; exit 1; }
[[ $(stat -c '%a' "$CFG_DIR") == "755" ]] || { echo "config dir must be 0755." >&2; exit 1; }
[[ $(stat -c '%a' "$CFG_DIR/secrets") == "700" ]] || { echo "secrets dir must be 0700." >&2; exit 1; }

# settings.yml is recovered from the example template when absent.
rm -rf "$CFG_DIR" "$EXAMPLES_DIR"
mkdir -p "$CFG_DIR/secrets" "$CFG_DIR/ssl" "$EXAMPLES_DIR"
printf 'example: content\n' > "$EXAMPLES_DIR/settings.yml.example"
make_valid_runtime "$PREFIX"
run_postinst 0
log_has 'Recovered missing settings.yml from example template (unexpected).'
grep -q 'example: content' "$CFG_DIR/settings.yml" \
  || { echo "settings.yml must be recovered from the example." >&2; exit 1; }

# settings.yml absent and no example template only warns.
rm -rf "$CFG_DIR" "$EXAMPLES_DIR"
mkdir -p "$CFG_DIR/secrets" "$CFG_DIR/ssl" "$EXAMPLES_DIR"
make_valid_runtime "$PREFIX"
run_postinst 0
log_has 'WARNING: settings.yml missing and no example template found.'
[[ ! -e "$CFG_DIR/settings.yml" ]] || { echo "no example must not create settings.yml." >&2; exit 1; }

# Missing portable python aborts with exit 1.
make_valid_runtime "$PREFIX"
rm -f "$PREFIX/runtime/python/bin/python3"
run_postinst 1
log_has 'Portable python missing'

# Missing venv python aborts with exit 1.
make_valid_runtime "$PREFIX"
rm -f "$PREFIX/runtime/venv/bin/python3"
run_postinst 1
log_has 'Embedded venv missing'

# Missing ansible aborts with exit 1.
make_valid_runtime "$PREFIX"
rm -f "$PREFIX/runtime/venv/bin/ansible"
run_postinst 1
log_has 'Embedded ansible missing'

# Missing requirements file aborts with exit 1.
make_valid_runtime "$PREFIX"
rm -f "$PREFIX/runtime/build_assets/requirements-ansible.txt"
run_postinst 1
log_has 'Requirements file missing'

# Missing requirements hash aborts with exit 1.
make_valid_runtime "$PREFIX"
rm -f "$PREFIX/runtime/build_assets/requirements-ansible.sha256"
run_postinst 1
log_has 'Requirements hash missing'

# Hash mismatch aborts.
make_valid_runtime "$PREFIX"
printf 'different-hash\n' > "$PREFIX/runtime/build_assets/requirements-ansible.sha256"
run_postinst 1
log_has 'Requirements hash mismatch.'

# --- pkg-prerm.sh -----------------------------------------------------------

run_prerm() {
  local expect_rc=$1
  shift
  set +e
  OT_PREFIX="$PREFIX" bash "$PRERM" "$@" >"$TMP_DIR/log" 2>&1
  local rc=$?
  set -e
  [[ $rc -eq $expect_rc ]] || { echo "pkg-prerm.sh expected rc $expect_rc, got $rc." >&2; cat "$TMP_DIR/log" >&2; exit 1; }
}

# remove/upgrade/deconfigure remove bytecode caches under the prefix.
rm -rf "$PREFIX"
mkdir -p "$PREFIX/app/__pycache__"
printf 'x' > "$PREFIX/app/__pycache__/mod.cpython-311.pyc"
printf 'x' > "$PREFIX/app/script.pyc"
for action in remove upgrade deconfigure; do
  run_prerm 0 "$action"
  [[ ! -e "$PREFIX/app/__pycache__" ]] || { echo "$action must remove __pycache__." >&2; exit 1; }
  [[ ! -e "$PREFIX/app/script.pyc" ]] || { echo "$action must remove .pyc files." >&2; exit 1; }
  mkdir -p "$PREFIX/app/__pycache__"
  printf 'x' > "$PREFIX/app/__pycache__/mod.pyc"
  printf 'x' > "$PREFIX/app/script.pyc"
done

# Unknown actions leave everything untouched.
run_prerm 0 unknown-action
[[ -e "$PREFIX/app/__pycache__/mod.pyc" ]] || { echo "unknown action must keep caches." >&2; exit 1; }

# Absent prefix is a no-op.
rm -rf "$PREFIX"
run_prerm 0 remove

echo "Packaging maintainer-script tests passed."