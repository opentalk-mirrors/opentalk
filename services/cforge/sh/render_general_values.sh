#!/bin/bash

# Load secrets from file if it exists
load_secrets() {
  if [ -e /data/.secrets ]; then
    set -o allexport
    # shellcheck source=/dev/null
    source /data/.secrets
    set +o allexport
  else
    echo "[ERROR] /data/.secrets file not found"
    exit 1
  fi
}

# Generate values.yaml if it doesn't exist or FORCE_CONFIG is true
generate_config() {
  if [ ! -e /tpl/values.yaml ] || [ "$FORCE_CONFIG" == "true" ]; then
    echo "[INFO] Create general values.yaml file from template"
    envsubst < /templates/values.yaml.tpl > /tpl/values.yaml
  else
    echo "[SKIP] General values.yaml file already present. Skip writing config."
  fi
}

# Create secret files if they don't exist or FORCE_SECRETS is true
create_secret_file() {
  local file_path="$1"
  local secret_value="$2"
  
  if [ ! -s "/tpl/$file_path" ] || [ "$FORCE_SECRETS" == "true" ]; then
    echo "[INFO] Create secrets file $file_path"
    echo "$secret_value" > "/tpl/$file_path"
  else
    echo "[SKIP] Secrets file $file_path already present"
  fi
}

# Main script
load_secrets
generate_config

create_secret_file "services/etherpad/.etherpad_api.secret" "$ETHERPAD_API_KEY"
create_secret_file "services/spacedeck/.spacedeck_api.secret" "$SPACEDECK_API_KEY"
create_secret_file "services/spacedeck/.spacedeck_invite.secret" "$SPACEDECK_INVITE_CODE"
create_secret_file "services/3rd_party/postgres/.db_controller.secret" "$CONTROLLER_DB_PASSWORD"
create_secret_file "services/3rd_party/postgres/.db_keycloak.secret" "$KEYCLOAK_DB_PASSWORD"
create_secret_file "services/3rd_party/minio/.minio_root.secret" "$MINIO_ROOT_PASSWORD"
