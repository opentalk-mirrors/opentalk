#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

if ! command -v ansible-playbook >/dev/null 2>&1; then
  echo "ansible-playbook is required to run the migrations test." >&2
  exit 1
fi

INVENTORY="$TMP_DIR/inventory"
printf 'localhost ansible_connection=local ansible_python_interpreter=%s\n' "$(command -v python3)" > "$INVENTORY"

GUARD_PLAYBOOK="$REPO_ROOT/extras/scripts/tests/fixtures/migrations_guard_playbook.yml"
STATUS_PLAYBOOK="$REPO_ROOT/automation/playbooks/migrate_status.yml"
SETTINGS="$TMP_DIR/settings.yml"

# docker stub: 'volume ls --quiet' prints OT_TEST_VOLUMES, one per line.
STUB_BIN="$TMP_DIR/bin"
mkdir -p "$STUB_BIN"
cat > "$STUB_BIN/docker" <<'STUB'
#!/usr/bin/env bash
if [[ "${OT_TEST_DOCKER_FAIL:-0}" == 1 ]]; then
  echo "Cannot connect to the Docker daemon" >&2
  exit 1
fi
if [[ "$1 $2" == "volume ls" ]]; then
  for v in ${OT_TEST_VOLUMES:-}; do printf '%s\n' "$v"; done
fi
STUB
chmod +x "$STUB_BIN/docker"

run_playbook() {
  local playbook=$1
  shift
  PATH="$STUB_BIN:$PATH" \
  ANSIBLE_ROLES_PATH="$REPO_ROOT/automation/roles" \
  OT_TEST_SETTINGS_FILE="$SETTINGS" \
  OT_SETTINGS_FILE="$SETTINGS" \
  ansible-playbook -i "$INVENTORY" "$playbook" "$@"
}

expect_guard() {
  local expected=$1 log=$2
  shift 2
  local rc=0
  run_playbook "$GUARD_PLAYBOOK" "$@" > "$log" 2>&1 || rc=$?
  if [[ "$expected" == pass && $rc -ne 0 ]]; then
    echo "Guard must pass, but failed:" >&2; cat "$log" >&2; exit 1
  fi
  if [[ "$expected" == block && $rc -eq 0 ]]; then
    echo "Guard must block, but passed:" >&2; cat "$log" >&2; exit 1
  fi
}

# --- fresh host: no volumes, nothing pending ---
printf 'compose_project_name: opentalk\n' > "$SETTINGS"
expect_guard pass "$TMP_DIR/fresh.log"

# --- legacy MinIO data without an explicit provider blocks the deploy ---
export OT_TEST_VOLUMES="opentalk_minio opentalk_rabbitmq"
expect_guard block "$TMP_DIR/legacy.log"
grep -q 'Deployment stopped before any change: 1 migration(s) required' "$TMP_DIR/legacy.log"
grep -q 'S3 storage provider not set' "$TMP_DIR/legacy.log"
grep -q "otctl migrate minio-to-garage" "$TMP_DIR/legacy.log"
grep -q "otctl migrate status" "$TMP_DIR/legacy.log"

# --- an explicit choice resolves it, in either spelling ---
printf 'compose_project_name: opentalk\ns3_storage_provider: garage\n' > "$SETTINGS"
expect_guard pass "$TMP_DIR/explicit-provider.log"
printf 'compose_project_name: opentalk\ncompose_service_minio_enabled: true\n' > "$SETTINGS"
expect_guard pass "$TMP_DIR/explicit-flag.log"

# --- volumes of another compose project are ignored ---
printf 'compose_project_name: other\n' > "$SETTINGS"
expect_guard pass "$TMP_DIR/other-project.log"

# --- the check can be disabled for a state the user verified ---
printf 'compose_project_name: opentalk\n' > "$SETTINGS"
expect_guard pass "$TMP_DIR/disabled.log" -e migration_check_enabled=false

# --- an unreachable docker daemon does not block the guard ---
OT_TEST_DOCKER_FAIL=1 expect_guard pass "$TMP_DIR/docker-down.log"

# --- status reports pending migrations and never fails ---
run_playbook "$STATUS_PLAYBOOK" > "$TMP_DIR/status-pending.log" 2>&1 || {
  cat "$TMP_DIR/status-pending.log" >&2; exit 1; }
grep -q "1 migration(s) pending" "$TMP_DIR/status-pending.log"
grep -q 'S3 storage provider not set' "$TMP_DIR/status-pending.log"

unset OT_TEST_VOLUMES
run_playbook "$STATUS_PLAYBOOK" > "$TMP_DIR/status-clean.log" 2>&1 || {
  cat "$TMP_DIR/status-clean.log" >&2; exit 1; }
grep -q "No migration pending" "$TMP_DIR/status-clean.log"

echo "Migration detection, guard and status tests passed."
