#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

for cmd in ansible-playbook jq python3; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "$cmd is required to run the secrets/certificates test." >&2
    exit 1
  fi
done

INVENTORY="$TMP_DIR/inventory"
printf 'localhost ansible_connection=local ansible_python_interpreter=%s\n' "$(command -v python3)" > "$INVENTORY"

COLLECTIONS="$REPO_ROOT/.ansible/collections"
FILTER_PLUGINS="$REPO_ROOT/automation/playbooks/filter_plugins"
ROLES_PATH="$REPO_ROOT/automation/roles"

CONFIG_DIR="$TMP_DIR/config"
SSL_DIR="$TMP_DIR/ssl"

run_secrets() {
  ANSIBLE_COLLECTIONS_PATH="$COLLECTIONS" \
  ANSIBLE_FILTER_PLUGINS="$FILTER_PLUGINS" \
  ANSIBLE_ROLES_PATH="$ROLES_PATH" \
  ansible-playbook -i "$INVENTORY" -e "config_dir=$CONFIG_DIR" \
    "$REPO_ROOT/extras/scripts/tests/fixtures/secrets_playbook.yml"
}

# --- Secrets: first run creates all secrets with correct length/charset ---
mkdir -p "$CONFIG_DIR"
run_secrets >"$TMP_DIR/secrets1.log" 2>&1

SECRETS_FILE="$CONFIG_DIR/secrets/secrets.yml"
[[ -f "$SECRETS_FILE" ]] || { echo "secrets.yml not written." >&2; exit 1; }
[[ $(stat -c '%a' "$SECRETS_FILE") == "600" ]] || { echo "secrets.yml mode must be 0600." >&2; exit 1; }

# All required keys present with correct length
check_keys() {
  python3 - "$SECRETS_FILE" <<'PY'
import sys, yaml
with open(sys.argv[1]) as f:
    secrets = yaml.safe_load(f)
required = {
    "CONTROLLER_DB_PASSWORD": 32,
    "KEYCLOAK_DB_PASSWORD": 32,
    "KC_BOOTSTRAP_ADMIN_PASSWORD": 32,
    "KC_PERMANENT_ADMIN_PASSWORD": 32,
    "OIDC_FRONTEND_CLIENT_SECRET": 32,
    "OIDC_CONTROLLER_CLIENT_SECRET": 32,
    "OIDC_RECORDER_CLIENT_SECRET": 32,
    "OIDC_OBELISK_CLIENT_SECRET": 32,
    "ROOMSERVER_API_SECRET": 32,
    "CONTROLLER_API_SECRET": 32,
    "ORCHESTRATOR_API_SECRET": 32,
    "LIVEKIT_API_SECRET": 32,
    "LIVEKIT_E2EE_SALT": 22,
    "SPACEDECK_API_KEY": 32,
    "SPACEDECK_INVITE_CODE": 32,
    "ETHERPAD_API_KEY": 32,
    "RABBITMQ_PASSWORD": 32,
    "RABBITMQ_ERLANG_COOKIE": 32,
    "MINIO_ROOT_PASSWORD": 32,
    "MINIO_ACC_KEY": 32,
    "MINIO_SECRET_KEY": 32,
    "GARAGE_ADMIN_TOKEN": 32,
    "DEMO_USER_PASSWORD": 32,
    "TRAEFIK_DASHBOARD_ADMIN_PASSWORD": 32,
    "TRAEFIK_DASHBOARD_ADMIN_PASSWORD_SALT": 16,
}
import string
for name, length in required.items():
    if name not in secrets:
        print(f"missing key {name}", file=sys.stderr); sys.exit(1)
    value = str(secrets[name])
    if len(value) != length:
        print(f"{name} has length {len(value)}, expected {length}", file=sys.stderr); sys.exit(1)
    if name == "GARAGE_RPC_SECRET":
        if not all(c in string.hexdigits and value == value.lower() for c in value):
            print("GARAGE_RPC_SECRET must be lowercase hex", file=sys.stderr); sys.exit(1)
        continue
    if not all(c in string.ascii_letters + string.digits for c in value):
        print(f"{name} contains disallowed characters", file=sys.stderr); sys.exit(1)
g = str(secrets.get("GARAGE_RPC_SECRET", ""))
if len(g) != 64 or not all(c in string.hexdigits for c in g) or g != g.lower():
    print("GARAGE_RPC_SECRET must be 64 lowercase hex chars", file=sys.stderr); sys.exit(1)
print("all secrets have expected length/charset")
PY
}
check_keys || { echo "secret key contract failed." >&2; exit 1; }

# Container secret files written with 0600 and matching value
for entry in "db_controller CONTROLLER_DB_PASSWORD" "garage_rpc GARAGE_RPC_SECRET" "etherpad_api ETHERPAD_API_KEY"; do
  name=${entry%% *}; key=${entry##* }
  f="$CONFIG_DIR/secrets/.$name.secret"
  [[ -f "$f" ]] || { echo "container secret file .$name.secret missing." >&2; exit 1; }
  [[ $(stat -c '%a' "$f") == "600" ]] || { echo ".$name.secret mode must be 0600." >&2; exit 1; }
  expected=$(python3 -c "import yaml;print(yaml.safe_load(open('$SECRETS_FILE'))['$key'])")
  actual=$(cat "$f")
  [[ "$actual" == "$expected" ]] || { echo ".$name.secret content mismatch." >&2; exit 1; }
done

# Existing secrets are preserved across reruns (idempotency)
before=$(sha256sum "$SECRETS_FILE" | cut -d' ' -f1)
run_secrets >"$TMP_DIR/secrets2.log" 2>&1
after=$(sha256sum "$SECRETS_FILE" | cut -d' ' -f1)
[[ "$before" == "$after" ]] || { echo "secrets.yml changed on idle rerun." >&2; exit 1; }

# --- Self-signed certificates: generation and idempotency ---
run_certs() {
  ANSIBLE_COLLECTIONS_PATH="$COLLECTIONS" \
  ANSIBLE_FILTER_PLUGINS="$FILTER_PLUGINS" \
  ANSIBLE_ROLES_PATH="$ROLES_PATH" \
  ansible-playbook -i "$INVENTORY" \
    -e '{"self_signed_certificates_generate": true, "self_signed_certificates_install_system_trust": false}' \
    -e "ot_domain=opentalk.test" \
    -e "custom_certificates_certs_dir=$SSL_DIR" \
    -e "self_signed_certificates_ca_key_size=2048" \
    -e "self_signed_certificates_server_key_size=2048" \
    "$REPO_ROOT/extras/scripts/tests/fixtures/self_signed_cert_playbook.yml"
}

run_certs >"$TMP_DIR/certs1.log" 2>&1

for f in ca.crt fullchain.crt privkey.key; do
  [[ -f "$SSL_DIR/$f" ]] || { echo "expected certificate file $f missing." >&2; tail -20 "$TMP_DIR/certs1.log" >&2; exit 1; }
done
[[ $(stat -c '%a' "$SSL_DIR/privkey.key") == "600" ]] || { echo "private key mode must be 0600." >&2; exit 1; }

# Key matches certificate in fullchain
if command -v openssl >/dev/null 2>&1; then
  key_mod=$(openssl pkey -in "$SSL_DIR/privkey.key" -pubout 2>/dev/null | openssl sha256)
  crt_mod=$(openssl x509 -in "$SSL_DIR/fullchain.crt" -pubkey -noout 2>/dev/null | openssl sha256)
  [[ "$key_mod" == "$crt_mod" ]] || { echo "certificate public key does not match private key." >&2; exit 1; }
  openssl x509 -in "$SSL_DIR/fullchain.crt" -noout -checkend 0 >/dev/null 2>&1 \
    || { echo "certificate is not valid." >&2; exit 1; }
fi

# CA is authoritative for the fullchain
if command -v openssl >/dev/null 2>&1; then
  openssl verify -CAfile "$SSL_DIR/ca.crt" "$SSL_DIR/fullchain.crt" >/dev/null 2>&1 \
    || { echo "fullchain does not verify against CA." >&2; exit 1; }
fi

cert1=$(sha256sum "$SSL_DIR/fullchain.crt" | cut -d' ' -f1)
run_certs >"$TMP_DIR/certs2.log" 2>&1
cert2=$(sha256sum "$SSL_DIR/fullchain.crt" | cut -d' ' -f1)
[[ "$cert1" == "$cert2" ]] || { echo "certificates regenerated on idle rerun (not idempotent)." >&2; exit 1; }

echo "Secrets and self-signed certificate tests passed."