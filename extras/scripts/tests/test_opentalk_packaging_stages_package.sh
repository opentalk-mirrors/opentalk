#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# Stage scripts 60-80 (package chain): pkg-stage, pkg-build-deb, pkg-build-rpm.
# Each reads build inputs from $PACKAGING_ROOT, which the test points at a temp
# "repo" it builds itself. Real filesystem tools (rsync, install, find, ...) are
# symlinked into a strict PATH; heavy package builders (fpm, dpkg-deb, rpmbuild,
# rpm, cpio) are provided as deterministic stubs so the tests never touch the
# host's package tooling and stay Docker-free.

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
STAGES="$REPO_ROOT/extras/packaging"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT
ROOT="$TMP_DIR/root"
FAKE_TOOLS="$TMP_DIR/tools"
export PACKAGING_ROOT="$ROOT"
export SOURCE_DATE_EPOCH=1700000000
export LC_ALL=C

mkdir -p "$ROOT/build" "$ROOT/extras/packaging"
mkdir -p "$FAKE_TOOLS"

log_has() {
  grep -q "$1" "$TMP_DIR/log" || { echo "log missing: $1" >&2; cat "$TMP_DIR/log" >&2; exit 1; }
}

make_stub() {
  printf '#!/usr/bin/env bash\nexit 0\n' > "$FAKE_TOOLS/$1"
  chmod +x "$FAKE_TOOLS/$1"
}

build_fake_path() {
  # Clean tool overrides AND stale build state from previous test cases.
  rm -rf "$FAKE_TOOLS"
  mkdir -p "$FAKE_TOOLS"
  rm -rf "$ROOT/build" "$ROOT/automation" "$ROOT/extras" "$ROOT/opt" "$ROOT/artifacts"
  mkdir -p "$ROOT/build" "$ROOT/extras/packaging"
  for tool in bash env dirname pwd echo cat rm mkdir cp mv chmod ln find \
      sort tail printf awk grep sed tr sha256sum install mktemp date head wc ls rsync which touch git gzip xz realpath; do
    if command -v "$tool" >/dev/null 2>&1; then
      ln -s "$(command -v "$tool")" "$FAKE_TOOLS/$tool"
    fi
  done
}

run_stage() {
  local script=$1
  local expect_rc=$2
  shift 2
  set +e
  PATH="$FAKE_TOOLS" bash "$STAGES/$script" "$@" >"$TMP_DIR/log" 2>&1
  local rc=$?
  set -e
  [[ $rc -eq $expect_rc ]] || { echo "$script expected rc $expect_rc, got $rc." >&2; cat "$TMP_DIR/log" >&2; exit 1; }
}

# Build a minimal but complete staging tree that scripts 70/80 accept as input.
# Mirrors what 60-pkg-stage.sh produces (without the extras/packaging leak that
# 70 rejects).
make_pkgroot() {
  local bin="$ROOT/build/pkgroot/opt/opentalk-compose/bin"
  local venv="$ROOT/build/pkgroot/opt/opentalk-compose/runtime/venv/bin"
  local py="$ROOT/build/pkgroot/opt/opentalk-compose/runtime/python/bin"
  mkdir -p "$bin" "$venv" "$py" \
    "$ROOT/build/pkgroot/opt/opentalk-compose/automation" \
    "$ROOT/build/pkgroot/opt/opentalk-compose/runtime/build_assets" \
    "$ROOT/build/pkgroot/etc/opentalk-compose/secrets" \
    "$ROOT/build/pkgroot/etc/opentalk-compose/ssl" \
    "$ROOT/build/pkgroot/usr/share/bash-completion/completions" \
    "$ROOT/build/pkgroot/usr/share/zsh/site-functions"
  local f
  for f in ot-ansible ot-ansible-playbook ot-ansible-galaxy pkg-postinst.sh; do
    printf '#!/usr/bin/env bash\nexit 0\n' > "$bin/$f"
    chmod +x "$bin/$f"
  done
  printf '#!/usr/bin/env bash\nexit 0\n' > "$bin/pkg-postrm.sh"
  chmod +x "$bin/pkg-postrm.sh"
  for f in python3 ansible; do
    printf '#!/usr/bin/env bash\nexit 0\n' > "$venv/$f"
    chmod +x "$venv/$f"
  done
  printf '#!/usr/bin/env bash\nexit 0\n' > "$py/python3"
  chmod +x "$py/python3"
  printf '[defaults]\nhost_key_checking = False\n' > "$ROOT/build/pkgroot/opt/opentalk-compose/automation/ansible.cfg"
  printf 'ansible-core==2.16.0\n' > "$ROOT/build/pkgroot/opt/opentalk-compose/runtime/build_assets/requirements-ansible.txt"
  printf 'package: opentalk-compose\n' > "$ROOT/build/pkgroot/etc/opentalk-compose/settings.yml"
  printf 'otctl\n' > "$ROOT/build/pkgroot/usr/share/bash-completion/completions/otctl"
  printf '_otctl\n' > "$ROOT/build/pkgroot/usr/share/zsh/site-functions/_otctl"
}

# dpkg-deb stub handling both --build (create empty package) and -R (emit a
# conffiles file the 70 sanity check greps for).
make_dpkg_deb_stub() {
  cat > "$FAKE_TOOLS/dpkg-deb" <<'DBEOF'
#!/usr/bin/env bash
if [[ "${1:-}" == "--build" ]]; then
  args=("$@"); : > "${args[-1]}"
  exit 0
fi
if [[ "${1:-}" == "-R" || "${1:-}" == "--contents" ]]; then
  args=("$@"); d="${args[-1]}"
  mkdir -p "$d/DEBIAN"
  printf '/etc/opentalk-compose/settings.yml\n' > "$d/DEBIAN/conffiles"
  mkdir -p "$d/opt/opentalk-compose/runtime/python/bin" "$d/opt/opentalk-compose/runtime/venv/bin"
  for p in opt/opentalk-compose/runtime/python/bin/python3 \
           opt/opentalk-compose/runtime/venv/bin/python3 \
           opt/opentalk-compose/runtime/venv/bin/ansible; do
    printf '#!/usr/bin/env bash\nexit 0\n' > "$d/$p"
    chmod +x "$d/$p"
  done
  exit 0
fi
  exit 0
DBEOF
  chmod +x "$FAKE_TOOLS/dpkg-deb"
}

# dpkg-deb stub that also advertises --root-owner-group on --help so 70's
# DPKG_DEB_BUILD_ARGS branch (line 230) is exercised.
make_dpkg_deb_stub_verbose() {
  cat > "$FAKE_TOOLS/dpkg-deb" <<'DBEOF'
#!/usr/bin/env bash
if [[ "${1:-}" == "--help" ]]; then
  echo "  --root-owner-group"
  exit 0
fi
if [[ "${1:-}" == "--build" ]]; then
  args=("$@"); : > "${args[-1]}"
  exit 0
fi
if [[ "${1:-}" == "-R" || "${1:-}" == "--contents" ]]; then
  args=("$@"); d="${args[-1]}"
  mkdir -p "$d/DEBIAN"
  printf '/etc/opentalk-compose/settings.yml\n' > "$d/DEBIAN/conffiles"
  mkdir -p "$d/opt/opentalk-compose/runtime/python/bin" "$d/opt/opentalk-compose/runtime/venv/bin"
  for p in opt/opentalk-compose/runtime/python/bin/python3 \
           opt/opentalk-compose/runtime/venv/bin/python3 \
           opt/opentalk-compose/runtime/venv/bin/ansible; do
    printf '#!/usr/bin/env bash\nexit 0\n' > "$d/$p"
    chmod +x "$d/$p"
  done
  exit 0
fi
exit 0
DBEOF
  chmod +x "$FAKE_TOOLS/dpkg-deb"
}

# fpm stub that materializes the output package at -p <file>.
make_fpm_stub() {
  cat > "$FAKE_TOOLS/fpm" <<'FMPEOF'
#!/usr/bin/env bash
echo "FPM_CALLED"
prev=""
for a in "$@"; do
  [[ "$prev" == "-p" ]] && { : > "$a"; exit 0; }
  prev="$a"
done
  exit 0
FMPEOF
  chmod +x "$FAKE_TOOLS/fpm"
}

# fpm stub that also advertises --deb-no-default-config-files and xzmt on
# --help so the conditional fpm-arg branches (70 line 182, 80 line 167) run.
make_fpm_stub_verbose() {
  cat > "$FAKE_TOOLS/fpm" <<'FMPEOF'
#!/usr/bin/env bash
echo "FPM_CALLED"
if [[ "${1:-}" == "--help" ]]; then
  echo "  --deb-no-default-config-files"
  echo "  --rpm-compression xzmt"
  exit 0
fi
prev=""
for a in "$@"; do
  [[ "$prev" == "-p" ]] && { : > "$a"; exit 0; }
  prev="$a"
done
exit 0
FMPEOF
  chmod +x "$FAKE_TOOLS/fpm"
}

# rpm/rpm2cpio/cpio stubs so 80's fpm-path validate_rpm passes deterministically.
make_rpm_stubs() {
  cat > "$FAKE_TOOLS/rpm" <<'RPMEOF'
#!/usr/bin/env bash
if [[ "${1:-}" == "-qip" ]]; then exit 0; fi
if [[ "${1:-}" == "-qlp" ]]; then
  cat <<'LIST'
/opt/opentalk-compose/runtime/python/bin/python3
/opt/opentalk-compose/runtime/venv/bin/python3
/opt/opentalk-compose/runtime/venv/bin/ansible
/opt/opentalk-compose/bin/pkg-postinst.sh
LIST
  exit 0
fi
exit 0
RPMEOF
  cat > "$FAKE_TOOLS/rpm2cpio" <<'CPIOEOF'
#!/usr/bin/env bash
# Produce a fake extracted tree in the current dir (validate_rpm cd's here).
mkdir -p "opt/opentalk-compose/runtime/python/bin" "opt/opentalk-compose/runtime/venv/bin" "opt/opentalk-compose/bin"
for p in opt/opentalk-compose/runtime/python/bin/python3 \
         opt/opentalk-compose/runtime/venv/bin/ansible \
         opt/opentalk-compose/bin/pkg-postinst.sh; do
  printf '#!/usr/bin/env bash\nexit 0\n' > "$p"
  chmod +x "$p"
done
exit 0
CPIOEOF
  chmod +x "$FAKE_TOOLS/rpm" "$FAKE_TOOLS/rpm2cpio"
  make_stub cpio
}

# rpmbuild stub: fabricate a binary RPM under <cwd>/RPMS/x86_64/ so the copy-back
# step (script 80 line 283) has something to move. Runs with CWD=$RPMROOT.
make_rpmbuild_stub() {
  cat > "$FAKE_TOOLS/rpmbuild" <<'RPEOF'
#!/usr/bin/env bash
# Extract version/release from --define "version X"/"release Y" for a stable name.
ver="0"; rel="1"
prev=""
for a in "$@"; do
  if [[ "$prev" == "--define" ]]; then
    case "$a" in
      version*) ver="${a#version }" ;;
      release*) rel="${a#release }" ;;
    esac
  fi
  prev="$a"
done
mkdir -p "RPMS/x86_64"
printf 'FAKE-RPM %s-%s' "$ver" "$rel" > "RPMS/x86_64/opentalk-compose-${ver}-${rel}.x86_64.rpm"
exit 0
RPEOF
  chmod +x "$FAKE_TOOLS/rpmbuild"
}

# Initialise $ROOT as a git repo with a single commit so `git archive HEAD`
# (script 80 line 215) and `rev-parse HEAD` (line 226) succeed for real.
init_git_repo() {
  rm -rf "$ROOT/.git"
  git init -q "$ROOT"
  git -C "$ROOT" config user.email "test@example.com"
  git -C "$ROOT" config user.name "Test"
  printf 'opentalk-compose\n' > "$ROOT/README"
  git -C "$ROOT" add README
  git -C "$ROOT" commit -q -m "init"
}

# A minimal spec with a Version: line that script 80's sync (lines 220-222) can
# rewrite; the rpmbuild stub ignores its contents.
make_spec() {
  local spec="$ROOT/extras/packaging/opentalk-compose.spec"
  mkdir -p "$(dirname "$spec")"
  cat > "$spec" <<'SPECEOF'
Name: opentalk-compose
Version: 0
Release: 1
Summary: Test package
License: EUPL-1.2
%description
Test package for coverage.
%files
SPECEOF
}

# ===========================================================================
# 60-pkg-stage.sh
# ===========================================================================

# Minimal repo that lets 60 complete successfully.
make_repo_60() {
  mkdir -p "$ROOT/automation"
  printf '[defaults]\nhost_key_checking = False\n' > "$ROOT/automation/ansible.cfg"
  mkdir -p "$ROOT/extras/packaging/wrappers" "$ROOT/extras/packaging/completions"
  local w
  for w in ot-ansible ot-ansible-playbook ot-ansible-galaxy; do
    printf '#!/usr/bin/env bash\necho %s\n' "$w" > "$ROOT/extras/packaging/wrappers/$w"
  done
  printf '# otctl bash completion\n' > "$ROOT/extras/packaging/completions/otctl.bash"
  printf '# _otctl zsh completion\n' > "$ROOT/extras/packaging/completions/_otctl"
}

# Missing automation/ dir aborts.
build_fake_path
run_stage 60-pkg-stage.sh 1
log_has 'automation/ directory missing.'

# Missing automation/ansible.cfg aborts.
build_fake_path
mkdir -p "$ROOT/automation"
run_stage 60-pkg-stage.sh 1
log_has 'automation/ansible.cfg missing.'

# Missing wrapper aborts.
build_fake_path
mkdir -p "$ROOT/automation" "$ROOT/extras/packaging/wrappers"
printf '[defaults]\n' > "$ROOT/automation/ansible.cfg"
printf '#!/usr/bin/env bash\nexit 0\n' > "$ROOT/extras/packaging/wrappers/ot-ansible"
run_stage 60-pkg-stage.sh 1
log_has 'Missing wrapper: ot-ansible'

# Missing shell completion aborts (install of a nonexistent file fails).
build_fake_path
make_repo_60
rm -f "$ROOT/extras/packaging/completions/_otctl"
run_stage 60-pkg-stage.sh 1
log_has 'cannot stat'

# Successful staging: full tree, conffile + examples + wrappers installed.
build_fake_path
make_repo_60
mkdir -p "$ROOT/extras/packaging/config-dist"
printf 'settings: sample\n' > "$ROOT/extras/packaging/config-dist/settings.yml-sample"
run_stage 60-pkg-stage.sh 0 "1.2.3"
log_has 'Completed staging'
log_has 'Installing wrappers'
[[ -x "$ROOT/build/pkgroot/opt/opentalk-compose/bin/ot-ansible" ]] || { echo "wrapper missing." >&2; exit 1; }
[[ -L "$ROOT/build/pkgroot/usr/bin/ot-ansible" ]] || { echo "usr symlink missing." >&2; exit 1; }
[[ -f "$ROOT/build/pkgroot/etc/opentalk-compose/settings.yml" ]] || { echo "conffile missing." >&2; exit 1; }
[[ -f "$ROOT/build/pkgroot/usr/share/opentalk-compose/examples/settings.yml.example" ]] || { echo "example missing." >&2; exit 1; }

# No vendor collections -> WARN but still succeeds.
build_fake_path
make_repo_60
run_stage 60-pkg-stage.sh 0
log_has 'WARN: No vendor collections found.'

# Full staging: every optional content branch (realm export, user samples,
# embedded vendor collections, otctl, realm-sanitizer, keycloak-theme-generator,
# portable Python, wheelhouse, requirements) is exercised.
make_repo_60_full() {
  make_repo_60
  # embedded vendor collections
  mkdir -p "$ROOT/build/embedded/vendor/collections/ansible_collections/dummy"
  : > "$ROOT/build/embedded/vendor/collections/ansible_collections/dummy/__init__.py"
  # portable Python runtime
  mkdir -p "$ROOT/build/python/bin"
  printf '#!/usr/bin/env bash\necho Python 3.11.5\n' > "$ROOT/build/python/bin/python3"
  chmod +x "$ROOT/build/python/bin/python3"
  printf 'https://example.org/python-3.11.5.tgz\n' > "$ROOT/build/python/.source-url"
  # wheelhouse + requirements
  mkdir -p "$ROOT/build/wheelhouse"
  : > "$ROOT/build/wheelhouse/dummy-0.1-py3-none-any.whl"
  printf 'ansible-core==2.16.0\n' > "$ROOT/extras/packaging/requirements-ansible.txt"
  # config-dist example files
  mkdir -p "$ROOT/extras/packaging/config-dist"
  printf '{}\n' > "$ROOT/extras/packaging/config-dist/export-opentalk-realm-and-users.json"
  printf 'id,name\n1,alice\n' > "$ROOT/extras/packaging/config-dist/users.csv-sample"
  printf '{"users": []}\n' > "$ROOT/extras/packaging/config-dist/users.json-sample"
  # primary control wrapper
  printf '#!/usr/bin/env bash\nexit 0\n' > "$ROOT/extras/packaging/wrappers/otctl"
  chmod +x "$ROOT/extras/packaging/wrappers/otctl"
  # keycloak-theme-generator tool
  mkdir -p "$ROOT/build/tools"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$ROOT/build/tools/keycloak-theme-generator"
  chmod +x "$ROOT/build/tools/keycloak-theme-generator"
  # realm-export sanitizer (read directly from $ROOT/extras/scripts by 60)
  mkdir -p "$ROOT/extras/scripts"
  cp "$REPO_ROOT/extras/scripts/opentalk_keycloak_realm_export_sanitize.sh" \
     "$ROOT/extras/scripts/opentalk_keycloak_realm_export_sanitize.sh"
}

build_fake_path
make_repo_60_full
run_stage 60-pkg-stage.sh 0 "1.2.3"
log_has 'Including example Keycloak realm export'
log_has 'Including example user import file: users.csv-sample'
log_has 'Including example user import file: users.json-sample'
log_has 'Copying vendor collections (embedded)'
log_has 'Installing otctl wrapper'
log_has 'Installing opentalk_keycloak_realm_export_sanitize maintenance script'
log_has 'Installing keycloak-theme-generator'
log_has 'Including portable Python runtime'
log_has 'Copying wheelhouse'
[[ -f "$ROOT/build/pkgroot/usr/share/opentalk-compose/examples/export-opentalk-realm-and-users.json" ]] \
  || { echo "realm example missing." >&2; exit 1; }
[[ -f "$ROOT/build/pkgroot/usr/share/opentalk-compose/examples/users.csv-sample" ]] \
  || { echo "users.csv example missing." >&2; exit 1; }
[[ -f "$ROOT/build/pkgroot/usr/share/opentalk-compose/examples/users.json-sample" ]] \
  || { echo "users.json example missing." >&2; exit 1; }
[[ -L "$ROOT/build/pkgroot/usr/bin/keycloak-theme-generator" ]] \
  || { echo "keycloak-theme symlink missing." >&2; exit 1; }
[[ -f "$ROOT/build/pkgroot/opt/opentalk-compose/runtime/python/.BUILD_INFO" ]] \
  || { echo "BUILD_INFO missing." >&2; exit 1; }
grep -q 'python_version=3.11.5' "$ROOT/build/pkgroot/opt/opentalk-compose/runtime/python/.BUILD_INFO" \
  || { echo "BUILD_INFO version wrong." >&2; exit 1; }
[[ -f "$ROOT/build/pkgroot/opt/opentalk-compose/runtime/build_assets/wheelhouse.list" ]] \
  || { echo "wheelhouse.list missing." >&2; exit 1; }
[[ -f "$ROOT/build/pkgroot/opt/opentalk-compose/runtime/build_assets/requirements-ansible.sha256" ]] \
  || { echo "requirements sha256 missing." >&2; exit 1; }

# Vendor collections from build/vendor fallback (line 129-132) when embedded is absent.
build_fake_path
make_repo_60
mkdir -p "$ROOT/build/vendor/collections/ansible_collections/dummy"
: > "$ROOT/build/vendor/collections/ansible_collections/dummy/__init__.py"
run_stage 60-pkg-stage.sh 0 "1.2.3"
log_has 'Copying vendor collections (fallback build/vendor)'
[[ -L "$ROOT/build/pkgroot/usr/bin/ot-ansible" ]] || { echo "usr symlink missing." >&2; exit 1; }

# ===========================================================================
# 70-pkg-build-deb.sh
# ===========================================================================

# No version -> usage error.
build_fake_path
run_stage 70-pkg-build-deb.sh 1
log_has 'Usage:'

# Invalid DEB_REVISION aborts.
build_fake_path
make_pkgroot
DEB_REVISION="bad revision" run_stage 70-pkg-build-deb.sh 2 1.2.3
log_has 'Invalid DEB_REVISION'

# Missing required runtime file aborts.
build_fake_path
make_pkgroot
rm -f "$ROOT/build/pkgroot/opt/opentalk-compose/runtime/venv/bin/ansible"
run_stage 70-pkg-build-deb.sh 1 1.2.3
log_has 'Missing expected file'

# Missing conffile aborts.
build_fake_path
make_pkgroot
rm -f "$ROOT/build/pkgroot/etc/opentalk-compose/settings.yml"
run_stage 70-pkg-build-deb.sh 1 1.2.3
log_has 'Missing conffile'

# extras/packaging leak aborts.
build_fake_path
make_pkgroot
mkdir -p "$ROOT/build/pkgroot/opt/opentalk-compose/extras/packaging"
run_stage 70-pkg-build-deb.sh 1 1.2.3
log_has 'extras/packaging accidentally included'

# Version normalization: prerelease + meta are rewritten to Debian form.
build_fake_path
make_pkgroot
make_dpkg_deb_stub
run_stage 70-pkg-build-deb.sh 0 v1.2.3-prerelease.4+git-c069
log_has 'Using deb revision: 1'

# Successful build via dpkg-deb fallback, inspected afterward.
build_fake_path
make_pkgroot
make_dpkg_deb_stub
run_stage 70-pkg-build-deb.sh 0 1.2.3
log_has 'fpm not found - using raw dpkg-deb fallback'
log_has 'Built: '
[[ -e "$ROOT/artifacts/opentalk-compose_1.2.3-1_amd64.deb" ]] || { echo "deb artifact missing." >&2; exit 1; }

# Successful build via fpm path when fpm is present.
build_fake_path
make_pkgroot
make_fpm_stub_verbose
make_dpkg_deb_stub
run_stage 70-pkg-build-deb.sh 0 1.2.3
log_has 'FPM_CALLED'
[[ -e "$ROOT/artifacts/opentalk-compose_1.2.3-1_amd64.deb" ]] || { echo "deb artifact missing." >&2; exit 1; }

# Explicit Debian revision extracted from a tag like 1.2.3-4 (line 40-59).
build_fake_path
make_pkgroot
make_dpkg_deb_stub_verbose
run_stage 70-pkg-build-deb.sh 0 1.2.3-4
log_has 'Using deb revision: 4 (from tag)'
[[ -e "$ROOT/artifacts/opentalk-compose_1.2.3-4_amd64.deb" ]] || { echo "deb artifact missing." >&2; exit 1; }

# Existing artifact + AUTO_REVISION=1 -> bump Debian revision (line 148-166).
build_fake_path
make_pkgroot
make_dpkg_deb_stub_verbose
mkdir -p "$ROOT/artifacts"
: > "$ROOT/artifacts/opentalk-compose_1.2.3-1_amd64.deb"
AUTO_REVISION=1 run_stage 70-pkg-build-deb.sh 0 1.2.3
log_has 'Bumped deb revision to 2'
[[ -e "$ROOT/artifacts/opentalk-compose_1.2.3-2_amd64.deb" ]] || { echo "bumped deb missing." >&2; exit 1; }

# Existing artifact + explicit revision from tag -> abort instead of bumping (line 151-154).
build_fake_path
make_pkgroot
make_dpkg_deb_stub_verbose
mkdir -p "$ROOT/artifacts"
: > "$ROOT/artifacts/opentalk-compose_1.2.3-4_amd64.deb"
run_stage 70-pkg-build-deb.sh 3 1.2.3-4
log_has 'Not auto-incrementing'

# Existing artifact without AUTO_REVISION -> abort (line 156-158).
build_fake_path
make_pkgroot
make_dpkg_deb_stub_verbose
mkdir -p "$ROOT/artifacts"
: > "$ROOT/artifacts/opentalk-compose_1.2.3-1_amd64.deb"
run_stage 70-pkg-build-deb.sh 3 1.2.3
log_has 'Artifact already exists'

# ===========================================================================
# 80-pkg-build-rpm.sh
# ===========================================================================

# No version -> usage error.
build_fake_path
run_stage 80-pkg-build-rpm.sh 2
log_has 'Usage:'

# Version normalization: prerelease + meta are rewritten to RPM form.
build_fake_path
make_pkgroot
make_fpm_stub_verbose
make_rpm_stubs
run_stage 80-pkg-build-rpm.sh 0 v1.2.3-prerelease.4+git-c069
log_has 'Using fpm path'
log_has 'Package validation passed.'

# RPM_RELEASE override wins over the default release 1 (fpm path, line 119-120).
build_fake_path
make_pkgroot
make_fpm_stub_verbose
make_rpm_stubs
RPM_RELEASE=2 run_stage 80-pkg-build-rpm.sh 0 1.2.3
log_has 'release=2'

# Explicit release from tag (1.2.3-4) in the fpm path (line 117-118).
build_fake_path
make_pkgroot
make_fpm_stub_verbose
make_rpm_stubs
run_stage 80-pkg-build-rpm.sh 0 1.2.3-4
log_has 'release=4'

# Existing artifact + AUTO_REVISION=1 -> bump release in the fpm path (line 126-153).
build_fake_path
make_pkgroot
make_fpm_stub_verbose
make_rpm_stubs
mkdir -p "$ROOT/artifacts"
: > "$ROOT/artifacts/opentalk-compose-1.2.3-1.x86_64.rpm"
AUTO_REVISION=1 run_stage 80-pkg-build-rpm.sh 0 1.2.3
log_has 'Bumped release to 2'
[[ -e "$ROOT/artifacts/opentalk-compose-1.2.3-2.x86_64.rpm" ]] || { echo "bumped rpm missing." >&2; exit 1; }

# Existing artifact + explicit release from tag -> abort in the fpm path (line 129-132).
build_fake_path
make_pkgroot
make_fpm_stub_verbose
make_rpm_stubs
mkdir -p "$ROOT/artifacts"
: > "$ROOT/artifacts/opentalk-compose-1.2.3-4.x86_64.rpm"
run_stage 80-pkg-build-rpm.sh 3 1.2.3-4
log_has 'Not auto-incrementing'

# Existing artifact without AUTO_REVISION -> abort in the fpm path (line 134-136).
build_fake_path
make_pkgroot
make_fpm_stub_verbose
make_rpm_stubs
mkdir -p "$ROOT/artifacts"
: > "$ROOT/artifacts/opentalk-compose-1.2.3-1.x86_64.rpm"
run_stage 80-pkg-build-rpm.sh 3 1.2.3
log_has 'Artifact already exists'

# rpmbuild fallback when fpm is absent -> needs rpmbuild, else aborts.
build_fake_path
make_pkgroot
rm -f "$FAKE_TOOLS/fpm"
run_stage 80-pkg-build-rpm.sh 3 1.2.3
log_has 'rpmbuild not found'

# rpmbuild happy path: git repo + spec + stubbed rpmbuild fabricates an RPM.
build_fake_path
init_git_repo
make_spec
make_rpmbuild_stub
run_stage 80-pkg-build-rpm.sh 0 1.2.3
log_has 'fpm not found; falling back to rpmbuild spec method'
log_has 'Building RPM for version 1.2.3 via rpmbuild'
log_has 'rpmbuild -ba'
log_has 'Done. Artifacts in'
shopt -s nullglob
rpms=("$ROOT/artifacts/"*.rpm)
[[ ${#rpms[@]} -gt 0 ]] || { echo "rpm artifact missing." >&2; exit 1; }

# Explicit release from tag (1.2.3-4) -> release taken from tag, no auto bump.
build_fake_path
init_git_repo
make_spec
make_rpmbuild_stub
run_stage 80-pkg-build-rpm.sh 0 1.2.3-4
log_has 'Building RPM for version 1.2.3 via rpmbuild'

# RPM_RELEASE override wins over the default release 1.
build_fake_path
init_git_repo
make_spec
make_rpmbuild_stub
RPM_RELEASE=7 run_stage 80-pkg-build-rpm.sh 0 1.2.3
log_has 'release=7'

# Artifact conflict without AUTO_REVISION -> abort (line 238/239/241).
build_fake_path
init_git_repo
make_spec
make_rpmbuild_stub
mkdir -p "$ROOT/artifacts"
: > "$ROOT/artifacts/opentalk-compose-1.2.3-1.x86_64.rpm"
run_stage 80-pkg-build-rpm.sh 3 1.2.3
log_has 'Artifact already exists'

# Artifact conflict with AUTO_REVISION=1 -> bump integer release (line 238/247).
build_fake_path
init_git_repo
make_spec
make_rpmbuild_stub
mkdir -p "$ROOT/artifacts"
: > "$ROOT/artifacts/opentalk-compose-1.2.3-1.x86_64.rpm"
AUTO_REVISION=1 run_stage 80-pkg-build-rpm.sh 0 1.2.3
log_has 'rpmbuild: bumped release to 2'

# Explicit release already present -> abort instead of incrementing (line 243).
build_fake_path
init_git_repo
make_spec
make_rpmbuild_stub
mkdir -p "$ROOT/artifacts"
: > "$ROOT/artifacts/opentalk-compose-1.2.3-4.x86_64.rpm"
AUTO_REVISION=1 run_stage 80-pkg-build-rpm.sh 3 1.2.3-4
log_has "Explicit release '4' already exists (rpmbuild path)"

# xz without -T threading support uses the non-threaded XZ_OPT (line 263).
build_fake_path
init_git_repo
make_spec
make_rpmbuild_stub
rm -f "$FAKE_TOOLS/xz"
cat > "$FAKE_TOOLS/xz" <<'XZEOF'
#!/usr/bin/env bash
echo "xz: options: --iterations"
exit 0
XZEOF
chmod +x "$FAKE_TOOLS/xz"
run_stage 80-pkg-build-rpm.sh 0 1.2.3
log_has 'xz has no threading support'

echo "Stage scripts 60-80 tests passed."
