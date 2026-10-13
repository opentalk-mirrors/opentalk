#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# End-to-end test of 'otctl migrate postgres' against real containers: legacy
# PostgreSQL volumes are migrated to the major in .versions.yml, a failed run
# rolls back completely, and status and guard agree before and after.
# Uses its own compose project name and removes everything it creates.
#
# By default it runs the playbooks of this checkout with ansible-playbook from
# PATH. With OT_PREFIX set, it runs the installed package instead, through its
# ot-ansible-playbook wrapper, as CI does after installing the .deb or .rpm.

set -euo pipefail

if [[ ${OPENTALK_E2E:-} != 1 ]]; then
  echo "Set OPENTALK_E2E=1 to run the PostgreSQL migration test (needs Docker and registry access)." >&2
  exit 2
fi
# The password files must be root:root 0600 as the package writes them;
# readable copies hid an image that does not start as root.
if [[ $(id -u) -ne 0 ]]; then
  echo "The PostgreSQL migration test needs root to create the password files like the package does." >&2
  exit 1
fi

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../../.." && pwd)
# shellcheck disable=SC1091
source "$REPO_ROOT/extras/packaging/lib/common.sh"
# shellcheck disable=SC2034 # VERSIONS_FILE is read by lib/versions.sh below.
if [[ -n ${OT_PREFIX:-} ]]; then
  PLAYBOOK_DIR="$OT_PREFIX/automation/playbooks"
  VERSIONS_FILE="$OT_PREFIX/.versions.yml"
else
  PLAYBOOK_DIR="$REPO_ROOT/automation/playbooks"
  VERSIONS_FILE="$REPO_ROOT/.versions.yml"
fi
# shellcheck disable=SC1091
source "$REPO_ROOT/extras/packaging/lib/versions.sh"

PROJECT="otpgmig$$"
TMP_DIR=$(mktemp -d)
IMAGE_SRC=registry.heinlein.group/opentalk/postgres
SOURCE_IMAGE="$IMAGE_SRC:$(versions_get postgres_migration_source_version)"
TARGET_IMAGE="$IMAGE_SRC:$(versions_get postgres_version)"
TARGET_MAJOR=$(versions_get postgres_version | cut -d. -f1)

cleanup() {
  local -a ids
  mapfile -t ids < <(docker ps -aq --filter "name=^${PROJECT}-")
  [[ ${#ids[@]} -eq 0 ]] || docker rm -f "${ids[@]}" >/dev/null
  mapfile -t ids < <(docker volume ls -q --filter "name=^${PROJECT}_")
  [[ ${#ids[@]} -eq 0 ]] || docker volume rm "${ids[@]}" >/dev/null
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

mkdir -p "$TMP_DIR/secrets" "$TMP_DIR/backups"
printf 'controller-secret-%s' "$$" > "$TMP_DIR/secrets/.db_controller.secret"
printf 'keycloak-secret-%s' "$$" > "$TMP_DIR/secrets/.db_keycloak.secret"
chmod 0600 "$TMP_DIR/secrets/".db_*.secret
SETTINGS="$TMP_DIR/settings.yml"
cat > "$SETTINGS" <<YAML
compose_project_name: $PROJECT
config_dir: $TMP_DIR
backup_dir: $TMP_DIR/backups
s3_storage_provider: garage
YAML

run_playbook() {
  if [[ -n ${OT_PREFIX:-} ]]; then
    OT_SETTINGS_FILE="$SETTINGS" \
    "$OT_PREFIX/bin/ot-ansible-playbook" -i localhost, -c local "$PLAYBOOK_DIR/$1" "${@:2}"
  else
    OT_SETTINGS_FILE="$SETTINGS" \
    ANSIBLE_ROLES_PATH="$REPO_ROOT/automation/roles" \
    ANSIBLE_CALLBACK_RESULT_FORMAT=yaml \
    ansible-playbook -i localhost, -c local "$PLAYBOOK_DIR/$1" "${@:2}"
  fi
}

wait_ready() {
  for _ in $(seq 1 60); do
    docker exec "$1" pg_isready -h 127.0.0.1 -U ot >/dev/null 2>&1 && return 0
    sleep 1
  done
  echo "PostgreSQL in $1 did not become ready." >&2
  docker logs "$1" >&2
  exit 1
}

psql_in() {
  local container=$1 db=$2
  shift 2
  docker exec -i "$container" psql -X -At -v ON_ERROR_STOP=1 -U ot -d "$db" "$@"
}

# --- a deployment as PostgreSQL 17 left it: legacy volumes, running containers ---
for db in controller keycloak; do
  docker volume create "${PROJECT}_${db}-db" >/dev/null
  docker run -d --name "${PROJECT}-${db}-db-1" \
    -e POSTGRES_USER=ot -e POSTGRES_DB="$db" -e POSTGRES_PASSWORD_FILE=/run/secrets/db \
    -v "$TMP_DIR/secrets/.db_${db}.secret:/run/secrets/db:ro" \
    -v "${PROJECT}_${db}-db:/var/lib/postgresql/data" "$SOURCE_IMAGE" >/dev/null
  # Stand-in for the application that writes to the database.
  docker run -d --name "${PROJECT}-${db}-1" --entrypoint sleep "$SOURCE_IMAGE" infinity >/dev/null
done
wait_ready "${PROJECT}-controller-db-1"
wait_ready "${PROJECT}-keycloak-db-1"
psql_in "${PROJECT}-controller-db-1" controller <<'SQL'
CREATE TABLE rooms (id serial PRIMARY KEY, name text NOT NULL, created timestamptz DEFAULT now());
INSERT INTO rooms (name) SELECT 'room ' || g FROM generate_series(1, 2500) g;
CREATE SCHEMA extra;
CREATE TABLE extra.events (id bigint PRIMARY KEY, payload jsonb);
INSERT INTO extra.events SELECT g, jsonb_build_object('n', g) FROM generate_series(1, 300) g;
CREATE TABLE empty_table (id int);
SQL
psql_in "${PROJECT}-keycloak-db-1" keycloak <<'SQL'
CREATE TABLE realm (id text PRIMARY KEY, name text);
INSERT INTO realm VALUES ('opentalk', 'OpenTalk'), ('master', 'Master');
SQL

# --- status and plan list both databases; nothing changes yet ---
run_playbook migrate_status.yml > "$TMP_DIR/status-before.log" 2>&1
grep -q '2 migration(s) pending' "$TMP_DIR/status-before.log" || { cat "$TMP_DIR/status-before.log" >&2; exit 1; }
run_playbook postgres_migrate.yml -e "postgres_migrate_plan_only=true postgres_migrate_plan_file=$TMP_DIR/plan" \
  > "$TMP_DIR/plan.log" 2>&1 || { cat "$TMP_DIR/plan.log" >&2; exit 1; }
[[ $(cat "$TMP_DIR/plan") == 2 ]] || { echo "plan must report 2 pending databases." >&2; exit 1; }
grep -q "${PROJECT}_controller-db -> ${PROJECT}_controller-db-pg${TARGET_MAJOR}" "$TMP_DIR/plan.log"
[[ $(docker inspect -f '{{.State.Running}}' "${PROJECT}-controller-db-1") == true ]] || {
  echo "the plan must not stop anything." >&2; exit 1; }

# --- without confirmation nothing happens ---
if run_playbook postgres_migrate.yml > "$TMP_DIR/noconfirm.log" 2>&1; then
  echo "migration without confirmation must fail." >&2; exit 1
fi
grep -q 'requested without confirmation' "$TMP_DIR/noconfirm.log"

# --- a failing run rolls back completely ---
if run_playbook postgres_migrate.yml -e postgres_migrate_confirmation=yes \
  -e "postgres_migrate_target_image=$IMAGE_SRC:does-not-exist" > "$TMP_DIR/fail.log" 2>&1; then
  echo "migration with a missing target image must fail." >&2; exit 1
fi
grep -q 'OpenTalk runs as before on the old data' "$TMP_DIR/fail.log" || { cat "$TMP_DIR/fail.log" >&2; exit 1; }
for db in controller keycloak; do
  [[ $(docker inspect -f '{{.State.Running}}' "${PROJECT}-${db}-db-1") == true ]] || {
    echo "${db} database must run again after a failed migration." >&2; exit 1; }
  [[ $(docker inspect -f '{{.State.Running}}' "${PROJECT}-${db}-1") == true ]] || {
    echo "${db} application must run again after a failed migration." >&2; exit 1; }
  ! docker volume inspect "${PROJECT}_${db}-db-pg${TARGET_MAJOR}" >/dev/null 2>&1 || {
    echo "the new ${db} volume must be removed after a failed migration." >&2; exit 1; }
done
[[ -z $(docker ps -aq --filter "name=^${PROJECT}-pgmigrate-") ]] || {
  echo "temporary containers must be removed after a failed migration." >&2; exit 1; }

# --- the real run ---
run_playbook postgres_migrate.yml -e postgres_migrate_confirmation=yes > "$TMP_DIR/migrate.log" 2>&1 || {
  cat "$TMP_DIR/migrate.log" >&2; exit 1; }
grep -q 'PostgreSQL migration finished' "$TMP_DIR/migrate.log"
for db in controller keycloak; do
  [[ $(docker inspect -f '{{.State.Running}}' "${PROJECT}-${db}-1") == false ]] || {
    echo "${db} application must stay stopped until the deploy." >&2; exit 1; }
  docker volume inspect "${PROJECT}_${db}-db" >/dev/null || { echo "old ${db} volume must be kept." >&2; exit 1; }
  [[ $(docker volume inspect -f '{{index .Labels "com.docker.compose.project"}}' "${PROJECT}_${db}-db-pg${TARGET_MAJOR}") == "$PROJECT" ]] || {
    echo "new ${db} volume must carry the compose project label." >&2; exit 1; }
done
ls "$TMP_DIR/backups/dumps/"pg-migrate-controller-*.sql >/dev/null
[[ -z $(docker ps -aq --filter "name=^${PROJECT}-pgmigrate-") ]] || {
  echo "temporary containers must be removed after the migration." >&2; exit 1; }

# The migration makes the password files readable for group root.
[[ $(stat -c '%a %U %G' "$TMP_DIR/secrets/.db_controller.secret") == "640 root root" ]] || {
  echo "the migration must leave the password file root:root 0640." >&2; exit 1; }

# --- the new volume works the way the compose template mounts it ---
docker run -d --name "${PROJECT}-verify" --group-add 0 \
  -e POSTGRES_PASSWORD_FILE=/run/secrets/db \
  -v "$TMP_DIR/secrets/.db_controller.secret:/run/secrets/db:ro" \
  -v "${PROJECT}_controller-db-pg${TARGET_MAJOR}:/var/lib/postgresql" "$TARGET_IMAGE" >/dev/null
wait_ready "${PROJECT}-verify"
[[ $(psql_in "${PROJECT}-verify" controller -c 'SHOW server_version_num' | cut -c1-2) == "$TARGET_MAJOR" ]]
[[ $(psql_in "${PROJECT}-verify" controller -c 'SELECT count(*) FROM rooms') == 2500 ]]
[[ $(psql_in "${PROJECT}-verify" controller -c 'SELECT max(id) FROM extra.events') == 300 ]]
[[ $(psql_in "${PROJECT}-verify" controller -c "SELECT nextval('rooms_id_seq')") == 2501 ]] || {
  echo "sequences must continue after the restore." >&2; exit 1; }
# Password authentication over TCP, as the controller connects.
docker exec -e PGPASSWORD="controller-secret-$$" "${PROJECT}-verify" \
  psql -X -At -h 127.0.0.1 -U ot -d controller -c 'SELECT 1' >/dev/null || {
  echo "the new database must accept the existing password." >&2; exit 1; }
docker rm -f "${PROJECT}-verify" >/dev/null

# --- afterwards status is clean and lists the old volumes for removal ---
run_playbook migrate_status.yml > "$TMP_DIR/status-after.log" 2>&1
grep -q 'No migration pending' "$TMP_DIR/status-after.log" || { cat "$TMP_DIR/status-after.log" >&2; exit 1; }
grep -q "volume ${PROJECT}_keycloak-db" "$TMP_DIR/status-after.log"

# --- a second run finds nothing to do ---
run_playbook postgres_migrate.yml -e "postgres_migrate_plan_only=true postgres_migrate_plan_file=$TMP_DIR/plan" \
  > "$TMP_DIR/plan-again.log" 2>&1
[[ $(cat "$TMP_DIR/plan") == 0 ]] || { echo "a finished migration must leave nothing pending." >&2; exit 1; }

echo "PostgreSQL migration integration test passed."
