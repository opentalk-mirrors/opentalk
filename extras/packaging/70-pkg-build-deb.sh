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
mkdir -p "$OUTDIR"

# Validate essentials
for p in \
  "$PKGROOT/opt/opentalk-compose/bin/ot-ansible" \
  "$PKGROOT/opt/opentalk-compose/bin/ot-ansible-playbook" \
  "$PKGROOT/opt/opentalk-compose/bin/ot-ansible-galaxy" \
  "$PKGROOT/opt/opentalk-compose/bin/pkg-postinst.sh" \
  "$PKGROOT/opt/opentalk-compose/automation/ansible.cfg" \
  "$PKGROOT/opt/opentalk-compose/runtime/build_assets/requirements-ansible.txt"; do
  [ -e "$p" ] || { echo "[build-deb] Missing expected file: $p"; exit 1; }
done

POSTRM_IN_BIN=0
[ -f "$PKGROOT/opt/opentalk-compose/bin/pkg-postrm.sh" ] && POSTRM_IN_BIN=1

if [ -d "$PKGROOT/opt/opentalk-compose/extras/packaging" ]; then
  echo "[build-deb] ERROR: extras/packaging accidentally included"
  exit 1
fi

# Symlinks
for w in ot-ansible ot-ansible-playbook ot-ansible-galaxy; do
  LNK="$PKGROOT/usr/bin/$w"
  TGT="/opt/opentalk-compose/bin/$w"
  if [[ ! -L "$LNK" && -x "$PKGROOT$TGT" ]]; then
    mkdir -p "$(dirname "$LNK")"
    ln -s "$TGT" "$LNK"
    echo "[build-deb] Created missing symlink /usr/bin/$w"
  fi
  [[ -L "$LNK" ]] || { echo "[build-deb] ERROR: symlink /usr/bin/$w missing"; exit 1; }
done

# Metadata
RUNTIME_META="$PKGROOT/opt/opentalk-compose/BUILD_INFO"
GIT_COMMIT=""
[ -d "$ROOT/.git" ] && GIT_COMMIT="$(git -C "$ROOT" rev-parse --short=12 HEAD || true)"
if [ ! -f "$RUNTIME_META" ]; then
  {
    echo "name=$NAME"
    echo "version=$VERSION"
    [ -n "$GIT_COMMIT" ] && echo "git_commit=$GIT_COMMIT"
    date -u +"build_utc=%Y-%m-%dT%H:%M:%SZ"
  } > "$RUNTIME_META"
fi

PKG_FILE="$OUTDIR/${NAME}_${VERSION}_${ARCH}.deb"
echo "[build-deb] Building $PKG_FILE"

POSTINST="$PKGROOT/opt/opentalk-compose/bin/pkg-postinst.sh"
POSTRM_BIN="$PKGROOT/opt/opentalk-compose/bin/pkg-postrm.sh"
POSTRM_SRC="$ROOT/extras/packaging/pkg-postrm.sh"

if command -v fpm >/dev/null 2>&1; then
  DEPENDS=(--depends "openssh-client")
  AFTER_REMOVE_ARGS=()
  if [ $POSTRM_IN_BIN -eq 1 ]; then
    AFTER_REMOVE_ARGS=(--after-remove "$POSTRM_BIN")
  elif [ -f "$POSTRM_SRC" ]; then
    AFTER_REMOVE_ARGS=(--after-remove "$POSTRM_SRC")
  fi
  TRANSITION_FLAGS=(
    --provides "$OLD_NAME"
    --replaces "$OLD_NAME"
    --conflicts "$OLD_NAME"
  )
  fpm -s dir -t deb -n "$NAME" -v "$VERSION" -a "$ARCH" \
    "${DEPENDS[@]}" \
    "${TRANSITION_FLAGS[@]}" \
    --description "OpenTalk Compose runtime (including embedded offline Ansible)" \
    --license "Proprietary" \
    --vendor "OpenTalk" \
    --maintainer "OpenTalk DevOps <devops@opentalk.example>" \
    --url "https://opentalk.eu" \
    --deb-compression xz \
    --after-install "$POSTINST" \
    "${AFTER_REMOVE_ARGS[@]}" \
    --no-deb-use-file-permissions \
    -C "$PKGROOT" \
    opt etc usr
  mv ./*.deb "$PKG_FILE"
else
  echo "[build-deb] fpm not found – fallback to dpkg-deb"
  CTRL_DIR="$PKGROOT/DEBIAN"
  rm -rf "$CTRL_DIR"; mkdir -p "$CTRL_DIR"
  cat > "$CTRL_DIR/control" <<EOF
Package: $NAME
Version: $VERSION
Section: admin
Priority: optional
Architecture: $ARCH
Maintainer: OpenTalk DevOps <devops@opentalk.example>
Description: OpenTalk Compose runtime (including embedded offline Ansible)
Depends: openssh-client
Provides: $OLD_NAME
Replaces: $OLD_NAME
Conflicts: $OLD_NAME
EOF
  install -m0755 "$POSTINST" "$CTRL_DIR/postinst"
  if [ $POSTRM_IN_BIN -eq 1 ]; then
    install -m0755 "$POSTRM_BIN" "$CTRL_DIR/postrm"
  elif [ -f "$POSTRM_SRC" ]; then
    install -m0755 "$POSTRM_SRC" "$CTRL_DIR/postrm"
  fi
  dpkg-deb -Zxz --build "$PKGROOT" "$PKG_FILE"
fi

echo "[build-deb] Built: $PKG_FILE"
ls -lh "$PKG_FILE"