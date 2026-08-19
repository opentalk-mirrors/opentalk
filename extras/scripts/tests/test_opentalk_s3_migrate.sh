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
grep -q "compose_service_minio_enabled: false" "$SETTINGS" || { echo "minio must remain disabled." >&2; exit 1; }

# --- append-if-absent: disabling source line missing from settings ---
cat > "$SETTINGS" <<'YAML'
s3_storage_provider: minio
compose_service_garage_enabled: true
YAML
run_logic "$TMP_DIR/append.json" -e "s3_migration_source=minio" -e "config_dir=$TMP_DIR"
grep -q "^compose_service_minio_enabled: false" "$SETTINGS" || { echo "missing minio enable line not appended." >&2; exit 1; }

# --- invalid source rejected ---
cat > "$SETTINGS" <<'YAML'
s3_storage_provider: minio
YAML
expect_fail "$TMP_DIR/invalid.log" "s3_migration_source must be 'minio' or 'garage'" \
  run_logic "$TMP_DIR/invalid.json" -e "s3_migration_source=ftp"

echo "S3 migration logic and settings-update tests passed."