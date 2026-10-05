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

# Neither docker nor podman on PATH: COMPOSE_CMD stays at its default "docker".
# Only `bash` and `cat` (used by the help text) are reachable, so the
# docker/podman fallback branch is what actually runs.
EMPTY_PATH_DIR="$TMP_DIR/empty-path"
mkdir -p "$EMPTY_PATH_DIR"
ln -s "$(command -v bash)" "$EMPTY_PATH_DIR/bash"
ln -s "$(command -v cat)" "$EMPTY_PATH_DIR/cat"
empty_path_out=$(PATH="$EMPTY_PATH_DIR" bash "$OTCTL" help < /dev/null)
grep -q 'OpenTalk Control (otctl)' <<<"$empty_path_out"

cat > "$PREFIX/bin/ot-ansible-playbook" <<'STUB'
#!/usr/bin/env bash
log="${0%/bin/ot-ansible-playbook}/ot-ansible-playbook.calls"
printf '%s\n' "$*" >> "$log"
for arg in "$@"; do
  [[ -n "$arg" ]] || printf '%s\n' "$*" >> "$log.empty-arg"
done
if [ "${OTSMOKE_TAGS_FAIL:-0}" = "1" ]; then
  exit 1
fi
if [ "${OTSMOKE_TAGS_LEGACY:-0}" = "1" ]; then
  printf '  play # tags=secrets,docker\n'
  exit 0
fi
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

# version also reports the installed Debian package when dpkg-query is present.
cat > "$PREFIX/bin/dpkg-query" <<'STUB'
#!/usr/bin/env bash
case "$*" in
  *'${Version}'*) printf '26.2.0\n' ;;
  *'${Architecture}'*) printf 'amd64\n' ;;
esac
STUB
chmod +x "$PREFIX/bin/dpkg-query"
dpkg_version_out=$(PATH="$PREFIX/bin:$PATH" bash "$OTCTL" version < /dev/null)
grep -q 'Package: opentalk-compose_26.2.0_amd64' <<<"$dpkg_version_out"
rm -f "$PREFIX/bin/dpkg-query"

# version reports only the version component when Architecture is unknown.
cat > "$PREFIX/bin/dpkg-query" <<'STUB'
#!/usr/bin/env bash
case "$*" in
  *'${Version}'*) printf '26.2.0\n' ;;
  *'${Architecture}'*) exit 1 ;;
esac
STUB
chmod +x "$PREFIX/bin/dpkg-query"
dpkg_archless_out=$(PATH="$PREFIX/bin:$PATH" bash "$OTCTL" version < /dev/null)
grep -q 'Package: opentalk-compose_26.2.0$' <<<"$dpkg_archless_out"
if grep -q 'opentalk-compose_26.2.0_amd64' <<<"$dpkg_archless_out"; then
  echo "otctl version must not append an unknown architecture." >&2; exit 1
fi
rm -f "$PREFIX/bin/dpkg-query"

# tags lists TASK TAGS from the playbook wrapper.
tags_out=$(bash "$OTCTL" tags < /dev/null)
grep -qx 'docker' <<<"$tags_out"
grep -qx 'secrets' <<<"$tags_out"

# tags failure from the wrapper is propagated (exit 1).
set +e
tags_fail=$(OTSMOKE_TAGS_FAIL=1 bash "$OTCTL" tags 2>&1 < /dev/null)
tags_fail_rc=$?
set -e
[[ $tags_fail_rc -eq 1 ]] || { echo "otctl tags must exit 1 when the wrapper fails (got $tags_fail_rc)." >&2; exit 1; }
grep -q 'failed to obtain tags' <<<"$tags_fail"

# tags also parses the legacy `play # tags=...` format from the wrapper.
legacy_tags=$(OTSMOKE_TAGS_LEGACY=1 bash "$OTCTL" tags < /dev/null)
grep -qx 'docker' <<<"$legacy_tags"
grep -qx 'secrets' <<<"$legacy_tags"

# ---------------------------------------------------------------------------
# Ansible collections path wiring from ansible.cfg (cfg_paths / VENDOR merge).
# ---------------------------------------------------------------------------
VENDOR_COLL="$PREFIX/runtime/vendor/collections/ansible_collections"
mkdir -p "$VENDOR_COLL"
cat > "$PREFIX/runtime/venv/bin/ansible" <<'STUB'
#!/usr/bin/env bash
printf 'ANSIBLE_COLLECTIONS_PATHS=%s\n' "${ANSIBLE_COLLECTIONS_PATHS:-}"
STUB
chmod +x "$PREFIX/runtime/venv/bin/ansible"

# cfg_paths is set and unrelated to VENDOR: VENDOR is prepended (Z.142).
cat > "$PREFIX/automation/ansible.cfg" <<'CFG'
[defaults]
collections_path = /usr/share/ansible/collections
CFG
coll_custom=$(env -u ANSIBLE_COLLECTIONS_PATHS bash "$OTCTL" version < /dev/null)
grep -q 'ANSIBLE_COLLECTIONS_PATHS=.*:/usr/share/ansible/collections' <<<"$coll_custom"

# cfg_paths is empty: the default search chain is used (Z.141).
cat > "$PREFIX/automation/ansible.cfg" <<'CFG'
[defaults]
CFG
coll_empty=$(env -u ANSIBLE_COLLECTIONS_PATHS bash "$OTCTL" version < /dev/null)
grep -q "ANSIBLE_COLLECTIONS_PATHS=$PREFIX/runtime/vendor/collections:$HOME/.ansible/collections:" <<<"$coll_empty"

# cfg_paths already contains VENDOR: it is used as-is (Z.140).
cat > "$PREFIX/automation/ansible.cfg" <<CFG
[defaults]
collections_path = $PREFIX/runtime/vendor/collections
CFG
coll_vendor=$(env -u ANSIBLE_COLLECTIONS_PATHS bash "$OTCTL" version < /dev/null)
grep -q "ANSIBLE_COLLECTIONS_PATHS=$PREFIX/runtime/vendor/collections\$" <<<"$coll_vendor"

rm -f "$PREFIX/automation/ansible.cfg"
rm -rf "$PREFIX/runtime/vendor"
cat > "$PREFIX/runtime/venv/bin/ansible" <<'STUB'
#!/usr/bin/env bash
printf 'ansible [core 2.99.0]\n'
STUB
chmod +x "$PREFIX/runtime/venv/bin/ansible"

# deploy passes --diff and forwarded tags to the playbook wrapper.
bash "$OTCTL" deploy --tags secrets < /dev/null >/dev/null
grep -q -- '--diff --tags secrets' "$CALLS_LOG"

# playbook passes all arguments through unmodified (raw passthrough).
: > "$CALLS_LOG"
bash "$OTCTL" playbook --check -i inventory.yml site.yml < /dev/null >/dev/null
grep -q -- '--check -i inventory.yml site.yml' "$CALLS_LOG"

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

# backup ls reports a missing category directory as "(dir missing)".
PARTIAL="$TMP_DIR/partial-backups"
mkdir -p "$PARTIAL/dumps" "$PARTIAL/config"
partial_ls=$(OT_BACKUP_DIR="$PARTIAL" bash "$OTCTL" backup ls 2>&1 < /dev/null)
grep -q '(dir missing)' <<<"$partial_ls"

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

# backup help (no sub-mode) prints usage and exits 0.
backup_help=$(bash "$OTCTL" backup 2>&1 < /dev/null)
grep -q 'otctl backup USAGE' <<<"$backup_help"

# backup sub-mode tag mapping: cron, clean/prune, dump variants, config, keycloak.
: > "$CALLS_LOG"
bash "$OTCTL" backup cron < /dev/null >/dev/null
grep -q -- '--tags controller_db_dump_cron' "$CALLS_LOG"
: > "$CALLS_LOG"
bash "$OTCTL" backup clean < /dev/null >/dev/null
grep -q -- '--tags backup_retention' "$CALLS_LOG"
bash "$OTCTL" backup prune < /dev/null >/dev/null
grep -q -- '--tags backup_retention' "$CALLS_LOG"
: > "$CALLS_LOG"
bash "$OTCTL" backup dump keycloak < /dev/null >/dev/null
grep -q -- '--tags keycloak_db_dump' "$CALLS_LOG"
bash "$OTCTL" backup dump < /dev/null >/dev/null
grep -q -- '--tags controller_db_dump,keycloak_db_dump' "$CALLS_LOG"
: > "$CALLS_LOG"
bash "$OTCTL" backup config < /dev/null >/dev/null
grep -q -- '--tags config_backup' "$CALLS_LOG"
bash "$OTCTL" backup cfg < /dev/null >/dev/null
grep -q -- '--tags config_backup' "$CALLS_LOG"
bash "$OTCTL" backup keycloak < /dev/null >/dev/null
grep -q -- '--tags keycloak_db_dump' "$CALLS_LOG"
bash "$OTCTL" backup dump-controller < /dev/null >/dev/null
grep -q -- '--tags controller_db_dump' "$CALLS_LOG"

# backup dump with an unknown qualifier falls back to both dump tags.
: > "$CALLS_LOG"
bash "$OTCTL" backup dump bogus < /dev/null >/dev/null
grep -q -- '--tags controller_db_dump,keycloak_db_dump' "$CALLS_LOG"

# backup volume without a name runs all volumes and must not set a volume list.
: > "$CALLS_LOG"
unset OTCTL_BACKUP_EXTRA_VARS
bash "$OTCTL" backup volume < /dev/null >/dev/null
grep -q -- '--tags docker_volumes_backup' "$CALLS_LOG"
if grep -q 'backup_docker_volume_list' "$CALLS_LOG"; then
  echo "otctl backup volume without a name must not set a volume list." >&2; exit 1
fi

# backup passthrough keeps the mode in argv (custom tags).
: > "$CALLS_LOG"
bash "$OTCTL" backup --tags secrets < /dev/null >/dev/null
grep -q -- '--tags backup --tags secrets' "$CALLS_LOG"

# restore help (no sub-mode) prints usage and exits 0.
restore_help=$(bash "$OTCTL" restore 2>&1 < /dev/null)
grep -q 'otctl restore USAGE' <<<"$restore_help"

# restore sub-modes resolve the newest matching artifact.
: > "$CALLS_LOG"
bash "$OTCTL" restore controller-db --yes < /dev/null >/dev/null
grep -q -- '--tags controller_db_restore' "$CALLS_LOG"
grep -qF '"controller_db_dump_restore_file": "'"$BACKUP_DIR/dumps/controller-20260819.sql"'"' "$CALLS_LOG"
: > "$CALLS_LOG"
bash "$OTCTL" restore keycloak-db --yes < /dev/null >/dev/null
grep -q -- '--tags keycloak_db_restore' "$CALLS_LOG"
grep -qF '"keycloak_db_dump_restore_file": "'"$BACKUP_DIR/dumps/keycloak-20260819.sql"'"' "$CALLS_LOG"
: > "$CALLS_LOG"
bash "$OTCTL" restore config --yes < /dev/null >/dev/null
grep -q -- '--tags config_restore' "$CALLS_LOG"
grep -qF '"config_backup_restore_file": "opentalk-compose_config_backup-20260819.tar.gz"' "$CALLS_LOG"

# restore volume with and without an explicit volume name.
: > "$CALLS_LOG"
bash "$OTCTL" restore volume opentalk_postgres --yes < /dev/null >/dev/null
grep -q -- '--tags docker_volumes_restore' "$CALLS_LOG"
grep -qF '"backup_docker_volume_list": ["opentalk_postgres"]' "$CALLS_LOG"
: > "$CALLS_LOG"
bash "$OTCTL" restore volume --yes < /dev/null >/dev/null
grep -q -- '--tags docker_volumes_restore' "$CALLS_LOG"
if grep -q 'backup_docker_volume_list' "$CALLS_LOG"; then
  echo "otctl restore volume without a name must not set a volume list." >&2; exit 1
fi

# restore with an explicit absolute path is passed through unchanged.
: > "$CALLS_LOG"
bash "$OTCTL" restore controller-db "$BACKUP_DIR/dumps/controller-20260819.sql" --yes < /dev/null >/dev/null
grep -qF '"controller_db_dump_restore_file": "'"$BACKUP_DIR/dumps/controller-20260819.sql"'"' "$CALLS_LOG"

# restore with an explicit relative path is resolved against the CWD.
(
  cd "$TMP_DIR"
  : > "$CALLS_LOG"
  bash "$OTCTL" restore controller-db "backups/dumps/controller-20260819.sql" --yes < /dev/null >/dev/null
)
grep -qF '"controller_db_dump_restore_file": "'"$BACKUP_DIR/dumps/controller-20260819.sql"'"' "$CALLS_LOG"

# restore with an unknown mode reports an error and exits 1.
set +e
bogus_restore=$(bash "$OTCTL" restore bogus --yes 2>&1 < /dev/null)
bogus_restore_rc=$?
set -e
[[ $bogus_restore_rc -eq 1 ]] || { echo "otctl restore with unknown mode must exit 1 (got $bogus_restore_rc)." >&2; exit 1; }
grep -q 'Unknown restore mode: bogus' <<<"$bogus_restore"

# ---------------------------------------------------------------------------
# Compose passthrough paths with a fake `docker` executable.
# ---------------------------------------------------------------------------
DOCKER_STUB="$PREFIX/bin/ot-docker-stub"
DOCKER_CALLS_LOG="$PREFIX/ot-docker.calls"
cat > "$DOCKER_STUB" <<'STUB'
#!/usr/bin/env bash
log="${0%/bin/ot-docker-stub}/ot-docker.calls"
printf 'stub docker: %s\n' "$*" >> "$log"
cmd="$1"; shift
case "$cmd" in
  compose)
    sub=""; for a in "$@"; do [[ "$a" == ps ]] && sub=ps; done
    if [[ "$sub" == ps ]]; then
      if [[ "$*" == *"--format json"* ]]; then
        printf '[{"Service":"controller","State":"running","Health":"healthy"}]\n'
      elif [[ "$*" == *"--format"* ]]; then
        printf 'controller\trunning\thealthy\n'
      else
        printf 'NAME\tIMAGE\tCOMMAND\tSERVICE\tSTATUS\n'
        printf 'ot-controller\timg\t""\tcontroller\trunning\n'
      fi
      exit 0
    fi
    printf 'stub compose: %s\n' "$*"
    ;;
  volume)
    sub="$1"; shift
    case "$sub" in
      ls) printf 'opentalk_controller\t/mnt/controller\nopentalk_postgres\t/mnt/postgres\nforeign_other\t/mnt/other\n' ;;
      rm) printf 'removed %s\n' "$*" ;;
      *) echo "unknown volume action: $sub" >&2; exit 1 ;;
    esac
    ;;
  inspect) printf 'running\n' ;;
  *) printf 'stub docker: %s\n' "$cmd $*" ;;
esac
STUB
chmod +x "$DOCKER_STUB"
export DOCKER_CMD="$DOCKER_STUB"
cat > "$PREFIX/compose.yml" <<'YAML'
services:
  controller:
    image: example
YAML

: > "$DOCKER_CALLS_LOG"
# up/down/ps/logs/restart/recreate/compose forward to compose_exec.
bash "$OTCTL" up < /dev/null >/dev/null
bash "$OTCTL" down < /dev/null >/dev/null
bash "$OTCTL" ps < /dev/null >/dev/null
bash "$OTCTL" logs controller < /dev/null >/dev/null
bash "$OTCTL" restart controller < /dev/null >/dev/null
bash "$OTCTL" recreate controller < /dev/null >/dev/null
bash "$OTCTL" compose config --services < /dev/null >/dev/null
grep -q 'stub docker: compose .* up -d' "$DOCKER_CALLS_LOG"
grep -q 'stub docker: compose .* down' "$DOCKER_CALLS_LOG"
grep -q 'stub docker: compose .* logs controller' "$DOCKER_CALLS_LOG"
grep -q 'stub docker: compose .* restart controller' "$DOCKER_CALLS_LOG"
grep -q 'stub docker: compose .* up -d --force-recreate controller' "$DOCKER_CALLS_LOG"

# recreate without arguments force-recreates the whole stack (no containers arg).
: > "$DOCKER_CALLS_LOG"
bash "$OTCTL" recreate < /dev/null >/dev/null
grep -q 'stub docker: compose .* up -d --force-recreate$' "$DOCKER_CALLS_LOG"

# status renders a JSON service table.
status_out=$(bash "$OTCTL" status 2>&1 < /dev/null)
grep -q '^SERVICE.*STATE.*HEALTH' <<<"$status_out"
grep -q 'controller  running  healthy' <<<"$status_out"

# ---------------------------------------------------------------------------
# status fallbacks: no JSON support (old compose) -> go-template, then inspect.
# ---------------------------------------------------------------------------
cat > "$PREFIX/bin/ot-docker-legacy-stub" <<'STUB'
#!/usr/bin/env bash
cmd="$1"; shift
case "$cmd" in
  compose)
    sub=""; for a in "$@"; do [[ "$a" == ps ]] && sub=ps; done
    if [[ "$sub" == ps ]]; then
      if [[ "$*" == *"--format json"* ]]; then
        exit 0
      elif [[ "$*" == *"--format"* ]]; then
        printf 'controller\trunning\thealthy\n'
      else
        printf 'NAME\tIMAGE\tCOMMAND\tSERVICE\tSTATUS\n'
        printf 'ot-controller\timg\t""\tcontroller\trunning\n'
      fi
      exit 0
    fi
    printf 'stub compose: %s\n' "$*"
    ;;
  inspect)
    fmt=""; for a in "$@"; do [[ "$a" == -f ]] && fmt=1; done
    if [[ "$fmt" == 1 ]]; then
      if [[ "$*" == *".State.Health"* ]]; then
        printf 'healthy\n'
      else
        printf 'running\n'
      fi
    else
      printf 'running\n'
    fi
    ;;
  *) printf 'stub docker: %s\n' "$cmd $*" ;;
esac
STUB
chmod +x "$PREFIX/bin/ot-docker-legacy-stub"

# Old compose that only supports go-template: JSON ps returns nothing, so the
# go-template fallback parses the {{.Service}} rows.
legacy_status=$(DOCKER_CMD="$PREFIX/bin/ot-docker-legacy-stub" bash "$OTCTL" status 2>&1 < /dev/null)
grep -q '^SERVICE.*STATE.*HEALTH' <<<"$legacy_status"
grep -q 'controller  running  healthy' <<<"$legacy_status"

# No --format support at all: last-resort table path uses docker inspect.
cat > "$PREFIX/bin/ot-docker-inspect-stub" <<'STUB'
#!/usr/bin/env bash
cmd="$1"; shift
case "$cmd" in
  compose)
    sub=""; for a in "$@"; do [[ "$a" == ps ]] && sub=ps; done
    if [[ "$sub" == ps ]]; then
      if [[ "$*" == *"--format"* ]]; then
        exit 0
      fi
printf 'NAME\tIMAGE\tCOMMAND\tSERVICE\tSTATUS\n'
        printf 'ot_controller\timg\t""\tcontroller\trunning\n'
      exit 0
    fi
    printf 'stub compose: %s\n' "$*"
    ;;
  inspect)
    fmt=""; for a in "$@"; do [[ "$a" == -f ]] && fmt=1; done
    if [[ "$fmt" == 1 ]]; then
      if [[ "$*" == *".State.Health"* ]]; then
        printf 'healthy\n'
      else
        printf 'running\n'
      fi
    else
      printf 'running\n'
    fi
    ;;
  *) printf 'stub docker: %s\n' "$cmd $*" ;;
esac
STUB
chmod +x "$PREFIX/bin/ot-docker-inspect-stub"

# otctl calls `docker inspect` directly (not via DOCKER_CMD), so expose the
# stub as `docker` on PATH for the inspect-based status path.
cp "$PREFIX/bin/ot-docker-inspect-stub" "$PREFIX/bin/docker"
chmod +x "$PREFIX/bin/docker"

inspect_status=$(PATH="$PREFIX/bin:$PATH" DOCKER_CMD="$PREFIX/bin/ot-docker-inspect-stub" bash "$OTCTL" status 2>&1 < /dev/null)
grep -q '^SERVICE.*STATE.*HEALTH' <<<"$inspect_status"
grep -q 'controller  running  healthy' <<<"$inspect_status"
rm -f "$PREFIX/bin/docker"

# status with no services at all (all fallbacks empty, no inspect on PATH).
cat > "$PREFIX/bin/ot-docker-empty-stub" <<'STUB'
#!/usr/bin/env bash
exit 0
STUB
chmod +x "$PREFIX/bin/ot-docker-empty-stub"
empty_status=$(PATH="$PREFIX/bin:$PATH" DOCKER_CMD="$PREFIX/bin/ot-docker-empty-stub" bash "$OTCTL" status 2>&1 < /dev/null)
grep -q '(no services)' <<<"$empty_status"
rm -f "$PREFIX/bin/docker"

# volumes list/rm/prune flow.
: > "$DOCKER_CALLS_LOG"
vol_list=$(bash "$OTCTL" volumes < /dev/null 2>&1)
grep -q 'opentalk_controller' <<<"$vol_list"
grep -q 'opentalk_postgres' <<<"$vol_list"
grep -q 'foreign_other' <<<"$vol_list" || true
bash "$OTCTL" volumes rm opentalk_controller < /dev/null >/dev/null
grep -q 'volume rm opentalk_controller' "$DOCKER_CALLS_LOG"
set +e
prune_out=$(bash "$OTCTL" volumes prune 2>&1 < /dev/null)
prune_rc=$?
set -e
[[ $prune_rc -eq 3 ]] || { echo "otctl volumes prune without --force must exit 3 (got $prune_rc)." >&2; exit 1; }
grep -q 'Re-run: otctl volumes prune --force' <<<"$prune_out"
bash "$OTCTL" volumes prune --force < /dev/null >/dev/null
set +e
bogus_vol=$(bash "$OTCTL" volumes bogus 2>&1 < /dev/null)
set -e
grep -q 'unknown volumes action' <<<"$bogus_vol" || {
  echo "otctl volumes with unknown action must report an error." >&2; exit 1; }

# volumes rm without a volume name reports usage and exits 2.
set +e
vol_rm_none=$(bash "$OTCTL" volumes rm 2>&1 < /dev/null)
vol_rm_none_rc=$?
set -e
[[ $vol_rm_none_rc -eq 2 ]] || { echo "otctl volumes rm without a name must exit 2 (got $vol_rm_none_rc)." >&2; exit 1; }
grep -q 'specify one or more volume names' <<<"$vol_rm_none"

# volumes prune with no project volumes is a no-op.
: > "$DOCKER_CALLS_LOG"
cat > "$PREFIX/bin/ot-docker-stub" <<'STUB'
#!/usr/bin/env bash
printf 'foreign_other\nforeign_extra\n'
STUB
prune_none=$(bash "$OTCTL" volumes prune 2>&1 < /dev/null)
grep -q 'no project volumes to prune' <<<"$prune_none"
if grep -q 'volume rm' "$DOCKER_CALLS_LOG"; then
  echo "otctl volumes prune must not remove non-project volumes." >&2; exit 1
fi

# backup ls without a backup root reports it and exits 3.
set +e
backup_ls_missing=$(env -u OT_BACKUP_DIR bash "$OTCTL" backup ls 2>&1 < /dev/null)
backup_ls_missing_rc=$?
set -e
[[ $backup_ls_missing_rc -eq 3 ]] || { echo "otctl backup ls without a root must exit 3 (got $backup_ls_missing_rc)." >&2; exit 1; }
grep -q 'Backup root not found' <<<"$backup_ls_missing"

# undeploy removes compose.yml and services contents but keeps the directory.
bash "$OTCTL" undeploy < /dev/null >/dev/null 2>&1
[[ ! -f "$PREFIX/compose.yml" ]] || { echo "otctl undeploy must remove compose.yml." >&2; exit 1; }
mkdir -p "$PREFIX/services"
touch "$PREFIX/services/svc.yml"
cat > "$PREFIX/compose.yml" <<'YAML'
services:
  controller:
    image: example
YAML
bash "$OTCTL" undeploy < /dev/null >/dev/null 2>&1
[[ ! -f "$PREFIX/compose.yml" ]] || { echo "otctl undeploy must remove compose.yml." >&2; exit 1; }
[[ ! -f "$PREFIX/services/svc.yml" ]] || { echo "otctl undeploy must remove services contents." >&2; exit 1; }
[[ -d "$PREFIX/services" ]] || { echo "otctl undeploy must keep the services directory." >&2; exit 1; }

# A second undeploy with no compose.yml and an empty services directory is a no-op.
idempotent_undeploy=$(bash "$OTCTL" undeploy 2>&1 < /dev/null)
grep -q 'compose.yml not present' <<<"$idempotent_undeploy"
grep -q 'services directory empty' <<<"$idempotent_undeploy"
[[ ! -f "$PREFIX/compose.yml" ]] || { echo "otctl undeploy must stay idempotent." >&2; exit 1; }

# up without compose.yml reports the missing compose file (exit 1).
set +e
no_compose_up=$(bash "$OTCTL" up 2>&1 < /dev/null)
no_compose_up_rc=$?
set -e
[[ $no_compose_up_rc -eq 1 ]] || { echo "otctl up without compose.yml must exit 1 (got $no_compose_up_rc)." >&2; exit 1; }
grep -q 'compose file not found' <<<"$no_compose_up"

# migrate forwards to the S3 migration playbook.
: > "$CALLS_LOG"
bash "$OTCTL" migrate minio-to-garage --yes < /dev/null >/dev/null 2>&1
grep -q 's3_migrate.yml' "$CALLS_LOG"
grep -q 's3_migration_source=minio' "$CALLS_LOG"
bash "$OTCTL" migrate garage-to-minio --no-deploy --yes < /dev/null >/dev/null 2>&1
grep -q 's3_migration_source=garage' "$CALLS_LOG"
grep -q 's3_migrate_run_deploy=false' "$CALLS_LOG"
# Regression: stripping --no-deploy must not leave an empty argument, which
# ansible-playbook reads as a playbook path. Other arguments must survive.
bash "$OTCTL" migrate minio-to-garage --no-deploy --yes -v < /dev/null >/dev/null 2>&1
tail -n 1 "$CALLS_LOG" | grep -q -- ' -v$'
[[ ! -e "$CALLS_LOG.empty-arg" ]] || {
  echo "otctl migrate --no-deploy passed an empty argument to ansible-playbook:" >&2
  cat "$CALLS_LOG.empty-arg" >&2; exit 1; }
! tail -n 1 "$CALLS_LOG" | grep -q -- '--yes' || {
  echo "otctl migrate must not forward --yes to ansible-playbook." >&2; exit 1; }
# The plan names the downtime and what happens afterwards.
migrate_plan=$(bash "$OTCTL" migrate garage-to-minio --no-deploy --yes 2>&1 < /dev/null)
grep -q 'MIGRATION PLAN: copy all S3 objects from garage to minio' <<<"$migrate_plan"
grep -q 'running meetings are interrupted' <<<"$migrate_plan"
grep -q 'controller stays stopped' <<<"$migrate_plan"
# Without --yes a non-interactive session must refuse and run nothing.
: > "$CALLS_LOG"
set +e
migrate_noyes=$(bash "$OTCTL" migrate minio-to-garage 2>&1 < /dev/null)
migrate_noyes_rc=$?
set -e
[[ $migrate_noyes_rc -eq 1 ]] || { echo "otctl migrate without --yes must exit 1 (got $migrate_noyes_rc)." >&2; exit 1; }
grep -q 're-run with --yes' <<<"$migrate_noyes"
[[ ! -s "$CALLS_LOG" ]] || { echo "otctl migrate without confirmation must not run a playbook." >&2; exit 1; }
# status is read-only and needs no confirmation.
bash "$OTCTL" migrate status < /dev/null >/dev/null 2>&1
grep -q 'migrate_status.yml' "$CALLS_LOG"
migrate_help=$(bash "$OTCTL" migrate help 2>&1 < /dev/null)
grep -q 'migrate minio-to-garage' <<<"$migrate_help"
grep -q 'migrate status' <<<"$migrate_help"
set +e
migrate_bogus=$(bash "$OTCTL" migrate bogus-depth 2>&1 < /dev/null)
migrate_bogus_rc=$?
set -e
[[ $migrate_bogus_rc -eq 1 ]] || { echo "otctl migrate with an unknown migration must exit 1." >&2; exit 1; }
grep -q 'Unknown migration: bogus-depth' <<<"$migrate_bogus" || {
  echo "otctl migrate with unknown migration must report an error." >&2; exit 1; }

# import users builds extra vars and forwards to the import playbook.
printf 'first,last\nAlice,User\n' > "$TMP_DIR/users.csv"
: > "$CALLS_LOG"
bash "$OTCTL" import users --users-file="$TMP_DIR/users.csv" --no-temporary \
  --password-reset-email --default-password='ChangeMe!' --password-length=32 foo < /dev/null >/dev/null
grep -q 'import_users.yml' "$CALLS_LOG"
grep -qF "import_users_file=$TMP_DIR/users.csv" "$CALLS_LOG"
grep -q 'import_users_password_temporary=false' "$CALLS_LOG"
grep -q 'import_users_force_password_reset_email=true' "$CALLS_LOG"
grep -qF 'import_users_default_password=ChangeMe!' "$CALLS_LOG"
grep -q 'import_users_password_length=32' "$CALLS_LOG"
set +e
import_no_file=$(bash "$OTCTL" import users 2>&1 < /dev/null)
import_bogus=$(bash "$OTCTL" import bogus 2>&1 < /dev/null)
set -e
grep -q 'Missing --users-file' <<<"$import_no_file" || {
  echo "otctl import users without --users-file must report an error." >&2; exit 1; }
grep -q 'Unknown import target' <<<"$import_bogus" || {
  echo "otctl import with unknown target must report an error." >&2; exit 1; }

# import help (no sub-target) prints the usage and exits 0.
import_help=$(bash "$OTCTL" import help 2>&1 < /dev/null)
grep -q 'otctl import users USAGE' <<<"$import_help"
import_noarg=$(bash "$OTCTL" import 2>&1 < /dev/null)
grep -q 'otctl import users USAGE' <<<"$import_noarg"

# import users supports separated option forms and rejects empty values.
: > "$CALLS_LOG"
bash "$OTCTL" import users --users-file "$TMP_DIR/users.csv" \
  --default-password 'SeparatedPass!' --password-length 40 < /dev/null >/dev/null
grep -qF "import_users_file=$TMP_DIR/users.csv" "$CALLS_LOG"
grep -qF 'import_users_default_password=SeparatedPass!' "$CALLS_LOG"
grep -q 'import_users_password_length=40' "$CALLS_LOG"
set +e
import_uflag_empty=$(bash "$OTCTL" import users --users-file 2>&1 < /dev/null)
import_pflag_empty=$(bash "$OTCTL" import users --users-file="$TMP_DIR/users.csv" --default-password 2>&1 < /dev/null)
import_len_empty=$(bash "$OTCTL" import users --users-file="$TMP_DIR/users.csv" --password-length 2>&1 < /dev/null)
set -e
grep -q -- '--users-file requires a value' <<<"$import_uflag_empty" || {
  echo "otctl import users with empty --users-file must report an error." >&2; exit 1; }
grep -q -- '--default-password requires a value' <<<"$import_pflag_empty" || {
  echo "otctl import users with empty --default-password must report an error." >&2; exit 1; }
grep -q -- '--password-length requires a value' <<<"$import_len_empty" || {
  echo "otctl import users with empty --password-length must report an error." >&2; exit 1; }

# restore --yes is still required even when all sub-mode args are present.
set +e
restore_interactive=$(bash "$OTCTL" restore config 2>&1 < /dev/null)
restore_interactive_rc=$?
set -e
[[ $restore_interactive_rc -eq 1 ]] || { echo "otctl restore without --yes must exit 1 (got $restore_interactive_rc)." >&2; exit 1; }
grep -q 're-run with --yes' <<<"$restore_interactive"

# help/usage paths and unknown subcommands.
help_out=$(bash "$OTCTL" help 2>&1 < /dev/null)
grep -q 'OpenTalk Control (otctl)' <<<"$help_out"
noargs_out=$(bash "$OTCTL" 2>&1 < /dev/null)
grep -q 'otctl status|logs|restart' <<<"$noargs_out"
set +e
unknown_out=$(bash "$OTCTL" bogus-subcommand 2>&1 < /dev/null)
unknown_rc=$?
set -e
[[ $unknown_rc -eq 1 ]] || { echo "otctl unknown subcommand must exit 1 (got $unknown_rc)." >&2; exit 1; }
grep -q 'Unknown subcommand: bogus-subcommand' <<<"$unknown_out"

echo "otctl smoke tests passed."