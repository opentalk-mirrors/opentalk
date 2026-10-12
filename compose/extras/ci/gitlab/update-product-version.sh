#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

if [ -z "${CI_COMMIT_TAG:-}" ]; then
    exit 0
fi

[[ "$CI_COMMIT_TAG" =~ ^compose-v[0-9]+\.[0-9]+\.[0-9]+(-(alpha|beta)(\.[0-9]+)?)?(-[0-9]+)?$ ]] \
    || { echo "Invalid product version tag: $CI_COMMIT_TAG" >&2; exit 1; }

VERSION_FILE=.versions.yml
[[ -f "$VERSION_FILE" ]] || { echo "$VERSION_FILE not found" >&2; exit 1; }

# The `compose-` prefix only selects the compose pipeline; the product version
# itself stays a bare `vX.Y.Z`, as used in .versions.yml and the package build.
PRODUCT_VERSION="${CI_COMMIT_TAG#compose-}"

sed -i -E "s/^opentalk_product_version:[[:space:]]*.*/opentalk_product_version: $PRODUCT_VERSION/" "$VERSION_FILE"
echo "Using product version $PRODUCT_VERSION from CI_COMMIT_TAG ($CI_COMMIT_TAG)."
