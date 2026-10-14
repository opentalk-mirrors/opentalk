# SPDX-License-Identifier: EUPL-1.2
# SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
# SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
#
# This file can be used with the [`just`](https://just.systems) tool.

set quiet

# Services that participate in a monorepo release. Each entry is the display name used in the release notes; its
# lowercased form is the directory at the repository root which must contain a `CHANGELOG.md` with an entry for the
# component version being released.
services := "Controller Obelisk Orchestrator Recorder RoomServer SMTP-Mailer Transcription"

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
        echo 'yq is not available, see https://github.com/mikefarah/yq' >&2
        exit 1
    fi

[no-exit-message]
_check_opentalk_git_cliff:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! opentalk-git-cliff --help &>/dev/null; then
        echo 'opentalk-git-cliff is not available, you can install it with:' >&2
        echo 'cargo install --locked --git https://git.opentalk.dev/opentalk/tools/check-changelog.git opentalk-git-cliff' >&2
        exit 1
    fi

[no-exit-message]
_check_glab:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! glab --version &>/dev/null; then
        echo 'glab is not available, see https://gitlab.com/gitlab-org/cli#installation' >&2
        exit 1
    fi

[no-exit-message]
_check_ci_doc_updater:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v opentalk-ci-doc-updater > /dev/null; then
        echo 'opentalk-ci-doc-updater is not available, you can install it with' >&2
        echo '`cargo install --git https://git.opentalk.dev/opentalk/tools/opentalk-ci-doc-updater.git`' >&2
        exit 1
    fi

[no-exit-message]
_check_k6:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v k6 > /dev/null; then
        echo 'k6 is not available, see https://grafana.com/docs/k6/latest/set-up/install-k6/' >&2
        exit 1
    fi

# Update generated or derived parts of the documentation
update-docs: _check_ci_doc_updater
    #!/usr/bin/env bash
    set -euo pipefail

    cargo build --release

    for service in {{ services }}; do
        script="${service,,}/ci/docs-update-generated-parts.sh"
        if [[ -x $script ]]; then
            "$script"
        fi
    done

# Update the version in the OpenAPI spec
update-openapi:
    # Update OpenAPI specification (which contains the version number)
    cargo run -p opentalk-roomserver -- openapi dump > roomserver/api/docs/openapi.yml
    # Trim whitespace
    sed -i 's/[[:space:]]*$//' roomserver/api/docs/openapi.yml
    # Add trailing new line (removed by previous command)
    echo '' >> roomserver/api/docs/openapi.yml
    cargo run -p opentalk-controller -- openapi dump > docs/developer/controller/api.yaml

install-latest-typst-packages:
    #!/usr/bin/env bash
    # pull the image with the typst packages
    docker pull git.opentalk.dev:5050/opentalk/backend/containers/typst-plugins:scratch-dev
    # create a new container
    # sh is necessary because creating containers without entry point is not possible
    CONTAINER_NAME=typst-plugins
    docker create --name "$CONTAINER_NAME" git.opentalk.dev:5050/opentalk/backend/containers/typst-plugins:scratch-dev sh >/dev/null 2>&1 || true
    # create the typst package directory
    TYPST_DIR=${XDG_CACHE_HOME:-$HOME/.cache}/typst/packages/preview/
    mkdir -p "$TYPST_DIR"
    # remove existing package
    rm -r "$TYPST_DIR/linguify" >/dev/null 2>&1 || true
    # copy the typst packages from the container
    docker cp typst-plugins:/usr/share/typst/packages/preview/linguify/ "$TYPST_DIR"
    # delete the container
    docker container rm "$CONTAINER_NAME"

test-coverage:
    cargo llvm-cov nextest --lcov --output-path lcov.info

test-load TEST: _check_k6
    #!/usr/bin/env bash

    export OPENTALK_ROOM_HTTP__PUBLIC_URL=http://localhost:11333
    export OPENTALK_ROOM_MONITORING__ADDR=127.0.0.1
    export LOAD_TEST_BASE_URL=http://localhost:11333

    pushd roomserver/load-test
    ../ci/run-load-test.sh src/tests/"{{ TEST }}".js

run-dui *ARGS:
    RUST_LOG=opentalk=debug cargo run -p opentalk-roomserver-dui -- {{ ARGS }}

# Prepare a release
prepare-release VERSION: (set-version VERSION) (update-changelog VERSION)

# Sets the version in the Cargo.toml and updates the Cargo.lock
set-version VERSION: _check_cargo_set_version _check_yq
    # Set the version number for all packages in the workspace
    cargo set-version --workspace {{ VERSION }}
    # Regenerate the lockfile
    cargo update --workspace
    # update the frontend apis
    yq '.info.version = "{{ VERSION }}"' -i roomserver/api/docs/openapi.yml
    yq '.info.version = "{{ VERSION }}"' -i controller/docs/developer/api.yaml

# Update the changelog
update-changelog VERSION: _check_opentalk_git_cliff
    #!/usr/bin/env bash
    # pipefail is required so that `wait` reports git-cliff failures instead of the exit status of `sed`
    set -euo pipefail

    if [ -z "${GITLAB_TOKEN:-}" ] && [ -f "$HOME/.gitlab_token" ]; then
        GITLAB_TOKEN=$(< "$HOME/.gitlab_token")
    fi

    if [ -z "${GITLAB_TOKEN:-}" ]; then
        echo 'GITLAB_TOKEN must be set, either as environment variable or in ~/.gitlab_token' >&2
        exit 1
    fi

    git_cliff() {
        # Bash starts background jobs in scripts with SIGINT/SIGQUIT ignored, see
        # https://www.gnu.org/software/bash/manual/html_node/Signals.html
        # Restore the defaults so Ctrl-C also stops git-cliff.
        trap - SIGINT SIGQUIT
        GITLAB_TOKEN=$GITLAB_TOKEN \
        GITLAB_API_URL=https://git.opentalk.dev/api/v4 \
        GITLAB_REPO=opentalk/opentalk \
        opentalk-git-cliff --unreleased --use-branch-tags --tag "v{{ VERSION }}" "$@"
    }

    # Update the changelog for each service in parallel
    declare -A pids
    for service in {{ services }}; do
        service_lower="${service,,}"
        git_cliff \
            --include-path "$service_lower/**/*" \
            --prepend "$service_lower/CHANGELOG.md" 2>&1 | sed "s/^/[$service]: /" &
        pids[$service]=$!
    done

    # Update the changelog for dependency updates
    # TODO: The commit range (e0c734dcf..HEAD) can be removed when the first release from the monorepo has been published
    # It is only necessary because the monorepo does not have tags yet, which would mean the first release includes all commits
    git_cliff \
        --config cliff-deps.toml \
        --include-path Cargo.toml \
        --include-path Cargo.lock \
        --prepend "DEPS_CHANGELOG.md" \
        e0c734dcf..HEAD 2>&1 | sed "s/^/[Dependencies]: /" &
    pids[Dependencies]=$!

    failed=()
    for service in "${!pids[@]}"; do
        wait "${pids[$service]}" || failed+=("$service")
    done

    if [ ${#failed[@]} -gt 0 ]; then
        printf -v failed_list '%s, ' "${failed[@]}"
        # Echo the failed services removing the trailing comma
        echo "Failed to update the changelog for: ${failed_list%, }" >&2
        exit 1
    fi

# Create the release commit
commit-release: _check_yq
    #!/usr/bin/env bash
    current_version=$(yq .workspace.package.version Cargo.toml)
    git commit -a -m "chore(release): prepare release $current_version"
    git log HEAD^..HEAD

# Create the release tag
tag-release: _check_yq
    #!/usr/bin/env bash
    current_version=$(yq .workspace.package.version Cargo.toml)
    git tag -s -m "v$current_version" "release-v$current_version"
    git show --no-patch "release-v$current_version"

# Create a GitLab release assembling notes from each service's CHANGELOG.md
create-release PRODUCT_VERSION COMPONENT_VERSION: _check_glab
    #!/usr/bin/env bash
    set -euo pipefail

    notes="# OpenTalk {{ PRODUCT_VERSION }}"

    extract_section() {
        local heading=$1
        local changelog=$2

        local section="" in_section=false line
        while IFS= read -r line; do
            if [[ $line == "## [{{ COMPONENT_VERSION }}]"* ]]; then
                # The heading for the component version starts the section
                in_section=true
            elif [[ $in_section == false ]]; then
                # Skip lines before the section is found
                continue
            elif [[ $line == "## ["* ]]; then
                # Stop when the next section is found
                break
            elif [[ -z $section && -z $line ]]; then
                # Skip leading empty lines
                continue
            elif [[ $line == "[{{ COMPONENT_VERSION }}]:"* ]]; then
                # Skip the link to the GitLab changes
                continue
            else
                section+="$line"$'\n'
            fi
        done < "$changelog"

        if [[ -n "$section" ]]; then
            printf '%s\n\n%s' "## $heading" "$section"
        else
            echo "No entry for {{ COMPONENT_VERSION }} found in $changelog, skipping" >&2
            return 1
        fi
    }

    # Collect the `## [COMPONENT_VERSION]` section from each `<service>/CHANGELOG.md` into a combined release notes
    # document, grouped by service.
    for service in {{ services }}; do
        changelog="${service,,}/CHANGELOG.md"
        if [ ! -f "$changelog" ]; then
            echo "Skipping ${service}: no ${changelog} found." >&2
            continue
        fi

        if section=$(extract_section "$service {{ COMPONENT_VERSION }}" "$changelog"); then
            notes+=$'\n\n'"$section"
        fi
    done

    # Collect the same section from the DEPS_CHANGELOG.md file.
    if section=$(extract_section "📦 Dependencies" DEPS_CHANGELOG.md); then
        notes+=$'\n\n'"$section"
    fi

    printf "\nThis will create a release with the following notes:\n\n"

    if command -v bat >/dev/null 2>&1; then
        printf "%s\n\n" "$notes" | bat --file-name "OpenTalk v{{ PRODUCT_VERSION }}" --language=md --style=plain --paging=never
    else
        printf "%s\n\n" "$notes"
    fi

    read -e -p "Publish? [y/N] " choice
    if [[ "$choice" =~ ^[Yy](es)?$ ]]; then
        glab release create "release-v{{ COMPONENT_VERSION }}" --name "v{{ COMPONENT_VERSION }}" --notes "$notes"
    fi
