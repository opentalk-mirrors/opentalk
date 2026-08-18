#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

if [[ ${OPENTALK_E2E:-} != 1 ]]; then
  echo "Set OPENTALK_E2E=1 to run the destructive Compose end-to-end test." >&2
  exit 2
fi

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../../.." && pwd)
RUNTIME_TEST="$REPO_ROOT/extras/scripts/tests/integration/test_deployment_runtime.sh"
SMTP_RABBITMQ_TEST="$REPO_ROOT/extras/scripts/tests/integration/test_smtp_mailer_rabbitmq.sh"
BACKUP_RESTORE_TEST="$REPO_ROOT/extras/scripts/tests/integration/test_backup_restore_config.sh"

cleanup() {
  just --justfile "$REPO_ROOT/justfile" compose-clean || true
  mapfile -t volumes < <(docker volume ls -q --filter 'label=com.docker.compose.project=opentalk')
  if (( ${#volumes[@]} > 0 )); then
    docker volume rm "${volumes[@]}" || true
  fi
}

trap cleanup EXIT

cd "$REPO_ROOT"
cleanup
just compose-deploy
COMPOSE_PROJECT_NAME=opentalk bash "$RUNTIME_TEST"

for endpoint in \
  http://opentalk.dev.localhost:8081/ \
  http://whiteboard.dev.localhost:8081/whiteboard/ \
  http://localhost:8001/auth/; do
  status=$(curl --max-time 20 --silent --output /dev/null --write-out '%{http_code}' "$endpoint")
  if [[ $endpoint == *'/auth/' ]]; then
    [[ $status == 302 ]] || { echo "$endpoint returned HTTP $status." >&2; exit 1; }
  else
    [[ $status == 200 ]] || { echo "$endpoint returned HTTP $status." >&2; exit 1; }
  fi
done

COMPOSE_PROJECT_NAME=opentalk bash "$SMTP_RABBITMQ_TEST"
OPENTALK_E2E=1 bash "$BACKUP_RESTORE_TEST"
echo "Compose end-to-end tests passed."