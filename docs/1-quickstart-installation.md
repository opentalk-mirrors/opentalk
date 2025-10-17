# Quickstart Guide to Self-Hosting OpenTalk

OpenTalk Compose is a set of tools and utilities for running the OpenTalk video conferencing solution on your own server. This quickstart guide covers the minimal steps required to get the necessary services up and running. It assumes a standard scenario with a publicly accessible server where all components run on a single host and SSL/TLS certificates are obtained via Let's Encrypt.  
For customized setups or special conditions, refer to the OpenTalk Administration Guide. The essentials are explained here.

## Table of Contents

- [1. Preparations](#1-preparations)  
  - [1.1 Minimal System Requirements](#11-minimal-system-requirements)  
  - [1.2 Supported Operating Systems](#12-supported-operating-systems)  
  - [1.3 Firewall Configuration](#13-firewall-configuration)  
  - [1.4 DNS Domain Record](#14-dns-domain-record)  
  - [1.5 Docker Requirement](#15-docker-requirement)  
- [2. Installation Procedure](#2-installation-procedure)  
  - [2.1 Add the OpenTalk Package Repository](#21-add-the-opentalk-package-repository)  
  - [2.2 Install the opentalk-compose Package](#22-install-the-opentalk-compose-package)  
  - [2.3 Set the OpenTalk Server Domain](#23-set-the-opentalk-server-domain)  
  - [2.4 Initialize the Deployment Environment](#24-initialize-the-deployment-environment)  
  - [2.5 Optional: Adjust Configuration & Redeploy](#25-optional-adjust-configuration--redeploy)  
  - [2.6 Check Service Status and Health](#26-check-service-status-and-health)  
  - [2.7 Backups (Optional but Recommended)](#27-backups-optional-but-recommended)  
  - [2.8 Undeploy (Tear Down)](#28-undeploy-tear-down)  
  - [2.9 Optional: Enable Demo User](#29-optional-enable-demo-user)  
  - [2.10 Reference: otctl Overview](#210-reference-otctl-overview)

## 1. Preparations

### 1.1 Minimal System Requirements

The target server should meet at least:

- CPU: 2 cores
- RAM: 4 GB
- Storage: 20 GB
- Network: Public network interface
- DNS: A domain record pointing to the public interface

### 1.2 Supported Operating Systems

Releases are provided as DEB and RPM packages. The packages are architecture-independent and not tied to a specific Linux distribution.

Recommended distributions:

| Distributions using `.deb`                 | Distributions using `.rpm`  |
|--------------------------------------------|-----------------------------|
| Debian 12 "Bookworm"                       | RHEL 9                      |
| Debian 13 "Trixie"                         | RHEL 10                     |
| Ubuntu 22.04 LTS "Jammy Jellyfish"         | AlmaLinux 9                 |
| Ubuntu 24.04 LTS "Noble Numbat"            | AlmaLinux 10                |
|                                            | Rocky Linux 9               |
|                                            | Rocky Linux 10              |
|                                            | openSUSE Leap 15            |
|                                            | SLES 15                     |

### 1.3 Firewall Configuration

Ensure these ports are reachable in a standard deployment:

- 80, 443/tcp (HTTP, HTTPS)
- 40000-49999/udp (RTP Obelisk)
- 5060/udp (SIP Obelisk)
- 50000-64535/udp (RTP LiveKit)
- 7881/tcp (WebRTC ICE over TCP, LiveKit)
- 3478/udp (LiveKit TURN)

### 1.4 DNS Domain Record

Create a DNS record pointing to the server's public IPv4 and IPv6 addresses. This is the domain used to access the web application (e.g. <https://example.com>).

### 1.5 Docker Requirement

Install Docker and the Docker Compose plugin before installing `opentalk-compose`. If not installed, follow the official Docker documentation.

## 2. Installation Procedure

### 2.1 Add the OpenTalk Package Repository

Debian / Ubuntu:

```bash
apt update && apt install -y curl gnupg
curl http://anfi.opentalk.run:7376/deb-testing/public.gpg | gpg --yes --dearmor -o /usr/share/keyrings/openrepo-deb-testing.gpg
echo "deb [arch=any signed-by=/usr/share/keyrings/openrepo-deb-testing.gpg] http://anfi.opentalk.run:7376/deb-testing/ stable main" > /etc/apt/sources.list.d/openrepo-deb-testing.list
```

RHEL- and SUSE-based:

```bash
cat > /etc/yum.repos.d/rpm-testing.repo <<'EOF'
[rpm-testing]
name=rpm-testing
baseurl=http://anfi.opentalk.run:7376/rpm-testing
enabled=1
repo_gpgcheck=1
gpgkey=http://anfi.opentalk.run:7376/rpm-testing/public.gpg
EOF
```

### 2.2 Install the `opentalk-compose` Package

Debian / Ubuntu:

```bash
apt update && apt install -y opentalk-compose
```

RHEL:

```bash
dnf install -y opentalk-compose
```

SUSE:

```bash
zypper install -y opentalk-compose
```

### 2.3 Set the OpenTalk Server Domain

Edit `/etc/opentalk-compose/settings.yml` and set:

```yaml
ot_domain: example.com
```

### 2.4 Initialize the Deployment Environment

Run the first deployment:

```bash
otctl deploy
```

First run effects:

- Generates required secrets (if absent)
- Renders `compose.yml` and service configs
- Prepares runtime directories and symlinks
- Starts containers (equivalent to docker compose up -d)

Note: Initial configuration and startup may take several minutes. Wait until completion before accessing the application.  
After initialization the web UI is available at: <https://example.com> (replace with your domain).

### 2.5 Optional: Adjust Configuration & Redeploy

Modify `/etc/opentalk-compose/settings.yml` (symlink at `/opt/opentalk-compose/config/settings.yml`) to adapt domain, certificate handling, external services, or feature toggles. Maintain valid YAML indentation.

Then redeploy (idempotent):

```bash
otctl deploy
```

Only changed settings are re-applied. Re-running is safe.

Note: `otctl` prints the underlying command (`[otctl] + <command>`) for transparency.

### 2.6 Check Service Status and Health

```bash
otctl status
```

Example:

```text
SERVICE       STATE     HEALTH
controller    running   healthy
keycloak      running   healthy
minio         running   healthy
obelisk       running   healthy
webapp        running   healthy
```

Follow-up examples:

```bash
otctl logs controller         # Tail one service
otctl restart webapp          # Restart web frontend
otctl recreate controller     # Recreate container with new image
otctl version                 # Show bundled versions
```

If unhealthy:

- Logs: `otctl logs <service>`
- Re-apply config: `otctl deploy`
- Inspect raw compose: `otctl compose ps`, `otctl compose logs <service>`

### 2.7 Backups (Optional but Recommended)

Default backup root: `/var/opt/opentalk-compose/backups` (falls back to `/opt/opentalk-compose/backups`).

Core commands:

```bash
otctl backup                 # Full backup (databases + config + volumes)
otctl backup ls              # List backups
otctl backup dump            # All DB dumps
otctl backup dump controller # Controller DB only
otctl backup dump keycloak   # Keycloak DB only
otctl backup config          # Config snapshot
otctl backup volume          # All defined volumes
otctl backup volume <name>   # Single volume
otctl backup clean           # Apply retention policy
```

Notes:

- Retention is controlled via Ansible tags (e.g. `backup_retention`).
- Volume names must match exactly (no automatic prefixing).
- Extra vars can be passed via `OTCTL_BACKUP_EXTRA_VARS` for advanced use.

### 2.8 Undeploy (Tear Down)

Remove containers (keep data volumes) and generated artifacts:

```bash
otctl undeploy
```

Performs:

1. docker compose down (if compose.yml exists)  
2. Removes generated `compose.yml`  
3. Cleans contents of `services/` (keeps directory)

For full purge (volumes, images), use Docker commands manually (e.g. review `docker volume ls | grep opentalk_` before pruning).

### 2.9 Optional: Enable Demo User

A demo user can be created for testing. Username: `demo`. The password is generated (or taken from settings) and shown with `--diff`.

Enable:

```bash
otctl deploy --diff --tags demo-user
```

Remove:

```bash
otctl deploy --tags demo-user -e postInstall_demoUser_state=absent
```

Regular user management is done via Keycloak.

### 2.10 Reference: `otctl` Overview

| Command | Purpose |
|---------|---------|
| otctl deploy | Render and (re)start stack |
| otctl status | Show service + health summary |
| otctl logs [svc] | Stream logs |
| otctl restart [svc...] | Restart services |
| otctl recreate [svc...] | Recreate containers |
| otctl backup … | Backup workflows |
| otctl version | Show versions |
| otctl undeploy | Stop stack & remove compose.yml |
| otctl compose … | Pass through to docker compose |
| otctl playbook … | Advanced Ansible usage |

Health column semantics (`otctl status`):

- healthy: Healthcheck passed
- unhealthy: Healthcheck failing
- starting: Healthcheck not yet passing
- n/a: No healthcheck defined
