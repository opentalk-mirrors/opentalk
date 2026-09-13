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

OPENTALK_CONTROLLER_PROJECT=${OPENTALK_CONTROLLER_PROJECT:-opentalk-controller}
OPENTALK_CONTROLLER_CMD=${OPENTALK_CONTROLLER_CMD:-target/release/opentalk-controller}

CLI_DIR="$DOCS_TEMP_DIR"/cli-usage
JOBS_DIR="$DOCS_TEMP_DIR"/jobs
CONFIG_DIR="$DOCS_TEMP_DIR"/config
DB_DIR="$DOCS_TEMP_DIR"/database
MODULES_DIR="$DOCS_TEMP_DIR"/modules

CMDNAME=opentalk-controller
ER_DIAGRAM_MERMAID="$DB_DIR/er-diagram.mermaid"

# shellcheck source=ci/include/codify.sh
source "$SCRIPT_DIR"/include/codify.sh

if ! command -v opentalk-ci-doc-updater; then
  echo "please install 'opentalk-ci-doc-updater' https://git.opentalk.dev/opentalk/tools/opentalk-ci-doc-updater"
  exit 1
fi

if [ ! "$(command -v "$OPENTALK_CONTROLLER_CMD")" ] || [ ! -f "$OPENTALK_CONTROLLER_CMD" ]; then
  echo "The variable 'OPENTALK_CONTROLLER_CMD' needs to be set to a path pointing \
to a valid controller binary or the controller needs to be build using \
'cargo build --release' prior to executing this script"
  exit 1
fi

mkdir -p "$CLI_DIR" "$JOBS_DIR" "$CONFIG_DIR" "$DB_DIR" "$MODULES_DIR"

# Generate mermaid diagrams only if sqlant is available. Otherwise use already provided mermaid
# sources (e.g. provided by other ci jobs).
if command -v sqlant; then
  "$SCRIPT_DIR"/docs-generate-mermaid.sh
fi

if [ ! -f $ER_DIAGRAM_MERMAID ]; then
  echo "Mermaid diagram file ($ER_DIAGRAM_MERMAID) not found."
  exit 1
fi

cat $ER_DIAGRAM_MERMAID | codify mermaid > $DB_DIR/er-diagram.md

codify toml < example/controller.toml > "$CONFIG_DIR"/controller.toml.md

$OPENTALK_CONTROLLER_CMD help | codify text > "$CLI_DIR"/"$CMDNAME"-help.md
$OPENTALK_CONTROLLER_CMD migrate-db --help | codify text > "$CLI_DIR"/"$CMDNAME"-migrate-db-help.md
$OPENTALK_CONTROLLER_CMD tenants --help | codify text > "$CLI_DIR"/"$CMDNAME"-tenants-help.md
$OPENTALK_CONTROLLER_CMD tenants list --help | codify text > "$CLI_DIR"/"$CMDNAME"-tenants-list-help.md
$OPENTALK_CONTROLLER_CMD tenants set-oidc-id --help | codify text > "$CLI_DIR"/"$CMDNAME"-tenants-set-oidc-id-help.md
$OPENTALK_CONTROLLER_CMD tariffs --help | codify text > "$CLI_DIR"/"$CMDNAME"-tariffs-help.md
$OPENTALK_CONTROLLER_CMD tariffs create --help | codify text > "$CLI_DIR"/"$CMDNAME"-tariffs-create.md
$OPENTALK_CONTROLLER_CMD tariffs delete --help | codify text > "$CLI_DIR"/"$CMDNAME"-tariffs-delete.md
$OPENTALK_CONTROLLER_CMD tariffs edit --help | codify text > "$CLI_DIR"/"$CMDNAME"-tariffs-edit.md
$OPENTALK_CONTROLLER_CMD jobs --help | codify text > "$CLI_DIR"/"$CMDNAME"-jobs-help.md
$OPENTALK_CONTROLLER_CMD jobs execute --help | codify text > "$CLI_DIR"/"$CMDNAME"-jobs-execute-help.md
$OPENTALK_CONTROLLER_CMD jobs default-parameters --help | codify text > "$CLI_DIR"/"$CMDNAME"-jobs-default-parameters-help.md
$OPENTALK_CONTROLLER_CMD modules --help | codify text > "$CLI_DIR"/"$CMDNAME"-modules-help.md
$OPENTALK_CONTROLLER_CMD modules list --help | codify text > "$CLI_DIR"/"$CMDNAME"-modules-list-help.md
$OPENTALK_CONTROLLER_CMD openapi --help | codify text > "$CLI_DIR"/"$CMDNAME"-openapi-help.md
$OPENTALK_CONTROLLER_CMD openapi dump --help | codify text > "$CLI_DIR"/"$CMDNAME"-openapi-dump-help.md
$OPENTALK_CONTROLLER_CMD health --help | codify text > "$CLI_DIR"/"$CMDNAME"-health-help.md
$OPENTALK_CONTROLLER_CMD reload --help | codify text > "$CLI_DIR"/"$CMDNAME"-reload-help.md

$OPENTALK_CONTROLLER_CMD --config example/controller.toml jobs default-parameters self-check | codify json > "$JOBS_DIR"/parameters-self-check.json.md
$OPENTALK_CONTROLLER_CMD --config example/controller.toml jobs default-parameters event-cleanup | codify json > "$JOBS_DIR"/parameters-event-cleanup.json.md
$OPENTALK_CONTROLLER_CMD --config example/controller.toml jobs default-parameters user-cleanup | codify json > "$JOBS_DIR"/parameters-user-cleanup.json.md
$OPENTALK_CONTROLLER_CMD --config example/controller.toml jobs default-parameters adhoc-event-cleanup | codify json > "$JOBS_DIR"/parameters-adhoc-event-cleanup.json.md
$OPENTALK_CONTROLLER_CMD --config example/controller.toml jobs default-parameters invite-cleanup | codify json > "$JOBS_DIR"/parameters-invite-cleanup.json.md
$OPENTALK_CONTROLLER_CMD --config example/controller.toml jobs default-parameters sync-storage-files | codify json > "$JOBS_DIR"/parameters-sync-storage-files.json.md
$OPENTALK_CONTROLLER_CMD --config example/controller.toml jobs default-parameters room-cleanup | codify json > "$JOBS_DIR"/parameters-room-cleanup.json.md
$OPENTALK_CONTROLLER_CMD --config example/controller.toml jobs default-parameters keycloak-account-sync | codify json > "$JOBS_DIR"/parameters-keycloak-account-sync.json.md

$OPENTALK_CONTROLLER_CMD --config example/controller.toml modules list | codify text > "$CLI_DIR"/"$CMDNAME"-modules-list.md

$OPENTALK_CONTROLLER_CMD --config example/controller.toml modules print-documentation > "$MODULES_DIR"/module-features-documentation.md

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
