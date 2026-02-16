#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail
# Fetch external tool binaries
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUTDIR="$ROOT/build/tools"
VERSIONS_FILE="$ROOT/.versions.yml"
FORCE=${FORCE_REFRESH:-0}

GITLAB_BASE_URL="https://git.opentalk.dev"

# Simple YAML value parser (key: value format only)
get_version() {
  local key="$1"
  grep -E "^${key}:" "$VERSIONS_FILE" | sed 's/^[^:]*:[[:space:]]*//' | tr -d '"' | tr -d "'"
}

mkdir -p "$OUTDIR"

# --- keycloak-theme-generator ---
KC_THEME_GEN_VERSION=$(get_version "keycloak-theme-generator_version")
KC_THEME_GEN_BIN="$OUTDIR/keycloak-theme-generator"
KC_THEME_GEN_MARKER="$KC_THEME_GEN_BIN.version"

if [[ -z "$KC_THEME_GEN_VERSION" ]]; then
  echo "[tools] WARN: keycloak-theme-generator_version not set in .versions.yml - skipping"
else
  KC_THEME_GEN_URL="${GITLAB_BASE_URL}/opentalk/opentalk-devops/keycloak-theme-generator/-/releases/${KC_THEME_GEN_VERSION}/downloads/keycloak-theme-generator"

  if [[ -x "$KC_THEME_GEN_BIN" && -f "$KC_THEME_GEN_MARKER" && $FORCE -eq 0 ]]; then
    CACHED_VERSION=$(cat "$KC_THEME_GEN_MARKER")
    if [[ "$CACHED_VERSION" == "$KC_THEME_GEN_VERSION" ]]; then
      echo "[tools] keycloak-theme-generator $KC_THEME_GEN_VERSION already cached. Skip."
    else
      echo "[tools] Version mismatch (cached: $CACHED_VERSION, wanted: $KC_THEME_GEN_VERSION). Re-downloading..."
      rm -f "$KC_THEME_GEN_BIN" "$KC_THEME_GEN_MARKER"
    fi
  fi

  if [[ ! -x "$KC_THEME_GEN_BIN" ]]; then
    echo "[tools] Downloading keycloak-theme-generator $KC_THEME_GEN_VERSION"
    curl -fL "$KC_THEME_GEN_URL" -o "$KC_THEME_GEN_BIN"
    chmod +x "$KC_THEME_GEN_BIN"
    echo "$KC_THEME_GEN_VERSION" > "$KC_THEME_GEN_MARKER"
    echo "[tools] keycloak-theme-generator installed at $KC_THEME_GEN_BIN"
  fi
fi

echo "[tools] Tool fetch completed"
