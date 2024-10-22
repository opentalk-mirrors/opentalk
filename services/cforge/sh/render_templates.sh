#!/bin/bash

generate_config() {
  local output_path
  local template_path
  local service_name
  service_name=$1
  service_path=$2
  output_path="${service_path}/${service_name}/${3}"
  template_path=$4

  echo "[INFO] Write $output_path"
  yq -nM "{\"$service_name\": load(\"$service_path/$service_name/values.yaml\")} * load(\"./values.yaml\")" | yq eval '{"Values": .}' - \
  	| tpl -d yaml --file "$template_path" > "$output_path" --option missingkey="$MISSING_KEY_BEHAVIOR"
  
  if [ $? -ne 0 ]; then
    echo "[ERROR]: render template $template_path failed."
    exit 1
  fi
}

# generate_config "service_name" "service_path" "output_path" "template_path"
generate_config "controller" "services" "config/controller.toml" "/templates/controller.toml.tpl"
generate_config "mailer" "services" "config/mailer.toml" "/templates/mailer.toml.tpl"
generate_config "recorder" "services" "config/recorder.toml" "/templates/recorder.toml.tpl"
generate_config "obelisk" "services" "config/obelisk.toml" "/templates/obelisk.toml.tpl" 
generate_config "keycloak" "services/3rd_party" "config/keycloak.conf" "/templates/keycloak.conf.tpl"
generate_config "keycloak" "services/3rd_party" "data/kc_data/import/export.json" "/templates/keycloak_export.json.tpl"
generate_config "livekit" "services/3rd_party" "config/livekit.yaml" "/templates/livekit.yaml.tpl"

