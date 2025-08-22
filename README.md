# OpenTalk Compose Administration Guide

OpenTalk Compose provides tools and utilities for deploying the OpenTalk video conferencing solution on your own server. Begin with the [Quickstart Guide](docs/1-quickstart-installation.md), which walks you through the installation steps required to set up all necessary services for your OpenTalk instance. This guide assumes a standard scenario: a publicly accessible server where all components run on a single host, and SSL/TLS certificates are obtained via Let's Encrypt.

For customized setups or special requirements, refer to the [Advanced Installation Scenarios](docs/4-advanced-installation-scenarios.md) in this documentation to explore configuration options that meet your needs. The Quickstart Guide covers the essential steps for a common installation.

## Table of Contents

[1. Quickstart Installation ](docs/1-quickstart-installation.md)

[2. Accessing Web Interfaces](docs/2-access-web-interfaces.md)

[3. Managing OpenTalk Users](docs/3-managing-opentalk-users-in-keycloak.md)

[4. Advanced Installation Scenarios](docs/4-advanced-installation-scenarios.md)

[5. Configuration Option Reference](docs/5-configuration-options-reference.md)

[6. Local Development Environment](docs/6-local-dev-environment.md)


```
/opt/opentalk-compose/
├── bin/                           # CLI Tools (User-facing)
│   ├── ot-stack                   # Haupt-CLI für Stack-Management
│   ├── ot-deploy                  # Deployment-Wrapper
│   └── ot-manage                  # Management-Befehle
├── config -> /etc/opentalk-compose # Symlink (bleibt)
├── automation/                    # Ansible Automation Code
│   ├── playbooks/
│   │   ├── site.yml               # Haupt-Deployment
│   │   ├── deploy-stack.yml       # OpenTalk Stack Deployment
│   │   ├── generate-compose.yml   # Compose Template Rendering
│   │   └── manage/
│   │       ├── backup.yml
│   │       ├── restore.yml
│   │       └── cleanup.yml
│   ├── roles/                     # Ansible Roles
│   │   ├── opentalk_stack/        # Main Stack Role
│   │   ├── compose_generator/     # Template → compose.yaml
│   │   ├── container_runtime/     # Docker/Podman Detection
│   │   └── monitoring/
│   ├── inventory/                 # Default Inventories
│   │   ├── local.yml
│   │   └── templates/
│   │       ├── production.yml
│   │       └── staging.yml
│   ├── templates/                 # Jinja2 Templates
│   │   ├── compose/
│   │   │   ├── base.compose.yml.j2
│   │   │   ├── backend.compose.yml.j2
│   │   │   ├── frontend.compose.yml.j2
│   │   │   └── monitoring.compose.yml.j2
│   │   ├── config/
│   │   │   ├── traefik.yml.j2
│   │   │   └── env.j2
│   │   └── systemd/
│   │       └── opentalk-stack.service.j2
│   └── vars/                      # Variable Files
│       ├── defaults.yml
│       ├── docker.yml
│       └── podman.yml
├── runtime/                       # Embedded Ansible Runtime
│   ├── .venv/                     # Python Virtual Environment
│   ├── .python/                   # Portable Python (optional)
│   ├── collections/               # Ansible Collections
│   ├── ansible.cfg                # Ansible Configuration
│   └── build_assets/              # Self-heal Assets
├── compose/                       # Generated Compose Files (Target)
│   ├── docker-compose.yml         # Generated main file
│   ├── docker-compose.override.yml
│   └── .env                       # Generated environment
├── data/                          # Persistent Data Mount Point
│   ├── postgresql/
│   ├── redis/
│   └── uploads/
├── logs/                          # Log Directory
└── docs/                          # Documentation
    ├── README.md
    ├── deployment-guide.md
    └── configuration.md
```