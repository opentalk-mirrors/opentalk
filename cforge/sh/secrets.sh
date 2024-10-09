#!/bin/bash

if [ ! -e /data/.secrets ] || [ "$FORCE_SECRETS" == "true" ]; then
echo "[INFO] Generate .secrets file." 
cat > /data/.secrets << EOF
CONTROLLER_DB_PASSWORD="$(pwgen -s 32 1)"
KEYCLOAK_DB_PASSWORD="$(pwgen -s 32 1)"
KEYCLOAK_ADMIN_PASSWORD="$(pwgen -s 32 1)"
KEYCLOAK_CLIENT_SECRET_FRONTEND="$(pwgen -s 32 1)"
KEYCLOAK_CLIENT_SECRET_BACKEND="$(pwgen -s 32 1)"
KEYCLOAK_CLIENT_SECRET_RECORDER="$(pwgen -s 32 1)"
KEYCLOAK_CLIENT_SECRET_OBELISK="$(pwgen -s 32 1)"
WHITEBOARD_API_KEY="$(pwgen -s 32 1)"
WHITEBOARD_API_TOKEN_INVITE_CODE="$(pwgen -s 32 1)"
ETHERPAD_API_KEY="$(pwgen -s 32 1)"
RABBITMQ_PASSWORD="$(pwgen -s 32 1)"
RABBITMQ_ERLANG_COOKIE="$(pwgen -s 32 1)"
MINIO_ACC_KEY="$(pwgen -s 32 1)"
MINIO_SECRET_KEY="$(pwgen -s 32 1)"
EOF
else
  echo "[SKIP] .secrets file already present. Skip generating secrets." 
fi