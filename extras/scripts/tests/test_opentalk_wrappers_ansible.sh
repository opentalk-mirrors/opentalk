#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
WRAPPERS="$REPO_ROOT/extras/packaging/wrappers"
ANSIBLE_W="$WRAPPERS/ot-ansible"
PLAYBOOK_W="$WRAPPERS/ot-ansible-playbook"
GALAXY_W="$WRAPPERS/ot-ansible-galaxy"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

PREFIX="$TMP_DIR/prefix"
export OT_PREFIX="$PREFIX"
export NO_COLOR=1

VENV_BIN="$PREFIX/runtime/venv/bin"
PORTABLE_PY="$PREFIX/runtime/python/bin/python3"
VENDOR="$PREFIX/runtime/vendor/collections"
AUTOMATION="$PREFIX/automation"
ANSIBLE_BIN="$VENV_BIN/ansible"
PLAYBOOK_BIN="$VENV_BIN/ansible-playbook"
GALAXY_BIN="$VENV_BIN/ansible-galaxy"

ANSIBLE_LOG="$TMP_DIR/ansible.log"
PLAYBOOK_LOG="$TMP_DIR/playbook.log"
GALAXY_LOG="$TMP_DIR/galaxy.log"

mkdir -p "$VENV_BIN" "$PORTABLE_PY%/" "$AUTOMATION"
rm -f "$ANSIBLE_LOG" "$PLAYBOOK_LOG" "$GALAXY_LOG"

make_bin() {
  # shellcheck disable=SC2317
  local path=$1
  local log=$2
  printf '#!/usr/bin/env bash\n' > "$path"
  # shellcheck disable=SC2016
  printf '{ printf "ARGS:"; printf " <%%s>" "$@"; printf "\\nACP:%%s\\nINTERPRETER:%%s\\nCONFIG:%%s\\n" "${ANSIBLE_COLLECTIONS_PATHS:-}" "${ANSIBLE_PYTHON_INTERPRETER:-}" "${ANSIBLE_CONFIG:-}"; } >> %s\n' "$log" >> "$path"
  chmod +x "$path"
}

make_python() {
  local path=$1
  local mode=$2 # ok|fail
  printf '#!/usr/bin/env bash\n' > "$path"
  if [[ $mode == ok ]]; then
    printf 'exit 0\n' >> "$path"
  else
    printf 'exit 1\n' >> "$path"
  fi
  chmod +x "$path"
}

# ---------------------------------------------------------------------------
# ot-ansible
# ---------------------------------------------------------------------------

# 1. Missing ansible binary fails fast.
set +e
missing_out=$(bash "$ANSIBLE_W" ping 2>&1 < /dev/null)
missing_rc=$?
set -e
[[ $missing_rc -eq 1 ]] || { echo "ot-ansible must exit 1 when ansible is missing (got $missing_rc)." >&2; exit 1; }
grep -q 'ansible unavailable' <<<"$missing_out"

# 2. venv python present (requests OK), no -e in args: interpreter extra var injected.
make_bin "$ANSIBLE_BIN" "$ANSIBLE_LOG"
make_python "$VENV_BIN/python3" ok
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$ANSIBLE_W" ping -i inventory > /dev/null
grep -q "ARGS: <-e> <ansible_python_interpreter=$VENV_BIN/python3> <ping> <-i> <inventory>" "$ANSIBLE_LOG"
grep -q "INTERPRETER:$VENV_BIN/python3" "$ANSIBLE_LOG"

# 3. A -e flag in the caller's args suppresses injection.
: > "$ANSIBLE_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$ANSIBLE_W" ping -e custom > /dev/null
grep -q "^ARGS: <ping> <-e> <custom>" "$ANSIBLE_LOG"

# 4. --extra-vars also suppresses injection.
: > "$ANSIBLE_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$ANSIBLE_W" ping --extra-vars foo > /dev/null
grep -q "^ARGS: <ping> <--extra-vars> <foo>" "$ANSIBLE_LOG"

# 5. venv python without requests fails.
make_python "$VENV_BIN/python3" fail
set +e
req_out=$(env -u ANSIBLE_COLLECTIONS_PATHS bash "$ANSIBLE_W" ping 2>&1 < /dev/null)
req_rc=$?
set -e
[[ $req_rc -eq 1 ]] || { echo "ot-ansible must exit 1 when requests is missing (got $req_rc)." >&2; exit 1; }
grep -q 'requests missing from embedded venv' <<<"$req_out"

# 6. No venv python: portable python is used.
rm -f "$VENV_BIN/python3"
make_python "$PORTABLE_PY" ok
: > "$ANSIBLE_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$ANSIBLE_W" ping > /dev/null
grep -q "ARGS: <-e> <ansible_python_interpreter=$PORTABLE_PY> <ping>" "$ANSIBLE_LOG"
grep -q "INTERPRETER:$PORTABLE_PY" "$ANSIBLE_LOG"

# 7. No python at all: no interpreter extra var, ansible still runs.
rm -f "$PORTABLE_PY" "$VENV_BIN/python3"
: > "$ANSIBLE_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$ANSIBLE_W" ping > /dev/null
grep -q "^ARGS: <ping>" "$ANSIBLE_LOG"
grep -q '^INTERPRETER:$' "$ANSIBLE_LOG"

# 8. Pre-set ANSIBLE_COLLECTIONS_PATHS is respected untouched.
make_python "$VENV_BIN/python3" ok
: > "$ANSIBLE_LOG"
ANSIBLE_COLLECTIONS_PATHS=custom bash "$ANSIBLE_W" ping > /dev/null
grep -q "ACP:custom" "$ANSIBLE_LOG"

# 9. Vendor dir + empty cfg: default search chain.
make_python "$VENV_BIN/python3" ok
mkdir -p "$VENDOR/ansible_collections" "$AUTOMATION"
printf '[defaults]\n' > "$AUTOMATION/ansible.cfg"
: > "$ANSIBLE_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$ANSIBLE_W" ping > /dev/null
grep -q "ACP:$VENDOR:$HOME/.ansible/collections:/usr/share/ansible/collections" "$ANSIBLE_LOG"

# 10. Vendor dir + cfg with another path: VENDOR is prepended.
printf '[defaults]\ncollections_path = /usr/share/ansible/collections\n' > "$AUTOMATION/ansible.cfg"
: > "$ANSIBLE_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$ANSIBLE_W" ping > /dev/null
grep -q "ACP:$VENDOR:/usr/share/ansible/collections" "$ANSIBLE_LOG"

# 11. cfg already contains VENDOR: used as-is.
printf '[defaults]\ncollections_path = %s\n' "$VENDOR" > "$AUTOMATION/ansible.cfg"
: > "$ANSIBLE_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$ANSIBLE_W" ping > /dev/null
grep -q "ACP:$VENDOR\$" "$ANSIBLE_LOG"

# 12. No vendor dir: cfg path only.
rm -rf "$VENDOR"
printf '[defaults]\ncollections_path = /usr/share/ansible/collections\n' > "$AUTOMATION/ansible.cfg"
: > "$ANSIBLE_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$ANSIBLE_W" ping > /dev/null
grep -q "ACP:/usr/share/ansible/collections" "$ANSIBLE_LOG"

# 13. No cfg, no vendor: ANSIBLE_COLLECTIONS_PATHS stays unset.
rm -f "$AUTOMATION/ansible.cfg"
: > "$ANSIBLE_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$ANSIBLE_W" ping > /dev/null
grep -q '^ACP:$' "$ANSIBLE_LOG"

# ---------------------------------------------------------------------------
# ot-ansible-playbook
# ---------------------------------------------------------------------------

# 14. Missing playbook binary fails fast.
set +e
pb_missing_out=$(bash "$PLAYBOOK_W" site.yml 2>&1 < /dev/null)
pb_missing_rc=$?
set -e
[[ $pb_missing_rc -eq 1 ]] || { echo "ot-ansible-playbook must exit 1 when ansible-playbook is missing (got $pb_missing_rc)." >&2; exit 1; }
grep -q 'ansible-playbook unavailable' <<<"$pb_missing_out"

# 15. Interpreter extra var is always injected additively (even with -e).
make_bin "$PLAYBOOK_BIN" "$PLAYBOOK_LOG"
make_python "$VENV_BIN/python3" ok
: > "$PLAYBOOK_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$PLAYBOOK_W" -e custom site.yml > /dev/null
grep -q "ARGS: <-e> <ansible_python_interpreter=$VENV_BIN/python3> <-e> <custom> <site.yml>" "$PLAYBOOK_LOG"

# 16. venv python without requests fails.
make_python "$VENV_BIN/python3" fail
set +e
pb_req_out=$(env -u ANSIBLE_COLLECTIONS_PATHS bash "$PLAYBOOK_W" site.yml 2>&1 < /dev/null)
pb_req_rc=$?
set -e
[[ $pb_req_rc -eq 1 ]] || { echo "ot-ansible-playbook must exit 1 when requests is missing (got $pb_req_rc)." >&2; exit 1; }
grep -q 'requests missing from embedded venv' <<<"$pb_req_out"

# 17. No venv python: portable python interpreter extra var.
make_python "$VENV_BIN/python3" ok
rm -f "$VENV_BIN/python3"
make_python "$PORTABLE_PY" ok
: > "$PLAYBOOK_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$PLAYBOOK_W" site.yml > /dev/null
grep -q "ARGS: <-e> <ansible_python_interpreter=$PORTABLE_PY> <site.yml>" "$PLAYBOOK_LOG"

# 18. No python at all: runs without interpreter extra var.
rm -f "$PORTABLE_PY" "$VENV_BIN/python3"
: > "$PLAYBOOK_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$PLAYBOOK_W" site.yml > /dev/null
grep -q "^ARGS: <site.yml>" "$PLAYBOOK_LOG"
grep -q '^INTERPRETER:$' "$PLAYBOOK_LOG"

# 18a. Vendor dir + empty cfg: default search chain (case `"::"`).
mkdir -p "$VENDOR/ansible_collections" "$AUTOMATION"
printf '[defaults]\n' > "$AUTOMATION/ansible.cfg"
: > "$PLAYBOOK_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$PLAYBOOK_W" site.yml > /dev/null
grep -q "ACP:$VENDOR:$HOME/.ansible/collections:/usr/share/ansible/collections" "$PLAYBOOK_LOG"

# 18b. cfg already contains VENDOR: used as-is (case `*":$VENDOR:"*`).
printf '[defaults]\ncollections_path = %s\n' "$VENDOR" > "$AUTOMATION/ansible.cfg"
: > "$PLAYBOOK_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$PLAYBOOK_W" site.yml > /dev/null
grep -q "ACP:$VENDOR\$" "$PLAYBOOK_LOG"

# 18c. cfg with another path: VENDOR is prepended (case `*)`).
printf '[defaults]\ncollections_path = /usr/share/ansible/collections\n' > "$AUTOMATION/ansible.cfg"
: > "$PLAYBOOK_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$PLAYBOOK_W" site.yml > /dev/null
grep -q "ACP:$VENDOR:/usr/share/ansible/collections" "$PLAYBOOK_LOG"

# 18d. No vendor dir: cfg path only.
rm -rf "$VENDOR"
: > "$PLAYBOOK_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS -u ANSIBLE_PYTHON_INTERPRETER bash "$PLAYBOOK_W" site.yml > /dev/null
grep -q "ACP:/usr/share/ansible/collections" "$PLAYBOOK_LOG"
rm -f "$AUTOMATION/ansible.cfg"

# ---------------------------------------------------------------------------
# ot-ansible-galaxy
# ---------------------------------------------------------------------------

# 19. Missing galaxy binary fails fast.
set +e
gal_missing_out=$(bash "$GALAXY_W" collection list 2>&1 < /dev/null)
gal_missing_rc=$?
set -e
[[ $gal_missing_rc -eq 1 ]] || { echo "ot-ansible-galaxy must exit 1 when ansible-galaxy is missing (got $gal_missing_rc)." >&2; exit 1; }
grep -q 'ansible-galaxy unavailable' <<<"$gal_missing_out"

# 20. collection install with vendor dir: --collections-path injected.
make_bin "$GALAXY_BIN" "$GALAXY_LOG"
make_python "$VENV_BIN/python3" ok
mkdir -p "$VENDOR/ansible_collections" "$AUTOMATION"
: > "$GALAXY_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS bash "$GALAXY_W" collection install community.general --version '1.0.0' > /dev/null
grep -q "^ARGS: <collection> <install> <--collections-path> <$VENDOR> <community.general> <--version> <1.0.0>" "$GALAXY_LOG"

# 21. Existing -p flag: no injection.
: > "$GALAXY_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS bash "$GALAXY_W" collection install -p /x community.general > /dev/null
grep -q "^ARGS: <collection> <install> <-p> </x> <community.general>" "$GALAXY_LOG"

# 22. Existing --collections-path flag: no injection.
: > "$GALAXY_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS bash "$GALAXY_W" collection install --collections-path /x community.general > /dev/null
grep -q "^ARGS: <collection> <install> <--collections-path> </x> <community.general>" "$GALAXY_LOG"

# 23. Existing --collections-path=... flag: no injection.
: > "$GALAXY_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS bash "$GALAXY_W" collection install --collections-path=/x community.general > /dev/null
grep -q "^ARGS: <collection> <install> <--collections-path=/x> <community.general>" "$GALAXY_LOG"

# 24. cfg already contains VENDOR: used as-is (case `*":$VENDOR:"*`).
printf '[defaults]\ncollections_path = %s\n' "$VENDOR" > "$AUTOMATION/ansible.cfg"
: > "$GALAXY_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS bash "$GALAXY_W" collection install community.general > /dev/null
grep -q "ACP:$VENDOR\$" "$GALAXY_LOG"

# 25. cfg with another path: VENDOR is prepended (case `*)`).
printf '[defaults]\ncollections_path = /usr/share/ansible/collections\n' > "$AUTOMATION/ansible.cfg"
: > "$GALAXY_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS bash "$GALAXY_W" collection install community.general > /dev/null
grep -q "ACP:$VENDOR:/usr/share/ansible/collections" "$GALAXY_LOG"

# 26. Non-collection subcommand: no injection.
: > "$GALAXY_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS bash "$GALAXY_W" role list > /dev/null
grep -q "^ARGS: <role> <list>" "$GALAXY_LOG"

# 27. collection with unsupported action: no injection.
: > "$GALAXY_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS bash "$GALAXY_W" collection verify community.general > /dev/null
grep -q "^ARGS: <collection> <verify> <community.general>" "$GALAXY_LOG"

# 28. No vendor dir: no collection path injection.
rm -rf "$VENDOR"
: > "$GALAXY_LOG"
env -u ANSIBLE_COLLECTIONS_PATHS bash "$GALAXY_W" collection install community.general > /dev/null
grep -q "^ARGS: <collection> <install> <community.general>" "$GALAXY_LOG"

echo "ansible wrapper smoke tests passed."