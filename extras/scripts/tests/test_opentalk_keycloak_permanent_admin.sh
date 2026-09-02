#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TASK="$REPO_ROOT/automation/roles/opentalk_compose_keycloak/tasks/keycloak_permanent_admin.yml"

permanent_admin_exists_task=$(awk '
  BEGIN { in_task=0 }
  /Check permanent admin exists \|/ { in_task=1 }
  in_task && /- name:/ && !/Check permanent admin exists \|/ { exit }
  in_task { print }
' "$TASK")

if ! grep -q 'until: permanent_admin_login_attempt.status in \[200, 400, 401\]' <<<"$permanent_admin_exists_task"; then
  echo "Check permanent admin task must accept 400 (invalid_grant) alongside 200/401." >&2
  exit 1
fi

echo "Keycloak permanent admin task contract passed."