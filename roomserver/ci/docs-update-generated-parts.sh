#!/usr/bin/env bash

# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

# This script generates parts of the documentation that are stored at `docs/`.
# When ever the documentation gets outdated, this script can be used to update the documentation.
#
# prerequisites:
# * the roomserver is build in release mode or a roomserver binary is provided with `OPENTALK_ROOMSERVER_CMD`
# * opentalk-ci-doc-updater is installed: https://git.opentalk.dev/opentalk/tools/opentalk-ci-doc-updater

set -e
set -o pipefail

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

DOCS_TEMP_DIR=target/docs/temporary

OPENTALK_ROOMSERVER_PROJECT=${OPENTALK_ROOMSERVER_PROJECT:-opentalk-roomserver}
OPENTALK_ROOMSERVER_CMD=${OPENTALK_ROOMSERVER_CMD:-target/release/opentalk-roomserver}

CLI_DIR="$DOCS_TEMP_DIR"/cli-usage
CONFIG_DIR="$DOCS_TEMP_DIR"/example

# shellcheck source=ci/include/codify.sh
source "$SCRIPT_DIR"/include/codify.sh

if ! command -v opentalk-ci-doc-updater > /dev/null; then
  echo "please install 'opentalk-ci-doc-updater' https://git.opentalk.dev/opentalk/tools/opentalk-ci-doc-updater"
  exit 1
fi

if ! command -v "$OPENTALK_ROOMSERVER_CMD" > /dev/null || [ ! -f "$OPENTALK_ROOMSERVER_CMD" ]; then
  echo "The variable 'OPENTALK_ROOMSERVER_CMD' needs to be set to a path pointing \
to a valid roomserver binary or the roomserver needs to be build using \
'cargo build --release' prior to executing this script"
  exit 1
fi

mkdir -p "$CLI_DIR" "$CONFIG_DIR"

codify toml < example/roomserver.toml > "$CONFIG_DIR"/roomserver.toml.md

$OPENTALK_ROOMSERVER_CMD help | codify text > "$CLI_DIR"/help.md
$OPENTALK_ROOMSERVER_CMD health --help | codify text > "$CLI_DIR"/health-help.md
$OPENTALK_ROOMSERVER_CMD openapi --help | codify text > "$CLI_DIR"/openapi-help.md
$OPENTALK_ROOMSERVER_CMD openapi dump --help | codify text > "$CLI_DIR"/openapi-dump-help.md
$OPENTALK_ROOMSERVER_CMD modules --help | codify text > "$CLI_DIR"/modules-help.md

# Remove trailing spaces to prevent markdownlint from triggering *MD009 - Trailing spaces*
# https://github.com/markdownlint/markdownlint/blob/main/docs/RULES.md#md009---trailing-spaces
for file in "$CLI_DIR"/*; do
 # Check if the script is running on macOS or BSD
 if [[ "$(uname)" == "Darwin" ]] || [[ "$(uname)" == "BSD" ]]; then
    sed -i '' -E 's/[[:space:]]+$//' "$file"
 else
    # For other Linux/Unix-like systems
    sed --in-place --regexp-extended 's/[[:space:]]+$//' "$file"
 fi
done

opentalk-ci-doc-updater generate --raw-files-dir target/docs/temporary/ --documentation-dir docs/
