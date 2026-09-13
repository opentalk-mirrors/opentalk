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

[no-exit-message]
_check_yq:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! yq --help &>/dev/null; then
        echo 'yq is not available, see https://github.com/kislyuk/yq' >&2
        exit 1
    fi

[no-exit-message]
_check_opentalk_git_cliff:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! opentalk-git-cliff --help &>/dev/null; then
        echo 'opentalk-git-cliff is not available, you can install it with:' >&2
        echo '    cargo install --git https://git.opentalk.dev/opentalk/tools/check-changelog.git opentalk-git-cliff' >&2
        exit 1
    fi

[no-exit-message]
_check_glab:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v glab > /dev/null; then
        echo 'glab is not available, see https://gitlab.com/gitlab-org/cli' >&2
        exit 1
    fi

# Prepare a release for a single crate
prepare-release CRATE VERSION: (set-version CRATE VERSION) (update-changelog CRATE VERSION)

# Sets the version in the crate's Cargo.toml and updates the Cargo.lock
set-version CRATE VERSION: _check_cargo_set_version
    # Set the version number for the specified package
    cargo set-version --package {{ CRATE }} {{ VERSION }}
    # Regenerate the lockfile
    cargo check

# Update the changelog for a single crate
update-changelog CRATE VERSION: _check_opentalk_git_cliff
    #!/usr/bin/env bash

    if [ -z "$GITLAB_TOKEN" ] && [ -f "$HOME/.gitlab_token" ]; then
        GITLAB_TOKEN=$(cat $HOME/.gitlab_token)
    fi

    # Update Changelog
    GITLAB_TOKEN=$GITLAB_TOKEN \
    GITLAB_API_URL=https://git.opentalk.dev/api/v4 \
    GITLAB_REPO=opentalk/backend/libs/opentalk-shared-folder \
    opentalk-git-cliff \
        --use-branch-tags \
        --unreleased \
        --include-path "crates/{{ CRATE }}/**" \
        --tag-pattern "{{ CRATE }}-v.*" \
        --tag "{{ CRATE }}-v{{ VERSION }}" \
        --prepend crates/{{ CRATE }}/CHANGELOG.md

# Create the release commit for a single crate
commit-release CRATE: _check_yq
    #!/usr/bin/env bash
    set -eu -o pipefail
    VERSION=$(cat crates/{{ CRATE }}/Cargo.toml | yq -ptoml ".package.version")
    git commit -a -m "chore(release): prepare {{ CRATE }} release ${VERSION}"
    git log HEAD^..HEAD

# Create the release tag for a single crate
tag-release CRATE: _check_yq
    #!/usr/bin/env bash
    set -eu -o pipefail
    VERSION=$(cat crates/{{ CRATE }}/Cargo.toml | yq -ptoml ".package.version")
    TAG="{{ CRATE }}-v${VERSION}"
    git tag -s -m "$TAG" "$TAG"
    git show --no-patch "$TAG"

# Create a GitLab release from the current version tag for a single crate
create-release CRATE: _check_yq _check_glab
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION=$(cat crates/{{ CRATE }}/Cargo.toml | yq -ptoml ".package.version")
    TAG="{{ CRATE }}-v${VERSION}"

    # Extract the changelog section for this version
    notes=$(awk "/^## \\[$TAG\\]/{found=1; next} /^## \\[/{if(found) exit} /^\\[$TAG\\]:/{next} found{print}" crates/{{ CRATE }}/CHANGELOG.md)

    if [ -z "$notes" ]; then
        echo "No changelog entry found for version $TAG" >&2
        exit 1
    fi

    glab release create "$TAG" --notes "$notes"
