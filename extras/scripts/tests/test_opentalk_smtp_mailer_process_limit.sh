#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
COMPOSE_TEMPLATE="$REPO_ROOT/automation/roles/opentalk_compose_opentalk_smtp_mailer/templates/compose.yml.j2"
DEFAULTS="$REPO_ROOT/automation/roles/opentalk_compose_opentalk_smtp_mailer/defaults/main.yml"
REFERENCE="$REPO_ROOT/docs/6-configuration-options-reference.md"

if grep -q 'nproc:' "$COMPOSE_TEMPLATE"; then
	echo "SMTP mailer compose template must not set nproc." >&2
	exit 1
fi

if grep -q 'mailer_docker_compose_ulimits_nproc' "$DEFAULTS" "$REFERENCE"; then
	echo "SMTP mailer nproc setting must not be configured or documented." >&2
	exit 1
fi

echo "SMTP mailer process-limit regression tests passed."