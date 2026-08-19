# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# OpenTalk Compose developer commands

set shell := ["bash", "-ceu"]

version := "26.2.0"
release := "1"
dev_dir := env_var_or_default("OPENTALK_DEV_DIR", "build/opentalk-compose-dev")
dev_settings_sample := "extras/packaging/config-dist/settings.dev.yml-sample"
dev_settings := dev_dir + "/config/settings.yml"
dev_collections := ".ansible/collections"
dev_venv := dev_dir + "/.venv"
docs_port := env_var_or_default("DOCS_PORT", "8082")

_default:
    @just --list

# Run the standard repository checks.
check: check-packaging

# Install build dependencies and development tools.
setup: install-deps setup-tools

# Install the tools used for packaging, documentation, and linting.
setup-tools:
    uv tool install --upgrade pre-commit
    uv tool install --upgrade 'mkdocs<2' \
        --with mkdocs-material \
        --with mkdocs-literate-nav \
        --with pymdown-extensions
    if python3 -c 'import importlib.util; raise SystemExit(0 if importlib.util.find_spec("privacy") else 1)' 2>/dev/null; then :; else echo "Warning: MkDocs privacy plugin is not available from PyPI; install the project-specific plugin before running just docs." >&2; fi
    if command -v gem >/dev/null 2>&1; then gem install --user-install mdl; else echo "RubyGems is required for mdl." >&2; exit 1; fi
    pre-commit install

# Verify that packaging prerequisites and scripts are available.
check-packaging:
    extras/packaging/check-packaging.sh

# Check whether all native packaging dependencies are installed.
check-deps:
    BUILD_TARGET=both extras/packaging/install-build-deps.sh --check

# Install native dependencies required to build packages.
install-deps:
    extras/packaging/install-build-deps.sh --install

# Build the Debian package.
build-deb:
    extras/packaging/build-all.sh --deb --version {{version}} --release {{release}}

# Build the RPM package.
build-rpm:
    extras/packaging/build-all.sh --rpm --version {{version}} --release {{release}}

# Build both Debian and RPM packages.
build:
    extras/packaging/build-all.sh --deb --rpm --version {{version}} --release {{release}}

# Build both packages with an automatically derived local revision.
build-local:
    extras/packaging/build-all.sh --deb --rpm --auto-revision --version {{version}} --release {{release}}

# Build both packages without downloading external dependencies.
build-offline:
    extras/packaging/build-all.sh --deb --rpm --offline --version {{version}} --release {{release}}

# Create the local, uncommitted Compose settings file from the sample.
compose-settings:
    dev_root="$(realpath -m "{{dev_dir}}")"; settings_file="$dev_root/config/settings.yml"; mkdir -p "$(dirname "$settings_file")"; if test -e "$settings_file"; then echo "Using existing local settings: $settings_file"; else cp "{{dev_settings_sample}}" "$settings_file"; echo "Created local settings: $settings_file"; fi

# Create or update the local Python and Ansible virtual environment.
compose-python:
    dev_root="$(realpath -m "{{dev_dir}}")"; mkdir -p "$dev_root"; uv venv --allow-existing "$dev_root/.venv"; uv pip install --python "$dev_root/.venv/bin/python" --requirement extras/packaging/requirements-ansible.txt

# Install the pinned Ansible Collections used by the local Compose deployment.
compose-collections: compose-python
    collections_root="$(realpath -m "{{dev_collections}}")"; mkdir -p "$collections_root"; dev_root="$(realpath -m "{{dev_dir}}")"; ANSIBLE_COLLECTIONS_PATH="$collections_root" "$dev_root/.venv/bin/ansible-galaxy" collection install --requirements-file automation/collections/requirements.yml --collections-path "$collections_root"

# Prepare settings, Python, and Ansible Collections for the local Compose environment.
compose-setup: compose-settings compose-collections

_dev-ansible args="":
    dev_root="$(realpath -m "{{dev_dir}}")"; collections_root="$(realpath -m "{{dev_collections}}")"; env OT_SETTINGS_FILE="$dev_root/config/settings.yml" OPENTALK_DEV_DIR="$dev_root" ANSIBLE_CONFIG="$PWD/automation/ansible.cfg" ANSIBLE_COLLECTIONS_PATH="$collections_root" "$dev_root/.venv/bin/ansible-playbook" -i automation/inventory.yml automation/playbooks/opentalk_compose.yml {{args}}

_dev-compose args="":
    dev_root="$(realpath -m "{{dev_dir}}")"; docker compose --project-directory "$dev_root" -f "$dev_root/compose.yml" {{args}}

# Deploy and start the complete local OpenTalk Compose environment.
compose-deploy: compose-setup
    dev_root="$(realpath -m "{{dev_dir}}")"; ingress_port="$(awk '/^ot_ingress_port:/ {print $2; exit}' "$dev_root/config/settings.yml")"; just _dev-ansible "-e ansible_become=false"; printf '\nLocal OpenTalk services:\n  Web UI:      http://opentalk.dev.localhost:%s/\n  Keycloak:   http://keycloak.localhost:8001/auth/\n  Traefik:    http://localhost:7777/\n  Etherpad:   http://opentalk.dev.localhost:%s/etherpad/\n  Whiteboard: http://whiteboard.dev.localhost:%s/whiteboard/\n\n' "$ingress_port" "$ingress_port" "$ingress_port"

# Create or update the local demo user in Keycloak.
compose-demo-user:
    just _dev-ansible "--tags demo-user -e ansible_become=false"

# Show the status of containers in the local OpenTalk Compose environment.
compose-status:
    just _dev-compose "ps"

# Follow logs from the local OpenTalk Compose environment, optionally filtered by service.
compose-logs service="":
    just _dev-compose "logs -f {{service}}"

# Restart all or one service in the local OpenTalk Compose environment.
compose-restart service="":
    just _dev-compose "restart {{service}}"

# Stop the local OpenTalk Compose environment without deleting its files.
compose-down:
    dev_root="$(realpath -m "{{dev_dir}}")"; if test -f "$dev_root/compose.yml"; then docker compose --project-directory "$dev_root" -f "$dev_root/compose.yml" down; else echo "No local development stack found at $dev_root."; fi

# Stop and remove the local OpenTalk Compose environment and its generated files.
compose-clean: compose-down
    rm -rf "$(realpath -m "{{dev_dir}}")"

# Verify the contents and metadata of a Debian package.
verify-deb package:
    extras/packaging/verify-packages.sh {{package}}

# Verify the contents and metadata of an RPM package.
verify-rpm package:
    extras/packaging/verify-packages.sh {{package}}

# Build the documentation site with strict validation.
docs:
    DISABLE_MKDOCS_2_WARNING=true mkdocs build --config-file mkdocs.yml --strict

# Serve the documentation site locally for interactive development.
docs-serve:
    printf '\nDocumentation: http://127.0.0.1:{{docs_port}}/\n\n'; DISABLE_MKDOCS_2_WARNING=true mkdocs serve --config-file mkdocs.yml --dev-addr 127.0.0.1:{{docs_port}}

# Run all configured pre-commit checks.
lint:
    pre-commit run --all-files

# Run fast, Docker-free regression and contract tests.
test-fast: lint
    find extras/scripts/tests -maxdepth 1 -type f -name 'test_*.sh' -print0 | xargs -0r -n1 bash

# Measure shell line coverage via bash -x tracing and enforce baseline thresholds.
test-coverage:
    bash extras/scripts/tests/test_opentalk_coverage_regression.sh

# Run only a single shell coverage measurement (TARGET is a test script path).
test-coverage-one target:
    python3 extras/scripts/coverage/measure_shell_coverage.py --verbose {{target}}

# Run the destructive Docker Compose end-to-end test in an isolated local environment.
test-e2e:
    OPENTALK_E2E=1 bash extras/scripts/tests/integration/test_compose_e2e.sh

# Remove generated build files while keeping package artifacts.
clean:
    extras/packaging/90-cleanup.sh --build-only -y

# Remove generated package artifacts.
clean-artifacts:
    extras/packaging/90-cleanup.sh --artifacts -y

# Remove all generated build files and package artifacts.
clean-all:
    extras/packaging/90-cleanup.sh --all -y
