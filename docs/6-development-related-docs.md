<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# Development related docs

## Building Packages

You can use `--deb` or `--rpm` to create packages in the respective format. If you omit these options, packages in both formats will be created.

### Building `.deb` Packages

```bash
extras/packaging/build-all.sh --deb --version 25.3.0 --release pre.$(date +"%y%m%d%H%M%S")
```

### Building `.rpm` Packages

```bash
extras/packaging/build-all.sh --rpm --version 25.3.0 --release pre.$(date +"%y%m%d%H%M%S")
```

## Setup local Dev environment for OpenTalk

[WIP]

## Pre-Commit Hooks

This repository uses [`pre-commit`](https://github.com/pre-commit/pre-commit) to automatically run linters and consistency checks before code is committed.  
By running these checks locally, we ensure consistent formatting, licensing, and documentation quality across all contributions.

### What It Does

The configured hooks perform the following checks:

| Hook | Description |
|------|--------------|
| **yamllint** | Validates YAML files for syntax and style. |
| **reuse** | Verifies license headers and REUSE compliance. |
| **markdown-lint** | Lints Markdown files using `mdl` |
| **shellcheck** | Checks shell scripts for common errors and bad practices. |
| **ansible-lint** | Lints Ansible playbooks, roles, and tasks for best practices using `ansible-lint`. |

### Installation

#### Install `pre-commit`

Depending on you envoronment install `pre-commit` e.g.:

```bash
# Fedora
dnf install pre-commit
# Debian
apt install pre-commit
# via python pip
pip install pre-commit
```

#### Set up the Git hooks

```bash
pre-commit install
```

#### (Optional) Run all checks manually

```bash
pre-commit run --all-files
```

#### pre-commit autoupdate

To update hook versions defined in .pre-commit-config.yaml:

```bash
pre-commit autoupdate
```

## Versioning of DEB or RPM Packages

Because the OpenTalk product version cannot be mapped 1:1 to DEB or RPM package versions without breaking update mechanics, product versions are converted accordingly. Nothing changes for authoring: keep using the product version schema when createing releases or git tags. The conversion happens when running the build scripts.

## DEB-Pakages

| Product Version / Input Tag                     | Package Version                      | Debian Filename                                                     |
|-------------------------------------|---------------------------------------|---------------------------------------------------------------------|
| v25.3.2                             | 25.3.2-1                              | opentalk-compose_25.3.2-1_amd64.deb                                 |
| v25.3.2-1                           | 25.3.2-1                              | opentalk-compose_25.3.2-1_amd64.deb                                 |
| v25.3.2-2                           | 25.3.2-2                              | opentalk-compose_25.3.2-2_amd64.deb                                 |
| v25.3.2-beta                        | 25.3.2~beta-1                         | opentalk-compose_25.3.2~beta-1_amd64.deb                            |
| v25.3.2-beta.2                      | 25.3.2~beta2-1                        | opentalk-compose_25.3.2~beta2-1_amd64.deb                           |
| v25.3.2-beta.3+git-4e7d8a1f         | 25.3.2~beta3+git4e7d8a1f-1            | opentalk-compose_25.3.2~beta3+git4e7d8a1f-1_amd64.deb              |
| v25.3.2-beta+git4e7d8a1f            | 25.3.2~beta+git4e7d8a1f-1             | opentalk-compose_25.3.2~beta+git4e7d8a1f-1_amd64.deb               |
| v25.3.2+git-4e7d8a1f                | 25.3.2+git4e7d8a1f-1                  | opentalk-compose_25.3.2+git4e7d8a1f-1_amd64.deb                    |

## RPM-Pakages

| Product Version / Input Tag | RPM Version | RPM Release | RPM Filename |
|-------------------------------------|----------------------|------------------------|---------------------------------------------------|
| v25.3.2                             | 25.3.2               | 1                      | opentalk-compose-25.3.2-1.x86_64.rpm             |
| v25.3.2-1                           | 25.3.2               | 1                      | opentalk-compose-25.3.2-1.x86_64.rpm             |
| v25.3.2-2                           | 25.3.2               | 2                      | opentalk-compose-25.3.2-2.x86_64.rpm             |
| v25.3.2-beta                        | 25.3.2~beta          | 1                      | opentalk-compose-25.3.2~beta-1.x86_64.rpm        |
| v25.3.2-beta.2                      | 25.3.2~beta2         | 1                      | opentalk-compose-25.3.2~beta2-1.x86_64.rpm       |
| v25.3.2-beta.3+git-4e7d8a1f         | 25.3.2~beta3+git4e7d8a1f   | 1                | opentalk-compose-25.3.2~beta3+git4e7d8a1f-1.x86_64.rpm |
| v25.3.2-beta+git4e7d8a1f           | 25.3.2~beta+git4e7d8a1f     | 1                | opentalk-compose-25.3.2~beta+git4e7d8a1f-1.x86_64.rpm |
| v25.3.2+git-4e7d8a1f                | 25.3.2+git4e7d8a1f   | 1                      | opentalk-compose-25.3.2+git4e7d8a1f-1.x86_64.rpm |

## Revovate local test

```bash
 docker run --rm -v "$(pwd):/usr/src/app" \
   -e LOG_LEVEL=debug \
   -e RENOVATE_CONFIG_FILE=/usr/src/app/renovate.json \
   -e GITHUB_COM_TOKEN="$GITHUB_COM_TOKEN" \
   ghcr.io/renovatebot/renovate:full \
   --platform=local \
   --dry-run=lookup
```
