#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# Regression: compose-deploy read the demo password with yq, which neither the
# CI image nor every development host has, so it failed after a successful
# deploy. It must read the password with the venv Python instead.

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

if ! python3 -c 'import yaml' >/dev/null 2>&1; then
  echo "PyYAML is required to run the compose-deploy demo password test." >&2
  exit 1
fi

mkdir -p "$TMP_DIR/bin" "$TMP_DIR/dev/.venv/bin" "$TMP_DIR/dev/config/secrets"
cat > "$TMP_DIR/bin/uv" <<'SH'
#!/usr/bin/env bash
exit 0
SH
cat > "$TMP_DIR/bin/yq" <<'SH'
#!/usr/bin/env bash
echo "yq must not be called" >&2
exit 1
SH
for tool in ansible-galaxy ansible-playbook; do
  printf '#!/usr/bin/env bash\nexit 0\n' > "$TMP_DIR/dev/.venv/bin/$tool"
done
chmod +x "$TMP_DIR/bin/uv" "$TMP_DIR/bin/yq" "$TMP_DIR"/dev/.venv/bin/*
ln -s "$(command -v python3)" "$TMP_DIR/dev/.venv/bin/python"
echo 'DEMO_USER_PASSWORD: "s3cr3t-demo"' > "$TMP_DIR/dev/config/secrets/secrets.yml"

output="$(
  PATH="$TMP_DIR/bin:$PATH" \
  OPENTALK_DEV_DIR="$TMP_DIR/dev" \
    just --justfile "$REPO_ROOT/justfile" --working-directory "$REPO_ROOT" compose-deploy 2>&1
)" || { echo "$output" >&2; echo "compose-deploy failed" >&2; exit 1; }

[[ "$output" == *"Password: s3cr3t-demo"* ]] \
  || { echo "$output" >&2; echo "compose-deploy must print the demo password from secrets.yml" >&2; exit 1; }

echo "compose-deploy demo password tests passed."
