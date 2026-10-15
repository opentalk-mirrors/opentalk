#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

for cmd in ansible-playbook jq python3; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "$cmd is required to run the S3 migration test." >&2
    exit 1
  fi
done

INVENTORY="$TMP_DIR/inventory"
printf 'localhost ansible_connection=local ansible_python_interpreter=%s\n' "$(command -v python3)" > "$INVENTORY"

COLLECTIONS="$REPO_ROOT/.ansible/collections"
FILTER_PLUGINS="$REPO_ROOT/automation/playbooks/filter_plugins"
ROLES_PATH="$REPO_ROOT/automation/roles"
PLAYBOOK="$REPO_ROOT/extras/scripts/tests/fixtures/s3_migrate_playbook.yml"

run_logic() {
  local result=$1
  shift
  ANSIBLE_COLLECTIONS_PATH="$COLLECTIONS" \
  ANSIBLE_FILTER_PLUGINS="$FILTER_PLUGINS" \
  ANSIBLE_ROLES_PATH="$ROLES_PATH" \
  OT_TEST_SETTINGS_FILE="$SETTINGS" \
  OT_TEST_RESULT_FILE="$result" \
  ansible-playbook --tags s3_migrate_logic \
    --skip-tags s3_migrate_integration \
    -i "$INVENTORY" "$PLAYBOOK" "$@"
}

expect_fail() {
  local log=$1
  local pattern=$2
  shift 2
  local rc=0
  "$@" >"$log" 2>&1 || rc=$?
  if [[ $rc -eq 0 ]]; then
    echo "Expected failure with '$pattern', but run succeeded." >&2
    cat "$log" >&2
    exit 1
  fi
  if ! grep -q "$pattern" "$log"; then
    echo "Expected failure message missing: $pattern" >&2
    cat "$log" >&2
    exit 1
  fi
}

SETTINGS="$TMP_DIR/settings.yml"
cat > "$SETTINGS" <<'YAML'
s3_storage_provider: minio
compose_service_minio_enabled: true
compose_service_garage_enabled: false
YAML

# --- minio -> garage: destination and endpoints derived, settings updated ---
RESULT="$TMP_DIR/minio-garage.json"
run_logic "$RESULT" -e "s3_migration_source=minio" -e "config_dir=$TMP_DIR"
[[ $(jq -r '.destination' "$RESULT") == "garage" ]] || { echo "wrong destination." >&2; exit 1; }
[[ $(jq -r '.source' "$RESULT") == "minio" ]] || { echo "wrong source." >&2; exit 1; }
[[ $(jq -r '.source_endpoint' "$RESULT") == "http://127.0.0.1:9000" ]] || { echo "wrong minio endpoint." >&2; exit 1; }
[[ $(jq -r '.dest_endpoint' "$RESULT") == "http://127.0.0.1:3900" ]] || { echo "wrong garage endpoint." >&2; exit 1; }

grep -q "^s3_storage_provider: garage" "$SETTINGS" || { echo "settings s3_storage_provider not switched to garage." >&2; exit 1; }
grep -q "^compose_service_minio_enabled: false" "$SETTINGS" || { echo "minio service not disabled." >&2; exit 1; }

# --- garage -> minio: inverse direction ---
cat > "$SETTINGS" <<'YAML'
s3_storage_provider: garage
compose_service_minio_enabled: false
compose_service_garage_enabled: true
YAML
RESULT="$TMP_DIR/garage-minio.json"
run_logic "$RESULT" -e "s3_migration_source=garage" -e "config_dir=$TMP_DIR"
[[ $(jq -r '.destination' "$RESULT") == "minio" ]] || { echo "wrong destination (garage->minio)." >&2; exit 1; }
grep -q "^s3_storage_provider: minio" "$SETTINGS" || { echo "settings s3_storage_provider not switched to minio." >&2; exit 1; }
grep -q "^compose_service_garage_enabled: false" "$SETTINGS" || { echo "garage service not disabled." >&2; exit 1; }
grep -q "^compose_service_minio_enabled: true" "$SETTINGS" || { echo "minio must be enabled as the new destination." >&2; exit 1; }

# --- append-if-absent: disabling source line missing from settings ---
cat > "$SETTINGS" <<'YAML'
s3_storage_provider: minio
compose_service_garage_enabled: true
YAML
run_logic "$TMP_DIR/append.json" -e "s3_migration_source=minio" -e "config_dir=$TMP_DIR"
grep -q "^compose_service_minio_enabled: false" "$SETTINGS" || { echo "missing minio enable line not appended." >&2; exit 1; }

# --- an explicitly disabled destination is enabled, or the next deploy stops it ---
cat > "$SETTINGS" <<'YAML'
s3_storage_provider: minio
compose_service_minio_enabled: true
compose_service_garage_enabled: false
YAML
run_logic "$TMP_DIR/dest-disabled.json" -e "s3_migration_source=minio" -e "config_dir=$TMP_DIR"
grep -q "^compose_service_garage_enabled: true" "$SETTINGS" || { echo "destination not enabled in settings." >&2; exit 1; }
! grep -q "^compose_service_garage_enabled: false" "$SETTINGS" || { echo "stale destination line left behind." >&2; exit 1; }

# --- a missing destination line is appended ---
cat > "$SETTINGS" <<'YAML'
compose_service_minio_enabled: true
YAML
run_logic "$TMP_DIR/dest-missing.json" -e "s3_migration_source=minio" -e "config_dir=$TMP_DIR"
grep -q "^compose_service_garage_enabled: true" "$SETTINGS" || { echo "destination line not appended." >&2; exit 1; }

# --- invalid source rejected ---
cat > "$SETTINGS" <<'YAML'
s3_storage_provider: minio
YAML
expect_fail "$TMP_DIR/invalid.log" "s3_migration_source must be 'minio' or 'garage'" \
  run_logic "$TMP_DIR/invalid.json" -e "s3_migration_source=ftp"

# --- copy runs with the controller stopped (stubbed docker and rclone) ---
STUB_BIN="$TMP_DIR/bin"
CALLS="$TMP_DIR/calls.log"
mkdir -p "$STUB_BIN" "$TMP_DIR/secrets"
cat > "$TMP_DIR/secrets/secrets.yml" <<'YAML'
GARAGE_S3_ACCESS_KEY: test-access
GARAGE_S3_SECRET_KEY: test-secret
MINIO_ROOT_PASSWORD: test-minio
YAML
cat > "$STUB_BIN/docker" <<'STUB'
#!/usr/bin/env bash
printf 'docker %s\n' "$*" >> "$OT_TEST_CALLS"
if [[ "$1" == ps && "${OT_TEST_CONTROLLER_RUNNING:-1}" == 1 ]]; then
  printf 'abc123\n'
fi
STUB
cat > "$STUB_BIN/rclone" <<'STUB'
#!/usr/bin/env bash
printf 'rclone %s\n' "$1" >> "$OT_TEST_CALLS"
case "$1" in
  sync) [[ "${OT_TEST_RCLONE_FAIL:-0}" == 1 ]] && exit 1; printf 'Transferred: 2 / 2\n' ;;
  ls) printf '1 a\n2 b\n' ;;
esac
STUB
chmod +x "$STUB_BIN/docker" "$STUB_BIN/rclone"

run_sync() {
  local result=$1
  shift
  : > "$CALLS"
  PATH="$STUB_BIN:$PATH" OT_TEST_CALLS="$CALLS" \
  ANSIBLE_COLLECTIONS_PATH="$COLLECTIONS" \
  ANSIBLE_FILTER_PLUGINS="$FILTER_PLUGINS" \
  ANSIBLE_ROLES_PATH="$ROLES_PATH" \
  OT_TEST_SETTINGS_FILE="$SETTINGS" \
  OT_TEST_RESULT_FILE="$result" \
  ansible-playbook --tags s3_migrate_logic \
    --skip-tags s3_migrate_preflight \
    -i "$INVENTORY" "$PLAYBOOK" \
    -e "s3_migration_source=minio" -e "config_dir=$TMP_DIR" \
    -e "s3_migrate_run_deploy=false" "$@"
}

write_minio_settings() {
  cat > "$SETTINGS" <<'YAML'
s3_storage_provider: minio
compose_service_minio_enabled: true
compose_service_garage_enabled: true
YAML
}

# Success: the controller is stopped before the copy and stays stopped
# without a deploy, because it would otherwise write to the old backend.
write_minio_settings
sync_log="$TMP_DIR/sync-ok.log"
run_sync "$TMP_DIR/sync-ok.json" > "$sync_log" 2>&1 || { cat "$sync_log" >&2; exit 1; }
order=$(grep -n -E '^(docker stop|rclone sync)' "$CALLS" | cut -d: -f2 | tr '\n' '|')
[[ "$order" == "docker stop opentalk-controller-1|rclone sync|" ]] || {
  echo "controller must be stopped before rclone sync (got '$order')." >&2; cat "$CALLS" >&2; exit 1; }
! grep -q '^docker start' "$CALLS" || { echo "controller must stay stopped without a deploy." >&2; exit 1; }
grep -q "controller stays stopped until you run 'otctl deploy'" "$sync_log" || {
  echo "missing notice that the controller stays stopped." >&2; exit 1; }
grep -q "^s3_storage_provider: garage" "$SETTINGS" || { echo "settings not switched after copy." >&2; exit 1; }

# Failure: the controller is restarted on the unchanged source.
write_minio_settings
export OT_TEST_RCLONE_FAIL=1
expect_fail "$TMP_DIR/sync-fail.log" "The source is unchanged and remains active" \
  run_sync "$TMP_DIR/sync-fail.json"
unset OT_TEST_RCLONE_FAIL
grep -q '^docker start opentalk-controller-1' "$CALLS" || { echo "controller not restarted after a failed copy." >&2; cat "$CALLS" >&2; exit 1; }
grep -q "^s3_storage_provider: minio" "$SETTINGS" || { echo "settings must stay on the source after a failed copy." >&2; exit 1; }

# When otctl deploys afterwards, the notice about the stopped controller is
# not shown.
write_minio_settings
run_sync "$TMP_DIR/sync-caller.json" -e s3_migrate_caller_deploys=true > "$TMP_DIR/sync-caller.log" 2>&1 || {
  cat "$TMP_DIR/sync-caller.log" >&2; exit 1; }
! grep -q "controller stays stopped until you run 'otctl deploy'" "$TMP_DIR/sync-caller.log" || {
  echo "notice must be skipped when the caller deploys." >&2; exit 1; }

# A controller that was not running is neither stopped nor started.
write_minio_settings
OT_TEST_CONTROLLER_RUNNING=0 run_sync "$TMP_DIR/sync-idle.json" > "$TMP_DIR/sync-idle.log" 2>&1 || {
  cat "$TMP_DIR/sync-idle.log" >&2; exit 1; }
! grep -q -E '^docker (stop|start)' "$CALLS" || { echo "idle controller must not be stopped or started." >&2; cat "$CALLS" >&2; exit 1; }

echo "S3 migration logic and settings-update tests passed."