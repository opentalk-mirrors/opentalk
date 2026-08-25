#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail
# shellcheck disable=SC1091
. "$(dirname "${BASH_SOURCE[0]}")/lib/common.sh"
#
# Build Debian package for opentalk-compose
#

VERSION_RAW="${1:-}"
AUTO_REVISION="${AUTO_REVISION:-0}"
[ -n "$VERSION_RAW" ] || { echo "Usage: $0 <version>"; exit 1; }

# Normalize provided version/tag to Debian/RPM-compatible upstream-version
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
  # remove hyphens inside meta to keep Debian upstream-version valid
  meta_part="$(printf '%s' "$meta_part" | tr -d '-')"
  VERSION="${base_part}+${meta_part}"
else
  VERSION="${base_part}"
fi

# Extract explicit Debian revision from trailing -N if pattern matches semver (+ optional ~prerelease) before meta.
explicit_deb_revision_flag=0
if [[ "$VERSION" =~ ^([^+]+)-([0-9]+)(\+.+)?$ ]]; then
  core_part="${BASH_REMATCH[1]}"
  rev_part="${BASH_REMATCH[2]}"
  meta_tail="${BASH_REMATCH[3]}"
  if [[ "$core_part" =~ ^[0-9]+(\.[0-9]+){2}(~[A-Za-z0-9]+)?$ ]]; then
    VERSION="${core_part}${meta_tail}"  # strip -N from upstream version, keep +meta if any
    DEB_REVISION="$rev_part"
    explicit_deb_revision_flag=1
  fi
fi

# Debian revision (packaging iteration). Default to 1 per versioning schema.
DEB_REVISION="${DEB_REVISION:-${DEB_REVISION:-1}}"
# Basic validation: allow alphanum plus . and + and ~; reject spaces
if ! [[ "$DEB_REVISION" =~ ^[0-9A-Za-z.+~_-]+$ ]]; then
  echo "[build-deb] ERROR: Invalid DEB_REVISION: $DEB_REVISION" >&2
  exit 2
fi
if [ $explicit_deb_revision_flag -eq 1 ]; then
  echo "[build-deb] Using deb revision: $DEB_REVISION (from tag)"
else
  echo "[build-deb] Using deb revision: $DEB_REVISION"
fi

ROOT="$PACKAGING_ROOT"
PKGROOT="$ROOT/build/pkgroot"
OUTDIR="$ROOT/artifacts"

NAME="opentalk-compose"
ARCH="amd64"

# Unified package metadata (override via environment if needed)
PACKAGE_NAME="${PACKAGE_NAME:-$NAME}"
PACKAGE_DESCRIPTION="${PACKAGE_DESCRIPTION:-Compose and orchestrate OpenTalk services.}"
PACKAGE_LICENSE="${PACKAGE_LICENSE:-EUPL-1.2}"
PACKAGE_MAINTAINER="${PACKAGE_MAINTAINER:-OpenTalk GmbH mail@opentalk.eu}"
PACKAGE_VENDOR="${PACKAGE_VENDOR:-OpenTalk GmbH}"
PACKAGE_VENDOR_URL="${PACKAGE_VENDOR_URL:-https://opentalk.eu}"
# Requested category (fpm --category) vs Debian Section mapping: use Section 'net' as specified
PACKAGE_CATEGORY="${PACKAGE_CATEGORY:-comm}"
DEB_SECTION="${DEB_SECTION:-net}"

# Final assembled human-readable description for tooling
FPM_DESCRIPTION="OpenTalk Compose Package. ${PACKAGE_DESCRIPTION}"

mkdir -p "$OUTDIR"

fail() { echo "[build-deb] ERROR: $*" >&2; exit 1; }

# Expected runtime artifacts
REQUIRED_FILES=(
  "$PKGROOT/opt/opentalk-compose/bin/ot-ansible"
  "$PKGROOT/opt/opentalk-compose/bin/ot-ansible-playbook"
  "$PKGROOT/opt/opentalk-compose/bin/ot-ansible-galaxy"
  "$PKGROOT/opt/opentalk-compose/bin/pkg-postinst.sh"
  "$PKGROOT/opt/opentalk-compose/runtime/python/bin/python3"
  "$PKGROOT/opt/opentalk-compose/runtime/venv/bin/python3"
  "$PKGROOT/opt/opentalk-compose/runtime/venv/bin/ansible"
  "$PKGROOT/opt/opentalk-compose/automation/ansible.cfg"
  "$PKGROOT/opt/opentalk-compose/runtime/build_assets/requirements-ansible.txt"
)

for p in "${REQUIRED_FILES[@]}"; do
  [ -e "$p" ] || fail "Missing expected file: $p"
done

# Conffile (must exist now – created by staging)
[ -f "$PKGROOT/etc/opentalk-compose/settings.yml" ] || fail "Missing conffile: $PKGROOT/etc/opentalk-compose/settings.yml"
[ -d "$PKGROOT/etc/opentalk-compose/secrets" ] || fail "Missing secrets dir: $PKGROOT/etc/opentalk-compose/secrets"
[ -d "$PKGROOT/etc/opentalk-compose/ssl" ] || fail "Missing ssl dir: $PKGROOT/etc/opentalk-compose/ssl"

# Shell completions (bash + zsh) must be present
[ -f "$PKGROOT/usr/share/bash-completion/completions/otctl" ] || fail "Missing bash completion: $PKGROOT/usr/share/bash-completion/completions/otctl"
[ -f "$PKGROOT/usr/share/zsh/site-functions/_otctl" ] || fail "Missing zsh completion: $PKGROOT/usr/share/zsh/site-functions/_otctl"

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
RUNTIME_META="$PKGROOT/opt/opentalk-compose/.BUILD_INFO"
GIT_COMMIT=""
if [ -d "$ROOT/.git" ]; then
  GIT_COMMIT="$(git -C "$ROOT" rev-parse --short=12 HEAD 2>/dev/null || true)"
fi
if [ ! -f "$RUNTIME_META" ]; then
  {
    echo "name=$NAME"
    echo "version=$VERSION"
    [ -n "$GIT_COMMIT" ] && echo "git_commit=$GIT_COMMIT"
    date -u -d "@$SOURCE_DATE_EPOCH" +"build_utc=%Y-%m-%dT%H:%M:%SZ"
  } >"$RUNTIME_META"
fi
find "$PKGROOT" -exec touch -h -d "@$SOURCE_DATE_EPOCH" {} +

BASE_FILENAME="${NAME}_${VERSION}-${DEB_REVISION}_${ARCH}.deb"
PKG_FILE="$OUTDIR/$BASE_FILENAME"
if [[ -f "$PKG_FILE" ]] && [[ "${FORCE_DEB_OVERWRITE:-0}" != "1" ]]; then
  echo "[build-deb] Existing artifact detected: $PKG_FILE"
  # If explicit revision extracted from tag, abort instead of auto-incrementing.
  if [[ $explicit_deb_revision_flag -eq 1 ]]; then
    echo "[build-deb] ERROR: Explicit revision '$DEB_REVISION' from tag already exists. Not auto-incrementing." >&2
    echo "[build-deb]        Use FORCE_DEB_OVERWRITE=1 or choose a new version/revision." >&2
    exit 3
  fi
  if [[ "$AUTO_REVISION" != 1 ]]; then
    echo "[build-deb] ERROR: Artifact already exists. Use a new revision, --auto-revision, or FORCE_DEB_OVERWRITE=1." >&2
    exit 3
  fi
  # Otherwise increment Debian revision until free.
  while [[ -f "$OUTDIR/${NAME}_${VERSION}-${DEB_REVISION}_${ARCH}.deb" ]]; do
    DEB_REVISION=$(( DEB_REVISION + 1 ))
  done
  BASE_FILENAME="${NAME}_${VERSION}-${DEB_REVISION}_${ARCH}.deb"
  PKG_FILE="$OUTDIR/$BASE_FILENAME"
  echo "[build-deb] Bumped deb revision to $DEB_REVISION -> $PKG_FILE"
fi
echo "[build-deb] Building $PKG_FILE"

POSTINST_SRC="$PKGROOT/opt/opentalk-compose/bin/pkg-postinst.sh"
POSTRM_SRC="$PKGROOT/opt/opentalk-compose/bin/pkg-postrm.sh"
[ -x "$POSTINST_SRC" ] || fail "Postinst script not executable: $POSTINST_SRC"
[ -f "$POSTRM_SRC" ] || echo "[build-deb] (optional) postrm script not present"

if command -v fpm >/dev/null 2>&1; then
  # Build via fpm. Use --iteration for revision if provided (maps to Debian revision after a dash)
  FPM_CMD=(fpm -s dir -t deb -n "$PACKAGE_NAME" -v "$VERSION" -a "$ARCH" \
    --source-date-epoch-default "$SOURCE_DATE_EPOCH" \
    --config-files /etc/opentalk-compose/settings.yml \
    --deb-compression xz \
    --after-install "$POSTINST_SRC")
  if fpm --help 2>&1 | grep -q -- 'deb-no-default-config-files'; then
    FPM_CMD+=(--deb-no-default-config-files)
  fi
  if [ -f "$POSTRM_SRC" ]; then
    FPM_CMD+=(--after-remove "$POSTRM_SRC")
  fi
  FPM_CMD+=(--iteration "$DEB_REVISION")
  # Metadata & dependencies
  FPM_CMD+=(--description "$FPM_DESCRIPTION" \
    --license "$PACKAGE_LICENSE" \
    --maintainer "$PACKAGE_MAINTAINER" \
    --vendor "$PACKAGE_VENDOR" \
    --url "$PACKAGE_VENDOR_URL" \
    --category "$PACKAGE_CATEGORY" \
    --provides "$PACKAGE_NAME" \
    --depends "docker-ce | docker-ee" \
    --depends "docker-compose-plugin")
  FPM_CMD+=(-C "$PKGROOT" -p "$PKG_FILE" .)
  "${FPM_CMD[@]}"
  [ -f "$PKG_FILE" ] || fail "fpm did not produce expected package at $PKG_FILE"
else
  echo "[build-deb] fpm not found - using raw dpkg-deb fallback"
  mkdir -p "$PKGROOT/DEBIAN"

  FULL_VERSION="${VERSION}-${DEB_REVISION}"
  cat >"$PKGROOT/DEBIAN/control" <<EOF
Package: $PACKAGE_NAME
Version: $FULL_VERSION
Section: $DEB_SECTION
Priority: optional
Architecture: $ARCH
Maintainer: $PACKAGE_MAINTAINER
Homepage: $PACKAGE_VENDOR_URL
Depends: docker-ce | docker-ee, docker-compose-plugin
Provides: $PACKAGE_NAME
Description: $FPM_DESCRIPTION
EOF

  {
    echo "/etc/opentalk-compose/settings.yml"
  } > "$PKGROOT/DEBIAN/conffiles"

  install -m 0755 "$POSTINST_SRC" "$PKGROOT/DEBIAN/postinst"
  if [ -f "$POSTRM_SRC" ]; then
    install -m 0755 "$POSTRM_SRC" "$PKGROOT/DEBIAN/postrm"
  fi

  DPKG_DEB_BUILD_ARGS=()
  if dpkg-deb --help 2>&1 | grep -q -- '--root-owner-group'; then
    DPKG_DEB_BUILD_ARGS+=(--root-owner-group)
  fi
  dpkg-deb --build "${DPKG_DEB_BUILD_ARGS[@]}" "$PKGROOT" "$PKG_FILE"
fi

# Sanity checks (inspect package contents in an isolated temp dir)
INSPECT_DIR="$(mktemp -d "$ROOT/build/inspect.XXXXXX")"
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
rm -rf "$INSPECT_DIR"