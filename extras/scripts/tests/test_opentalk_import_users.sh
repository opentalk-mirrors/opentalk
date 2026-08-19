#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

if ! command -v ansible-playbook >/dev/null 2>&1; then
  echo "ansible-playbook is required to run the import_users test." >&2
  exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to run the import_users test." >&2
  exit 1
fi

INVENTORY="$TMP_DIR/inventory"
printf 'localhost ansible_connection=local ansible_python_interpreter=%s\n' "$(command -v python3)" > "$INVENTORY"

PLAYBOOK="$TMP_DIR/playbook.yml"
cat > "$PLAYBOOK" <<'YAML'
- hosts: localhost
  gather_facts: false
  roles:
    - role: opentalk_compose_import_users
  vars:
    import_users_file: "{{ lookup('ansible.builtin.env', 'OT_TEST_IMPORT_FILE') }}"
  post_tasks:
    - name: Export enriched user list as JSON
      ansible.builtin.copy:
        content: "{{ _import_users_enriched | to_json }}"
        dest: "{{ lookup('ansible.builtin.env', 'OT_TEST_RESULT_FILE') }}"
  tags:
    - import_users_logic
YAML

run_logic() {
  local results=$1
  local file=$2
  shift 2
  ANSIBLE_COLLECTIONS_PATH="$REPO_ROOT/.ansible/collections" \
  ANSIBLE_FILTER_PLUGINS="$REPO_ROOT/automation/playbooks/filter_plugins" \
  ANSIBLE_ROLES_PATH="$REPO_ROOT/automation/roles" \
  OT_TEST_IMPORT_FILE="$file" \
  OT_TEST_RESULT_FILE="$results" \
  ansible-playbook --tags import_users_logic --skip-tags import_users_keycloak \
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

CSV_FILE="$TMP_DIR/users.csv"
cat > "$CSV_FILE" <<'CSV'
username,email,firstName,lastName,password,password_reset_email,tariff_id
inlinepw,inlinepw@example.test,Inline,Pw,plain-secret,false,free
defaulted,defaulted@example.test,Defaulted,Example,,false,premium
resettoken,resettoken@example.test,Reset,Token,,true,free
CSV

JSON_FILE="$TMP_DIR/users.json"
cat > "$JSON_FILE" <<'JSON'
[
  {"username": "jsonuser", "email": "jsonuser@example.test", "firstName": "Json", "lastName": "User", "password": "json-secret", "password_reset_email": false, "tariff_id": "free"},
  {"username": "jsonreset", "email": "jsonreset@example.test", "firstName": "Json", "lastName": "Reset", "password_reset_email": true, "tariff_id": "premium"}
]
JSON

# --- CSV: default password fallback ---
# inlinepw keeps its inline password, defaulted gets the fallback, resettoken is
# flagged for the password-reset email (send_email true) and must NOT get a password.
RESULT="$TMP_DIR/csv-default.json"
run_logic "$RESULT" "$CSV_FILE" -e "import_users_default_password=fallback-no-auto" \
  -e "import_users_password_length=24"
pw_inline=$(jq -r '.[] | select(.username=="inlinepw") | ._password' "$RESULT")
pw_defaulted=$(jq -r '.[] | select(.username=="defaulted") | ._password' "$RESULT")
[[ $pw_inline == "plain-secret" ]] || { echo "inline password not preserved (got: $pw_inline)." >&2; exit 1; }
[[ $pw_defaulted == "fallback-no-auto" ]] || { echo "default password fallback not applied (got: $pw_defaulted)." >&2; exit 1; }
jq -e '.[] | select(.username=="resettoken") | ._send_email == true' "$RESULT" >/dev/null \
  || { echo "resettoken must be flagged for password reset email." >&2; exit 1; }

# --- CSV: auto-generated password ---
# Password policy: generated password must match [A-Za-z0-9]{24} (charset
# ascii_letters,digits and configured length), reset-flagged users get none.
RESULT="$TMP_DIR/csv-auto.json"
run_logic "$RESULT" "$CSV_FILE" -e "import_users_password_length=24"
pw_auto_len=$(jq -r '.[] | select(.username=="defaulted") | ._password | length' "$RESULT")
pw_auto_charset=$(jq -r '.[] | select(.username=="defaulted") | ._password' "$RESULT")
[[ $pw_auto_len == "24" ]] || { echo "auto-generated password length mismatch (got $pw_auto_len)." >&2; exit 1; }
[[ $pw_auto_charset =~ ^[A-Za-z0-9]+$ ]] || { echo "auto-generated password charset violation (got: $pw_auto_charset)." >&2; exit 1; }
[[ $pw_auto_charset != "fallback-no-auto" ]] || { echo "auto password must not equal a known literal." >&2; exit 1; }
jq -e '.[] | select(.username=="resettoken") | ._send_email == true' "$RESULT" >/dev/null \
  || { echo "resettoken must be flagged for password reset email." >&2; exit 1; }

# --- CSV: force password reset email for all users ---
RESULT="$TMP_DIR/csv-force.json"
run_logic "$RESULT" "$CSV_FILE" -e "import_users_force_password_reset_email=true"
jq -e 'all(.[]; ._send_email == true)' "$RESULT" >/dev/null \
  || { echo "force_password_reset_email must flag all users." >&2; exit 1; }

# --- JSON input ---
RESULT="$TMP_DIR/json.json"
run_logic "$RESULT" "$JSON_FILE" -e "import_users_default_password=json-dflt"
pw_json=$(jq -r '.[] | select(.username=="jsonuser") | ._password' "$RESULT")
[[ $pw_json == "json-secret" ]] || { echo "JSON inline password not preserved." >&2; exit 1; }
jq -e '.[] | select(.username=="jsonreset") | ._send_email == true' "$RESULT" >/dev/null \
  || { echo "JSON reset flag not honored." >&2; exit 1; }

# --- Validation: unsupported extension fails ---
BAD="$TMP_DIR/users.yaml"
printf 'username: x\n' > "$BAD"
expect_fail "$TMP_DIR/bad.log" "Unsupported file format" \
  run_logic "$TMP_DIR/bad.json" "$BAD"

# --- Validation: missing file fails ---
expect_fail "$TMP_DIR/missing.log" "Import file not found" \
  run_logic "$TMP_DIR/missing.json" "$TMP_DIR/does-not-exist.csv"

# --- Validation: unsupported tariff_id fails against live user profile ---
BAD_TARIFF="$TMP_DIR/users-bad-tariff.csv"
cat > "$BAD_TARIFF" <<'CSV'
username,email,firstName,lastName,password,password_reset_email,tariff_id
golduser,gold@example.test,Gold,User,secret,,goldplan
CSV
expect_fail "$TMP_DIR/tariff.log" "Invalid tariff_id 'goldplan' for user 'golduser'" \
  run_logic "$TMP_DIR/tariff.json" "$BAD_TARIFF" \
  -e "_import_tariff_id_allowed=[\"free\",\"premium\"]"

# --- Validation: supported tariff_id passes against live user profile ---
GOOD_TARIFF="$TMP_DIR/users-good-tariff.csv"
cat > "$GOOD_TARIFF" <<'CSV'
username,email,firstName,lastName,password,password_reset_email,tariff_id
premiumuser,premium@example.test,Pre,Mium,pw,,premium
CSV
run_logic "$TMP_DIR/good-tariff.json" "$GOOD_TARIFF" \
  -e "_import_tariff_id_allowed=[\"free\",\"premium\"]"

echo "import_users logic and validation tests passed."