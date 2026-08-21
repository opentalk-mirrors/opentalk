#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail
AUTO_REVISION="${AUTO_REVISION:-0}"
# shellcheck disable=SC1091
. "$(dirname "${BASH_SOURCE[0]}")/lib/common.sh"

VERSION_RAW="${1:-}"
[[ -z "$VERSION_RAW" ]] && { echo "Usage: $0 <version>" >&2; exit 2; }

# Normalize provided version/tag to RPM-compatible version
# Rules:
#  - strip leading 'v'
#  - convert prerelease in base part: '-prerelease[.N]' -> '~prereleaseN'
#  - preserve '+meta' part but remove '-' inside meta (e.g. '+git-c069' -> '+gitc069')
strip_v="$(printf '%s' "$VERSION_RAW" | sed -E 's/^v//')"
base_part="$strip_v"
meta_part=""
if [[ "$strip_v" == *+* ]]; then
  base_part="${strip_v%%+*}"
  meta_part="${strip_v#*+}"
fi
base_part="$(printf '%s' "$base_part" | sed -E 's/-([[:alpha:]]+)\.([0-9]+)/~\1\2/g; s/-([[:alpha:]]+)/~\1/g')"
if [ -n "$meta_part" ]; then
  meta_part="$(printf '%s' "$meta_part" | tr -d '-')"
  VERSION="${base_part}+${meta_part}"
else
  VERSION="${base_part}"
fi

# Extract explicit release suffix (-N) from normalized VERSION, if present.
# We only treat a trailing -N as release if the part before -N matches a semver (+ optional ~prerelease) pattern.
RPM_VERSION="$VERSION"
explicit_release_flag=0
if [[ "$RPM_VERSION" =~ ^([^+]+)-([0-9]+)(\+.+)?$ ]]; then
  core_part="${BASH_REMATCH[1]}"
  rel_part="${BASH_REMATCH[2]}"
  meta_tail="${BASH_REMATCH[3]}"
  if [[ "$core_part" =~ ^[0-9]+(\.[0-9]+){2}(~[A-Za-z0-9]+)?$ ]]; then
    RPM_VERSION="${core_part}${meta_tail}"
    RELEASE_FROM_TAG="$rel_part"
    explicit_release_flag=1
  fi
fi

ROOT="$PACKAGING_ROOT"
PKGDIR="$ROOT/extras/packaging"
ARTifacts_DIR="$ROOT/artifacts"
SPEC_FILE="$PKGDIR/opentalk-compose.spec"

mkdir -p "$ARTifacts_DIR"

validate_rpm() {
  local package_file="$1"
  local inspect_dir
  inspect_dir="$(mktemp -d "$ROOT/build/rpm-inspect.XXXXXX")"
  trap 'rm -rf "$inspect_dir"' RETURN

  rpm -qip "$package_file" >/dev/null || { echo "[rpm] ERROR: invalid RPM metadata" >&2; return 1; }
  rpm -qlp "$package_file" > "$inspect_dir/file-list"
  grep -q '^/opt/opentalk-compose/runtime/python/' "$inspect_dir/file-list" || {
    echo "[rpm] ERROR: embedded Python runtime missing" >&2
    return 1
  }
  grep -q '^/opt/opentalk-compose/runtime/venv/bin/ansible$' "$inspect_dir/file-list" || {
    echo "[rpm] ERROR: embedded Ansible missing" >&2
    return 1
  }
  grep -q '^/opt/opentalk-compose/bin/pkg-postinst.sh$' "$inspect_dir/file-list" || {
    echo "[rpm] ERROR: postinst script missing" >&2
    return 1
  }
  if grep -q 'extras/packaging' "$inspect_dir/file-list"; then
    echo "[rpm] ERROR: packaging sources leaked into RPM" >&2
    return 1
  fi

  (cd "$inspect_dir" && rpm2cpio "$package_file" | cpio -idm --quiet)
  "$inspect_dir/opt/opentalk-compose/runtime/python/bin/python3" \
    -c 'import ssl, hashlib, sqlite3, ctypes'
  "$inspect_dir/opt/opentalk-compose/runtime/venv/bin/ansible" --version >/dev/null
  echo "[rpm] Package validation passed."
}

# Prefer fpm if available for simpler conversion of staged tree
if command -v fpm >/dev/null 2>&1; then
  echo "[rpm] Using fpm path"
  PKGROOT="$ROOT/build/pkgroot"
  [ -d "$PKGROOT" ] || { echo "[rpm] ERROR: pkgroot missing (run staging first)" >&2; exit 4; }

  NAME="opentalk-compose"
  ARCH="x86_64"
  REQUIRED_FILES=(
    "$ROOT/build/pkgroot/opt/opentalk-compose/runtime/python/bin/python3"
    "$ROOT/build/pkgroot/opt/opentalk-compose/runtime/venv/bin/python3"
    "$ROOT/build/pkgroot/opt/opentalk-compose/runtime/venv/bin/ansible"
    "$ROOT/build/pkgroot/opt/opentalk-compose/bin/pkg-postinst.sh"
  )
  for required_file in "${REQUIRED_FILES[@]}"; do
    [[ -e "$required_file" ]] || { echo "[rpm] ERROR: missing embedded runtime file: $required_file" >&2; exit 4; }
  done
  find "$PKGROOT" -exec touch -h -d "@$SOURCE_DATE_EPOCH" {} +
  PACKAGE_NAME="${PACKAGE_NAME:-$NAME}"
  PACKAGE_DESCRIPTION="${PACKAGE_DESCRIPTION:-Compose and orchestrate OpenTalk services.}"
  PACKAGE_LICENSE="${PACKAGE_LICENSE:-EUPL-1.2}"
  PACKAGE_MAINTAINER="${PACKAGE_MAINTAINER:-OpenTalk GmbH mail@opentalk.eu}"
  PACKAGE_VENDOR="${PACKAGE_VENDOR:-OpenTalk GmbH}"
  PACKAGE_VENDOR_URL="${PACKAGE_VENDOR_URL:-https://opentalk.eu}"
  PACKAGE_CATEGORY="${PACKAGE_CATEGORY:-Applications/Communications}" # fpm --category maps to RPM Group
  FPM_DESCRIPTION="OpenTalk Compose Package. ${PACKAGE_DESCRIPTION}"
  # Determine release baseline (default 1). If RPM_RELEASE is set, use that.
  # If DYNAMIC_RPM_RELEASE=1, only escalate to dynamic timestamp release when a conflict is detected
  # (existing artifact with same NEVRA or installed package with same version-release).
  if [[ $explicit_release_flag -eq 1 ]]; then
    RELEASE="$RELEASE_FROM_TAG"
  elif [[ -n "${RPM_RELEASE:-}" ]]; then
    RELEASE="$RPM_RELEASE"
  else
    RELEASE="1"
  fi
  # Bump release if artifact with same NEVR already exists (no timestamps)
  BASE_RPM_FILE="$ARTifacts_DIR/${NAME}-${RPM_VERSION}-${RELEASE}.${ARCH}.rpm"
  if [[ -f "$BASE_RPM_FILE" ]] && [[ "${FORCE_RPM_OVERWRITE:-0}" != "1" ]]; then
    echo "[rpm] Existing artifact detected: $BASE_RPM_FILE"
    # If explicit release extracted from tag, abort instead of bumping.
    if [[ $explicit_release_flag -eq 1 ]]; then
      echo "[rpm] ERROR: Tag provided explicit release '$RELEASE' and artifact already exists. Not auto-incrementing." >&2
      echo "[rpm]        Use FORCE_RPM_OVERWRITE=1 or choose a new version/release." >&2
      exit 3
    fi
    if [[ "$AUTO_REVISION" != 1 ]]; then
      echo "[rpm] ERROR: Artifact already exists. Use a new release, --auto-revision, or FORCE_RPM_OVERWRITE=1." >&2
      exit 3
    fi
    # Otherwise bump release to avoid conflict.
    if [[ "$RELEASE" =~ ^[0-9]+$ ]]; then
      while [[ -f "$ARTifacts_DIR/${NAME}-${RPM_VERSION}-${RELEASE}.${ARCH}.rpm" ]]; do RELEASE=$(( RELEASE + 1 )); done
    else
      suffix="${RELEASE##*.}"
      if [[ "$RELEASE" == *.* ]] && [[ "$suffix" =~ ^[0-9]+$ ]]; then
        base_rel="${RELEASE%.*}"
        while [[ -f "$ARTifacts_DIR/${NAME}-${RPM_VERSION}-${base_rel}.${suffix}.${ARCH}.rpm" ]]; do suffix=$(( suffix + 1 )); done
        RELEASE="${base_rel}.${suffix}"
      else
        suffix=1
        while [[ -f "$ARTifacts_DIR/${NAME}-${RPM_VERSION}-${RELEASE}.${suffix}.${ARCH}.rpm" ]]; do suffix=$(( suffix + 1 )); done
        RELEASE="${RELEASE}.${suffix}"
      fi
    fi
    echo "[rpm] Bumped release to $RELEASE"
  fi
  RPM_FILE="$ARTifacts_DIR/${NAME}-${RPM_VERSION}-${RELEASE}.${ARCH}.rpm"
  FORCE=${FORCE_RPM_OVERWRITE:-0}
  if [[ -f "$RPM_FILE" ]] && [[ "$FORCE" != "1" ]]; then
    echo "[rpm] Existing file detected at target path; release should have been bumped earlier"
    exit 7
  fi

  echo "[rpm] fpm building $RPM_FILE (version=$RPM_VERSION release=$RELEASE)"
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
  fpm -s dir -t rpm -n "$PACKAGE_NAME" -v "$RPM_VERSION" -a "$ARCH" \
    --source-date-epoch-default "$SOURCE_DATE_EPOCH" \
    --rpm-rpmbuild-define "_buildtime $SOURCE_DATE_EPOCH" \
    --iteration "$RELEASE" \
    --config-files /etc/opentalk-compose/settings.yml \
    --rpm-compression "$COMP_ALGO" \
    --rpm-compression-level "$COMP_LEVEL" \
    --description "$FPM_DESCRIPTION" \
    --license "$PACKAGE_LICENSE" \
    --vendor "$PACKAGE_VENDOR" \
    --maintainer "$PACKAGE_MAINTAINER" \
    --url "$PACKAGE_VENDOR_URL" \
    --category "$PACKAGE_CATEGORY" \
    --provides "$PACKAGE_NAME" \
    --depends "(docker-ce or docker-ee or docker)" \
    --depends "(docker-compose-plugin or docker-compose)" \
    -C "$PKGROOT" \
    --after-install "$PKGROOT/opt/opentalk-compose/bin/pkg-postinst.sh" \
    "${AFTER_REMOVE_ARGS[@]}" \
    -p "$RPM_FILE"
  [ -f "$RPM_FILE" ] || { echo "[rpm] ERROR: fpm did not produce rpm" >&2; exit 5; }
  validate_rpm "$RPM_FILE" || exit 6
  echo "[rpm] Built (fpm): $RPM_FILE"
  exit 0
fi

echo "[rpm] fpm not found; falling back to rpmbuild spec method"
which rpmbuild >/dev/null 2>&1 || { echo "rpmbuild not found. Install rpm-build." >&2; exit 3; }

echo "[rpm] Building RPM for version $RPM_VERSION via rpmbuild"

NAME="opentalk-compose"
ARCH="x86_64"

RPMROOT="$ROOT/build/rpm"
mkdir -p "$RPMROOT/BUILD" "$RPMROOT/RPMS" "$RPMROOT/SOURCES" "$RPMROOT/SPECS" "$RPMROOT/SRPMS"

tar_src_dir="opentalk-compose-$VERSION"
tmp_tar_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_tar_dir"' EXIT

echo "[rpm] Creating source tarball"
git -C "$ROOT" archive --format=tar --prefix="$tar_src_dir/" "HEAD" | gzip > "$RPMROOT/SOURCES/opentalk-compose-$RPM_VERSION.tar.gz"

echo "[rpm] Syncing spec file"
cp "$SPEC_FILE" "$RPMROOT/SPECS/opentalk-compose.spec"

if ! grep -q "^Version:\s*$RPM_VERSION" "$RPMROOT/SPECS/opentalk-compose.spec"; then
  sed -i "s/^Version:.*/Version:        $RPM_VERSION/" "$RPMROOT/SPECS/opentalk-compose.spec"
fi

GIT_COMMIT=""
if [ -d "$ROOT/.git" ]; then
  GIT_COMMIT="$(git -C "$ROOT" rev-parse --short=12 HEAD 2>/dev/null || true)"
fi

pushd "$RPMROOT" >/dev/null
if [[ $explicit_release_flag -eq 1 ]]; then
  RELEASE="$RELEASE_FROM_TAG"
elif [[ -n "${RPM_RELEASE:-}" ]]; then
  RELEASE="$RPM_RELEASE"
else
  RELEASE="1"
fi
# Bump release if artifact exists in artifacts dir (only when no explicit release from tag)
if [[ -f "$ARTifacts_DIR/${NAME}-${RPM_VERSION}-${RELEASE}.${ARCH}.rpm" ]] && [[ "${FORCE_RPM_OVERWRITE:-0}" != "1" ]]; then
  if [[ "$AUTO_REVISION" != 1 ]]; then
    echo "[rpm] ERROR: Artifact already exists. Use a new release, --auto-revision, or FORCE_RPM_OVERWRITE=1." >&2
    exit 3
  fi
  if [[ $explicit_release_flag -eq 1 ]]; then
    echo "[rpm] ERROR: Explicit release '$RELEASE' already exists (rpmbuild path). Aborting." >&2
    exit 3
  fi
  if [[ "$RELEASE" =~ ^[0-9]+$ ]]; then
    while [[ -f "$ARTifacts_DIR/${NAME}-${RPM_VERSION}-${RELEASE}.${ARCH}.rpm" ]]; do RELEASE=$(( RELEASE + 1 )); done
  else
    suffix=1
    while [[ -f "$ARTifacts_DIR/${NAME}-${RPM_VERSION}-${RELEASE}.${suffix}.${ARCH}.rpm" ]]; do suffix=$(( suffix + 1 )); done
    RELEASE="${RELEASE}.${suffix}"
  fi
  echo "[rpm] rpmbuild: bumped release to $RELEASE to avoid artifact conflict"
fi
echo "[rpm] rpmbuild -ba (version=$RPM_VERSION release=$RELEASE git_commit=$GIT_COMMIT)"
  # rpmbuild path: configure XZ_OPT for multi-thread compression if xz supports -T.
  XZ_THREADS="${RPM_XZ_THREADS:-0}"  # 0 means all cores
  XZ_LEVEL="${RPM_COMPRESSION_LEVEL:-3}"
  if xz --help 2>&1 | grep -q -- "-T"; then
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
  --define "version $RPM_VERSION" \
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