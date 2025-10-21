#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2
set -euo pipefail

VERSION="${1:-}"
[[ -z "$VERSION" ]] && { echo "Usage: $0 <version>" >&2; exit 2; }

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PKGDIR="$ROOT/extras/packaging"
ARTifacts_DIR="$ROOT/artifacts"
SPEC_FILE="$PKGDIR/opentalk-compose.spec"

mkdir -p "$ARTifacts_DIR"

# Prefer fpm if available for simpler conversion of staged tree
if command -v fpm >/dev/null 2>&1; then
  echo "[rpm] Using fpm path"
  PKGROOT="$ROOT/build/pkgroot"
  [ -d "$PKGROOT" ] || { echo "[rpm] ERROR: pkgroot missing (run staging first)" >&2; exit 4; }

  NAME="opentalk-compose"
  ARCH="x86_64"
  DESC="OpenTalk compose deployment tooling"
  MAINTAINER="OpenTalk <support@opentalk.eu>"
  VENDOR="OpenTalk"
  URL="https://opentalk.eu"
  RPM_FILE="$ARTifacts_DIR/${NAME}-${VERSION}-1.${ARCH}.rpm"
  FORCE=${FORCE_RPM_OVERWRITE:-0}
  if [[ -f "$RPM_FILE" ]]; then
    if [[ "$FORCE" == "1" ]]; then
      echo "[rpm] Existing file $RPM_FILE will be overwritten (FORCE_RPM_OVERWRITE=1)"
      rm -f "$RPM_FILE"
    else
      TS="$(date +%Y%m%d%H%M%S)"
      RPM_FILE="$ARTifacts_DIR/${NAME}-${VERSION}-1.${TS}.${ARCH}.rpm"
      echo "[rpm] Existing file found; writing to timestamped file $RPM_FILE (set FORCE_RPM_OVERWRITE=1 to overwrite)"
    fi
  fi

  echo "[rpm] fpm building $RPM_FILE"
  fpm -s dir -t rpm -n "$NAME" -v "$VERSION" -a "$ARCH" \
    --config-files /etc/opentalk-compose/settings.yml \
    --rpm-compression xz \
    --rpm-compression-level 9 \
    --description "$DESC" \
  --license EUPL-1.2 \
    --vendor "$VENDOR" \
    --maintainer "$MAINTAINER" \
    --url "$URL" \
    -C "$PKGROOT" \
    --after-install "$PKGROOT/opt/opentalk-compose/bin/pkg-postinst.sh" \
    $( [ -f "$PKGROOT/opt/opentalk-compose/bin/pkg-postrm.sh" ] && printf -- "--after-remove %s" "$PKGROOT/opt/opentalk-compose/bin/pkg-postrm.sh" ) \
    -p "$RPM_FILE"
  [ -f "$RPM_FILE" ] || { echo "[rpm] ERROR: fpm did not produce rpm" >&2; exit 5; }
  echo "[rpm] Built (fpm): $RPM_FILE"
  exit 0
fi

echo "[rpm] fpm not found; falling back to rpmbuild spec method"
which rpmbuild >/dev/null 2>&1 || { echo "rpmbuild not found. Install rpm-build." >&2; exit 3; }

echo "[rpm] Building RPM for version $VERSION via rpmbuild"

RPMROOT="$ROOT/build/rpm"
mkdir -p "$RPMROOT/BUILD" "$RPMROOT/RPMS" "$RPMROOT/SOURCES" "$RPMROOT/SPECS" "$RPMROOT/SRPMS"

tar_src_dir="opentalk-compose-$VERSION"
tmp_tar_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_tar_dir"' EXIT

echo "[rpm] Creating source tarball"
git -C "$ROOT" archive --format=tar --prefix="$tar_src_dir/" "HEAD" | gzip > "$RPMROOT/SOURCES/opentalk-compose-$VERSION.tar.gz"

echo "[rpm] Syncing spec file"
cp "$SPEC_FILE" "$RPMROOT/SPECS/opentalk-compose.spec"

if ! grep -q "^Version:\s*$VERSION" "$RPMROOT/SPECS/opentalk-compose.spec"; then
  sed -i "s/^Version:.*/Version:        $VERSION/" "$RPMROOT/SPECS/opentalk-compose.spec"
fi

GIT_COMMIT=""
if [ -d "$ROOT/.git" ]; then
  GIT_COMMIT="$(git -C "$ROOT" rev-parse --short=12 HEAD 2>/dev/null || true)"
fi

pushd "$RPMROOT" >/dev/null
echo "[rpm] rpmbuild -ba (version=$VERSION release=1 git_commit=$GIT_COMMIT)"
rpmbuild \
  --define "_topdir $RPMROOT" \
  --define "version $VERSION" \
  --define "release 1" \
  --define "_source_payload w9.xzdio" \
  --define "_binary_payload w9.xzdio" \
  $( [ -n "$GIT_COMMIT" ] && printf -- "--define git_commit %s" "$GIT_COMMIT" ) \
  -ba SPECS/opentalk-compose.spec
popd >/dev/null

echo "[rpm] Copying resulting RPMs to artifacts/"
cp "$RPMROOT"/RPMS/*/*.rpm "$ARTifacts_DIR/" || true
cp "$RPMROOT"/SRPMS/*.src.rpm "$ARTifacts_DIR/" || true
echo "[rpm] Done. Artifacts in $ARTifacts_DIR"