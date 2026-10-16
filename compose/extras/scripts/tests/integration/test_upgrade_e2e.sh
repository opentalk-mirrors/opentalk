#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# Upgrade path a user takes: install the current stable release from the
# package repository, deploy it and write data, install the package under
# test, and check that 'otctl deploy' stops for the pending migrations, that
# 'otctl migrate postgres' moves the data, and that the stack comes back
# healthy with the data intact.
#
# Destructive: installs packages and deploys the stack system-wide. Run it
# only on a disposable Debian or Ubuntu host as root, such as the CI dind
# container. Requires network access to the package repository and the
# container registry.

set -euo pipefail

if [[ ${OPENTALK_E2E:-} != 1 ]]; then
  echo "Set OPENTALK_E2E=1 to run the upgrade test (destructive, disposable host only)." >&2
  exit 2
fi
if [[ $(id -u) -ne 0 ]]; then
  echo "The upgrade test installs packages and must run as root." >&2
  exit 1
fi
command -v apt-get >/dev/null || { echo "The upgrade test needs a Debian-based host." >&2; exit 1; }

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../../.." && pwd)
NEW_PACKAGE=${OT_UPGRADE_PACKAGE:-$(find "$REPO_ROOT/artifacts" -maxdepth 1 -name '*.deb' -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -n 1 | cut -d' ' -f2-)}
# Empty installs the newest stable release, the one users upgrade from.
FROM_VERSION=${OT_UPGRADE_FROM_VERSION:-}
REPO_URL=${OT_UPGRADE_REPO_URL:-https://repo.heinlein.group/repo/opentalk-compose}
SETTINGS_FILE=/etc/opentalk-compose/settings.yml
PROJECT=opentalk
LOG_DIR=$(mktemp -d)
trap 'rm -rf "$LOG_DIR"' EXIT

[[ -f $NEW_PACKAGE ]] || { echo "Package under test not found; set OT_UPGRADE_PACKAGE." >&2; exit 1; }

step() { printf '\n=== %s\n' "$*"; }

# Right after a deploy the controller restarts until Keycloak answers, and the
# runtime check fails on a restarting container. Give the stack a few rounds.
wait_healthy() {
  local attempt
  for attempt in 1 2 3 4 5 6 7 8; do
    if COMPOSE_PROJECT_NAME=$PROJECT bash "$REPO_ROOT/extras/scripts/tests/integration/test_deployment_runtime.sh"; then
      return 0
    fi
    echo "Stack not healthy yet (attempt $attempt), retrying in 15s."
    sleep 15
  done
  echo "The stack did not become healthy." >&2
  exit 1
}

sql() {
  local db=$1
  shift
  docker exec -i "${PROJECT}-${db}-db-1" psql -X -At -v ON_ERROR_STOP=1 -U ot -d "$db" "$@"
}

# Rows of tables both releases write, plus a marker row per database. The new
# controller adds its own schema migrations, so only the ones applied before
# the upgrade are compared.
snapshot() {
  {
    sql keycloak -c "SELECT 'realms', count(*) FROM realm"
    sql keycloak -c "SELECT 'users', count(*) FROM user_entity"
    sql keycloak -c "SELECT 'marker', note FROM upgrade_marker"
    sql controller -c "SELECT 'schema_history', count(*) FROM refinery_schema_history WHERE version <= $old_schema_version"
    sql controller -c "SELECT 'marker', note FROM upgrade_marker"
  } > "$1"
}

step "Install the current stable release"
apt-get update -qq
apt-get install -y -qq curl ca-certificates >/dev/null
curl -fsSL "$REPO_URL/opentalk-compose_sign.asc" > /etc/apt/trusted.gpg.d/opentalk-archive-keyring.asc
echo "deb [arch=amd64] $REPO_URL/deb/stable/ all main" > /etc/apt/sources.list.d/opentalk-compose.list
apt-get update -qq
apt-get install -y -qq "opentalk-compose${FROM_VERSION:+=$FROM_VERSION}"
old_version=$(dpkg-query -W -f '${Version}' opentalk-compose)
echo "Installed opentalk-compose $old_version"

mkdir -p "$(dirname "$SETTINGS_FILE")"
cp "$REPO_ROOT/extras/ci/gitlab/integration-settings.yml" "$SETTINGS_FILE"
# Host-specific overrides, for example a process limit that a nested Docker
# shares with the desktop session of the same UID.
if [[ -n ${OT_UPGRADE_EXTRA_SETTINGS:-} ]]; then
  cat "$OT_UPGRADE_EXTRA_SETTINGS" >> "$SETTINGS_FILE"
fi
grep -q 'opentalk-ci.test' /etc/hosts || echo "127.0.0.1 opentalk-ci.test" >> /etc/hosts

step "Deploy $old_version"
otctl deploy
wait_healthy
old_pg=$(sql controller -c 'SHOW server_version_num' | cut -c1-2)
echo "Running on PostgreSQL $old_pg"

step "Write data that must survive the upgrade"
for db in controller keycloak; do
  sql "$db" -c "CREATE TABLE upgrade_marker (note text); INSERT INTO upgrade_marker VALUES ('written by $old_version')"
done
old_schema_version=$(sql controller -c 'SELECT max(version) FROM refinery_schema_history')
snapshot "$LOG_DIR/before"
cat "$LOG_DIR/before"

step "Install the package under test"
# Keep the modified settings.yml, as the default answer of the conffile
# prompt does; the new template lands next to it as settings.yml.dpkg-dist.
install_rc=0
{
  dpkg -i --force-confdef --force-confold "$NEW_PACKAGE" ||
    apt-get install -f -y -o Dpkg::Options::=--force-confdef -o Dpkg::Options::=--force-confold
} > "$LOG_DIR/install.log" 2>&1 || install_rc=$?
cat "$LOG_DIR/install.log"
[[ $install_rc -eq 0 ]] || { echo "Installing the package under test failed ($install_rc)." >&2; exit 1; }
new_version=$(dpkg-query -W -f '${Version}' opentalk-compose)
[[ $new_version != "$old_version" ]] || { echo "The package under test was not installed." >&2; exit 1; }
grep -q "otctl migrate status" "$LOG_DIR/install.log" || {
  echo "The package upgrade must point at 'otctl migrate status'." >&2; exit 1; }

step "The deploy stops before any change"
if otctl deploy > "$LOG_DIR/deploy-blocked.log" 2>&1; then
  cat "$LOG_DIR/deploy-blocked.log"
  echo "'otctl deploy' must stop while the PostgreSQL migration is pending." >&2
  exit 1
fi
grep -q 'Deployment stopped before any change: 2 migration(s) required' "$LOG_DIR/deploy-blocked.log" || {
  cat "$LOG_DIR/deploy-blocked.log"; exit 1; }
[[ $(sql controller -c 'SHOW server_version_num' | cut -c1-2) == "$old_pg" ]] || {
  echo "The blocked deploy must leave the old database running." >&2; exit 1; }

step "Status names both databases"
otctl migrate status | tee "$LOG_DIR/status-before.log"
grep -q '2 migration(s) pending' "$LOG_DIR/status-before.log"

step "Migrate"
otctl migrate postgres --yes 2>&1 | tee "$LOG_DIR/migrate.log"

step "The stack runs on the new major with the old data"
wait_healthy
new_pg=$(sql controller -c 'SHOW server_version_num' | cut -c1-2)
(( new_pg > old_pg )) || { echo "Expected a newer PostgreSQL than $old_pg, got $new_pg." >&2; exit 1; }
snapshot "$LOG_DIR/after"
diff -u "$LOG_DIR/before" "$LOG_DIR/after" || { echo "Data changed during the upgrade." >&2; exit 1; }

step "Status is clean and lists the old volumes"
otctl migrate status | tee "$LOG_DIR/status-after.log"
grep -q 'No migration pending' "$LOG_DIR/status-after.log"
grep -q "volume ${PROJECT}_controller-db" "$LOG_DIR/status-after.log"

step "A second deploy is a no-op for the migration"
otctl deploy > "$LOG_DIR/deploy-again.log" 2>&1 || { cat "$LOG_DIR/deploy-again.log"; exit 1; }
wait_healthy

echo "Upgrade from $old_version (PostgreSQL $old_pg) to $new_version (PostgreSQL $new_pg) passed."
