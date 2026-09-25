#!/usr/bin/env bash

# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

# This script generates parts of the documentation that are stored at `docs/`.
# When ever the documentation gets outdated, this script can be used to update the documentation.
#
# prerequisites:
# * opentalk-ci-doc-updater is installed: https://git.opentalk.dev/opentalk/tools/opentalk-ci-doc-updater

set -e
set -o pipefail

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
COMPONENT_DIR=$( cd -- "$SCRIPT_DIR/.." &> /dev/null && pwd )

DOCS_TEMP_DIR="$COMPONENT_DIR"/target/docs/temporary/obelisk
CONFIG_DIR="$DOCS_TEMP_DIR"/config

if ! command -v opentalk-ci-doc-updater > /dev/null; then
  echo "please install 'opentalk-ci-doc-updater' https://git.opentalk.dev/opentalk/tools/opentalk-ci-doc-updater"
  exit 1
fi

mkdir -p "$CONFIG_DIR"

cp "$COMPONENT_DIR"/extra/example.toml "$CONFIG_DIR"/example.toml

opentalk-ci-doc-updater generate --raw-files-dir "$DOCS_TEMP_DIR" --documentation-dir "$COMPONENT_DIR"/docs/
