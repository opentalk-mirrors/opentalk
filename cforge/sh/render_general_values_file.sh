#!/bin/bash

if [ -e /data/.secrets ]; then
  set -o allexport
  # shellcheck source=/dev/null
  source /data/.secrets
  set +o allexport
else
  echo "[ERROR] /data/.secrets file not found."
  exit 1
fi

if [ ! -e /tpl/values.yaml ] || [ "$FORCE_CONFIG" == "true" ]; then
  echo "[INFO] Create general values.yaml file from template"
    envsubst < /data/templates/values.yaml.tpl > /tpl/values.yaml
else
  echo "[SKIP] General values.yaml file already present. Skip writing config."
fi