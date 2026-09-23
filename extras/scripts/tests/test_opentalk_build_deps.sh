#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
DEPS="$REPO_ROOT/extras/packaging/install-build-deps.sh"

# install-build-deps.sh probes build tools with `command -v` and, in install
# mode, dispatches to a package manager. All probed binaries are provided as
# deterministic stubs in a strict PATH (no host fallback), so the script runs
# without any real packaging tooling or privilege escalation.
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT
FAKE_TOOLS="$TMP_DIR/tools"

make_stub() {
  printf '#!/usr/bin/env bash\nexit 0\n' > "$FAKE_TOOLS/$1"
  chmod +x "$FAKE_TOOLS/$1"
}

base_tools() {
  rm -rf "$FAKE_TOOLS"
  mkdir -p "$FAKE_TOOLS"
  ln -s "$(command -v bash)" "$FAKE_TOOLS/bash"
  # Real system tools the script or the stubs rely on (usage() uses cat).
  for tool in env dirname cat; do
    ln -s "$(command -v "$tool")" "$FAKE_TOOLS/$tool"
  done
  # Every binary install-build-deps.sh --check probes for BUILD_TARGET=both.
  for tool in curl git jq rsync tar unzip sha256sum python3 pip3 ansible-galaxy \
      uv just gem file objdump readelf rpm rpm2cpio cpio dpkg-deb rpmbuild; do
    make_stub "$tool"
  done
}

run_deps() {
  local expect_rc=$1
  shift
  set +e
  PATH="$FAKE_TOOLS" BUILD_TARGET="${BUILD_TARGET:-}" bash "$DEPS" "$@" >"$TMP_DIR/run.log" 2>&1
  local rc=$?
  set -e
  [[ $rc -eq $expect_rc ]] || { echo "install-build-deps.sh expected rc $expect_rc, got $rc." >&2; cat "$TMP_DIR/run.log" >&2; exit 1; }
}

# --help prints usage and exits 0.
base_tools
run_deps 0 --help
grep -q '^Usage:' "$TMP_DIR/run.log" || { echo "--help must print usage." >&2; exit 1; }

# Unknown options are rejected with usage on stderr.
run_deps 2 --bogus
grep -q 'Unknown option' "$TMP_DIR/run.log" || { echo "unknown option must be reported." >&2; exit 1; }

# Happy path: --check with all tools present succeeds.
run_deps 0 --check
grep -q 'All required build dependencies are available' "$TMP_DIR/run.log" \
  || { echo "--check happy path must report all dependencies available." >&2; exit 1; }

# A missing tool is reported and exits 1.
base_tools
rm "$FAKE_TOOLS/uv"
run_deps 1 --check
grep -q 'Missing:' "$TMP_DIR/run.log" || { echo "missing tool must be reported." >&2; exit 1; }

# BUILD_TARGET=deb skips the rpm-specific probes.
base_tools
rm "$FAKE_TOOLS/rpm" "$FAKE_TOOLS/rpm2cpio" "$FAKE_TOOLS/cpio" "$FAKE_TOOLS/rpmbuild"
BUILD_TARGET=deb run_deps 0 --check
grep -q 'All required build dependencies are available' "$TMP_DIR/run.log" \
  || { echo "BUILD_TARGET=deb must succeed without rpm tools." >&2; exit 1; }

# BUILD_TARGET=rpm skips the deb-specific probe.
base_tools
rm "$FAKE_TOOLS/dpkg-deb"
BUILD_TARGET=rpm run_deps 0 --check
grep -q 'All required build dependencies are available' "$TMP_DIR/run.log" \
  || { echo "BUILD_TARGET=rpm must succeed without dpkg-deb." >&2; exit 1; }

# BUILD_TARGET=deb requires dpkg-deb.
base_tools
rm "$FAKE_TOOLS/dpkg-deb"
BUILD_TARGET=deb run_deps 1 --check
grep -q 'dpkg-deb' "$TMP_DIR/run.log" || { echo "missing dpkg-deb must be reported." >&2; exit 1; }

# BUILD_TARGET=rpm requires fpm or rpmbuild.
base_tools
rm "$FAKE_TOOLS/rpmbuild"
BUILD_TARGET=rpm run_deps 1 --check
grep -q 'fpm or rpmbuild' "$TMP_DIR/run.log" || { echo "missing rpmbuild must be reported." >&2; exit 1; }

if [[ $EUID -ne 0 ]]; then
  # --install without root and without sudo is rejected.
  base_tools
  run_deps 1 --install
  grep -q 'requires root privileges or sudo' "$TMP_DIR/run.log" \
    || { echo "--install without sudo must be rejected." >&2; exit 1; }
else
  echo "skipping --install-without-sudo case (running as root)."
fi

# --install via sudo dispatches to apt-get.
base_tools
make_stub sudo
make_stub apt-get
run_deps 0 --install
grep -q 'All required build dependencies are available' "$TMP_DIR/run.log" \
  || { echo "apt-get install path must complete." >&2; exit 1; }

# --install dispatches to dnf when apt-get is missing.
base_tools
make_stub sudo
make_stub dnf
run_deps 0 --install
grep -q 'All required build dependencies are available' "$TMP_DIR/run.log" \
  || { echo "dnf install path must complete." >&2; exit 1; }

# --install dispatches to zypper when apt-get and dnf are missing.
base_tools
make_stub sudo
make_stub zypper
run_deps 0 --install
grep -q 'All required build dependencies are available' "$TMP_DIR/run.log" \
  || { echo "zypper install path must complete." >&2; exit 1; }

# --install dispatches to pacman when all other managers are missing.
base_tools
make_stub sudo
make_stub pacman
run_deps 0 --install
grep -q 'All required build dependencies are available' "$TMP_DIR/run.log" \
  || { echo "pacman install path must complete." >&2; exit 1; }

# --install on an unsupported distribution fails.
base_tools
make_stub sudo
run_deps 1 --install
grep -q 'unsupported distribution' "$TMP_DIR/run.log" \
  || { echo "unsupported distribution must be reported." >&2; exit 1; }

echo "Build dependency check/install tests passed."