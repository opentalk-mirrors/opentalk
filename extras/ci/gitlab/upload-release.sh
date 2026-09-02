#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

# Upload artifacts to GitLab, Nexus, and OpenRepo
# Usage: upload-release.sh <flavor> <channel>
# flavor: deb|rpm
# channel: stable|testing|experimental

FLAVOR="${1:?Missing FLAVOR (deb|rpm)}"
CHANNEL="${2:?Missing CHANNEL (stable|testing|experimental)}"

[[ "$FLAVOR" =~ ^(deb|rpm)$ ]] || { echo "Invalid FLAVOR: $FLAVOR" >&2; exit 1; }
[[ "$CHANNEL" =~ ^(stable|testing|experimental)$ ]] || { echo "Invalid CHANNEL: $CHANNEL" >&2; exit 1; }

if [ -n "${CI_COMMIT_TAG:-}" ]; then
    PACKAGE_VERSION="$CI_COMMIT_TAG"
else
    PACKAGE_VERSION="$(yq -r '.opentalk_product_version' .versions.yml)+git${CI_COMMIT_SHORT_SHA}"
fi

for file in artifacts/*."${FLAVOR}"; do
    [ -f "$file" ] || continue
    filename=$(basename "$file")
    
    # GitLab: Upload to generic packages
    echo "Uploading $filename to GitLab packages..."
    curl --header "JOB-TOKEN: $CI_JOB_TOKEN" \
         --upload-file "$file" \
            "${CI_API_V4_URL}/projects/${CI_PROJECT_ID}/packages/generic/opentalk-compose/${PACKAGE_VERSION}/$filename"
    echo "Uploaded to gitlab: $filename"
    
    # Nexus: Select repository based on FLAVOR and CHANNEL
    if [ "$FLAVOR" = "deb" ]; then
        case "$CHANNEL" in
            stable)       NEXUS_REPO_VAR="NEXUS_REPO_NAME_DEB_STABLE" ;;
            testing)      NEXUS_REPO_VAR="NEXUS_REPO_NAME_DEB_BETA" ;;
            experimental) NEXUS_REPO_VAR="NEXUS_REPO_NAME_DEB_EXPERIMENTAL" ;;
        esac
    else  # rpm
        case "$CHANNEL" in
            stable)       NEXUS_REPO_VAR="NEXUS_REPO_NAME_RPM_STABLE" ;;
            testing)      NEXUS_REPO_VAR="NEXUS_REPO_NAME_RPM_BETA" ;;
            experimental) NEXUS_REPO_VAR="NEXUS_REPO_NAME_RPM_EXPERIMENTAL" ;;
        esac
    fi
    
    NEXUS_URL="${NEXUS_REPO_BASE_URL}/${!NEXUS_REPO_VAR}/"
    echo "Uploading $filename to Nexus ($CHANNEL/$FLAVOR)..."
    curl --progress-bar -v -u "${NEXUS_API_USER}:${NEXUS_API_KEY}" \
         -H "Content-Type: multipart/form-data" \
         --data-binary "@${file}" \
         "${NEXUS_URL}" | tee /dev/null
    echo "Uploaded to Nexus: $filename"
    
    # OpenRepo: Upload with channel suffix
    echo "Uploading $filename to OpenRepo..."
    openrepo --debug upload "$file" -r "${FLAVOR}-${CHANNEL}" -o
    echo "Uploaded to openrepo ($CHANNEL): $filename"
    
    sleep 2
done
