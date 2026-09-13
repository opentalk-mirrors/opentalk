#!/usr/bin/env bash
#
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2
set -xeu
set -o pipefail

is_latest_major() {
    local version=$1
    local filtered_versions=()
    local major minor patch versions v_major

    IFS='.' read -r major minor patch <<< "$version"

    IFS=' ' read -ra versions <<< "$2"
    for v in "${versions[@]}"; do
        IFS='.' read -r v_major _ _ <<< "$v"
        if [[ "$v_major" = "$major" ]]; then
            filtered_versions+=("$v")
        fi
    done
    is_latest "$version" "${filtered_versions[*]}" || return 1
}

is_latest() {
    local version versions latest
    version=$1
    IFS=" " read -ra versions <<< "$2"
    versions+=("$version")
    latest=$(printf '%s\n' "${versions[@]}" | sort -V | tail -n1)
    [[ "$version" = "$latest" ]] && return 0
    return 1
}

create_tags() {
    local ref all_tags flavor default_flavor tag tags rb_major rb_minor
    ref=$1
    all_tags=${2//v/}
    flavor=$3
    default_flavor=$4

    # Release branches like `release/v0.33.x` produce `v0.33-dev` images.
    if [[ "$ref" =~ ^release/v([0-9]+)\.([0-9]+)\.x$ ]]; then
        rb_major=${BASH_REMATCH[1]}
        rb_minor=${BASH_REMATCH[2]}
        tags=("v${rb_major}.${rb_minor}-dev-$flavor")
        [ "$flavor" = "$default_flavor" ] && tags+=("v${rb_major}.${rb_minor}-dev")
        echo "${tags[*]}"
        return
    fi

    if [ "$ref" = "main" ]; then
        tags=()
        tags+=("dev-$flavor")
        [ "$flavor" = "$default_flavor" ] && tags+=("dev")
        echo "${tags[*]}"
        return
    fi

    tag=${ref//v/}
    IFS='.' read -r major minor patch <<< "$tag"

    if [[ "$patch" =~ [a-zA-Z].* ]]; then
        tags=()
        tags=("$major.$minor.$patch-$flavor")
        if [ "$flavor" = "$default_flavor" ]; then
            tags+=("$major.$minor.$patch")
        fi
        echo "${tags[*]}"
        return
    fi

    # Always create tags for MAJOR.MINOR.PATCH and MAJOR.MINOR
    tags=("$major.$minor.$patch-$flavor" "$major.$minor-$flavor")

    # When this is the latest minor for a major branch create a tag for its major branch
    is_latest_major "$tag" "$all_tags" && tags+=("$major-$flavor")
    # Add each tag additionally prefixed with 'v'
    tags+=("${tags[@]/#/v}")

    # When this is the latest release, create a "latest" tag
    is_latest "$tag" "$all_tags" && tags+=("latest-$flavor")

    # When this is the default flavor, create a tag without the flavor for each version
    if [ "$flavor" = "$default_flavor" ]; then
        tags+=("${tags[@]//-$flavor}")
    fi

    echo "${tags[*]}"
}

create_tags "$@"
