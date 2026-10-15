#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
PLAYBOOK="$REPO_ROOT/automation/playbooks/opentalk_compose.yml"
ROLES_DIR="$REPO_ROOT/automation/roles"

if find "$ROLES_DIR" -mindepth 1 -maxdepth 1 -type d -name 'opentalk_compose.*' | grep -q .; then
  echo "Legacy dotted role directories must not exist." >&2
  exit 1
fi

while IFS= read -r role; do
  if [[ ! -d "$ROLES_DIR/$role" ]]; then
    echo "Playbook references missing role: $role" >&2
    exit 1
  fi
done < <(awk '/^[[:space:]]+- role: opentalk_compose_/ { print $3 }' "$PLAYBOOK")

echo "OpenTalk role contract tests passed."