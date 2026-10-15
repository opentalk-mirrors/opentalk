#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# Regression test: with the dev settings sample, containers must reach the
# public dev URLs (e.g. http://opentalk.dev.localhost:8081) through the Traefik
# network aliases instead of host-gateway. The host detour is dropped by host
# firewalls such as the NixOS default and made the controller time out on
# Keycloak.

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

for tool in ansible-playbook python3; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "$tool is required to run the dev settings routing test." >&2
    exit 1
  fi
done

if ! python3 -c 'import yaml' >/dev/null 2>&1; then
  echo "PyYAML is required to run the dev settings routing test." >&2
  exit 1
fi

ROLES_DIR="$REPO_ROOT/automation/roles"
FILTER_PLUGINS="$REPO_ROOT/automation/playbooks/filter_plugins"
DEV_SETTINGS="$REPO_ROOT/extras/packaging/config-dist/settings.dev.yml-sample"

mapfile -t template_files < <(find "$ROLES_DIR" -path '*/templates/compose.yml.j2' -type f | sort)
template_files+=("$ROLES_DIR/opentalk_compose_traefik/templates/traefik.yaml.j2")

INVENTORY="$TMP_DIR/inventory"
printf 'localhost ansible_connection=local ansible_python_interpreter=%s\n' "$(command -v python3)" > "$INVENTORY"

SECRETS="$TMP_DIR/secrets.yml"
python3 - "$ROLES_DIR" "$DEV_SETTINGS" "$SECRETS" <<'PY'
import re
import sys
from pathlib import Path

roles_dir, dev_settings, secrets_path = sys.argv[1], Path(sys.argv[2]), sys.argv[3]
pattern = re.compile(r"{{\s*([A-Z][A-Z0-9_]*)\s*}}")
sources = [dev_settings]
for role in Path(roles_dir).iterdir():
    sources += sorted(role.glob("defaults/*.yml")) + sorted(role.glob("vars/*.yml"))
secrets = set()
for path in sources:
    secrets.update(pattern.findall(path.read_text(encoding="utf-8")))
with Path(secrets_path).open("w", encoding="utf-8") as f:
    for secret in sorted(secrets):
        f.write(f"{secret}: stub\n")
PY

PLAYBOOK="$TMP_DIR/render.yml"
{
  cat <<'YAML'
---
- name: Render templates with the dev settings sample
  hosts: localhost
  connection: local
  gather_facts: false
  vars:
YAML
  printf '    role_defaults_files:\n'
  find "$ROLES_DIR" -path '*/defaults/main.yml' -type f | sort | sed 's/^/      - "/; s/$/"/'
  printf '    render_targets:\n'
  for template in "${template_files[@]}"; do
    name="$(basename "$(dirname "$(dirname "$template")")")_$(basename "$template" .j2)"
    printf '      - {"src": "%s", "dest": "%s"}\n' "$template" "$TMP_DIR/rendered/$name"
  done
  cat <<YAML
  tasks:
    - name: Load role defaults
      ansible.builtin.include_vars:
        file: "{{ item }}"
      loop: "{{ role_defaults_files }}"
    - name: Load dev settings sample
      ansible.builtin.include_vars:
        file: "$DEV_SETTINGS"
    - name: Render template
      ansible.builtin.template:
        src: "{{ item.src }}"
        dest: "{{ item.dest }}"
        mode: "0644"
      loop: "{{ render_targets }}"
YAML
} > "$PLAYBOOK"

mkdir -p "$TMP_DIR/rendered"
if ! OPENTALK_DEV_DIR="$TMP_DIR/dev" \
  ANSIBLE_FILTER_PLUGINS="$FILTER_PLUGINS" \
  ansible-playbook -i "$INVENTORY" -e "@$SECRETS" -e "@$REPO_ROOT/.versions.yml" "$PLAYBOOK" >"$TMP_DIR/render.log" 2>&1; then
  cat "$TMP_DIR/render.log" >&2
  echo "Ansible failed to render templates with the dev settings sample." >&2
  exit 1
fi

python3 - "$TMP_DIR/rendered" <<'PY'
import sys
from pathlib import Path

import yaml

rendered = Path(sys.argv[1])
traefik_compose = yaml.safe_load((rendered / "opentalk_compose_traefik_compose.yml").read_text())
traefik_config = yaml.safe_load((rendered / "opentalk_compose_traefik_traefik.yaml").read_text())
traefik = traefik_compose["services"]["traefik"]

aliases = set()
for network in traefik["networks"].values():
    aliases.update((network or {}).get("aliases", []))
if "opentalk.dev.localhost" not in aliases:
    sys.exit("Traefik must carry the opentalk.dev.localhost network alias in the dev settings.")

errors = []
for path in sorted(rendered.glob("*_compose.yml")):
    services = (yaml.safe_load(path.read_text()) or {}).get("services", {})
    for name, service in services.items():
        for entry in service.get("extra_hosts", []):
            host, _, target = entry.partition(":")
            if host in aliases:
                errors.append(f"{name}: extra_hosts entry '{entry}' shadows the Traefik alias {host}")

web_port = traefik_config["entryPoints"]["web"]["address"].rpartition(":")[2]
published = {str(p["target"]): str(p["published"]) for p in traefik["ports"]}
if published.get(web_port) != web_port:
    errors.append(
        f"Traefik web entrypoint listens on {web_port} but publishes {published}; "
        "the in-container port must equal the published ingress port"
    )

if errors:
    sys.exit("\n".join(errors))
PY

echo "Dev settings routing tests passed."
