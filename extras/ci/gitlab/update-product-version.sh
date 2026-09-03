#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

if [ -z "${CI_COMMIT_TAG:-}" ]; then
    exit 0
fi

[[ "$CI_COMMIT_TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-(alpha|beta)(\.[0-9]+)?)?(-[0-9]+)?$ ]] \
    || { echo "Invalid product version tag: $CI_COMMIT_TAG" >&2; exit 1; }

VERSION_FILE=.versions.yml
[[ -f "$VERSION_FILE" ]] || { echo "$VERSION_FILE not found" >&2; exit 1; }

yq -i ".opentalk_product_version = \"$CI_COMMIT_TAG\"" "$VERSION_FILE"
echo "Using product version $CI_COMMIT_TAG from CI_COMMIT_TAG."
