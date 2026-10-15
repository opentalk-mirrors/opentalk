#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
SCRIPT="$SCRIPT_DIR/../opentalk_keycloak_realm_export_sanitize.sh"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

INPUT="$TMP_DIR/input.json"
OUTPUT="$TMP_DIR/output.json"

cat > "$INPUT" <<'JSON'
{
  "components": {"key": "sensitive"},
  "eventsListeners": {"key": "sensitive"},
  "users": [
    {"username": "demo", "enabled": true},
    {"username": "alice", "enabled": true},
    {"username": "service-account-api", "enabled": true}
  ],
  "clients": [
    {
      "clientId": "web",
      "rootUrl": "https://old.example/",
      "baseUrl": "https://old.example/",
      "redirectUris": ["https://old.example/callback"],
      "webOrigins": ["https://old.example"]
    }
  ]
}
JSON

bash "$SCRIPT" --frontend-url new.example "$INPUT" "$OUTPUT" >/dev/null

[[ $(jq -r '.users | length' "$OUTPUT") -eq 2 ]]
[[ $(jq -r '.users[] | select(.username == "demo") | .enabled' "$OUTPUT") == false ]]
[[ $(jq -r '.users[] | select(.username == "service-account-api") | .username' "$OUTPUT") == service-account-api ]]
[[ $(jq -r '.users[] | select(.username == "demo") | .credentials[0].value | length' "$OUTPUT") -eq 32 ]]
[[ $(jq -r '.clients[0].secret | length' "$OUTPUT") -eq 32 ]]
[[ $(jq -r '.clients[0].rootUrl' "$OUTPUT") == https://new.example/ ]]
[[ $(jq -r '.clients[0].redirectUris[0]' "$OUTPUT") == https://new.example/callback ]]
[[ $(jq -r '.clients[0].webOrigins[0]' "$OUTPUT") == https://new.example ]]
[[ $(jq 'has("components") or has("eventsListeners")' "$OUTPUT") == false ]]

echo "Sanitizer regression tests passed."