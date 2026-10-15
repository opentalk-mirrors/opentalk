#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2
#
#
# opentalk_keycloak_realm_export_sanitize.sh — Sanitizes an OpenTalk Keycloak realm export for public distribution.
#
# This script is OpenTalk-focused: it keeps only the "demo" user (disabled), regenerates
# all client secrets, and removes certain potentially sensitive sections. Designed for
# preparing a sample realm export safe for sharing.
# 
# Realm export example:
# /opt/keycloak/bin/kc.sh export --realm opentalk --users same_file --file export-opentalk-realm-users.json
#
# Usage:
#   opentalk_keycloak_realm_export_sanitize.sh [--use-placeholder] [--frontend-url <url>] <input_file> <output_file>
#
# Options:
#   --use-placeholder      Use the literal placeholder CHANGE_ME instead of generating random secrets.
#   --frontend-url <url>   Set (and normalize) the public OpenTalk frontend base URL used inside client definitions.
#                           If omitted, defaults to https://example.com . If a scheme is missing, https:// is prepended.
#
# Exit codes:
#   1 usage / unknown option
#   2 input file missing
#   3 jq not installed
#
# NOTE: For broader / generic use, extend with flags to keep additional users, select
# specific clients, or mask further fields.
#
set -euo pipefail

USE_PLACEHOLDER=false
PLACEHOLDER=CHANGE_ME
FRONTEND_URL="https://example.com"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --use-placeholder)
      USE_PLACEHOLDER=true; shift ;;
    --frontend-url)
      [[ $# -ge 2 ]] || { echo "❌ --frontend-url expects a value"; exit 1; }
      FRONTEND_URL="$2"; shift 2 ;;
    -*) echo "❌ Unknown option: $1"; exit 1 ;;
    *) break ;;
  esac
done

if [[ $# -ne 2 ]]; then
  echo "Usage: $0 [--use-placeholder] [--frontend-url <url>] <input_file> <output_file>"; exit 1
fi

# Normalize FRONTEND_URL: add scheme if missing, drop trailing slashes (we keep a single trailing slash when assigning root/base/admin URLs)
if [[ ! "$FRONTEND_URL" =~ :// ]]; then
  FRONTEND_URL="https://$FRONTEND_URL"
fi
# Remove trailing slashes (store clean), will re-add one when assigning whole-base fields
FRONTEND_URL_CLEAN=${FRONTEND_URL%/}

INPUT="$1"; OUTPUT="$2"
[[ -f "$INPUT" ]] || { echo "❌ Input file '$INPUT' not found."; exit 2; }
command -v jq >/dev/null 2>&1 || { echo "❌ Required command 'jq' not found. Please install it."; exit 3; }

trap 'rm -f "$TMP" "$TMP.tmp" 2>/dev/null || true' EXIT

generate_secret() {
  if $USE_PLACEHOLDER; then
    echo "$PLACEHOLDER"
  elif command -v openssl >/dev/null 2>&1; then
    openssl rand -base64 48 | tr -dc 'A-Za-z0-9' | head -c 32
  elif command -v uuidgen >/dev/null 2>&1; then
    uuidgen | tr -d '-' | tr '[:lower:]' '[:upper:]'
  else
    head -c 48 /dev/urandom | base64 | tr -dc 'A-Za-z0-9' | head -c 32
  fi
}

echo "🧹 Sanitizing OpenTalk realm export '$INPUT' ..."
echo "🌐 Frontend URL set to: $FRONTEND_URL_CLEAN"

TMP=$(mktemp)

DEMO_PASSWORD=$(generate_secret)

# Single jq pass:
#   - Keep demo user (disabled, random password)
#   - Keep ALL service account users (username starts with "service-account-") to preserve their realm role assignments
#   - Drop any other real users
#   - Remove selected top-level fields (components, eventsListeners)
jq --arg pwd "$DEMO_PASSWORD" \
   --arg front "$FRONTEND_URL_CLEAN" '
  .users |= (
    map(select(.username == "demo" or (.username|startswith("service-account-")))
      | if .username == "demo" then
          .enabled = false |
          .credentials = [{ "type": "password", "value": $pwd, "temporary": false }]
        else
          .  # service-account users unchanged
        end
    )
  ) |
  # Rewrite client URL-related fields safely without replacing whole client objects
  (.clients[]? | .rootUrl)      |= (if . != null then $front + "/" else . end) |
  (.clients[]? | .baseUrl)      |= (if . != null then $front + "/" else . end) |
  (.clients[]? | .adminUrl)     |= (if . != null then $front + "/" else . end) |
  (.clients[]? | .redirectUris) |= (if type=="array" then map( if test("^[A-Za-z][A-Za-z0-9+.-]*://[^/]+") then sub("^[A-Za-z][A-Za-z0-9+.-]*://[^/]+"; $front) else . end ) else . end) |
  (.clients[]? | .webOrigins)   |= (if type=="array" then map( if . == "*" then . else if test("^[A-Za-z][A-Za-z0-9+.-]*://[^/]+") then sub("^[A-Za-z][A-Za-z0-9+.-]*://[^/]+"; $front) else $front end end ) else . end) |
  del(.components, .eventsListeners)
' "$INPUT" > "$TMP"

# Replace/rotate all client secrets
declare -A GENERATED_SECRETS
mapfile -t CLIENTS < <(jq -r '.clients[]?.clientId' "$TMP")
for CLIENT_ID in "${CLIENTS[@]}"; do
  NEW_SECRET=$(generate_secret)
  jq --arg cid "$CLIENT_ID" --arg secret "$NEW_SECRET" '(.clients[]? | select(.clientId == $cid) | .secret) = $secret' "$TMP" > "$TMP.tmp" && mv "$TMP.tmp" "$TMP"
  GENERATED_SECRETS["$CLIENT_ID"]="$NEW_SECRET"
done

mv "$TMP" "$OUTPUT"

echo "✅ Sanitized realm export created: $OUTPUT (service-account users retained, client URLs normalized)"

echo
echo "🔑 Client secrets:"
printf '%s\n' "---------------------------------------------------------------"
for cid in "${!GENERATED_SECRETS[@]}"; do
  printf "% -30s %s\n" "$cid" "${GENERATED_SECRETS[$cid]}"
done | sort
printf '%s\n' "---------------------------------------------------------------"

echo
echo "🔑 Demo user password (user is disabled by default):"
printf '%s\n' "---------------------------------------------------------------"
printf "% -30s %s\n" "demo" "$DEMO_PASSWORD"
printf '%s\n' "---------------------------------------------------------------"

if $USE_PLACEHOLDER; then
  echo
  echo "⚠️  Placeholder mode used: Replace CHANGE_ME secrets before any production use."
fi
