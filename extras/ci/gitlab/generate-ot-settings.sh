#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

# Generate OT_SETTINGS YAML for integration tests
# Usage: generate-ot-settings.sh <channel> [--with-smtp] [--with-orchestrator] [--with-obelisk] [--with-outlook]
# channel: stable|testing|experimental

CHANNEL="${1:?Missing CHANNEL (stable|testing|experimental)}"
shift

# Base settings common to all channels
cat << 'EOF'
ot_domain: 'opentalk-ci.test'
ot_ingress_port: 443
ot_ingress_protocol: https
config_dir: "/etc/opentalk-compose"
services_dir: "/opt/opentalk-compose/services"
deploy_dir: "/opt/opentalk-compose"
compose_project_name: "opentalk"
compose_service_opentalk_controller_enabled: true
compose_service_opentalk_roomserver_enabled: false
compose_service_opentalk_obelisk_enabled: false
compose_service_opentalk_outlook_add_in_enabled: false
compose_service_opentalk_recorder_enabled: true
EOF

# Channel-specific SMTP setting
case "$CHANNEL" in
    stable|experimental)
        echo "compose_service_opentalk_smtp_mailer_enabled: true"
        ;;
    testing|pre-release)
        echo "compose_service_opentalk_smtp_mailer_enabled: false"
        ;;
esac

cat << 'EOF'
compose_service_opentalk_web_frontend_enabled: true
compose_service_etherpad_enabled: true
compose_service_spacedeck_enabled: true
compose_service_keycloak_enabled: true
compose_service_livekit_server_enabled: true
compose_service_minio_enabled: true
compose_service_rabbitmq_enabled: true
compose_service_traefik_enabled: true
compose_service_ndt_enabled: true
compose_service_mailcrab_enabled: false
self_signed_certificates_generate: true
custom_certificates_enabled: true
custom_certificates_fullchain_cert_file: fullchain.crt
custom_certificates_private_key_file: privkey.key
custom_certificates_ca_root_cert_file: ca.crt
controller_docker_extra_hosts:
  - "host.docker.internal:host-gateway"
  - "opentalk-ci.test:host-gateway"
recorder_docker_extra_hosts:
  - "host.docker.internal:host-gateway"
  - "opentalk-ci.test:host-gateway"
obelisk_docker_extra_hosts:
  - "host.docker.internal:host-gateway"
  - "opentalk-ci.test:host-gateway"
EOF
