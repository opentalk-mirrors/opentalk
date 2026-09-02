#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

# Setup integration test environment
# - Authenticate with container registry
# - Map test domain in hosts file

echo "Setting up integration test environment..."

# Docker login for container registry access
docker login -u gitlab-ci-token -p "$CI_JOB_TOKEN" git.opentalk.dev:5050

# Map test domain (containers use extra_hosts; host process needs it in /etc/hosts)
echo "127.0.0.1 opentalk-ci.test" >> /etc/hosts

echo "Integration environment ready."
