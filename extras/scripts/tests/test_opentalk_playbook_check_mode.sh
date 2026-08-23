#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

if ! command -v ansible-playbook >/dev/null 2>&1; then
  echo "ansible-playbook is required to run the playbook check-mode test." >&2
  exit 1
fi

CONFIG_DIR="$TMP_DIR/config"
mkdir -p "$CONFIG_DIR/secrets"

cat > "$CONFIG_DIR/secrets/secrets.yml" <<'YAML'
CONTROLLER_DB_PASSWORD: stub-controller-db-password-32
KEYCLOAK_DB_PASSWORD: stub-keycloak-db-password-32
KC_BOOTSTRAP_ADMIN_PASSWORD: stub-kc-bootstrap-password-32
KC_PERMANENT_ADMIN_PASSWORD: stub-kc-permanent-password32
OIDC_FRONTEND_CLIENT_SECRET: stub-oidc-frontend-secret-32
OIDC_CONTROLLER_CLIENT_SECRET: stub-oidc-controller-sec-32
OIDC_RECORDER_CLIENT_SECRET: stub-oidc-recorder-secret-32
OIDC_OBELISK_CLIENT_SECRET: stub-oidc-obelisk-secret-32
ROOMSERVER_API_SECRET: stub-roomserver-api-secret-32
CONTROLLER_API_SECRET: stub-controller-api-secret32
ORCHESTRATOR_API_SECRET: stub-orchestrator-api-sec-32
LIVEKIT_API_SECRET: stub-livekit-api-secret-32
LIVEKIT_E2EE_SALT: stub-livekit-salt-22
SPACEDECK_API_KEY: stub-spacedeck-api-key-32
SPACEDECK_INVITE_CODE: stub-spacedeck-invite-32
ETHERPAD_API_KEY: stub-etherpad-api-key-32
RABBITMQ_PASSWORD: stub-rabbitmq-password-32
RABBITMQ_ERLANG_COOKIE: stub-rabbitmq-cookie-32
MINIO_ROOT_PASSWORD: stub-minio-root-password-32
MINIO_ACC_KEY: stub-minio-acc-key-32
MINIO_SECRET_KEY: stub-minio-secret-key-32
GARAGE_ADMIN_TOKEN: stub-garage-admin-token-32
GARAGE_RPC_SECRET: 0badc0de0badc0de0badc0de0badc0de
DEMO_USER_PASSWORD: stub-demo-user-password-32
TRAEFIK_DASHBOARD_ADMIN_PASSWORD: stub-traefik-admin-pass-32
TRAEFIK_DASHBOARD_ADMIN_PASSWORD_SALT: stub-traefik-salt16
YAML

SETTINGS="$TMP_DIR/settings.yml"
cat > "$SETTINGS" <<'YAML'
ot_domain: example.com
ot_ingress_port: 443
ot_ingress_protocol: https
config_dir: "{{ lookup('ansible.builtin.env', 'OT_TEST_CONFIG_DIR') }}"
services_dir: "{{ lookup('ansible.builtin.env', 'OT_TEST_SERVICES_DIR') }}"
deploy_dir: "{{ lookup('ansible.builtin.env', 'OT_TEST_DEPLOY_DIR') }}"
compose_project_name: opentalk-test
docker_network_name: opentalk-test-net
traefik_letsencrypt_enable: false
custom_certificates_enabled: false
s3_storage_provider: garage
keycloak_manage_enabled: false
self_signed_certificates_generate: false
compose_service_traefik_enabled: false
compose_service_keycloak_enabled: false
compose_service_garage_enabled: false
compose_service_minio_enabled: false
compose_service_rabbitmq_enabled: false
compose_service_mailcrab_enabled: false
compose_service_livekit_server_enabled: false
compose_service_spacedeck_enabled: false
compose_service_etherpad_enabled: false
compose_service_opentalk_controller_enabled: false
compose_service_opentalk_roomserver_enabled: false
compose_service_opentalk_orchestrator_enabled: false
compose_service_opentalk_web_frontend_enabled: false
compose_service_opentalk_recorder_enabled: false
compose_service_opentalk_obelisk_enabled: false
compose_service_opentalk_outlook_add_in_enabled: false
compose_service_opentalk_smtp_mailer_enabled: false
compose_service_ndt_enabled: false
YAML

INVENTORY="$TMP_DIR/inventory"
printf 'localhost ansible_connection=local ansible_python_interpreter=%s\n' "$(command -v python3)" > "$INVENTORY"

mkdir -p "$TMP_DIR/services" "$TMP_DIR/deploy"
cat > "$TMP_DIR/deploy/compose.yml" <<'YAML'
---
include:
  - path:
      - compose.seed.yml
YAML
: > "$TMP_DIR/deploy/common.yml"

COLLECTIONS="$REPO_ROOT/.ansible/collections"
FILTER_PLUGINS="$REPO_ROOT/automation/playbooks/filter_plugins"
PLAYBOOK="$REPO_ROOT/automation/playbooks/opentalk_compose.yml"

set +e
OT_SETTINGS_FILE="$SETTINGS" \
OT_TEST_CONFIG_DIR="$CONFIG_DIR" \
OT_TEST_SERVICES_DIR="$TMP_DIR/services" \
OT_TEST_DEPLOY_DIR="$TMP_DIR/deploy" \
ANSIBLE_COLLECTIONS_PATH="$COLLECTIONS" \
ANSIBLE_FILTER_PLUGINS="$FILTER_PLUGINS" \
ansible-playbook --check --skip-tags docker,s3 -i "$INVENTORY" -e "@$REPO_ROOT/.versions.yml" "$PLAYBOOK" >"$TMP_DIR/check.log" 2>&1
check_rc=$?
set -e

if [[ $check_rc -ne 0 ]]; then
  cat "$TMP_DIR/check.log" >&2
  echo "ansible-playbook --check failed for the OpenTalk compose playbook." >&2
  exit 1
fi

if grep -Eq "failed=[1-9]" "$TMP_DIR/check.log"; then
  tail -20 "$TMP_DIR/check.log" >&2
  echo "ansible-playbook --check reported failures." >&2
  exit 1
fi

echo "OpenTalk compose playbook check-mode tests passed."