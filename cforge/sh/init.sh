#!/bin/bash

if [ "$ENABLE_CFORGE" != "true" ]; then
  echo "cforge is disabled."
  exit 0
fi

if [ -z "$OT_DOMAIN" ]; then
  echo "[ERROR] Variable OT_DOMAIN is mandatory."
  exit 1
fi

 /usr/local/bin/secrets.sh
 /usr/local/bin/render_general_values_file.sh
 /usr/local/bin/render_templates.sh
