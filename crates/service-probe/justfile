# SPDX-License-Identifier: EUPL-1.2
# SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
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

[no-exit-message]
_check_git_cliff:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! git-cliff --help &>/dev/null; then
        echo 'git-cliff is not available, you can install it with `cargo install --git ssh://git@git.opentalk.dev:222/opentalk/tools/git-cliff.git`' >&2
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
    
# Update the changelog
update-changelog VERSION: _check_git_cliff
    # Update Changelog
    GITLAB_TOKEN=$(cat ~/.gitlab_token) \
    GITLAB_API_URL=https://git.opentalk.dev/api/v4 \
    GITLAB_REPO=opentalk/backend/libs/service-probe \
    git-cliff -vv \
        --config opentalk \
        --unreleased \
        --tag "v{{ VERSION }}" \
        --prepend CHANGELOG.md

# Create the release commit
commit-release: _check_yq
    #!/usr/bin/env bash
    current_version=$(cat Cargo.toml | yq -ptoml .workspace.package.version)
    git commit -a -m "chore(release): prepare release $current_version"
    git log HEAD^..HEAD

# Create the release tag
tag-release: _check_yq
    #!/usr/bin/env bash
    current_version=$(cat Cargo.toml | yq -ptoml .workspace.package.version)
    git tag -s -m "v$current_version" "v$current_version"
    git show --no-patch "v$current_version"

# Prepare a release
prepare-release VERSION: (set-version VERSION) (update-changelog VERSION)

# Sets the version in the Cargo.toml and updates the Cargo.lock
set-version VERSION: _check_cargo_set_version
    # Set the version number for all packages in the workspace
    cargo set-version --workspace {{ VERSION }}
    # Regenerate the lockfile
    cargo check
