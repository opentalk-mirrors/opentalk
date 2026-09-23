#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

project_name=${COMPOSE_PROJECT_NAME:-opentalk}
health_timeout_seconds=${OPENTALK_HEALTH_TIMEOUT_SECONDS:-180}
mapfile -t containers < <(docker ps -aq --filter "label=com.docker.compose.project=$project_name")

if (( ${#containers[@]} == 0 )); then
  echo "No containers found for Compose project '$project_name'." >&2
  exit 1
fi

for container in "${containers[@]}"; do
  name=$(docker inspect --format '{{.Name}}' "$container")
  deadline=$((SECONDS + health_timeout_seconds))

  while :; do
    state=$(docker inspect --format '{{.State.Status}}' "$container")
    health=$(docker inspect --format '{{if .Config.Healthcheck}}{{.State.Health.Status}}{{end}}' "$container")

    if [[ $state != running ]]; then
      echo "$name is not running (state: $state)." >&2
      docker logs --tail 100 "$container" >&2 || true
      exit 1
    fi

    if [[ -z $health || $health == healthy ]]; then
      break
    fi

    if (( SECONDS >= deadline )); then
      echo "$name did not become healthy within ${health_timeout_seconds}s (health: $health)." >&2
      docker inspect --format '{{range .State.Health.Log}}{{.ExitCode}} {{.Output}}{{end}}' "$container" >&2 || true
      exit 1
    fi

    sleep 2
  done
done

echo "All containers for Compose project '$project_name' are running and healthy."