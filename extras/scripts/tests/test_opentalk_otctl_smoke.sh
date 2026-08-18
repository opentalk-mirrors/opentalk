#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
OTCTL="$REPO_ROOT/extras/packaging/wrappers/otctl"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

PREFIX="$TMP_DIR/prefix"
BACKUP_DIR="$TMP_DIR/backups"
CALLS_LOG="$PREFIX/ot-ansible-playbook.calls"
export OT_PREFIX="$PREFIX"
export OT_BACKUP_DIR="$BACKUP_DIR"
export NO_COLOR=1

mkdir -p "$PREFIX/runtime/venv/bin" "$PREFIX/bin" "$PREFIX/automation"
mkdir -p "$PREFIX/automation/playbooks"
touch "$PREFIX/automation/playbooks/opentalk_compose.yml"
mkdir -p "$BACKUP_DIR"/dumps "$BACKUP_DIR"/volumes "$BACKUP_DIR"/config

# Missing Ansible runtime must fail fast.
set +e
missing_out=$(bash "$OTCTL" version 2>&1 < /dev/null)
missing_rc=$?
set -e
[[ $missing_rc -eq 1 ]] || { echo "otctl must exit 1 when the Ansible runtime is missing (got $missing_rc)." >&2; exit 1; }
grep -q 'ansible runtime missing' <<<"$missing_out"

cat > "$PREFIX/runtime/venv/bin/ansible" <<'STUB'
#!/usr/bin/env bash
printf 'ansible [core 2.99.0]\n'
STUB
chmod +x "$PREFIX/runtime/venv/bin/ansible"

cat > "$PREFIX/runtime/venv/bin/python" <<'STUB'
#!/usr/bin/env bash
printf 'Python 3.12.0 smoke\n'
STUB
chmod +x "$PREFIX/runtime/venv/bin/python"

cat > "$PREFIX/bin/ot-ansible-playbook" <<'STUB'
#!/usr/bin/env bash
log="${0%/bin/ot-ansible-playbook}/ot-ansible-playbook.calls"
printf '%s\n' "$*" >> "$log"
for arg in "$@"; do
  if [[ "$arg" == --list-tags ]]; then
    printf 'TASK TAGS: [secrets, docker]\n'
    exit 0
  fi
done
exit 0
STUB
chmod +x "$PREFIX/bin/ot-ansible-playbook"

cat > "$PREFIX/.versions.yml" <<'YAML'
---
opentalk_product_version: 26.2.0
opentalk_controller_version: v0.33.5
rabbitmq_version: 4.2.7
YAML

printf 'config snapshot' > "$BACKUP_DIR/config/opentalk-compose_config_backup-20260819.tar.gz"
printf 'sql dump' > "$BACKUP_DIR/dumps/controller-20260819.sql"
printf 'sql dump' > "$BACKUP_DIR/dumps/keycloak-20260819.sql"

# version prints product and component versions.
version_out=$(bash "$OTCTL" version < /dev/null)
grep -q 'OpenTalk Product Version: 26.2.0' <<<"$version_out"
grep -q 'opentalk-controller' <<<"$version_out"
grep -q 'rabbitmq' <<<"$version_out"

# tags lists TASK TAGS from the playbook wrapper.
tags_out=$(bash "$OTCTL" tags < /dev/null)
grep -qx 'docker' <<<"$tags_out"
grep -qx 'secrets' <<<"$tags_out"

# deploy passes --diff and forwarded tags to the playbook wrapper.
bash "$OTCTL" deploy --tags secrets < /dev/null >/dev/null
grep -q -- '--diff --tags secrets' "$CALLS_LOG"

# backup mode tag mapping.
bash "$OTCTL" backup all < /dev/null >/dev/null
grep -q -- '--tags backup' "$CALLS_LOG"
bash "$OTCTL" backup dump controller < /dev/null >/dev/null
grep -q -- '--tags controller_db_dump' "$CALLS_LOG"
bash "$OTCTL" backup volume myvol < /dev/null >/dev/null
grep -q -- '--tags docker_volumes_backup' "$CALLS_LOG"
grep -qF '"backup_docker_volume_list": ["myvol"]' "$CALLS_LOG"

# backup ls lists all three backup categories without invoking Ansible.
backup_ls=$(bash "$OTCTL" backup ls 2>&1 < /dev/null)
grep -q 'Backup root:' <<<"$backup_ls"
grep -q 'opentalk-compose_config_backup-20260819.tar.gz' <<<"$backup_ls"
grep -q 'controller-20260819.sql' <<<"$backup_ls"
grep -q 'keycloak-20260819.sql' <<<"$backup_ls"

# restore ls reuses the backup listing.
restore_ls=$(bash "$OTCTL" restore ls 2>&1 < /dev/null)
grep -q 'controller-20260819.sql' <<<"$restore_ls"

# restore without --yes must refuse in non-interactive sessions.
set +e
restore_out=$(bash "$OTCTL" restore all 2>&1 < /dev/null)
restore_rc=$?
set -e
[[ $restore_rc -eq 1 ]] || { echo "otctl restore without --yes must exit 1 (got $restore_rc)." >&2; exit 1; }
grep -q 're-run with --yes' <<<"$restore_out"

# restore all --yes resolves the newest dump and config artifacts.
: > "$CALLS_LOG"
bash "$OTCTL" restore all --yes < /dev/null >/dev/null
grep -q -- '--tags restore' "$CALLS_LOG"
grep -qF 'controller_db_dump_restore_file' "$CALLS_LOG"
grep -qF 'controller-20260819.sql' "$CALLS_LOG"
grep -qF 'keycloak_db_dump_restore_file' "$CALLS_LOG"
grep -qF '"config_backup_restore_file": "opentalk-compose_config_backup-20260819.tar.gz"' "$CALLS_LOG"

# restore controller-db with an explicit missing file must fail.
set +e
missing_restore=$(bash "$OTCTL" restore controller-db "$TMP_DIR/missing-controller.sql" --yes 2>&1 < /dev/null)
missing_restore_rc=$?
set -e
[[ $missing_restore_rc -eq 1 ]] || { echo "otctl restore with missing file must exit 1 (got $missing_restore_rc)." >&2; exit 1; }
grep -q 'File not found' <<<"$missing_restore"

echo "otctl smoke tests passed."