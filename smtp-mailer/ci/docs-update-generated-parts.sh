#!/usr/bin/env bash

# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

# This script generates parts of the documatation that are stored at `docs/`.
# When ever the documentation gets outdated, this script can be used to update the documentation.
# 'docs-generate-mermaid.sh' is called during execution to update the ER diagrams.
#
# prerequisites:
# * rabbitMQ is running
# * a postgres database is running
# * 'OPENTALK_CTRL_DATABASE__URL' is set to the postgres database URL
# * sqlant is installed (Or the er diagram is already generated): https://github.com/kurotych/sqlant
# * opentalk-ci-doc-updater is installed: https://git.opentalk.dev/opentalk/tools/opentalk-ci-doc-updater

set -xe
set -o pipefail

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

DOCS_TEMP_DIR=target/docs/temporary
CONFIG_DIR="$DOCS_TEMP_DIR"/config

# shellcheck source=ci/include/codify.sh
source "$SCRIPT_DIR"/include/codify.sh

if ! command -v opentalk-ci-doc-updater; then
  echo "please install 'opentalk-ci-doc-updater' https://git.opentalk.dev/opentalk/tools/opentalk-ci-doc-updater"
  exit 1
fi

mkdir -p "$CONFIG_DIR"

codify toml < example/smtp-mailer.toml > "$CONFIG_DIR"/smtp-mailer.toml.md

opentalk-ci-doc-updater generate --raw-files-dir "$DOCS_TEMP_DIR" --documentation-dir docs/
