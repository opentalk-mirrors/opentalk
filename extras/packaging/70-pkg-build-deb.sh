#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PKGROOT="$ROOT/build/pkgroot"
VER="${1:-1.0.0}"
NAME="opentalk-compose-embedded-ansible"
POSTINST_SRC="$ROOT/extras/packaging/pkg-postinst.sh"
PRERM_SRC="$ROOT/extras/packaging/pkg-prerm.sh"
POSTRM_SRC="$ROOT/extras/packaging/pkg-postrm.sh"

[ -d "$PKGROOT" ] || { echo "Missing $PKGROOT (run 60-pkg-stage.sh)"; exit 1; }

if command -v fpm >/dev/null; then
  fpm -s dir -t deb -n "$NAME" -v "$VER" \
    --after-install "$POSTINST_SRC" \
    --before-remove "$PRERM_SRC" \
    --after-remove "$POSTRM_SRC" \
    -C "$PKGROOT" \
    --description "OpenTalk Compose with embedded Ansible runtime (hybrid self-heal)" .
else
  DEBROOT="$ROOT/build/debroot"
  rm -rf "$DEBROOT"; mkdir -p "$DEBROOT/DEBIAN"
  cat > "$DEBROOT/DEBIAN/control" <<EOF
Package: $NAME
Version: $VER
Section: admin
Priority: optional
Architecture: amd64
Maintainer: admin@example.org
Description: OpenTalk Compose with embedded Ansible runtime (hybrid self-heal)
EOF
  install -m0755 "$POSTINST_SRC" "$DEBROOT/DEBIAN/postinst"
  install -m0755 "$PRERM_SRC"   "$DEBROOT/DEBIAN/prerm"
  install -m0755 "$POSTRM_SRC"  "$DEBROOT/DEBIAN/postrm"
  (cd "$PKGROOT" && tar cpf - .) | (cd "$DEBROOT" && tar xpf -)
  dpkg-deb --build "$DEBROOT" "$ROOT/build/${NAME}_${VER}_amd64.deb"
fi

echo "Built .deb in build/"