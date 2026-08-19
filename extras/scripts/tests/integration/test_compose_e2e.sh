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

# The E2E stack must not collide with a running local development environment:
# it uses its own deployment directory, its own Compose project name, and its
# own published ports (all overridable) so containers, volumes, networks, and
# host ports stay separate from the local stack.
INGRESS_PORT=${OPENTALK_E2E_INGRESS_PORT:-8082}
KEYCLOAK_PORT=${OPENTALK_E2E_KEYCLOAK_PORT:-8002}
PROJECT_NAME=${OPENTALK_E2E_PROJECT_NAME:-opentalk-e2e}
E2E_DEV_DIR=${OPENTALK_E2E_DEV_DIR:-"$REPO_ROOT/build/opentalk-compose-e2e-dev"}
for port in "$INGRESS_PORT" "$KEYCLOAK_PORT"; do
  if [[ ! $port =~ ^[0-9]+$ ]]; then
    echo "Port override must be numeric, got: $port" >&2
    exit 1
  fi
done
if [[ -z $PROJECT_NAME ]]; then
  echo "Compose project name must not be empty (OPENTALK_E2E_PROJECT_NAME)." >&2
  exit 1
fi
export OPENTALK_DEV_DIR="$E2E_DEV_DIR"

cleanup() {
  just --justfile "$REPO_ROOT/justfile" compose-clean || true
  mapfile -t volumes < <(docker volume ls -q --filter "label=com.docker.compose.project=$PROJECT_NAME")
  if (( ${#volumes[@]} > 0 )); then
    docker volume rm "${volumes[@]}" || true
  fi
}

trap cleanup EXIT

cd "$REPO_ROOT"
cleanup

dev_root="$E2E_DEV_DIR"
settings_file="$dev_root/config/settings.yml"
mkdir -p "$(dirname "$settings_file")"
sed -e "s/^ot_ingress_port:.*/ot_ingress_port: $INGRESS_PORT/" \
    -e "s/^keycloak_local_port:.*/keycloak_local_port: $KEYCLOAK_PORT/" \
    -e "s/^compose_project_name:.*/compose_project_name: \"$PROJECT_NAME\"/" \
    -e "s#http://keycloak.localhost:8001#http://keycloak.localhost:$KEYCLOAK_PORT#g" \
    "$REPO_ROOT/extras/packaging/config-dist/settings.dev.yml-sample" > "$settings_file"

just compose-deploy
COMPOSE_PROJECT_NAME="$PROJECT_NAME" bash "$RUNTIME_TEST"

for endpoint in \
  "http://opentalk.dev.localhost:$INGRESS_PORT/" \
  "http://whiteboard.dev.localhost:$INGRESS_PORT/whiteboard/" \
  "http://localhost:$KEYCLOAK_PORT/auth/"; do
  status=$(curl --max-time 20 --silent --output /dev/null --write-out '%{http_code}' "$endpoint")
  if [[ $endpoint == *'/auth/' ]]; then
    [[ $status == 302 ]] || { echo "$endpoint returned HTTP $status." >&2; exit 1; }
  else
    [[ $status == 200 ]] || { echo "$endpoint returned HTTP $status." >&2; exit 1; }
  fi
done

COMPOSE_PROJECT_NAME="$PROJECT_NAME" bash "$SMTP_RABBITMQ_TEST"
OPENTALK_E2E=1 bash "$BACKUP_RESTORE_TEST"
echo "Compose end-to-end tests passed."