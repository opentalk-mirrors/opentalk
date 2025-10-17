#!/usr/bin/env bash
#
# opentalk_keycloak_realm_export_sanitize.sh — Sanitizes an OpenTalk Keycloak realm export for public distribution.
#
# This script is OpenTalk-focused: it keeps only the "demo" user (disabled), regenerates
# all client secrets, and removes certain potentially sensitive sections. Designed for
# preparing a sample realm export safe for sharing.
#
# Usage:
#   opentalk_keycloak_realm_export_sanitize.sh [--use-placeholder] <input_file> <output_file>
#
# Options:
#   --use-placeholder   Use the literal placeholder CHANGE_ME instead of generating random secrets.
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

while [[ $# -gt 0 ]]; do
  case "$1" in
    --use-placeholder)
      USE_PLACEHOLDER=true; shift ;;
    -*) echo "❌ Unknown option: $1"; exit 1 ;;
    *) break ;;
  esac
done

if [[ $# -ne 2 ]]; then
  echo "Usage: $0 [--use-placeholder] <input_file> <output_file>"; exit 1
fi

INPUT="$1"; OUTPUT="$2"
[[ -f "$INPUT" ]] || { echo "❌ Input file '$INPUT' not found."; exit 2; }
command -v jq >/dev/null 2>&1 || { echo "❌ Required command 'jq' not found. Please install it."; exit 3; }

trap 'rm -f "$TMP" "$TMP.tmp" 2>/dev/null || true' EXIT

generate_secret() {
  if $USE_PLACEHOLDER; then
    echo "$PLACEHOLDER"
  elif command -v openssl >/dev/null 2>&1; then
    openssl rand -base64 24 | tr -dc 'A-Za-z0-9' | head -c 32
  elif command -v uuidgen >/dev/null 2>&1; then
    uuidgen | tr -d '-' | tr '[:lower:]' '[:upper:]'
  else
    head -c 32 /dev/urandom | base64 | tr -dc 'A-Za-z0-9' | head -c 32
  fi
}

echo "🧹 Sanitizing OpenTalk realm export '$INPUT' ..."

TMP=$(mktemp)

DEMO_PASSWORD=$(generate_secret)

# Single jq pass: keep only demo user, disable it, set password, remove selected fields
jq --arg pwd "$DEMO_PASSWORD" '
  .users |= (map(select(.username == "demo")
    | .enabled = false
    | .credentials = [{ "type": "password", "value": $pwd, "temporary": false }]
  )) |
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

echo "✅ Sanitized realm export created: $OUTPUT"

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
