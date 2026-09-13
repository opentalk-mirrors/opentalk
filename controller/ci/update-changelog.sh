#!/usr/bin/env bash
#
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

# This script automatically updates the CHANGELOG.md to contain the latest changes.
#
# This tool will overwrite the files CHANGELOG.md.tmp and CHANGELOG.should.md make
# that they don't exist before this script is executed.
#
# Make sure to store a GitLab access token at `~/.gitlab_token`.

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
PROJECT_DIR=$( dirname "$SCRIPT_DIR" )
# Remove Unreleased section from Changelog

GITLAB_TOKEN_FILE=$HOME/.gitlab_token
GITLAB_REPO=opentalk/backend/services/controller

if [ -f "$GITLAB_TOKEN_FILE" ]; then
    echo "Using '$GITLAB_TOKEN_FILE' to authenticate with GitLab."
else
    echo "Please provide a GitLab token at '$GITLAB_TOKEN_FILE'."
    echo "You can create one here: https://git.opentalk.dev/-/user_settings/personal_access_tokens"
    echo "The scope should at least contain read_api."
fi

docker run -it -v "$PROJECT_DIR":/app \
    -e GITLAB_REPO=$GITLAB_REPO \
    -e GITLAB_API_URL=https://git.opentalk.dev/api/v4 \
    -e GITLAB_TOKEN="$(cat "$GITLAB_TOKEN_FILE")" \
    -u "$(id -u):$(id -g)" \
    git.opentalk.dev:5050/opentalk/tools/check-changelog:v0.3.0
