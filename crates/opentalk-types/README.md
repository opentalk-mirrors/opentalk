<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# OpenTalk Types

This repository contains crates with datatypes used by OpenTalk in the Web API
(for managing events etc) and the signaling connection (during a conference
call).

# How To Release

1. Update the crates versions that need releasing
2. Check dependent crates in this repo
    - if the version is incompatible with the current version -> Update dependencies
    - if version is compatible, we don't need to update the dependency
    - [Check the cargo doc](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#version-requirement-syntax) for details about which version are compatible with which requirement
3. create a merge request with version updated
    - go through normal review process
    - merge
4. Tag and publish
    - create a tag for each release
    - tag name: `<crate-name>-<crate-version>`
    - `cargo publish -p`

> [!NOTE]
> We don't use prerelease versions (`X.Y.Z-dev`) on `main` since this would conflict with releasing only a subset of crates.
> When a subset of crates is published, we must ensure that no prerelease version leak into dependencies.

[!NOTE]: mdl-requires-a-link
