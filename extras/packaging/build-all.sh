#!/usr/bin/env bash
set -euo pipefail

# Orchestrates embedded Ansible hybrid build (prebuilt + self-heal) and Debian package.
# Steps:
#   (optional) cleanup
#   (optional) fetch portable Python
#   lock (cached)
#   wheelhouse (cached)
#   collections download/vendor (cached, optional)
#   venv build (cached hash-based)
#   assemble
#   stage
#   build deb
#
# Options:
#   --version X          Package version (default: 1.0.0)
#   --offline            Skip all network dependent steps (python fetch, lock, wheelhouse, collections)
#   --skip-collections   Skip Galaxy collections
#   --no-package         Do not build the .deb (still builds runtime)
#   --clean              Run cleanup (does NOT pass --all)
#   --no-python          Do not fetch portable Python even if fallback URL exists
#   --force              Force refresh (ignore caches; sets FORCE_REFRESH=1)
#   --verbose            set -x
#   -h|--help            Show help
#
# Environment:
#   PY_URL        Portable Python URL (override fallback; set empty "" to disable)
#   PY_FILE       Local tarball path for portable Python (alternative to PY_URL)
#   PY_SHA256     Expected SHA256 for tarball (optional but recommended)
#   COL_REQ       Collections requirements.yml path (default repo path)
#   FORCE_REFRESH=1  Force all rebuilds (same as --force)
#
# Fallback portable Python URL (disable via --no-python or PY_URL="")
FALLBACK_PY_URL="https://github.com/indygreg/python-build-standalone/releases/download/20250818/cpython-3.13.7+20250818-x86_64-unknown-linux-gnu-install_only.tar.gz"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PKGDIR="$ROOT/extras/packaging"

VERSION="1.0.0"
OFFLINE=false
SKIP_COLLECTIONS=false
NO_PACKAGE=false
DO_CLEAN=false
VERBOSE=false
NO_PYTHON_FETCH=false
FORCE_REFRESH=${FORCE_REFRESH:-0}

# Resolve env with fallback
PY_URL="${PY_URL:-$FALLBACK_PY_URL}"
PY_FILE="${PY_FILE:-}"
PY_SHA256="${PY_SHA256:-}"
COL_REQ="${COL_REQ:-$ROOT/services/cforge/ansible/collections/requirements.yml}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version) VERSION="${2:?missing version}"; shift ;;
    --offline) OFFLINE=true ;;
    --skip-collections) SKIP_COLLECTIONS=true ;;
    --no-package) NO_PACKAGE=true ;;
    --clean) DO_CLEAN=true ;;
    --no-python) NO_PYTHON_FETCH=true; PY_URL=""; PY_FILE="";;
    --force) FORCE_REFRESH=1 ;;
    --verbose) VERBOSE=true ;;
    -h|--help)
      grep -E '^#( |$)' "$0" | sed 's/^# \{0,1\}//'
      exit 0 ;;
    *) echo "Unknown option: $1" >&2; exit 2 ;;
  esac
  shift
done

export FORCE_REFRESH
$VERBOSE && set -x

run() { echo ">>> $*"; "$@"; }

echo "[info] Build version: $VERSION"
$OFFLINE && echo "[mode] OFFLINE" || echo "[mode] ONLINE"
echo "[cache] FORCE_REFRESH=$FORCE_REFRESH"

# 1. Cleanup (optional)
$DO_CLEAN && run bash "$PKGDIR/90-cleanup.sh"

# 2. Portable Python
if ! $OFFLINE; then
  if ! $NO_PYTHON_FETCH; then
    if [[ -n "$PY_FILE" ]]; then
      echo "[python] Using local tarball $PY_FILE"
      PY_SHA256="$PY_SHA256" run bash "$PKGDIR/12-python-runtime-fetch.sh" --file "$PY_FILE"
    elif [[ -n "$PY_URL" ]]; then
      echo "[python] Fetching portable runtime (override with PY_URL or disable via --no-python)"
      PY_SHA256="$PY_SHA256" run bash "$PKGDIR/12-python-runtime-fetch.sh" --url "$PY_URL"
    else
      echo "[python] No portable runtime requested."
    fi
  else
    echo "[python] Skipped (--no-python)."
  fi
else
  echo "[python] Offline: skip runtime fetch."
fi

# 3. Requirements lock + wheelhouse
if ! $OFFLINE; then
  run bash "$PKGDIR/10-ansible-lock.sh"
  run bash "$PKGDIR/20-ansible-wheelhouse.sh"
else
  echo "[deps] Offline: expecting existing requirements-ansible.txt & wheelhouse."
fi

# 4. Collections (optional)
if ! $SKIP_COLLECTIONS; then
  if ! $OFFLINE; then
    if command -v ansible-galaxy >/dev/null && [[ -f "$COL_REQ" ]]; then
      run bash "$PKGDIR/30-ansible-collections-download.sh" "$COL_REQ"
      run bash "$PKGDIR/31-ansible-collections-vendor.sh"
    else
      echo "[collections] Skip (ansible-galaxy missing or $COL_REQ absent)"
    fi
  else
    echo "[collections] Offline: expecting pre-vendored collections if needed."
  fi
else
  echo "[collections] Skipped (--skip-collections)."
fi

# 5. Venv build (cached) + assemble
run bash "$PKGDIR/40-ansible-venv-build.sh"
run bash "$PKGDIR/50-embed-assemble.sh"

# 6. Stage
run bash "$PKGDIR/60-pkg-stage.sh"

# 7. Build package
if ! $NO_PACKAGE; then
  run bash "$PKGDIR/70-pkg-build-deb.sh" "$VERSION"
  echo "[result] Debian package ready (build/)"
else
  echo "[result] Packaging skipped (--no-package)"
fi

echo "Done."