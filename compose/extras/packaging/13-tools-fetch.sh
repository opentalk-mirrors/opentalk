#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail
# shellcheck disable=SC1091
. "$(dirname "${BASH_SOURCE[0]}")/lib/common.sh"
# shellcheck disable=SC1091
. "$(dirname "${BASH_SOURCE[0]}")/lib/versions.sh"
# Fetch external tool binaries
ROOT="$PACKAGING_ROOT"
OUTDIR="$ROOT/build/tools"
FORCE=${FORCE_REFRESH:-0}

GITLAB_API_URL="https://git.opentalk.dev/api/v4"

# GitLab authentication: prefer GITLAB_TOKEN, fallback to CI_JOB_TOKEN in CI
CURL_AUTH_OPTS=()

mkdir -p "$OUTDIR"

# --- keycloak-theme-generator ---
# An absent pin is not fatal here: the tool is optional and the build skips it
# with a warning, so use the non-fatal lookup rather than the strict read.
KC_THEME_GEN_VERSION="$(versions_lookup 'keycloak-theme-generator_version' || true)"
KC_THEME_GEN_BIN="$OUTDIR/keycloak-theme-generator"
KC_THEME_GEN_MARKER="$KC_THEME_GEN_BIN.version"
KC_THEME_GEN_PROJECT_ID="1213"

if [[ -z "$KC_THEME_GEN_VERSION" ]]; then
  echo "[tools] WARN: keycloak-theme-generator_version not set in .versions.yml - skipping"
else
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
    echo "[tools] Downloading keycloak-theme-generator $KC_THEME_GEN_VERSION via Job Artifacts API"

    if [[ -n "${GITLAB_TOKEN:-}" ]]; then
      echo "[tools] Using GITLAB_TOKEN for authentication"
      CURL_AUTH_OPTS=(--header "PRIVATE-TOKEN: $GITLAB_TOKEN")
    elif [[ -n "${CI_JOB_TOKEN:-}" ]]; then
      echo "[tools] Using CI_JOB_TOKEN for authentication"
      CURL_AUTH_OPTS=(--header "JOB-TOKEN: $CI_JOB_TOKEN")
    else
      echo "[tools] ERROR: No GitLab token found (set GITLAB_TOKEN or CI_JOB_TOKEN)"
      exit 1
    fi
    
    KC_ARTIFACTS_URL="${GITLAB_API_URL}/projects/${KC_THEME_GEN_PROJECT_ID}/jobs/artifacts/${KC_THEME_GEN_VERSION}/download?job=package:build"
    KC_TEMP_ZIP=$(mktemp)
    
    curl -fLs "${CURL_AUTH_OPTS[@]}" "$KC_ARTIFACTS_URL" -o "$KC_TEMP_ZIP"
    
    unzip -p "$KC_TEMP_ZIP" "target/release/keycloak-theme-generator" > "$KC_THEME_GEN_BIN"
    rm -f "$KC_TEMP_ZIP"
    
    chmod +x "$KC_THEME_GEN_BIN"
    echo "$KC_THEME_GEN_VERSION" > "$KC_THEME_GEN_MARKER"
    echo "[tools] keycloak-theme-generator installed at $KC_THEME_GEN_BIN"
  fi
fi

echo "[tools] Tool fetch completed"
