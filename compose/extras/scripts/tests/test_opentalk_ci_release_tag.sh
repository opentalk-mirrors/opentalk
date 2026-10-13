#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

printf 'opentalk_product_version: v26.4.0\n' > "$TMP_DIR/.versions.yml"

check_tag() {
  (
    cd "$TMP_DIR"
    CI_COMMIT_TAG="$1" bash "$REPO_ROOT/extras/ci/gitlab/check-release-tag.sh" "$2"
  )
}

stable_suffix='(-[0-9]+)?'
pre_release_suffix='(-(alpha|beta)(\.[0-9]+)?)?'

check_tag compose-v26.4.0 "$stable_suffix"
check_tag compose-v26.4.0-1 "$stable_suffix"
check_tag compose-v26.4.0-beta.2 "$pre_release_suffix"

# Tags of the components or without the compose- prefix must never pass as a
# compose release, and neither may a tag for a different product version.
for tag in v26.4.0 release-v26.4.0 compose-v26.4.1 compose-v26.4.0-beta.2; do
  if check_tag "$tag" "$stable_suffix" >/dev/null 2>&1; then
    echo "Release tag check accepted $tag for product version v26.4.0." >&2
    exit 1
  fi
done

echo "CI release tag tests passed."
