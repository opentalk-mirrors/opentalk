#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

for tool in ansible-playbook python3; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "$tool is required to run the compose template rendering test." >&2
    exit 1
  fi
done

if ! python3 -c 'import yaml' >/dev/null 2>&1; then
  echo "PyYAML is required to run the compose template rendering test." >&2
  exit 1
fi

ROLES_DIR="$REPO_ROOT/automation/roles"
COLLECTIONS="$REPO_ROOT/.ansible/collections"
FILTER_PLUGINS="$REPO_ROOT/automation/playbooks/filter_plugins"

mapfile -t template_files < <(find "$ROLES_DIR" -path '*/templates/compose.yml.j2' -type f | sort)
(( ${#template_files[@]} > 0 )) || { echo "No compose templates found." >&2; exit 1; }

declare -a default_files=()
while IFS= read -r role; do
  [[ -f "$role/defaults/main.yml" ]] && default_files+=("$role/defaults/main.yml")
done < <(find "$ROLES_DIR" -mindepth 1 -maxdepth 1 -type d | sort)

declare -a role_vars_files=()
while IFS= read -r role; do
  [[ -f "$role/vars/main.yml" ]] && role_vars_files+=("$role/vars/main.yml")
done < <(find "$ROLES_DIR" -mindepth 1 -maxdepth 1 -type d | sort)

INVENTORY="$TMP_DIR/inventory"
printf 'localhost ansible_connection=local ansible_python_interpreter=%s\n' "$(command -v python3)" > "$INVENTORY"

SETTINGS="$TMP_DIR/settings.yml"
cat > "$SETTINGS" <<'YAML'
ot_domain: example.com
ot_ingress_port: 443
ot_ingress_protocol: https
config_dir: test/config
services_dir: test/services
deploy_dir: test/deploy
compose_project_name: opentalk
docker_network_name: opentalk-net
compose_service_traefik_enabled: true
traefik_letsencrypt_enable: false
custom_certificates_enabled: false
YAML

SECRETS="$TMP_DIR/secrets.yml"
python3 - "$ROLES_DIR" "$SECRETS" <<'PY'
import re
import sys
from pathlib import Path

roles_dir, secrets_path = sys.argv[1], sys.argv[2]
pattern = re.compile(r"{{\s*([A-Z][A-Z0-9_]*)\s*}}")
secrets = set()
for role in Path(roles_dir).iterdir():
    for yml in sorted(role.glob("defaults/*.yml")) + sorted(role.glob("vars/*.yml")):
        secrets.update(pattern.findall(yml.read_text(encoding="utf-8")))
with Path(secrets_path).open("w", encoding="utf-8") as f:
    for secret in sorted(secrets):
        f.write(f"{secret}: stub\n")
PY

PLAYBOOK="$TMP_DIR/render.yml"
{
  cat <<'YAML'
---
- name: Render compose templates
  hosts: localhost
  connection: local
  gather_facts: false
  vars:
YAML
  printf '    role_defaults_files:\n'
  for f in "${default_files[@]}"; do
    printf '      - "%s"\n' "$f"
  done
  printf '    role_vars_files:\n'
  for f in "${role_vars_files[@]}"; do
    printf '      - "%s"\n' "$f"
  done
  printf '    render_targets:\n'
  for template in "${template_files[@]}"; do
    role_name=$(basename "$(dirname "$(dirname "$template")")")
    printf '      - {"src": "%s", "dest": "%s"}\n' "$template" "$TMP_DIR/$role_name.yml"
  done
  cat <<'YAML'
  tasks:
    - name: Load role defaults
      ansible.builtin.include_vars:
        file: "{{ item }}"
      loop: "{{ role_defaults_files }}"
    - name: Load role vars
      ansible.builtin.include_vars:
        file: "{{ item }}"
      loop: "{{ role_vars_files }}"
    - name: Render compose template
      ansible.builtin.template:
        src: "{{ item.src }}"
        dest: "{{ item.dest }}"
      loop: "{{ render_targets }}"
YAML
} > "$PLAYBOOK"

set +e
ANSIBLE_COLLECTIONS_PATH="$COLLECTIONS" \
ANSIBLE_FILTER_PLUGINS="$FILTER_PLUGINS" \
ansible-playbook -i "$INVENTORY" -e "@$SETTINGS" -e "@$SECRETS" -e "@$REPO_ROOT/.versions.yml" "$PLAYBOOK" >"$TMP_DIR/render.log" 2>&1
render_rc=$?
set -e

if [[ $render_rc -ne 0 ]]; then
  cat "$TMP_DIR/render.log" >&2
  echo "Ansible failed to render one or more compose templates." >&2
  exit 1
fi

for template in "${template_files[@]}"; do
  role_name=$(basename "$(dirname "$(dirname "$template")")")
  rendered="$TMP_DIR/$role_name.yml"
  [[ -f "$rendered" ]] || { echo "Template was not rendered: $template" >&2; exit 1; }
  python3 - "$rendered" <<'PY'
import sys
import yaml
with open(sys.argv[1], encoding="utf-8") as f:
    yaml.safe_load(f)
PY
done

KEYCLOAK_RENDERED="$TMP_DIR/opentalk_compose_keycloak.yml"
if grep -q '"443:8080"' "$KEYCLOAK_RENDERED"; then
  echo "Keycloak must not publish the ingress port when Traefik is enabled." >&2
  exit 1
fi
grep -q '"9080:9000"' "$KEYCLOAK_RENDERED" \
  || { echo "Keycloak management port must remain published." >&2; exit 1; }

echo "Compose template rendering tests passed (${#template_files[@]} templates)."
