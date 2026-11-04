#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

BASE_SETTINGS_FILE=extras/ci/gitlab/integration-settings.yml
SETTINGS_FILE=/etc/opentalk-compose/settings.yml
COMPOSE_PROJECT_NAME=opentalk
export COMPOSE_PROJECT_NAME

install_package() {
    local os_id=$1

    case "$os_id" in
        debian|ubuntu)
            echo "Debian-/Ubuntu-based Distribution: $os_id"
            docker info
            DEB_FILE=$(ls artifacts/*.deb 2>/dev/null || true)
            echo "Installing ${DEB_FILE}..."
            dpkg -i "$DEB_FILE" || apt-get install -f -y
            ;;
        rocky)
            echo "RHEL-based Distribution: $os_id"
            docker info
            RPM_FILE=$(ls artifacts/*.rpm 2>/dev/null || true)
            echo "Installing ${RPM_FILE}..."
            dnf install -y "$RPM_FILE" --nogpgcheck
            ;;
        opensuse-leap)
            echo "SUSE-based Distribution: $os_id"
            docker info
            RPM_FILE=$(ls artifacts/*.rpm 2>/dev/null || true)
            echo "Installing ${RPM_FILE}..."
            zypper --non-interactive --no-gpg-checks install "$RPM_FILE"
            ;;
        *)
            echo "Unknown or unsupported Distribution: $os_id" >&2
            exit 1
            ;;
    esac
}

write_settings() {
    mkdir -p "$(dirname "$SETTINGS_FILE")"
    cp "$BASE_SETTINGS_FILE" "$SETTINGS_FILE"

    if [ "${SMTP_MAILER_ENABLED:-false}" = "true" ]; then
        sed -i \
            -e 's/^compose_service_opentalk_smtp_mailer_enabled: false$/compose_service_opentalk_smtp_mailer_enabled: true/' \
            "$SETTINGS_FILE"
    fi

    if [ "${EXTRA_OPTIONAL_SERVICES:-}" = "with-optional-services" ]; then
        sed -i \
            -e 's/^compose_service_opentalk_orchestrator_enabled: false$/compose_service_opentalk_orchestrator_enabled: true/' \
            -e 's/^compose_service_opentalk_obelisk_enabled: false$/compose_service_opentalk_obelisk_enabled: true/' \
            -e 's/^compose_service_opentalk_outlook_add_in_enabled: false$/compose_service_opentalk_outlook_add_in_enabled: true/' \
            "$SETTINGS_FILE"
        echo "Extra optional services (orchestrator, obelisk, outlook) enabled for this distro slot."
    fi

    # Point at this pipeline's freshly built component images instead of the
    # versions pinned in .versions.yml, so a merge that changes a component
    # is tested against its own build rather than the last release.
    if [ "${USE_DEV_IMAGES:-false}" = "true" ]; then
        local sha="$CI_COMMIT_SHORT_SHA"
        cat >>"$SETTINGS_FILE" <<EOF
controller_docker_image_src: "${CI_REGISTRY_IMAGE}/controller"
opentalk_controller_version: "ubuntu-${sha}"
obelisk_docker_image_src: "${CI_REGISTRY_IMAGE}/obelisk"
opentalk_obelisk_version: "ubuntu-${sha}"
orchestrator_docker_image_src: "${CI_REGISTRY_IMAGE}/orchestrator"
opentalk_orchestrator_version: "ubuntu-${sha}"
roomserver_docker_image_src: "${CI_REGISTRY_IMAGE}/roomserver"
opentalk_roomserver_version: "ubuntu-${sha}"
mailer_docker_image_src: "${CI_REGISTRY_IMAGE}/smtp-mailer"
opentalk_smtp_mailer_version: "ubuntu-${sha}"
recorder_docker_image_src: "${CI_REGISTRY_IMAGE}/recorder"
opentalk_recorder_version: "intel-${sha}"
EOF
        echo "Using dev-tagged component images built from ${sha}."
    fi
}

run_tests() {
    bash extras/scripts/tests/integration/test_deployment_runtime.sh

    if [ "${RUN_SMTP_RABBITMQ_TEST:-false}" = "true" ]; then
        bash extras/scripts/tests/integration/test_smtp_mailer_rabbitmq.sh
    fi
}

# shellcheck disable=SC1091
OS_ID="$(. /etc/os-release && echo "$ID")"
install_package "$OS_ID"
write_settings
otctl deploy
run_tests