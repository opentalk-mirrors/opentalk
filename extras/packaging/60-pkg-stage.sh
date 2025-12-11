#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail
VERSION="${1:-}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PKGROOT="$ROOT/build/pkgroot"
PREFIX="$PKGROOT/opt/opentalk-compose"
RUNTIME_DIR="$PREFIX/runtime"
AUTOMATION_DIR="$PREFIX/automation"
ETC_DIR="$PKGROOT/etc/opentalk-compose"
DOC_DIR="$PKGROOT/usr/share/doc/opentalk-compose"
USR_BIN_DIR="$PKGROOT/usr/bin"
RUNTIME_BIN_DIR="$PREFIX/bin"
BUILD_ASSETS_SUB="build_assets"
WHEEL_SRC="$ROOT/build/wheelhouse"
EMB="$ROOT/build/embedded"
WRAPPER_SRC="$ROOT/extras/packaging/wrappers"

err() { echo "ERROR: $*" >&2; exit 1; }

rm -rf "$PKGROOT"
mkdir -p "$PREFIX" "$RUNTIME_DIR" "$ETC_DIR" "$DOC_DIR" "$USR_BIN_DIR" "$RUNTIME_BIN_DIR"

echo "[stage] Copying repository (filtered)..."
RSYNC_EXCLUDES=(
  "--exclude" "/README.md"
  "--exclude" "/docs"
  "--exclude" "/.git"
  "--exclude" "/.gitignore"
  "--exclude" "/.gitlab-ci.yml"
  "--exclude" "/.ansible-lint.yml"
  "--exclude" "/.commitlintrc.json"
  "--exclude" "/.markdown_style.rb"
  "--exclude" "/.pre-commit-config.yaml"
  "--exclude" "/.yamllint.yml"
  "--exclude" "/mkdocs.yml"
  "--exclude" "/REUSE.toml"
  "--exclude" "/build"
  "--exclude" "/artifacts"
  "--exclude" "/extras"
  "--exclude" "/*.deb"
  "--exclude" "/*.rpm"
  "--exclude" "/*.tar.gz"
  "--exclude" "/config/secrets"
  "--exclude" "/config/secrets/**"
  "--exclude" "/services/**"
)
rsync -a "${RSYNC_EXCLUDES[@]}" "$ROOT/" "$PREFIX/"

mkdir -p "$PREFIX/services"

[[ -d "$AUTOMATION_DIR" ]] || err "automation/ directory missing."
[[ -f "$AUTOMATION_DIR/ansible.cfg" ]] || err "automation/ansible.cfg missing."

EXAMPLES_DIR="$PKGROOT/usr/share/opentalk-compose/examples"
mkdir -p "$EXAMPLES_DIR"

# Config -> ship settings.yml as real conffile (dpkg handles .dpkg-dist), plus example copy
if [[ -d "$PREFIX/config" ]]; then
  echo "[stage] Relocating config/ to /etc/opentalk-compose (conffile model)"
  if [[ -d "$PREFIX/config/secrets" ]]; then
    echo "[stage] Removing config/secrets from package context"
    rm -rf "$PREFIX/config/secrets"
  fi
  cp -a "$PREFIX/config/." "$ETC_DIR/"
  rm -rf "$PREFIX/config"
fi
# NOTE: Legacy symlink $PREFIX/config -> /etc/opentalk-compose removed (no longer required).

echo "[stage] Skipping embedded venv copy (build on target)"
rm -rf "$RUNTIME_DIR/venv"
mkdir -p "$RUNTIME_DIR/venv"

# Ensure secrets directory exists in /etc (even though content is generated later)
mkdir -p "$ETC_DIR/secrets"
chmod 0700 "$ETC_DIR/secrets" || true

# Ensure ssl directory exists (certs/keys may be dropped in later by operator)
mkdir -p "$ETC_DIR/ssl"
chmod 0700 "$ETC_DIR/ssl" || true

SAMPLE_SETTINGS_SRC=""
if [[ -f "$ROOT/extras/packaging/config-dist/settings.yml-sample" ]]; then
  SAMPLE_SETTINGS_SRC="$ROOT/extras/packaging/config-dist/settings.yml-sample"
else
  echo "[stage] WARN: No settings.yml sample or repo config/settings.yml found"
fi

# Ensure settings.yml exists (conffile) – if repo did not have one, create from sample
if [[ ! -f "$ETC_DIR/settings.yml" && -n "$SAMPLE_SETTINGS_SRC" ]]; then
  echo "[stage] Creating settings.yml from template (repo lacked one)"
  cp "$SAMPLE_SETTINGS_SRC" "$ETC_DIR/settings.yml"
fi

# Install example copy (not a conffile) for diffs
if [[ -f "$ETC_DIR/settings.yml" ]]; then
  cp "$ETC_DIR/settings.yml" "$EXAMPLES_DIR/settings.yml.example" || true
fi

# Ship cleaned example Keycloak realm export (if present)
REALM_EXAMPLE_SRC="$ROOT/extras/packaging/config-dist/export-opentalk-realm-and-users.json"
if [[ -f "$REALM_EXAMPLE_SRC" ]]; then
  echo "[stage] Including example Keycloak realm export"
  install -m0644 "$REALM_EXAMPLE_SRC" "$EXAMPLES_DIR/export-opentalk-realm-and-users.json"
fi

# Vendor Collections
SRC_VENDOR_EMB="$EMB/vendor"
SRC_VENDOR_BUILD="$ROOT/build/vendor"
if [[ -d "$SRC_VENDOR_EMB/collections/ansible_collections" ]]; then
  echo "[stage] Copying vendor collections (embedded)"
  rm -rf "$RUNTIME_DIR/vendor"
  cp -a "$SRC_VENDOR_EMB" "$RUNTIME_DIR/vendor"
elif [[ -d "$SRC_VENDOR_BUILD/collections/ansible_collections" ]]; then
  echo "[stage] Copying vendor collections (fallback build/vendor)"
  rm -rf "$RUNTIME_DIR/vendor"
  cp -a "$SRC_VENDOR_BUILD" "$RUNTIME_DIR/vendor"
else
  echo "[stage] WARN: No vendor collections found."
fi

# Self-heal / postrm scripts
for helper in pkg-postinst.sh pkg-postrm.sh; do
  [[ -f "$ROOT/extras/packaging/$helper" ]] && install -m0755 "$ROOT/extras/packaging/$helper" "$RUNTIME_BIN_DIR/$helper"
done

echo "[stage] Installing wrappers"
for w in ot-ansible ot-ansible-playbook ot-ansible-galaxy; do
  [[ -f "$WRAPPER_SRC/$w" ]] || err "Missing wrapper: $w"
  install -m0755 "$WRAPPER_SRC/$w" "$RUNTIME_BIN_DIR/$w"
  ln -sfn "/opt/opentalk-compose/bin/$w" "$USR_BIN_DIR/$w"
done

# Primary control wrapper (otctl)
if [[ -f "$WRAPPER_SRC/otctl" ]]; then
  echo "[stage] Installing otctl wrapper"
  install -m0755 "$WRAPPER_SRC/otctl" "$RUNTIME_BIN_DIR/otctl"
  ln -sfn "/opt/opentalk-compose/bin/otctl" "$USR_BIN_DIR/otctl"
else
  echo "[stage] WARN: otctl wrapper missing (expected $WRAPPER_SRC/otctl)"
fi

# Install auxiliary maintenance script (realm export sanitizer)
SCRIPT_NEW_SRC="$ROOT/extras/scripts/opentalk_keycloak_realm_export_sanitize.sh"
if [[ -f "$SCRIPT_NEW_SRC" ]]; then
  echo "[stage] Installing opentalk_keycloak_realm_export_sanitize maintenance script"
  install -m0755 "$SCRIPT_NEW_SRC" "$RUNTIME_BIN_DIR/opentalk_keycloak_realm_export_sanitize.sh"
  # Backwards compatibility symlink for older docs/tools (can be removed in a future major release)
  ln -sfn "/opt/opentalk-compose/bin/opentalk_keycloak_realm_export_sanitize.sh" "$RUNTIME_BIN_DIR/clean_realm_export.sh"
else
  echo "[stage] WARN: opentalk_keycloak_realm_export_sanitize.sh not found at $SCRIPT_NEW_SRC"
fi

# Portable Python
if [[ -x "$ROOT/build/python/bin/python3" ]]; then
  echo "[stage] Including portable Python runtime"
  mkdir -p "$RUNTIME_DIR/python"
  cp -a "$ROOT/build/python/." "$RUNTIME_DIR/python/"
  PY_VER="$("$ROOT/build/python/bin/python3" -V 2>/dev/null | awk '{print $2}')"
  SRC_URL_FILE="$ROOT/build/python/.source-url"
  SRC_URL=""; [[ -f "$SRC_URL_FILE" ]] && SRC_URL="$(cat "$SRC_URL_FILE")"
  {
    echo "python_version=$PY_VER"
    [[ -n "$SRC_URL" ]] && echo "source=$SRC_URL"
    [[ -n "$VERSION" ]] && echo "package_version=$VERSION"
    date -u +"build_utc=%Y-%m-%dT%H:%M:%SZ"
  } > "$RUNTIME_DIR/python/.BUILD_INFO"
fi

# Wheelhouse + requirements
mkdir -p "$RUNTIME_DIR/$BUILD_ASSETS_SUB"
if compgen -G "$WHEEL_SRC/*.whl" >/dev/null; then
  echo "[stage] Copying wheelhouse"
  mkdir -p "$RUNTIME_DIR/$BUILD_ASSETS_SUB/wheelhouse"
  cp -a "$WHEEL_SRC/." "$RUNTIME_DIR/$BUILD_ASSETS_SUB/wheelhouse/"
  (cd "$RUNTIME_DIR/$BUILD_ASSETS_SUB/wheelhouse" && \ls -1 > ../wheelhouse.list)
fi
if [[ -f "$ROOT/extras/packaging/requirements-ansible.txt" ]]; then
  cp -a "$ROOT/extras/packaging/requirements-ansible.txt" "$RUNTIME_DIR/$BUILD_ASSETS_SUB/"
  sha256sum "$ROOT/extras/packaging/requirements-ansible.txt" | awk '{print $1}' > "$RUNTIME_DIR/$BUILD_ASSETS_SUB/requirements-ansible.sha256"
fi

# Shell completions
install -d -m 0755 "$PKGROOT/usr/share/bash-completion/completions"
install -d -m 0755 "$PKGROOT/usr/share/zsh/site-functions"
install -m 0644 "$ROOT/extras/packaging/completions/otctl.bash" \
  "$PKGROOT/usr/share/bash-completion/completions/otctl"
install -m 0644 "$ROOT/extras/packaging/completions/_otctl" \
  "$PKGROOT/usr/share/zsh/site-functions/_otctl"

# Docs
[[ -f "$ROOT/README.md" ]] && cp -a "$ROOT/README.md" "$DOC_DIR/README.md"
[[ -d "$ROOT/docs" ]] && cp -a "$ROOT/docs/." "$DOC_DIR/"

# Permissions
find "$PREFIX" -type d -exec chmod 0755 {} + 2>/dev/null || true
find "$PREFIX/bin" -type f -maxdepth 1 -exec chmod 0755 {} + 2>/dev/null || true

echo "[stage] Completed staging at $PKGROOT (conffile settings.yml + example)"