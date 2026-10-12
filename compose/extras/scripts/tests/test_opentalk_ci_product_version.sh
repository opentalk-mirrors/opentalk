#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

printf 'opentalk_product_version: v0.0.0\nother_version: v1.2.3\n' > "$TMP_DIR/.versions.yml"

(
  cd "$TMP_DIR"
  CI_COMMIT_TAG=compose-v26.2.0 bash "$REPO_ROOT/extras/ci/gitlab/update-product-version.sh"
)
grep -q '^opentalk_product_version: v26.2.0$' "$TMP_DIR/.versions.yml"
grep -q '^other_version: v1.2.3$' "$TMP_DIR/.versions.yml"

before=$(cat "$TMP_DIR/.versions.yml")
(
  cd "$TMP_DIR"
  env -u CI_COMMIT_TAG bash "$REPO_ROOT/extras/ci/gitlab/update-product-version.sh"
)
[[ "$(cat "$TMP_DIR/.versions.yml")" == "$before" ]]

if (
  cd "$TMP_DIR"
  CI_COMMIT_TAG=compose-v26.2 bash "$REPO_ROOT/extras/ci/gitlab/update-product-version.sh"
); then
  echo "Invalid product version tag was accepted." >&2
  exit 1
fi

if (
  cd "$TMP_DIR"
  CI_COMMIT_TAG=v26.2.0 bash "$REPO_ROOT/extras/ci/gitlab/update-product-version.sh"
); then
  echo "A tag without the compose- prefix was accepted." >&2
  exit 1
fi

echo "CI product version tests passed."
