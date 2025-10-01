#!/usr/bin/env bash
set -euo pipefail
#
# Build Debian package for opentalk-compose
#

VERSION="${1:-}"
[ -n "$VERSION" ] || { echo "Usage: $0 <version>"; exit 1; }

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PKGROOT="$ROOT/build/pkgroot"
OUTDIR="$ROOT/artifacts"

NAME="opentalk-compose"
OLD_NAME="opentalk-compose-embedded-ansible"
ARCH="amd64"

# Metadata defaults (override via environment if needed)
DESC="${DESC:-OpenTalk compose deployment tooling}"
MAINTAINER="${MAINTAINER:-OpenTalk <support@opentalk.eu>}"
VENDOR="${VENDOR:-OpenTalk}"
URL="${URL:-https://opentalk.eu}"

mkdir -p "$OUTDIR"

fail() { echo "[build-deb] ERROR: $*" >&2; exit 1; }

# Expected runtime artifacts
REQUIRED_FILES=(
  "$PKGROOT/opt/opentalk-compose/bin/ot-ansible"
  "$PKGROOT/opt/opentalk-compose/bin/ot-ansible-playbook"
  "$PKGROOT/opt/opentalk-compose/bin/ot-ansible-galaxy"
  "$PKGROOT/opt/opentalk-compose/bin/pkg-postinst.sh"
  "$PKGROOT/opt/opentalk-compose/automation/ansible.cfg"
  "$PKGROOT/opt/opentalk-compose/runtime/build_assets/requirements-ansible.txt"
)

for p in "${REQUIRED_FILES[@]}"; do
  [ -e "$p" ] || fail "Missing expected file: $p"
done

# Conffile (must exist now – created by staging)
[ -f "$PKGROOT/etc/opentalk-compose/settings.yml" ] || fail "Missing conffile: $PKGROOT/etc/opentalk-compose/settings.yml"
[ -d "$PKGROOT/etc/opentalk-compose/secrets" ] || fail "Missing secrets dir: $PKGROOT/etc/opentalk-compose/secrets"

# Guard: packaging sources must not leak
[ -d "$PKGROOT/opt/opentalk-compose/extras/packaging" ] && fail "extras/packaging accidentally included"

# Ensure symlinks for wrapper helpers (idempotent)
for w in ot-ansible ot-ansible-playbook ot-ansible-galaxy; do
  LNK="$PKGROOT/usr/bin/$w"
  TGT="/opt/opentalk-compose/bin/$w"
  if [[ ! -L "$LNK" && -x "$PKGROOT$TGT" ]]; then
    mkdir -p "$(dirname "$LNK")"
    ln -s "$TGT" "$LNK"
    echo "[build-deb] Created symlink /usr/bin/$w"
  fi
  [[ -L "$LNK" ]] || fail "Symlink /usr/bin/$w missing"
done

# Embed build metadata (only once)
RUNTIME_META="$PKGROOT/opt/opentalk-compose/BUILD_INFO"
GIT_COMMIT=""
if [ -d "$ROOT/.git" ]; then
  GIT_COMMIT="$(git -C "$ROOT" rev-parse --short=12 HEAD 2>/dev/null || true)"
fi
if [ ! -f "$RUNTIME_META" ]; then
  {
    echo "name=$NAME"
    echo "version=$VERSION"
    [ -n "$GIT_COMMIT" ] && echo "git_commit=$GIT_COMMIT"
    date -u +"build_utc=%Y-%m-%dT%H:%M:%SZ"
  } >"$RUNTIME_META"
fi

PKG_FILE="$OUTDIR/${NAME}_${VERSION}_${ARCH}.deb"
echo "[build-deb] Building $PKG_FILE"

POSTINST_SRC="$PKGROOT/opt/opentalk-compose/bin/pkg-postinst.sh"
POSTRM_SRC="$PKGROOT/opt/opentalk-compose/bin/pkg-postrm.sh"
[ -x "$POSTINST_SRC" ] || fail "Postinst script not executable: $POSTINST_SRC"
[ -f "$POSTRM_SRC" ] || echo "[build-deb] (optional) postrm script not present"

if command -v fpm >/dev/null 2>&1; then
  # Build via fpm
  fpm -s dir -t deb -n "$NAME" -v "$VERSION" -a "$ARCH" \
    --config-files /etc/opentalk-compose/settings.yml \
    --deb-compression xz \
    --after-install  "$POSTINST_SRC" \
    $( [ -f "$POSTRM_SRC" ] && printf -- "--after-remove %s" "$POSTRM_SRC" ) \
    -C "$PKGROOT" \
    -p "$PKG_FILE" \
    --description "$DESC" \
    --license "Proprietary" \
    --vendor "$VENDOR" \
    --maintainer "$MAINTAINER" \
    --url "$URL"
  mv -f ./*.deb "$PKG_FILE"
else
  echo "[build-deb] fpm not found - using raw dpkg-deb fallback"
  mkdir -p "$PKGROOT/DEBIAN"

  cat >"$PKGROOT/DEBIAN/control" <<EOF
Package: $NAME
Version: $VERSION
Section: misc
Priority: optional
Architecture: $ARCH
Maintainer: $MAINTAINER
Description: $DESC
Homepage: $URL
EOF

  {
    echo "/etc/opentalk-compose/settings.yml"
  } > "$PKGROOT/DEBIAN/conffiles"

  install -m 0755 "$POSTINST_SRC" "$PKGROOT/DEBIAN/postinst"
  if [ -f "$POSTRM_SRC" ]; then
    install -m 0755 "$POSTRM_SRC" "$PKGROOT/DEBIAN/postrm"
  fi

  dpkg-deb --build "$PKGROOT" "$PKG_FILE"
fi

# Sanity checks
INSPECT_DIR="$ROOT/build/_inspect_pkg"
rm -rf "$INSPECT_DIR"
mkdir -p "$INSPECT_DIR"

if ! dpkg-deb -R "$PKG_FILE" "$INSPECT_DIR"; then
  echo "[build-deb] DEBUG: ar t output:"
  ar t "$PKG_FILE" || true
  echo "[build-deb] DEBUG: file header:"
  file "$PKG_FILE" || true
  fail "Failed to inspect built package"
fi

if ! grep -q "^/etc/opentalk-compose/settings.yml" "$INSPECT_DIR/DEBIAN/conffiles"; then
  fail "settings.yml not marked as conffile"
fi

echo "[build-deb] Built: $PKG_FILE"
\ls -lh "$PKG_FILE"