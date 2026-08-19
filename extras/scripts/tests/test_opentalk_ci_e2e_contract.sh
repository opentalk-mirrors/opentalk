#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
CI_FILE="$REPO_ROOT/.gitlab-ci.yml"
E2E_TEST="$REPO_ROOT/extras/scripts/tests/integration/test_compose_e2e.sh"
RUNTIME_TEST="$REPO_ROOT/extras/scripts/tests/integration/test_deployment_runtime.sh"

[[ -f "$CI_FILE" ]] || { echo ".gitlab-ci.yml not found." >&2; exit 1; }
[[ -f "$E2E_TEST" ]] || { echo "test_compose_e2e.sh not found." >&2; exit 1; }

failed=0
fail() {
  echo "$*" >&2
  failed=1
}

# int:e2e CI job: the destructive E2E test must only run behind an explicit
# manual trigger and must set the OPENTALK_E2E guard that gates the test.
grep -q '^int:e2e:' "$CI_FILE" || fail "int:e2e job is missing from .gitlab-ci.yml"
grep -q 'OPENTALK_E2E: "1"' "$CI_FILE" || fail "int:e2e job must set OPENTALK_E2E=1"
grep -q 'when: manual' "$CI_FILE" || fail "int:e2e job must be gated by a manual trigger"
grep -q 'bash extras/scripts/tests/integration/test_compose_e2e.sh' "$CI_FILE" \
  || fail "int:e2e job must run test_compose_e2e.sh"
grep -q '!reference \[.int_before_script, before_script\]' "$CI_FILE" \
  || fail "int:e2e job must reuse the shared integration before_script"

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
grep -q 'OPENTALK_E2E_KEYCLOAK_PORT:-8002' "$E2E_TEST" \
  || fail "E2E default Keycloak port must be 8002"
# shellcheck disable=SC2016
grep -q 'compose_project_name:.*\$PROJECT_NAME' "$E2E_TEST" \
  || fail "E2E settings must inject the isolated Compose project name"
# shellcheck disable=SC2016
grep -q 'docker volume ls -q --filter "label=com.docker.compose.project=$PROJECT_NAME"' "$E2E_TEST" \
  || fail "E2E cleanup must only remove volumes of the isolated project"
# shellcheck disable=SC2016
grep -q 'export OPENTALK_DEV_DIR="$E2E_DEV_DIR"' "$E2E_TEST" \
  || fail "E2E test must point the deployment at its own dev dir"

# The runtime health test the E2E stack runs against must accept the project
# name from the environment.
grep -q 'COMPOSE_PROJECT_NAME' "$E2E_TEST" \
  || fail "E2E test must pass COMPOSE_PROJECT_NAME to the runtime test"
[[ -f "$RUNTIME_TEST" ]] || fail "test_deployment_runtime.sh not found"

[[ $failed -eq 0 ]] || exit 1

echo "CI E2E job and isolation contract tests passed."