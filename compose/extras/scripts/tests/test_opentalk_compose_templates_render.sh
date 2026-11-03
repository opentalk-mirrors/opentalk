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

# Regression: ndt had no restart policy and stayed down after exiting, so
# every rendered service must come back on its own.
python3 - "$TMP_DIR" "${template_files[@]}" <<'PY' || exit 1
import sys
from pathlib import Path
import yaml
tmp_dir, missing = Path(sys.argv[1]), []
for template in sys.argv[2:]:
    role_name = Path(template).parent.parent.name
    with (tmp_dir / f"{role_name}.yml").open(encoding="utf-8") as f:
        services = (yaml.safe_load(f) or {}).get("services") or {}
    missing += [f"{role_name}: {name}" for name, service in services.items() if not service.get("restart")]
if missing:
    print("Services without a restart policy:", *missing, sep="\n  ", file=sys.stderr)
    sys.exit(1)
PY

KEYCLOAK_RENDERED="$TMP_DIR/opentalk_compose_keycloak.yml"
if grep -q '"443:8080"' "$KEYCLOAK_RENDERED"; then
  echo "Keycloak must not publish the ingress port when Traefik is enabled." >&2
  exit 1
fi
grep -q '"9080:9000"' "$KEYCLOAK_RENDERED" \
  || { echo "Keycloak management port must remain published." >&2; exit 1; }

# Regression: the database containers read their root:root 0640 password file
# through group 0 when the image starts as the postgres user (USER 70:70).
for pair in opentalk_compose_opentalk_controller:controller-db opentalk_compose_keycloak:keycloak-db; do
  python3 - "$TMP_DIR/${pair%%:*}.yml" "${pair#*:}" <<'PY' || { echo "${pair#*:} must join group 0 to read its password file." >&2; exit 1; }
import sys
import yaml
with open(sys.argv[1], encoding="utf-8") as f:
    service = yaml.safe_load(f)["services"][sys.argv[2]]
sys.exit(0 if [str(g) for g in service.get("group_add", [])] == ["0"] else 1)
PY
done

# Regression: the controller sees traefik as the peer of every request, so its
# metrics allowlist cannot keep /controller/metrics private. Traefik must deny
# the path, and the controller route must not catch "/controller%2Fmetrics".
python3 - "$TMP_DIR/opentalk_compose_opentalk_controller.yml" <<'PY' || exit 1
import sys
import yaml
with open(sys.argv[1], encoding="utf-8") as f:
    labels = dict(l.split("=", 1) for l in yaml.safe_load(f)["services"]["controller"]["labels"])
prefix = "traefik.http.routers."
errors = []
if "PathPrefix(`/controller/`)" not in labels.get(prefix + "controller.rule", ""):
    errors.append("controller router must match PathPrefix(`/controller/`) with the trailing slash")
if "PathPrefix(`/controller/metrics`)" not in labels.get(prefix + "controller-metrics.rule", ""):
    errors.append("controller-metrics router must match /controller/metrics")
# Routers without a priority get the length of their rule.
priorities = [int(v) for k, v in labels.items() if k.startswith(prefix) and k.endswith(".priority")]
priorities.append(len(labels.get(prefix + "controller.rule", "")) + 1)
if int(labels.get(prefix + "controller-metrics.priority", 0)) != max(priorities, default=0):
    errors.append("controller-metrics router must have the highest priority")
deny = labels.get(prefix + "controller-metrics.middlewares", "")
if labels.get(f"traefik.http.middlewares.{deny}.ipallowlist.sourcerange") != "127.0.0.1/32":
    errors.append("controller-metrics router must deny everything but traefik itself")
if errors:
    print("Controller metrics exposure:", *errors, sep="\n  ", file=sys.stderr)
    sys.exit(1)
PY

# Regression: obelisk calls the controller's /internal API, which Traefik does
# not route, so the default must reach the controller directly.
OBELISK_TEMPLATE="$ROLES_DIR/opentalk_compose_opentalk_obelisk/templates/obelisk.toml.j2"
OBELISK_PLAYBOOK="$TMP_DIR/render-obelisk.yml"
cat > "$OBELISK_PLAYBOOK" <<YAML
---
- name: Render obelisk config
  hosts: localhost
  connection: local
  gather_facts: false
  tasks:
    - name: Load obelisk defaults
      ansible.builtin.include_vars:
        file: "$ROLES_DIR/opentalk_compose_opentalk_obelisk/defaults/main.yml"
    - name: Render obelisk.toml
      ansible.builtin.template:
        src: "$OBELISK_TEMPLATE"
        dest: "{{ obelisk_dest }}"
        mode: "0644"
YAML

render_obelisk() {
  local dest=$1
  shift
  ANSIBLE_COLLECTIONS_PATH="$COLLECTIONS" \
  ansible-playbook -i "$INVENTORY" -e "@$SETTINGS" -e "@$SECRETS" -e "obelisk_dest=$dest" "$@" "$OBELISK_PLAYBOOK" >"$TMP_DIR/render-obelisk.log" 2>&1 \
    || { cat "$TMP_DIR/render-obelisk.log" >&2; echo "Ansible failed to render obelisk.toml." >&2; exit 1; }
}

assert_obelisk_url() {
  local file=$1 expected=$2
  grep -qxF "url = \"$expected\"" "$file" \
    || { echo "obelisk.toml: expected controller url $expected, got:" >&2; grep '^url' "$file" >&2; exit 1; }
}

render_obelisk "$TMP_DIR/obelisk-default.toml"
assert_obelisk_url "$TMP_DIR/obelisk-default.toml" "http://localhost:11311/"

render_obelisk "$TMP_DIR/obelisk-bridge.toml" -e obelisk_docker_network_mode=bridge
assert_obelisk_url "$TMP_DIR/obelisk-bridge.toml" "http://host.docker.internal:11311/"

render_obelisk "$TMP_DIR/obelisk-legacy.toml" -e '{"obelisk_controller_domain": "controller.example.com", "obelisk_controller_insecure": false}'
assert_obelisk_url "$TMP_DIR/obelisk-legacy.toml" "https://controller.example.com/"

# Regression: in the host network every Docker veth adds an fe80:: ICE
# candidate, and their zone lookups stall LiveKit (livekit/livekit#4591).
LIVEKIT_PLAYBOOK="$TMP_DIR/render-livekit.yml"
LIVEKIT_RENDERED="$TMP_DIR/livekit.yaml"
cat > "$LIVEKIT_PLAYBOOK" <<YAML
---
- name: Render livekit config
  hosts: localhost
  connection: local
  gather_facts: false
  tasks:
    - name: Load livekit defaults
      ansible.builtin.include_vars:
        file: "$ROLES_DIR/opentalk_compose_livekit_server/defaults/main.yml"
    - name: Render livekit.yaml
      ansible.builtin.template:
        src: "$ROLES_DIR/opentalk_compose_livekit_server/templates/livekit.yaml.j2"
        dest: "$LIVEKIT_RENDERED"
        mode: "0644"
YAML

ANSIBLE_COLLECTIONS_PATH="$COLLECTIONS" \
ansible-playbook -i "$INVENTORY" -e "@$SETTINGS" -e "@$SECRETS" "$LIVEKIT_PLAYBOOK" >"$TMP_DIR/render-livekit.log" 2>&1 \
  || { cat "$TMP_DIR/render-livekit.log" >&2; echo "Ansible failed to render livekit.yaml." >&2; exit 1; }

python3 - "$LIVEKIT_RENDERED" <<'PY' || { echo "livekit.yaml must exclude link-local IPv6 (fe80::/10) from ICE candidates." >&2; exit 1; }
import sys
import yaml
with open(sys.argv[1], encoding="utf-8") as f:
    excludes = ((yaml.safe_load(f).get("rtc") or {}).get("ips") or {}).get("excludes") or []
sys.exit(0 if "fe80::/10" in excludes else 1)
PY

echo "Compose template rendering tests passed (${#template_files[@]} templates)."
