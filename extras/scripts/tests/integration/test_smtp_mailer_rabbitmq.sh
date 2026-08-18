#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

project_name=${COMPOSE_PROJECT_NAME:-opentalk}
mailer=$(docker ps -q --filter "label=com.docker.compose.project=$project_name" --filter 'label=com.docker.compose.service=smtp-mailer')
rabbitmq=$(docker ps -q --filter "label=com.docker.compose.project=$project_name" --filter 'label=com.docker.compose.service=rabbitmq')

if [[ -z $mailer || -z $rabbitmq ]]; then
  echo "SMTP mailer or RabbitMQ is not running for Compose project '$project_name'." >&2
  exit 1
fi

health=$(docker inspect --format '{{.State.Health.Status}}' "$mailer")
if [[ $health != healthy ]]; then
  echo "SMTP mailer is not healthy (health: $health)." >&2
  docker logs --tail 100 "$mailer" >&2 || true
  exit 1
fi

if ! docker exec "$rabbitmq" rabbitmqctl list_deprecated_features | grep -Eq 'transient_nonexcl_queues.*permitted'; then
  echo "RabbitMQ does not permit transient non-exclusive queues required by the SMTP mailer." >&2
  exit 1
fi

echo "SMTP mailer is healthy and connected to RabbitMQ."