#!/bin/bash

generate_config() {
  local output_path=$1
  local template_path=$2
  local base_dir=$(echo "$output_path" | cut -d'/' -f1)
  
  echo "[INFO] Write $output_path"
  yq -nM "load(\"$base_dir/values.yaml\") * load(\"values.yaml\")" | yq eval '{"Values": .}' - \
  	| tpl -d yaml --file "$template_path" > "$output_path" --option missingkey="$MISSING_KEY_BEHAVIOR"
}

generate_config "keycloak/config/keycloak.conf" "keycloak/templates/keycloak.conf.tpl"
generate_config "keycloak/data/kc_data/import/export.json" "keycloak/templates/export.json.tpl"
generate_config "controller/config/controller.toml" "controller/templates/controller.toml.tpl"
generate_config "livekit/config/livekit.yaml" "livekit/templates/livekit.yaml.tpl"
generate_config "mailer/config/mailer.toml" "mailer/templates/mailer.toml.tpl"
generate_config "recorder/config/recorder.toml" "recorder/templates/recorder.toml.tpl"
generate_config "obelisk/config/obelisk.toml" "obelisk/templates/obelisk.toml.tpl"
