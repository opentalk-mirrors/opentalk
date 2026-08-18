#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
ROLES_DIR="$REPO_ROOT/automation/roles"
MAIN_COMPOSE_ROLE="opentalk_compose_docker_env"

failed=0

while IFS= read -r template; do
  [[ -f "$template" ]] || continue

  first_line=$(head -n1 "$template")
  if [[ "$first_line" != '---' ]]; then
    echo "Compose template must start with the YAML document marker '---': $template" >&2
    failed=1
  fi

  role_name=$(basename "$(dirname "$(dirname "$template")")")
  if [[ "$role_name" != "$MAIN_COMPOSE_ROLE" ]] && ! grep -q '^services:' "$template"; then
    echo "Compose template must define a top-level 'services:' key: $template" >&2
    failed=1
  fi
done < <(find "$ROLES_DIR" -path '*/templates/compose.yml.j2' -type f | sort)

[[ $failed -eq 0 ]] || exit 1

echo "Compose template YAML consistency tests passed."