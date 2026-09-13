# SPDX-License-Identifier: EUPL-1.2
# SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
#
# This file can be used with the [`just`](https://just.systems) tool.

set quiet := true

[no-exit-message]
_check_cargo_set_version:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! cargo set-version --help &>/dev/null; then
        echo 'cargo set-version is not available, you can install it with `cargo install cargo-edit`' >&2
        exit 1
    fi

[no-exit-message]
_check_opentalk_git_cliff:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! opentalk-git-cliff --help &>/dev/null; then
        echo 'opentalk-git-cliff is not available, you can install it with `cargo install --git ssh://git@git.opentalk.dev:222/opentalk/tools/check-changelog.git`' >&2
        exit 1
    fi

[no-exit-message]
_check_yq:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! yq --help &>/dev/null; then
        echo 'yq is not available, see https://github.com/mikefarah/yq' >&2
        exit 1
    fi

prepare-release VERSION: (set-version VERSION) (update-changelog VERSION)

# Sets the version in the Cargo.toml and updates the Cargo.lock
set-version VERSION: _check_cargo_set_version _check_yq
    # Set the version number for all packages in the workspace
    cargo set-version --workspace {{ VERSION }}
    # Regenerate the lockfile
    cargo check

# Update the changelog
update-changelog VERSION: _check_opentalk_git_cliff
    #!/usr/bin/env bash

    if [ -z "$GITLAB_TOKEN" ] && [ -f "$HOME/.gitlab_token" ]; then
        GITLAB_TOKEN=$(cat $HOME/.gitlab_token)
    fi

    # Update Changelog
    GITLAB_TOKEN=$GITLAB_TOKEN \
    GITLAB_API_URL=https://git.opentalk.dev/api/v4 \
    GITLAB_REPO=opentalk/backend/libs/opentalk-service-auth \
    opentalk-git-cliff \
        --unreleased \
        --tag "v{{ VERSION }}" \
        --prepend CHANGELOG.md

# Create the release commit
commit-release: _check_yq
    #!/usr/bin/env bash
    current_version=$(cat Cargo.toml | yq -ptoml .package.version)
    git commit -a -m "chore(release): prepare release $current_version"
    git log HEAD^..HEAD

# Create the release tag
tag-release: _check_yq
    #!/usr/bin/env bash
    current_version=$(cat Cargo.toml | yq -ptoml .package.version)
    git tag -s -m "v$current_version" "v$current_version"
    git show --no-patch "v$current_version"
