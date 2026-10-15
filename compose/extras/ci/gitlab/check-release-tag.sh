#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

EXPECTED_TAG_SUFFIX_PATTERN="${1:?Missing expected tag suffix pattern}"
BASE_VERSION=$(yq -r ".opentalk_product_version" .versions.yml)

echo "Compare product version in .versions.yml with CI_COMMIT_TAG"

if [[ ! "${CI_COMMIT_TAG:?CI_COMMIT_TAG is not set}" =~ ^${BASE_VERSION}${EXPECTED_TAG_SUFFIX_PATTERN}$ ]]; then
    echo "Product Version mismatch: .versions.yml has $BASE_VERSION but tag is $CI_COMMIT_TAG" >&2
    exit 1
fi