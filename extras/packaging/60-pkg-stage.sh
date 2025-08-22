#!/usr/bin/env bash
set -euo pipefail
#
# Stage files into build/pkgroot for packaging:
#   /opt/opentalk-compose              (runtime prefix)
#   /etc/opentalk-compose              (configuration)
#   /usr/bin                           (wrappers)
#   /usr/share/doc/opentalk-compose    (docs & README)
#
# Features:
#   - Moves config/ into /etc/opentalk-compose and creates symlink /opt/opentalk-compose/config -> /etc/opentalk-compose
#   - Excludes VCS / CI / dev artifacts (.git, .gitignore, .gitlab-ci.yml, build/)
#   - Copies prebuilt venv (.venv) + ansible.cfg + vendor collections
#   - Copies portable Python runtime (.python) if present and writes BUILD_INFO
#   - Copies wheelhouse + requirements into build_assets for self-heal
#   - Writes wheelhouse metadata + requirements hash
#
# Usage:
#   bash extras/packaging/60-pkg-stage.sh [version]
#
# Must run after:
#   40-ansible-venv-build.sh
#   50-embed-assemble.sh

VERSION="${1:-}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PKGROOT="$ROOT/build/pkgroot"
PREFIX="$PKGROOT/opt/opentalk-compose"
ETC_DIR="$PKGROOT/etc/opentalk-compose"
DOC_DIR="$PKGROOT/usr/share/doc/opentalk-compose"
BIN_DIR_SYS="$PKGROOT/usr/bin"
BUILD_ASSETS_REL="build_assets"
WHEEL_SRC="$ROOT/build/wheelhouse"
EMB="$ROOT/build/embedded"

err() { echo "ERROR: $*" >&2; exit 1; }

# Sanity checks
[[ -x "$EMB/.venv/bin/ansible" ]] || err "Embedded venv missing (run 40-ansible-venv-build.sh)."
[[ -f "$EMB/ansible.cfg" ]] || err "ansible.cfg missing (run 50-embed-assemble.sh)."

# Reset staging
rm -rf "$PKGROOT"
mkdir -p "$PREFIX" "$ETC_DIR" "$DOC_DIR" "$BIN_DIR_SYS"

echo "[stage] Copying repository (filtered)..."

RSYNC_EXCLUDES=(
  "--exclude" "/.git"
  "--exclude" "/.gitignore"
  "--exclude" "/.gitlab-ci.yml"
  "--exclude" "/build"
  "--exclude" "/extras/packaging/*.sh~"
  "--exclude" "/extras/packaging/*.bak"
  "--exclude" "/services/**/.DS_Store"
)

rsync -a "${RSYNC_EXCLUDES[@]}" "$ROOT/" "$PREFIX/"

# Move config to /etc and create symlink
if [[ -d "$PREFIX/config" ]]; then
  echo "[stage] Relocating config/ to /etc/opentalk-compose"
  cp -a "$PREFIX/config/." "$ETC_DIR/"
  rm -rf "$PREFIX/config"
  ln -s /etc/opentalk-compose "$PREFIX/config"
fi

# Copy venv
echo "[stage] Adding embedded venv"
cp -a "$EMB/.venv" "$PREFIX/.venv"

# ansible.cfg + vendor
echo "[stage] Adding ansible.cfg & vendor collections"
cp -a "$EMB/ansible.cfg" "$PREFIX/ansible.cfg"
if [[ -d "$EMB/vendor" ]]; then
  rm -rf "$PREFIX/vendor"
  cp -a "$EMB/vendor" "$PREFIX/vendor"
fi

# Wrappers (now /usr/bin)
if compgen -G "$EMB/wrappers/ot-ansible*" >/dev/null; then
  echo "[stage] Installing wrappers into /usr/bin"
  install -m0755 "$EMB"/wrappers/ot-ansible* "$BIN_DIR_SYS"/
fi

# Portable Python runtime
if [[ -x "$ROOT/build/python/bin/python3" ]]; then
  echo "[stage] Including portable Python runtime"
  mkdir -p "$PREFIX/.python"
  cp -a "$ROOT/build/python/." "$PREFIX/.python/"
  PY_VER="$("$ROOT/build/python/bin/python3" -V 2>/dev/null | awk '{print $2}')"
  SRC_URL_FILE="$ROOT/build/python/.source-url"
  SRC_URL=""
  [[ -f "$SRC_URL_FILE" ]] && SRC_URL="$(cat "$SRC_URL_FILE")"
  {
    echo "python_version=$PY_VER"
    [[ -n "$SRC_URL" ]] && echo "source=$SRC_URL"
    [[ -n "$VERSION" ]] && echo "package_version=$VERSION"
    date -u +"build_utc=%Y-%m-%dT%H:%M:%SZ"
  } > "$PREFIX/.python/BUILD_INFO"
fi

# Wheelhouse + requirements
mkdir -p "$PREFIX/$BUILD_ASSETS_REL"
if compgen -G "$WHEEL_SRC/*.whl" >/dev/null; then
  echo "[stage] Copying wheelhouse"
  mkdir -p "$PREFIX/$BUILD_ASSETS_REL/wheelhouse"
  cp -a "$WHEEL_SRC/." "$PREFIX/$BUILD_ASSETS_REL/wheelhouse/"
  (cd "$PREFIX/$BUILD_ASSETS_REL/wheelhouse" && ls -1 > ../wheelhouse.list)
fi

if [[ -f "$ROOT/extras/packaging/requirements-ansible.txt" ]]; then
  cp -a "$ROOT/extras/packaging/requirements-ansible.txt" "$PREFIX/$BUILD_ASSETS_REL/"
  sha256sum "$ROOT/extras/packaging/requirements-ansible.txt" | awk '{print $1}' > "$PREFIX/$BUILD_ASSETS_REL/requirements-ansible.sha256"
fi

# Postinst helper for self-heal
if [[ -f "$ROOT/extras/packaging/pkg-postinst.sh" ]]; then
  install -m0755 "$ROOT/extras/packaging/pkg-postinst.sh" "$PREFIX/extras/packaging/pkg-postinst.sh"
fi

# Documentation
if [[ -f "$PREFIX/README.md" ]]; then
  echo "[stage] Installing documentation"
  cp -a "$PREFIX/README.md" "$DOC_DIR/README.md"
fi
if [[ -d "$PREFIX/docs" ]]; then
  cp -a "$PREFIX/docs" "$DOC_DIR/"
fi
# Optional removal from prefix:
# rm -rf "$PREFIX/docs" "$PREFIX/README.md"

# Permissions
find "$PREFIX" -type d -exec chmod 0755 {} + 2>/dev/null || true
find "$PREFIX" -type f -name "*.sh" -exec chmod 0755 {} + 2>/dev/null || true