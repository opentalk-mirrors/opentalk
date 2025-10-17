#!/usr/bin/env bash
#
# opentalk_keycloak_realm_export_sanitize.sh — Sanitizes an OpenTalk Keycloak realm export for public distribution.
#
# Removes sensitive data, keeps a disabled demo user with a random password,
# regenerates client secrets, and prints secrets in formatted tables.
#

set -euo pipefail

USE_PLACEHOLDER=false
PLACEHOLDER=CHANGE_ME

# --- Parse flags ---
while [[ $# -gt 0 ]]; do
  case "$1" in
    --use-placeholder)
      USE_PLACEHOLDER=true
      shift
      ;;
    -*)
      echo "❌ Unknown option: $1"
      exit 1
      ;;
    *)
      break
      ;;
  esac
done

if [[ $# -ne 2 ]]; then
  echo "Usage: $0 [--use-placeholder] <input_file> <output_file>"
  exit 1
fi

INPUT="$1"
OUTPUT="$2"

if [[ ! -f "$INPUT" ]]; then
  echo "❌ Input file '$INPUT' not found."
  exit 2
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "❌ Required command 'jq' not found. Please install it."
  exit 3
fi

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

# Keep only demo user, disable it, assign random password
DEMO_PASSWORD=$(generate_secret)

jq --arg pwd "$DEMO_PASSWORD" '
  .users |= map(select(.username == "demo") |
    .enabled = false |
    .credentials = [{ "type": "password", "value": $pwd, "temporary": false }]
  )
' "$INPUT" > "$TMP"

# Remove all other users
jq '
  .users |= map(select(.username == "demo"))
' "$TMP" > "$TMP.tmp" && mv "$TMP.tmp" "$TMP"

# Remove sensitive fields
jq '
  del(.components) |
  del(.eventsListeners)
' "$TMP" > "$TMP.tmp" && mv "$TMP.tmp" "$TMP"

declare -A GENERATED_SECRETS

# Replace client secrets
mapfile -t CLIENTS < <(jq -r '.clients[]?.clientId' "$TMP")

for CLIENT_ID in "${CLIENTS[@]}"; do
  NEW_SECRET=$(generate_secret)
  jq --arg cid "$CLIENT_ID" --arg secret "$NEW_SECRET" '
    (.clients[]? | select(.clientId == $cid) | .secret) = $secret
  ' "$TMP" > "$TMP.tmp" && mv "$TMP.tmp" "$TMP"
  GENERATED_SECRETS["$CLIENT_ID"]="$NEW_SECRET"
done

mv "$TMP" "$OUTPUT"

# --- Print Client Secrets ---
echo "✅ Sanitized realm export created: $OUTPUT"
echo
echo "🔑 Client secrets:"
printf '%s\n' "---------------------------------------------------------------"
for cid in "${!GENERATED_SECRETS[@]}"; do
  printf "%-30s %s\n" "$cid" "${GENERATED_SECRETS[$cid]}"
done
printf '%s\n' "---------------------------------------------------------------"

# --- Print Demo User Password in its own table with same formatting ---
echo
echo "🔑 Demo user password (user is disabled by default):"
printf '%s\n' "---------------------------------------------------------------"
printf "%-30s %s\n" "demo" "$DEMO_PASSWORD"
printf '%s\n' "---------------------------------------------------------------"