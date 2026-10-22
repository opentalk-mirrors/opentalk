#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
CI_FILE="$REPO_ROOT/.gitlab-ci.yml"
INTEGRATION_CI_FILE="$REPO_ROOT/extras/ci/gitlab/integration.yml"
INTEGRATION_SETTINGS_FILE="$REPO_ROOT/extras/ci/gitlab/integration-settings.yml"
E2E_TEST="$REPO_ROOT/extras/scripts/tests/integration/test_compose_e2e.sh"
RUNTIME_TEST="$REPO_ROOT/extras/scripts/tests/integration/test_deployment_runtime.sh"
JUSTFILE="$REPO_ROOT/justfile"

[[ -f "$CI_FILE" ]] || { echo ".gitlab-ci.yml not found." >&2; exit 1; }
[[ -f "$INTEGRATION_CI_FILE" ]] || { echo "extras/ci/gitlab/integration.yml not found." >&2; exit 1; }
[[ -f "$INTEGRATION_SETTINGS_FILE" ]] || { echo "extras/ci/gitlab/integration-settings.yml not found." >&2; exit 1; }
[[ -f "$E2E_TEST" ]] || { echo "test_compose_e2e.sh not found." >&2; exit 1; }
[[ -f "$JUSTFILE" ]] || { echo "justfile not found." >&2; exit 1; }

failed=0
fail() {
  echo "$*" >&2
  failed=1
}

# int:e2e CI job: the destructive E2E test must only run behind an explicit
# manual trigger and must set the OPENTALK_E2E guard that gates the test.
# Note: int:e2e is defined in extras/ci/gitlab/integration.yml and included by root CI file
grep -q '^int:e2e:' "$INTEGRATION_CI_FILE" || fail "int:e2e job is missing from extras/ci/gitlab/integration.yml"
grep -q 'OPENTALK_E2E: "1"' "$INTEGRATION_CI_FILE" || fail "int:e2e job must set OPENTALK_E2E=1"
grep -q 'when: manual' "$INTEGRATION_CI_FILE" || fail "int:e2e job must be gated by a manual trigger"
grep -q 'bash extras/scripts/tests/integration/test_compose_e2e.sh' "$INTEGRATION_CI_FILE" \
  || fail "int:e2e job must run test_compose_e2e.sh"
grep -q 'bash extras/ci/gitlab/setup-integration-env.sh' "$INTEGRATION_CI_FILE" \
  || fail "integration jobs must run setup-integration-env.sh"
grep -q 'bash extras/ci/gitlab/run-integration-test.sh' "$INTEGRATION_CI_FILE" \
  || fail "integration jobs must run run-integration-test.sh"
! grep -q 'OT_SETTINGS:' "$INTEGRATION_CI_FILE" \
  || fail "integration.yml must keep shared settings in integration-settings.yml"
grep -q 'compose_service_opentalk_smtp_mailer_enabled: false' "$INTEGRATION_SETTINGS_FILE" \
  || fail "integration-settings.yml must disable SMTP mailer by default"

# The E2E test must guard itself so it cannot run without explicit opt-in.
grep -q 'OPENTALK_E2E:-} != 1' "$E2E_TEST" \
  || fail "test_compose_e2e.sh must refuse to run without OPENTALK_E2E=1"

# Isolation contract: the E2E stack uses its own dev dir, Compose project name,
# and published ports, all overridable, so it never collides with the local
# development stack (build/opentalk-compose-dev, project "opentalk", 8081/8001).
grep -q 'OPENTALK_E2E_PROJECT_NAME:-opentalk-e2e' "$E2E_TEST" \
  || fail "E2E default project name must be opentalk-e2e"
# shellcheck disable=SC2016
grep -q 'OPENTALK_E2E_DEV_DIR:-"$REPO_ROOT/build/opentalk-compose-e2e-dev"' "$E2E_TEST" \
  || fail "E2E default dev dir must be build/opentalk-compose-e2e-dev"
grep -q 'OPENTALK_E2E_INGRESS_PORT:-8082' "$E2E_TEST" \
  || fail "E2E default ingress port must be 8082"
# Regression: the ingress port 8082 equals Traefik's default ping port, and
# Traefik restarted in a loop because it could not bind the port twice.
grep -qF 'traefik_entryPoints_ping_address: ":%s"' "$E2E_TEST" \
  || fail "E2E settings must set the Traefik ping entry point"
# shellcheck disable=SC2016 # The script variable must be matched literally.
grep -qF '"$((INGRESS_PORT + 1))" >> "$settings_file"' "$E2E_TEST" \
  || fail "E2E settings must move the Traefik ping entry point off the ingress port"
# shellcheck disable=SC2016 # The script variable must be matched literally.
grep -q "keycloak_local_hostname: ''" "$E2E_TEST" \
  || fail "E2E Keycloak URL must use the isolated ingress hostname"
# shellcheck disable=SC2016 # The script variable must be matched literally.
grep -q 'keycloak_local_port: \$INGRESS_PORT' "$E2E_TEST" \
  || fail "E2E Keycloak URL must use the isolated ingress port"
grep -q 'http://keycloak.localhost:8001#http://opentalk.dev.localhost' "$E2E_TEST" \
  || fail "E2E Keycloak and controller OIDC URLs must use the Traefik host"
# shellcheck disable=SC2016
grep -q 'compose_project_name:.*\$PROJECT_NAME' "$E2E_TEST" \
  || fail "E2E settings must inject the isolated Compose project name"
# shellcheck disable=SC2016
grep -q 'docker volume ls -q --filter "label=com.docker.compose.project=$PROJECT_NAME"' "$E2E_TEST" \
  || fail "E2E cleanup must only remove volumes of the isolated project"
# shellcheck disable=SC2016
grep -q 'export OPENTALK_DEV_DIR="$E2E_DEV_DIR"' "$E2E_TEST" \
  || fail "E2E test must point the deployment at its own dev dir"

# The ignored lock file is absent in a fresh CI checkout. compose-python must
# create it from the tracked input before compose-deploy can install Ansible.
# shellcheck disable=SC2016 # The recipe variables must be matched literally.
grep -q 'uv pip compile .*--output-file "\$requirements_lock" "\$requirements_in"' "$JUSTFILE" \
  || fail "compose-python must create the Ansible lock file when it is missing"
# shellcheck disable=SC2016 # The recipe variable must be matched literally.
grep -q 'ansible_python_interpreter=\$dev_root/.venv/bin/python' "$JUSTFILE" \
  || fail "compose deployment must run Ansible modules with its venv Python"

# The runtime health test the E2E stack runs against must accept the project
# name from the environment.
grep -q 'COMPOSE_PROJECT_NAME' "$E2E_TEST" \
  || fail "E2E test must pass COMPOSE_PROJECT_NAME to the runtime test"
[[ -f "$RUNTIME_TEST" ]] || fail "test_deployment_runtime.sh not found"

[[ $failed -eq 0 ]] || exit 1

echo "CI E2E job and isolation contract tests passed."