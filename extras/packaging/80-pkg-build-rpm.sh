#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

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
  # Determine release baseline (default 1). If RPM_RELEASE is set, use that.
  # If DYNAMIC_RPM_RELEASE=1, only escalate to dynamic timestamp release when a conflict is detected
  # (existing artifact with same NEVRA or installed package with same version-release).
  if [[ -n "${RPM_RELEASE:-}" ]]; then
    RELEASE="$RPM_RELEASE"
  else
    RELEASE="1"
  fi
  if [[ "${DYNAMIC_RPM_RELEASE:-0}" == "1" ]]; then
    BASE_RPM_FILE="$ARTifacts_DIR/${NAME}-${VERSION}-${RELEASE}.${ARCH}.rpm"
    CONFLICT=0
    # Conflict if target artifact already exists
    [[ -f "$BASE_RPM_FILE" ]] && CONFLICT=1
    # Conflict if system already has same NEV (Name-Version-Release) installed
    if rpm -q --qf '%{NAME}-%{VERSION}-%{RELEASE}.%{ARCH}\n' "$NAME" 2>/dev/null | grep -q "^${NAME}-${VERSION}-${RELEASE}\."; then
      CONFLICT=1
    fi
    if [[ $CONFLICT -eq 1 ]]; then
      TS_REL="$(date -u +%Y%m%d%H%M%S)"
      if git -C "$ROOT" rev-parse --short=8 HEAD >/dev/null 2>&1; then
        GIT_SHORT="$(git -C "$ROOT" rev-parse --short=8 HEAD)"
        RELEASE="${TS_REL}.git${GIT_SHORT}"
      else
        RELEASE="${TS_REL}"
      fi
      echo "[rpm] Dynamic release escalation due to conflict -> $RELEASE"
    fi
  fi
  RPM_FILE="$ARTifacts_DIR/${NAME}-${VERSION}-${RELEASE}.${ARCH}.rpm"
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

  echo "[rpm] fpm building $RPM_FILE (release=$RELEASE)"
  # Multi-thread compression attempt: prefer xzmt if fpm supports it, else xz.
  FPM_HELP=$(fpm --help 2>&1 || true)
  COMP_LEVEL="${RPM_COMPRESSION_LEVEL:-3}"
  COMP_ALGO="xz"
  if echo "$FPM_HELP" | grep -q "xzmt"; then
    COMP_ALGO="xzmt"
  fi
  echo "[rpm] fpm compression: $COMP_ALGO (level $COMP_LEVEL)"
  # Conditional args arrays to avoid word splitting (SC2046)
  AFTER_REMOVE_ARGS=()
  if [ -f "$PKGROOT/opt/opentalk-compose/bin/pkg-postrm.sh" ]; then
    AFTER_REMOVE_ARGS=(--after-remove "$PKGROOT/opt/opentalk-compose/bin/pkg-postrm.sh")
  fi
  fpm -s dir -t rpm -n "$NAME" -v "$VERSION" -a "$ARCH" \
    --iteration "$RELEASE" \
    --config-files /etc/opentalk-compose/settings.yml \
    --rpm-compression "$COMP_ALGO" \
    --rpm-compression-level "$COMP_LEVEL" \
    --description "$DESC" \
  --license EUPL-1.2 \
    --vendor "$VENDOR" \
    --maintainer "$MAINTAINER" \
    --url "$URL" \
    -C "$PKGROOT" \
    --after-install "$PKGROOT/opt/opentalk-compose/bin/pkg-postinst.sh" \
    "${AFTER_REMOVE_ARGS[@]}" \
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
if [[ -n "${RPM_RELEASE:-}" ]]; then
  RELEASE="$RPM_RELEASE"
else
  # rpmbuild fallback (same logic as fpm, but need to recalc because we exited earlier there if fpm succeeded)
  RELEASE="1"
  if [[ "${DYNAMIC_RPM_RELEASE:-0}" == "1" ]]; then
    if rpm -q --qf '%{NAME}-%{VERSION}-%{RELEASE}.%{ARCH}\n' opentalk-compose 2>/dev/null | grep -q "^opentalk-compose-${VERSION}-1\."; then
      TS_REL="$(date -u +%Y%m%d%H%M%S)"
      if git -C "$ROOT" rev-parse --short=8 HEAD >/dev/null 2>&1; then
        GIT_SHORT="$(git -C "$ROOT" rev-parse --short=8 HEAD)"
        RELEASE="${TS_REL}.git${GIT_SHORT}"
      else
        RELEASE="${TS_REL}"
      fi
      echo "[rpm] Dynamic release escalation (rpmbuild) -> $RELEASE"
    fi
  fi
fi
echo "[rpm] rpmbuild -ba (version=$VERSION release=$RELEASE git_commit=$GIT_COMMIT)"
  # rpmbuild path: configure XZ_OPT for multi-thread compression if xz supports -T.
  XZ_THREADS="${RPM_XZ_THREADS:-0}"  # 0 means all cores
  XZ_LEVEL="${RPM_COMPRESSION_LEVEL:-3}"
  if xz --help 2>&1 | grep -q "-T"; then
    export XZ_OPT="-T${XZ_THREADS} -${XZ_LEVEL}"
    echo "[rpm] rpmbuild XZ_OPT=$XZ_OPT"
  else
    export XZ_OPT="-${XZ_LEVEL}"
    echo "[rpm] rpmbuild xz has no threading support; using XZ_OPT=$XZ_OPT"
  fi
  # Conditional git commit define (SC2046)
  GIT_DEFINE=()
  if [ -n "$GIT_COMMIT" ]; then
    GIT_DEFINE=(--define "git_commit $GIT_COMMIT")
  fi
rpmbuild \
  --define "_topdir $RPMROOT" \
  --define "version $VERSION" \
  --define "release $RELEASE" \
  --define "_source_payload w9.xzdio" \
  --define "_binary_payload w9.xzdio" \
  "${GIT_DEFINE[@]}" \
  -ba SPECS/opentalk-compose.spec || { echo "[rpm] ERROR: rpmbuild failed" >&2; exit 6; }
popd >/dev/null

echo "[rpm] Copying resulting RPMs to artifacts/"
cp "$RPMROOT"/RPMS/*/*.rpm "$ARTifacts_DIR/" || true
cp "$RPMROOT"/SRPMS/*.src.rpm "$ARTifacts_DIR/" || true
echo "[rpm] Done. Artifacts in $ARTifacts_DIR"