#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TASKS="$REPO_ROOT/automation/roles/opentalk_compose_keycloak/tasks"

# Every write operation in the realm-management task chain must be idempotent
# (safe against repeated runs). This test pins the structural contracts that
# provide idempotency:
#   - realm creation only when a GET check returned 404
#   - realm/client/user-profile writes accept 409 as a "already exists" alias
#   - client management goes through community.general modules with state: present

fail=0
check() {
  local desc=$1
  local needle=$2
  local file=$3
  if ! grep -q -- "$needle" "$file"; then
    echo "FAIL: $desc" >&2
    echo "      missing pattern: $needle" >&2
    echo "      in: $file" >&2
    fail=1
  fi
}

# 1) Realm creation must be gated on the 404 existence check (no blind POST).
check "realm creation is gated on 404 existence check" \
  "when: realm_check.status == 404" \
  "$TASKS/keycloak_realm.yml"

# 2) Realm update accepts 200/204, so a no-op PUT does not fail a re-run.
check "realm config update accepts 200/204 on re-runs" \
  "status_code: \[200, 204\]" \
  "$TASKS/keycloak_realm.yml"

# 3) Client session timeout reset is defensive (accepts 200/204 as well).
check "client session timeout reset accepts 200/204" \
  "status_code: \[200, 204\]" \
  "$TASKS/keycloak_realm.yml"

# 4) User profile provider POST is idempotent (201 create / 409 exists).
check "user profile provider POST accepts 201/409" \
  "status_code: \[201, 409\]" \
  "$TASKS/keycloak_user_profile.yml"

# 5) Client scope protocol mapper POSTs are idempotent.
check "client scope protocol mapper POST accepts 201/409" \
  "status_code: \[201, 409\]" \
  "$TASKS/keycloak_client_scopes.yml"

# 6) Default scope assignment PUT is idempotent (204 ok / 409 already assigned).
check "default scope assignment PUT accepts 204/409" \
  "status_code: \[204, 409\]" \
  "$TASKS/keycloak_client_scopes.yml"

# 7) OIDC clients use the module form with state: present (idempotent design).
for client in backend frontend outlook recorder realm-management; do
  file="$TASKS/keycloak_oidc_client_${client}.yml"
  if ! grep -q 'community.general.keycloak_client:' "$file"; then
    echo "FAIL: $file must manage the client via community.general.keycloak_client." >&2
    fail=1
  fi
  if ! grep -q '    state: present' "$file"; then
    echo "FAIL: $file must declare state: present for idempotent client management." >&2
    fail=1
  fi
done

# 8) Realm roles are declared idempotently via state: present.
check "realm roles use state: present" \
  '    state: present' \
  "$TASKS/keycloak_realm_roles.yml"

if [[ $fail -ne 0 ]]; then
  echo "Keycloak realm idempotency contract checks failed." >&2
  exit 1
fi

echo "Keycloak realm idempotency contract checks passed."