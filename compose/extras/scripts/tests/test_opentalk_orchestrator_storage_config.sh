#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

# Regression: the rendered orchestrator.toml must always carry a [storage]
# section. The orchestrator binary requires it and crash-loops without one.

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

for cmd in ansible-playbook python3; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "$cmd is required to run the orchestrator storage config test." >&2
    exit 1
  fi
done

INVENTORY="$TMP_DIR/inventory"
printf 'localhost ansible_connection=local ansible_python_interpreter=%s\n' "$(command -v python3)" > "$INVENTORY"

COLLECTIONS="$REPO_ROOT/.ansible/collections"
FILTER_PLUGINS="$REPO_ROOT/automation/playbooks/filter_plugins"
RENDERED_CONFIG="$TMP_DIR/orchestrator.toml"

ANSIBLE_COLLECTIONS_PATH="$COLLECTIONS" \
ANSIBLE_FILTER_PLUGINS="$FILTER_PLUGINS" \
ansible-playbook -i "$INVENTORY" \
  -e "roles_dir=$REPO_ROOT/automation/roles" \
  -e "rendered_config=$RENDERED_CONFIG" \
  "$REPO_ROOT/extras/scripts/tests/fixtures/orchestrator_config_playbook.yml" \
  >"$TMP_DIR/render.log" 2>&1 \
  || { cat "$TMP_DIR/render.log" >&2; echo "Failed to render orchestrator.toml." >&2; exit 1; }

[[ -f "$RENDERED_CONFIG" ]] || { echo "orchestrator.toml was not rendered." >&2; exit 1; }

python3 - "$RENDERED_CONFIG" <<'PY'
import sys
import tomllib

with open(sys.argv[1], "rb") as f:
    config = tomllib.load(f)

storage = config.get("storage")
if not isinstance(storage, dict) or "kind" not in storage:
    print("Rendered orchestrator.toml is missing a [storage] section with a 'kind' key.", file=sys.stderr)
    sys.exit(1)
PY

echo "Orchestrator storage config tests passed."
