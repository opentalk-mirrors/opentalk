#!/bin/bash

if [ "$ENABLE_CFORGE" != "true" ]; then
  echo "cforge is disabled."
  exit 0
fi

if [ -z "$OT_DOMAIN" ]; then
  echo "[ERROR] Env variable OT_DOMAIN is mandatory."
  exit 1
fi

cd /ansible || exit
ansible-playbook -i inventory.yaml playbooks/render_config.yaml --diff
