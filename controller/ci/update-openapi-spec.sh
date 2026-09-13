#!/usr/bin/env bash

# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

# This script generates the OpenAPI specification file for the OpenTalk Controller API

set -xe
set -o pipefail

OPENTALK_CONTROLLER_CMD=${OPENTALK_CONTROLLER_CMD:-target/release/opentalk-controller}

$OPENTALK_CONTROLLER_CMD --config example/controller.toml  openapi dump docs/developer/api.yaml
