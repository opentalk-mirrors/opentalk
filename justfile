# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2
#
# This file can be used with the [`just`](https://just.systems) tool.

[no-exit-message]
_check_cargo_set_version:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! cargo set-version --help &>/dev/null; then
        echo 'cargo set-version is not available, you can install it with `cargo install cargo-edit`' >&2
        exit 1
    fi
    if ! yq --help &>/dev/null; then
        echo 'yq is not available' >&2
        exit 1
    fi
    if ! git-cliff --help &>/dev/null; then
        echo 'git-cliff is not available, you can install it with `cargo install --git ssh://git@git.opentalk.dev:222/opentalk/tools/git-cliff.git`' >&2
        exit 1
    fi

# Prepare a release
prepare-release VERSION: _check_cargo_set_version (set-version VERSION) (update-changelog VERSION)

# Sets the version in the Cargo.toml and updates the Cargo.lock
set-version VERSION:
    # Set the version number for all packages in the workspace
    cargo set-version --workspace {{ VERSION }}
    # Regenerate the lockfile
    cargo check

# Update the changelog
update-changelog VERSION: _check_cargo_set_version
    # Update Changelog
    GITLAB_TOKEN=$(cat ~/.gitlab_token) \
    GITLAB_API_URL=https://git.opentalk.dev/api/v4 \
    GITLAB_REPO=opentalk/backend/services/obelisk \
    git-cliff -vv \
        --config opentalk \
        --unreleased \
        --tag "v{{ VERSION }}" \
        --prepend CHANGELOG.md

commit-release:
    #!/usr/bin/env bash
    VERSION=$(cat Cargo.toml | yq -ptoml ".package.version")
    git commit -a -m "chore(release): prepare release ${VERSION}"
    git log HEAD^..HEAD
