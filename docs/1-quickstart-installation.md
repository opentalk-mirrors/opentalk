# Quickstart Guide to Self-Host OpenTalk

OpenTalk Compose is a set of tools and utilities for running the OpenTalk video conferencing solution on your own server. This quickstart guide covers the minimal installation steps to get the necessary services up and running. It assumes a standard scenario with a publicly accessible server where all components run on a single host, and SSL/TLS certificates are obtained via Let's Encrypt.
For customized setups or special conditions, refer to the "OpenTalk Administration Guide" in the documentation to find options that suit your requirements. The essentials are explained in this Quickstart Guide.

## Table of Contents

- [1. Preparations](#1-preparations)
	- [1.1 Minimal System Requirements](#11-minimal-system-requirements)
	- [1.2 Supported Operating Systems](#12-supported-operating-systems)
	- [1.3 Firewall Configuration](#13-firewall-configuration)
	- [1.4 DNS Domain Record](#14-dns-domain-record)
	- [1.5 Docker as Container Engine is Required](#15-docker-as-container-engine-is-required)
- [2. Installation Procedure](#2-installation-procedure)
  - [2.1 Add the OpenTalk Package Repository](#21-add-the-opentalk-package-repository)
  - [2.2 Install the opentalk-compose Package](#22-install-the-opentalk-compose-package)
  - [2.3 Set the Server's DNS Domain Record](#23-set-the-servers-dns-domain-record)
  - [2.4 Initialize the Deployment Environment](#24-initialize-the-deployment-environment)
  - [2.5 Configure & Deploy (Edit settings.yml then otctl deploy)](#25-configure--deploy-edit-settingsyml-then-otctl-deploy)
  - [2.6 Check Service Status & Health](#26-check-service-status--health)
  - [2.7 Backups (Optional but Recommended)](#27-backups-optional-but-recommended)
  - [2.8 Undeploy (Tear Down)](#28-undeploy-tear-down)
  - [2.9 Optional: Enable Demo User](#29-optional-enable-demo-user)
  - [2.10 Reference: otctl Overview](#210-reference-otctl-overview)

## 1. Preparations

### 1.1 Minimal System Requirements

The target server for OpenTalk installation should fulfill the following minimum hardware specifications:

- **CPU:** 2 cores
- **RAM:** 4 GB
- **Storage:** 20 GB HDD
- **Network:** a public network interface
- **DNS:** a domain record pointing to the public network interface

### 1.2 Supported Operating Systems

Currently, releases are provided as DEB and RPM packages. The packages are neither architecture-dependent nor tied to a specific Linux distribution.

We recommend using one of the following distributions:

| Distributions using `.deb`     | Distributions using `.rpm`      |
|--------------------------------|--------------------------------------|
| Debian 12 "Bookworm"           | RHEL 9                               |
| Debian 13 "Trixie"             | RHEL 10                              |
| Ubuntu 22.04 LTS "Jammy Jellyfish" | AlmaLinux 9                     |
| Ubuntu 24.04 LTS "Noble Numbat"    | AlmaLinux 10                    |
|                                | Rocky Linux 9                        |
|                                | Rocky Linux 10                       |
|                                | openSUSE Leap 15                   |
|                                | SLES 15                           |

### 1.3 Firewall Configuration

The following firewall ports should be accessible for the standard deployment scenario. Installation scenarios behind restrictive networks are discussed elsewhere.

- **80, 443/tcp** (HTTP, HTTPS)
- **40000-49999/udp** (RTP Obelisk)
- **5060/udp** (SIP Obelisk)
- **50000-64535/udp** (RTP LiveKit)
- **7881/tcp** (WebRTC ICE over TCP, LiveKit)
- **3478/udp** (LiveKit TURN)

### 1.4 DNS Domain Record

Make sure that an appropriate DNS domain record for OpenTalk is configured and points to the public IPv4 and IPv6 addresses of your server.
It is the domain name used to access the OpenTalk web application, e.g., `https://example.com`.

### 1.5 Docker as Container Engine is Required

Docker and the Docker Compose plugin must be installed before you start with the `opentalk-compose` installation. If this is not yet the case, follow the [official Docker installation documentation](https://docs.docker.com/engine/install/debian/#install-using-the-repository).

## 2. Installation Procedure

### 2.1 Add the OpenTalk Package Repository

**Debian and Ubuntu-based Distributions:**

```bash
apt update && apt install -y curl gnupg
curl http://anfi.opentalk.run:7376/deb-testing/public.gpg | gpg --yes --dearmor -o /usr/share/keyrings/openrepo-deb-testing.gpg
echo "deb [arch=any signed-by=/usr/share/keyrings/openrepo-deb-testing.gpg] http://anfi.opentalk.run:7376/deb-testing/ stable main" > /etc/apt/sources.list.d/openrepo-deb-testing.list
```

**RHEL and SuSE-based Distributions:**

```bash
echo """
[rpm-testing]
name=rpm-testing
baseurl=http://anfi.opentalk.run:7376/rpm-testing
enabled=1
repo_gpgcheck=1
gpgkey=http://anfi.opentalk.run:7376/rpm-testing/public.gpg
""" > /etc/yum.repos.d/rpm-testing.repo
```

### 2.2 Install the `opentalk-compose` Package

**Debian and Ubuntu-based Distributions:**

```bash
apt update && apt install opentalk-compose -y
```

**RHEL-based Distributions:**

```bash
dnf install opentalk-compose -y
```

**SuSE-based Distributions:**

```bash
zypper install opentalk-compose -y
```

### 2.3 Set the OpenTalk Server's DNS Domain Record

Define your primary domain in the central configuration file `settings.yml` using the key `ot_domain`.

Edit `/etc/opentalk-compose/settings.yml` and add your domain:

```yaml
ot_domain: example.com
```

### 2.4 Initialize the Deployment Environment

Initialize secrets and baseline configuration using the embedded automation (a first run performs all required bootstrap steps automatically):

```bash
otctl deploy
```

First run effects:

- Generates required secrets (if not present)
- Renders `compose.yml` and service configuration files
- Prepares runtime directories & symlinks
- Starts containers (equivalent to docker compose up -d with generated spec)

> **Note:** Configuring and starting all components may take several minutes, especially during the initial run. Please wait until the process is complete before attempting to access the web application.
After the `init` phase the OpenTalk web application will be reachable at e.g. `https://example.com`.

### 2.5 Optional: Configure & Deploy (Edit `settings.yml`)

If the default configuration does not fit your needs and you prefer different settings, adjust `/etc/opentalk-compose/settings.yml` (symlink also at `/opt/opentalk-compose/config/settings.yml`) before or after the first run to tune configuration (domain, certificates scenario, external services, feature toggles). Keep YAML indentation valid because syntax errors will abort the playbook.

After editing `settings.yml`, simply re-run the deployment (idempotent):

```bash
otctl deploy
```

Only changed templates / settings are re-applied. You can safely repeat this; it is idempotent.

> **Note:** You will notice that `otctl` first outputs the command it is about to execute: `[otctl] + <command>`. Since `otctl` is just a wrapper around `ansible` and `docker compose`, this can help you to track what is happening and serves as a reference if you want to work outside of `otctl`.

### 2.6 Check Service Status & Health

Use `otctl status` to get a concise overview including container health (if a healthcheck is defined in the compose spec):

```bash
otctl status
```

Example output:

```text
SERVICE       STATE     HEALTH
controller    running   healthy
keycloak      running   healthy
minio         running   healthy
obelisk       running   healthy
webapp        running   healthy
```

Useful follow-up commands:

```bash
otctl logs controller            # Tail logs for a single service
otctl restart webapp             # Restart only the web frontend
otctl recreate controller        # Force recreate one service (new image)
otctl version                    # Show bundled component versions
```

If something is unhealthy:

- Check logs: `otctl logs <service>`
- Re-run automation (ensures configuration rendered): `otctl deploy`
- Inspect raw compose: `otctl compose ps` / `otctl compose logs <service>`

### 2.7 Backups (Optional but Recommended)

The embedded playbook provides structured backup targets. Default backup root is `/var/opt/opentalk-compose/backups` (falls back to `/opt/opentalk-compose/backups` if missing).

Core commands:

```bash
otctl backup            # Full backup tag (databases + config + volumes as defined)
otctl backup ls         # List existing backup artifacts
otctl backup dump       # DB dumps (controller & keycloak)
otctl backup dump controller   # Only controller DB
otctl backup dump keycloak     # Only keycloak DB
otctl backup config     # Configuration snapshot
otctl backup volume     # All defined docker volumes
otctl backup volume <name>   # Single docker volume by name
otctl backup clean      # Apply retention policy (prune old backups)
```

Notes:

- Retention logic is handled by dedicated Ansible tags (e.g. `backup_retention`).
- To target a single docker volume, its exact volume name is passed; no project prefix is auto-added.
- Extended variables (e.g. custom volume list) can be injected via `OTCTL_BACKUP_EXTRA_VARS` (advanced).

### 2.8 Undeploy (Tear Down)

To remove containers (keeping data volumes) and clean generated runtime artifacts:

```bash
otctl undeploy
```

This performs:

1. `docker compose down` (if `compose.yml` is present)
2. Removal of the generated `compose.yml`
3. Cleanup of contents inside `services/` (directory itself is retained)

To fully purge (volumes, images) afterwards, run manual docker commands as needed (e.g. `docker volume ls | grep opentalk_` then prune carefully).

### 2.9 Optional: Enable Demo User

By default, there is no user available to log in to the OpenTalk dashboard. For testing purposes, you can enable the demo user via an Ansible tag.
The user name is `demo`; the password is generated (or taken from settings) and shown when running with `--diff`.
Otherwise, manage regular users via the Keycloak administration console.

```bash
otctl deploy --diff --tags demo-user
```

If you later wish to remove the demo user:

```bash
otctl deploy --tags demo-user -e postInstall_demoUser_state=absent
```

### 2.10 Reference: `otctl` Overview

Frequently used subcommands (see `otctl help` for the exhaustive list):

| Command | Purpose |
|---------|---------|
| otctl deploy | Render & (re)start stack via Ansible + compose |
| otctl status | Show service + health summary |
| otctl logs [svc] | Stream logs (all or specific service) |
| otctl restart [svc...] | Restart one or multiple services |
| otctl recreate [svc...] | Force recreate (new container from image) |
| otctl backup … | Managed backup workflows |
| otctl version | Show product & component versions |
| otctl undeploy | Stop stack & remove generated compose.yml |
| otctl compose … | Raw passthrough to `docker compose` |
| otctl playbook … | Advanced direct Ansible usage |

Health Column Semantics (`otctl status`):

- `healthy` – Container healthcheck succeeded.
- `unhealthy` – Healthcheck failing (inspect logs).
- `starting` – Healthcheck not yet passing (grace period).
- `n/a` – No healthcheck defined in compose spec.
