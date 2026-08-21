#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# shellcheck disable=SC2015  # Intentional use of logical chaining for optional echo of combined targets.
set -euo pipefail

# Orchestrates embedded Ansible build and packages (Debian + RPM).
# Steps: clean? -> uv/python -> lock -> wheelhouse -> collections dl/vendor -> embed -> stage -> embedded venv -> deb+rpm
#
# Options:
#   --version X
#   --rpm-release N   (override RPM_RELEASE; packaging iteration for rpm)
#   --deb-revision N  (override DEB_REVISION; packaging revision for deb)
#   --release N       (generic iteration applied to both rpm+deb unless specific ones provided)
#   --offline
#   --skip-collections
#   --no-package
#   --clean
#   --no-python
#   --force
#   --auto-revision
#   --verbose
#   -h|--help
#
# Env:
#   COL_REQ (default automation/collections/requirements.yml)
#   FORCE_REFRESH=1
#   CLEAN=1 (alternative to --clean)
#   CHECK_BUILD_DEPS=1 (check host build dependencies before downloads)

# shellcheck disable=SC1091
. "$(dirname "${BASH_SOURCE[0]}")/lib/common.sh"
# shellcheck disable=SC1091
. "$(dirname "${BASH_SOURCE[0]}")/lib/versions.sh"
ROOT="$PACKAGING_ROOT"
PKGDIR="$ROOT/extras/packaging"

# Safety: ensure we are operating from repo root even if user invoked script from a directory that gets deleted (e.g. artifacts/ just cleaned).
if [[ "$PWD" != "$ROOT" ]]; then
  cd "$ROOT" || { echo "[fatal] Cannot change directory to repo root $ROOT" >&2; exit 1; }
fi

VERSION="1.0.0"
DO_DEB=true
DO_RPM=true
OFFLINE=false
SKIP_COLLECTIONS=false
NO_PACKAGE=false
DO_CLEAN=false
VERBOSE=false
NO_PYTHON_FETCH=false
AUTO_REVISION=false
PACKAGE_FLAG_SEEN=false
FORCE_REFRESH=${FORCE_REFRESH:-0}

COL_REQ="${COL_REQ:-$ROOT/automation/collections/requirements.yml}"

GENERIC_RELEASE=""
while [[ $# -gt 0 ]]; do
  case "$1" in
  --version) VERSION="${2:?missing version}"; shift ;;
  --rpm-release) RPM_RELEASE="${2:?missing release}"; shift ;;
  --deb-revision) DEB_REVISION="${2:?missing revision}"; shift ;;
  --release) GENERIC_RELEASE="${2:?missing generic release}"; shift ;;
    --offline) OFFLINE=true ;;
    --skip-collections) SKIP_COLLECTIONS=true ;;
    --no-package) NO_PACKAGE=true ;;
    --rpm)
      if ! $PACKAGE_FLAG_SEEN; then DO_DEB=false; PACKAGE_FLAG_SEEN=true; fi
      DO_RPM=true
      ;;
    --deb)
      if ! $PACKAGE_FLAG_SEEN; then DO_RPM=false; PACKAGE_FLAG_SEEN=true; fi
      DO_DEB=true
      ;;
    --auto-revision) AUTO_REVISION=true ;;
    --clean) DO_CLEAN=true ;;
    --no-python) NO_PYTHON_FETCH=true ;;
    --force) FORCE_REFRESH=1 ;;
    --verbose) VERBOSE=true ;;
    -h|--help) grep -E '^#( |$)' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "Unknown option: $1" >&2; exit 2 ;;
  esac
  shift
done

PYTHON_VERSION="${PYTHON_VERSION:-$(versions_get python_version)}"

UV_BIN="${UV_BIN:-$(command -v uv || true)}"
export UV_BIN

export FORCE_REFRESH
export AUTO_REVISION
$VERBOSE && set -x

if [[ "${CHECK_BUILD_DEPS:-0}" == "1" && "$NO_PACKAGE" == false ]]; then
  BUILD_TARGET=both
  if $DO_DEB && ! $DO_RPM; then BUILD_TARGET=deb; fi
  if $DO_RPM && ! $DO_DEB; then BUILD_TARGET=rpm; fi
  run env BUILD_TARGET="$BUILD_TARGET" bash "$PKGDIR/install-build-deps.sh" --check
fi

run bash "$PKGDIR/11-uv-check.sh"

echo "[info] Build version: $VERSION"
if [[ -n "$GENERIC_RELEASE" ]]; then
  echo "[info] Generic release flag detected: $GENERIC_RELEASE"
fi
if [[ -n "${DEB_REVISION:-}" ]]; then
  echo "[deb] Explicit deb revision: $DEB_REVISION"
fi
if [[ -n "${RPM_RELEASE:-}" ]]; then
  echo "[rpm] Explicit rpm release: $RPM_RELEASE"
fi
echo "[info] Package targets: $(
  $DO_DEB && $DO_RPM && echo "deb,rpm" || {
    $DO_DEB && echo deb
    $DO_RPM && echo rpm
  }
)"
$OFFLINE && echo "[mode] OFFLINE" || echo "[mode] ONLINE"
echo "[cache] FORCE_REFRESH=$FORCE_REFRESH"
echo "[paths] requirements.yml: $COL_REQ"

# 1. Cleanup
if $DO_CLEAN || [[ "${CLEAN:-0}" == "1" ]]; then
  run bash "$PKGDIR/90-cleanup.sh"
fi

# 2. Portable Python
if ! $OFFLINE; then
  if ! $NO_PYTHON_FETCH; then
    run bash "$PKGDIR/12-python-runtime-fetch.sh" --version "$PYTHON_VERSION"
  else
    echo "[python] Skipped (--no-python)."
  fi
else
  echo "[python] Offline: skip runtime fetch."
fi

# 2b. External Tools
if ! $OFFLINE; then
  run bash "$PKGDIR/13-tools-fetch.sh"
else
  echo "[tools] Offline: skip tools fetch."
fi

# 3. Lock + wheelhouse
if ! $OFFLINE; then
  run bash "$PKGDIR/10-ansible-lock.sh"
  run bash "$PKGDIR/20-ansible-wheelhouse.sh"
else
  echo "[deps] Offline: expecting existing wheelhouse."
fi

# 4. Collections
if ! $SKIP_COLLECTIONS; then
  if [[ -f "$COL_REQ" ]]; then
    if command -v ansible-galaxy >/dev/null 2>&1; then
      run bash "$PKGDIR/30-ansible-collections-download.sh" "$COL_REQ"
      run bash "$PKGDIR/31-ansible-collections-vendor.sh"
      # Verify vendor build
      if ! find "$ROOT/build/vendor/collections" -maxdepth 3 -type d -name ansible_collections -quit 2>/dev/null; then
        echo "[collections] ERROR: vendor layout missing after build." >&2
        exit 1
      fi
    else
      echo "[collections] ERROR: ansible-galaxy not in PATH and collections requested." >&2
      exit 1
    fi
  else
    echo "[collections] NOTE: requirements file missing ($COL_REQ) - skipping."
  fi
else
  echo "[collections] Skipped (--skip-collections)."
fi

# 5. Assemble vendor data and stage the package base.
run bash "$PKGDIR/50-embed-assemble.sh"
run bash "$PKGDIR/60-pkg-stage.sh" "$VERSION"

# 6. Build the venv directly at its final absolute package path.
run bash "$PKGDIR/40-ansible-venv-build.sh"
run bash "$PKGDIR/14-runtime-compatibility-check.sh"

# Verify vendor in staging (optional warn if intentionally skipped)
if find "$ROOT/build/vendor/collections" -maxdepth 3 -type d -name ansible_collections -quit 2>/dev/null; then
  if ! find "$ROOT/build/pkgroot/opt/opentalk-compose/runtime/vendor/collections" -maxdepth 3 -type d -name ansible_collections -quit 2>/dev/null; then
    echo "[stage] ERROR: vendor missing in pkgroot but existed pre-staging." >&2
    exit 1
  fi
else
  echo "[stage] INFO: No vendor collections built (none expected)."
fi

# 7. Package
if ! $NO_PACKAGE; then
  # Resolve unified release semantics:
  # Precedence: explicit per-format overrides generic. If generic set and specific missing -> apply.
  if [[ -n "$GENERIC_RELEASE" ]]; then
    if [[ -z "${RPM_RELEASE:-}" ]]; then RPM_RELEASE="$GENERIC_RELEASE"; fi
    if [[ -z "${DEB_REVISION:-}" ]]; then DEB_REVISION="$GENERIC_RELEASE"; fi
  fi
  if $DO_DEB; then
    if [[ -n "${DEB_REVISION:-}" ]]; then export DEB_REVISION; fi
    run bash "$PKGDIR/70-pkg-build-deb.sh" "$VERSION"
    echo "[result] Debian package ready (artifacts/)"
  fi
  if $DO_RPM; then
    # Enable dynamic release fallback to avoid NEVRA collisions when rebuilding same version.
    export DYNAMIC_RPM_RELEASE="${DYNAMIC_RPM_RELEASE:-1}"
    if [[ -n "${RPM_RELEASE:-}" ]]; then
      echo "[rpm] Using explicit rpm release: $RPM_RELEASE"
      export RPM_RELEASE
    fi
    run bash "$PKGDIR/80-pkg-build-rpm.sh" "$VERSION" || {
      echo "[rpm] ERROR: RPM build failed" >&2; exit 1; }
    echo "[result] RPM package ready (artifacts/)"
  fi
else
  echo "[result] Packaging skipped (--no-package)"
fi

echo "Done."