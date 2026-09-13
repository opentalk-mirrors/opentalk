#!/usr/bin/env bash

# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

# This script generates a mermaid diagram from the provided database.
# Before running this script make sure to set the `OPENTALK_CTRL_DATABASE__URL` variable to
# contain the URL of a running postgres instance and that sqlant is installed.

set -xe

if [ ! -v OPENTALK_CTRL_DATABASE__URL ]; then
  echo "Variable 'OPENTALK_CTRL_DATABASE__URL' wasn't set. Make sure to set this \
variable to the database URL."
  exit 1
fi


DOCS_TEMP_DIR=target/docs/temporary

OPENTALK_CONTROLLER_PROJECT=${OPENTALK_CONTROLLER_PROJECT:-opentalk-controller}
OPENTALK_CONTROLLER_CMD=${OPENTALK_CONTROLLER_CMD:-target/release/opentalk-controller}

DB_DIR="$DOCS_TEMP_DIR"/database
ER_DIAGRAM_MERMAID="$DB_DIR/er-diagram.mermaid"

mkdir -p "$DB_DIR"

# Initialize the database schema
$OPENTALK_CONTROLLER_CMD --config example/controller.toml migrate-db

# Generate the ER diagram
sqlant -o mermaid --direction rl "$OPENTALK_CTRL_DATABASE__URL" > $ER_DIAGRAM_MERMAID
