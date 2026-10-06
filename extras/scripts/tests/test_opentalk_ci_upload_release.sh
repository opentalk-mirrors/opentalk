#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/artifacts" "$TMP_DIR/bin"
touch "$TMP_DIR/artifacts/opentalk-compose_26.2.0+gitabcdef12-1_amd64.deb"
cat > "$TMP_DIR/.versions.yml" <<'YAML'
opentalk_product_version: v9.9.9
YAML

cat > "$TMP_DIR/bin/curl" <<'SH'
#!/usr/bin/env bash
echo "curl $*" >> "$UPLOAD_TEST_LOG"
SH

cat > "$TMP_DIR/bin/openrepo" <<'SH'
#!/usr/bin/env bash
echo "openrepo $*" >> "$UPLOAD_TEST_LOG"
SH

cat > "$TMP_DIR/bin/sleep" <<'SH'
#!/usr/bin/env bash
:
SH

chmod +x "$TMP_DIR/bin/curl" "$TMP_DIR/bin/openrepo" "$TMP_DIR/bin/sleep"

run_upload() {
  : > "$TMP_DIR/upload.log"
  (
    cd "$TMP_DIR"
    env -u CI_COMMIT_TAG \
    PATH="$TMP_DIR/bin:$PATH" \
    UPLOAD_TEST_LOG="$TMP_DIR/upload.log" \
    CI_JOB_TOKEN=fake-token \
    CI_API_V4_URL=https://gitlab.example/api/v4 \
    CI_PROJECT_ID=795 \
    CI_COMMIT_SHORT_SHA=abcdef12 \
    NEXUS_REPO_BASE_URL=https://nexus.example \
    NEXUS_API_USER=fake-user \
    NEXUS_API_KEY=fake-key \
    NEXUS_REPO_NAME_DEB_STABLE=deb-stable \
    NEXUS_REPO_NAME_DEB_EXPERIMENTAL=deb-experimental \
    "$@"
  )
}

run_upload bash "$REPO_ROOT/extras/ci/gitlab/upload-release.sh" deb experimental
grep -q '/packages/generic/opentalk-compose/v9.9.9+gitabcdef12/' "$TMP_DIR/upload.log" \
  || { cat "$TMP_DIR/upload.log" >&2; echo "experimental upload must derive a package version without CI_COMMIT_TAG" >&2; exit 1; }
grep -q 'https://nexus.example/deb-experimental/' "$TMP_DIR/upload.log" \
  || { cat "$TMP_DIR/upload.log" >&2; echo "experimental upload must use the experimental Nexus repository" >&2; exit 1; }

run_upload env CI_COMMIT_TAG=v26.2.0 bash "$REPO_ROOT/extras/ci/gitlab/upload-release.sh" deb stable
grep -q '/packages/generic/opentalk-compose/v26.2.0/' "$TMP_DIR/upload.log" \
  || { cat "$TMP_DIR/upload.log" >&2; echo "stable upload must use CI_COMMIT_TAG as package version" >&2; exit 1; }
grep -q 'https://nexus.example/deb-stable/' "$TMP_DIR/upload.log" \
  || { cat "$TMP_DIR/upload.log" >&2; echo "stable upload must use the stable Nexus repository" >&2; exit 1; }

echo "CI upload-release tests passed."