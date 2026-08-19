<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# Development related docs

## Building Packages

The build embeds Python, Ansible, all Python wheels, Ansible Collections and the required external tools. `uv` is used during the build; it is not required on the target system.

Install or check host dependencies first:

```bash
extras/packaging/install-build-deps.sh --install
extras/packaging/install-build-deps.sh --check
```

For a complete local development setup, use:

```bash
just setup
```

This installs the host build dependencies, the MkDocs and pre-commit tools, the Markdown linter, and configures the local Git hooks. Fedora, Debian, Ubuntu, openSUSE, and Arch Linux are supported. Bootstrap `just` and `uv` before running `just setup`; the setup otherwise requires RubyGems and network access.

The build requires x86-64 Linux with glibc 2.34 or newer for the embedded runtime. Use `--deb` or `--rpm` to build one package format; use both flags to build both formats. Without either flag, both formats are built.

### Building `.deb` Packages

```bash
extras/packaging/build-all.sh --deb --version 26.2.0 --release 1
```

### Building `.rpm` Packages

```bash
extras/packaging/build-all.sh --rpm --version 26.2.0 --release 1
```

Build both formats in one run:

```bash
extras/packaging/build-all.sh --deb --rpm --version 26.2.0 --release 1
```

The build is offline-capable after all caches are populated. Use `--offline` to prevent downloads. Repeated local builds can opt into automatic revision increments with `--auto-revision`; release builds fail if the requested artifact already exists.

The repository also provides a `justfile` with shortcuts for the common development tasks:

```bash
just check
just build-deb
just build-rpm
just build
just build-offline
just clean-artifacts
```

Run `just --list` to see all available recipes. The direct scripts remain useful for CI and advanced build options.

## Setup local Dev environment for OpenTalk

The repository contains a development configuration at `extras/packaging/config-dist/settings.dev.yml-sample`. It enables the local OpenTalk stack with HTTP on port `8081`, Keycloak on port `8001`, insecure local TLS handling, Mailcrab, Etherpad, Spacedeck, Garage, and the OpenTalk services.

The development sample stores generated Compose files, configuration, secrets, and service data below `build/opentalk-compose-dev/`. The complete `build/` directory is ignored by Git. Set `OPENTALK_DEV_DIR` to use another untracked location. Run the following commands from the repository root:

```bash
just compose-deploy
```

`just compose-deploy` runs the complete local deployment: it creates a uv-managed Ansible environment under `build/opentalk-compose-dev/.venv`, installs the pinned Python requirements and required Ansible Collections, renders all Compose files, starts the enabled containers, and then applies the Keycloak configuration. The local Docker user must be allowed to access the Docker socket. Docker Compose is required; Podman is not covered by this development path yet.

The command also deploys the demo user `demo`. Its generated password is stored in `build/opentalk-compose-dev/config/secrets/secrets.yml` under `DEMO_USER_PASSWORD`. To deploy the user separately or repeat the operation, use `just compose-demo-user`.

Open the local frontend at <http://localhost:8081>. The Traefik dashboard is available at <http://localhost:7777>; Mailcrab and the other optional service endpoints are configured by the generated Compose files.

Useful commands from the repository root:

```bash
just compose-status
just compose-logs controller
just compose-restart controller
just compose-down
```

To apply configuration changes, run `just compose-deploy` again. The generated `build/opentalk-compose-dev/` contents are runtime state and should not be edited manually or committed. `just compose-clean` stops the stack and removes its local runtime directory; `just clean-all` also removes `build/`.

## End-to-End Compose Tests

The repository contains a destructive, end-to-end Compose test that deploys the complete local stack, checks the running services, exercises the SMTP mailer and RabbitMQ, and runs a backup/restore lifecycle. It is intended for manual local runs and as the `int:e2e` CI job (GitLab, manual trigger on merge requests and schedules).

```bash
just test-e2e
```

The E2E stack is isolated from a running local development environment: it uses its own deployment directory (`OPENTALK_E2E_DEV_DIR`, default `build/opentalk-compose-e2e-dev`), its own Compose project name (`OPENTALK_E2E_PROJECT_NAME`, default `opentalk-e2e`), and its own published ports (`OPENTALK_E2E_INGRESS_PORT`/`OPENTALK_E2E_KEYCLOAK_PORT`, default `8082`/`8002`). This keeps containers, volumes, networks, and host ports separate from the local stack (`build/opentalk-compose-dev`, project `opentalk`, ports `8081`/`8001`). Cleanup only removes resources that belong to the E2E project name.

The test runs `just compose-deploy`, waits for all containers to become healthy, verifies the web, whiteboard, and Keycloak endpoints via HTTP, runs `test_deployment_runtime.sh` and `test_smtp_mailer_rabbitmq.sh` against the E2E project, and finally runs `test_backup_restore_config.sh`. It requires Docker, `just`, `uv`, and network access to the container registry; it is destructive and should not be run while the local development stack is in use.

The `int:e2e` CI job uses the `pkgtest-dind` Debian 12 image, installs `just` and `uv` on demand, and triggers the test manually from the GitLab UI.

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

| Product Version / Input Tag         | Package Version                       | Debian Filename                                       |
|-------------------------------------|---------------------------------------|-------------------------------------------------------|
| v25.3.2                             | 25.3.2-1                              | opentalk-compose_25.3.2-1_amd64.deb                   |
| v25.3.2-1                           | 25.3.2-1                              | opentalk-compose_25.3.2-1_amd64.deb                   |
| v25.3.2-2                           | 25.3.2-2                              | opentalk-compose_25.3.2-2_amd64.deb                   |
| v25.3.2-beta                        | 25.3.2~beta-1                         | opentalk-compose_25.3.2~beta-1_amd64.deb              |
| v25.3.2-beta.2                      | 25.3.2~beta2-1                        | opentalk-compose_25.3.2~beta2-1_amd64.deb             |
| v25.3.2-beta.3+git-4e7d8a1f         | 25.3.2~beta3+git4e7d8a1f-1            | opentalk-compose_25.3.2~beta3+git4e7d8a1f-1_amd64.deb |
| v25.3.2-beta+git4e7d8a1f            | 25.3.2~beta+git4e7d8a1f-1             | opentalk-compose_25.3.2~beta+git4e7d8a1f-1_amd64.deb  |
| v25.3.2+git-4e7d8a1f                | 25.3.2+git4e7d8a1f-1                  | opentalk-compose_25.3.2+git4e7d8a1f-1_amd64.deb       |

## RPM-Pakages

| Product Version / Input Tag         | RPM Version              | RPM Release            | RPM Filename                                           |
|-------------------------------------|--------------------------|------------------------|--------------------------------------------------------|
| v25.3.2                             | 25.3.2                   | 1                      | opentalk-compose-25.3.2-1.x86_64.rpm                   |
| v25.3.2-1                           | 25.3.2                   | 1                      | opentalk-compose-25.3.2-1.x86_64.rpm                   |
| v25.3.2-2                           | 25.3.2                   | 2                      | opentalk-compose-25.3.2-2.x86_64.rpm                   |
| v25.3.2-beta                        | 25.3.2~beta              | 1                      | opentalk-compose-25.3.2~beta-1.x86_64.rpm              |
| v25.3.2-beta.2                      | 25.3.2~beta2             | 1                      | opentalk-compose-25.3.2~beta2-1.x86_64.rpm             |
| v25.3.2-beta.3+git-4e7d8a1f         | 25.3.2~beta3+git4e7d8a1f | 1                      | opentalk-compose-25.3.2~beta3+git4e7d8a1f-1.x86_64.rpm |
| v25.3.2-beta+git4e7d8a1f            | 25.3.2~beta+git4e7d8a1f  | 1                      | opentalk-compose-25.3.2~beta+git4e7d8a1f-1.x86_64.rpm  |
| v25.3.2+git-4e7d8a1f                | 25.3.2+git4e7d8a1f       | 1                      | opentalk-compose-25.3.2+git4e7d8a1f-1.x86_64.rpm       |

## Renovate local test

Run Renovate to verify the `renovate.json` configuration or check which components are tracked:

```bash
 docker run --rm -v "$(pwd):/usr/src/app" \
   -e LOG_LEVEL=debug \
   -e RENOVATE_CONFIG_FILE=/usr/src/app/renovate.json \
   -e GITHUB_COM_TOKEN="$GITHUB_COM_TOKEN" \
   ghcr.io/renovatebot/renovate:full \
   --platform=local \
   --dry-run=lookup
```

## Building and Testing OpenTalk Compose Package on the local machine

This section describes how to build and test the OpenTalk Compose Debian package in a Docker container.

### Example for a Debian package

#### Build the package

```bash
extras/packaging/build-all.sh --deb --version v25.4.0
```

#### Start a Debian container with Docker-in-Docker support

**With Docker:**

```bash
docker run --rm -ti --name dind-debian12 \
  --privileged \
  --tmpfs /run:rw,noexec,nosuid,size=65536k \
  --tmpfs /run/lock \
  -v dind-debian-docker-root:/var/lib/docker \
  -v ./artifacts:/artifacts \
  -e DOCKER_TLS_CERTDIR= \
  git.opentalk.dev:5050/opentalk/opentalk-devops/containers/pkgtest-dind/debian12:dev bash
```

**With Podman:**

```bash
podman run --rm -ti --name dind-debian12 \
  --privileged \
  --tmpfs /run:rw,noexec,nosuid,size=65536k \
  --tmpfs /run/lock \
  -v dind-debian-docker-root:/var/lib/docker \
  -v ./artifacts:/artifacts \
  -e DOCKER_TLS_CERTDIR= \
  git.opentalk.dev:5050/opentalk/opentalk-devops/containers/pkgtest-dind/debian12:dev bash
```

#### Install the OpenTalk Compose package inside the DinD container

```bash
dpkg -i artifacts/opentalk-compose_25.4.0-1_amd64.deb
```

#### Activate the Python virtual environment

```bash
source /opt/opentalk-compose/runtime/venv/bin/activate
```

#### Run deplyment to execute ansible tasks

```bash
otctl deploy -e ot_domain=localhost
```
