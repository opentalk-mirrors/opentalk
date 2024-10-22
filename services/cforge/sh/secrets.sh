#!/bin/bash

if [ ! -e /data/.secrets ] || [ "$FORCE_SECRETS" == "true" ]; then
echo "[INFO] Generate .secrets file." 
cat > /data/.secrets << EOF
CONTROLLER_DB_PASSWORD="$(pwgen -s 32 1)"
KEYCLOAK_DB_PASSWORD="$(pwgen -s 32 1)"
KEYCLOAK_ADMIN_PASSWORD="$(pwgen -s 32 1)"
KEYCLOAK_WEBAPP_CLIENT_SECRET="$(pwgen -s 32 1)"
KEYCLOAK_CONTROLLER_CLIENT_SECRET="$(pwgen -s 32 1)"
KEYCLOAK_RECORDER_CLIENT_SECRET="$(pwgen -s 32 1)"
KEYCLOAK_OBELISK_CLIENT_SECRET="$(pwgen -s 32 1)"
LIVEKIT_API_SECRET="$(pwgen -s 32 1)"
SPACEDECK_API_KEY="$(pwgen -s 32 1)"
SPACEDECK_INVITE_CODE="$(pwgen -s 32 1)"
ETHERPAD_API_KEY="$(pwgen -s 32 1)"
RABBITMQ_PASSWORD="$(pwgen -s 32 1)"
RABBITMQ_ERLANG_COOKIE="$(pwgen -s 32 1)"
MINIO_ROOT_PASSWORD="$(pwgen -s 32 1)"
MINIO_ACC_KEY="$(pwgen -s 32 1)"
MINIO_SECRET_KEY="$(pwgen -s 32 1)"
EOF
else
  echo "[SKIP] .secrets file already present. Skip generating secrets." 
fi
