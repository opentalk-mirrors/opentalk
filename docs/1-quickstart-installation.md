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
  - [2.2 Install the `opentalk-compose` Package](#22-install-the-opentalk-compose-package)  
  - [2.3 Set the OpenTalk Server Domain](#23-set-the-opentalk-server-domain)  
  - [2.4 Initialize the Deployment Environment](#24-initialize-the-deployment-environment)  
  - [2.5 Optional: Enable Demo User](#25-optional-enable-demo-user)  
  - [2.6 Optional: Adjust Configuration & Redeploy](#26-optional-adjust-configuration--redeploy)  
  - [2.7 Check Service Status and Health](#27-check-service-status-and-health)  
  - [2.8 Backups (Optional but Recommended)](#28-backups-optional-but-recommended)  
  - [2.9 Reference: `otctl` Overview](#29-reference-otctl-overview)
- [3. Optional: SMTP Mailer Service](#3-optional-smtp-mailer-service)  
  - [3.1 What is the SMTP Mailer](#31-what-is-the-smtp-mailer)  
  - [3.2 Enable & Configure SMTP Mailer](#32-enable--configure-smtp-mailer)  
  - [3.3 Verify SMTP Mailer](#33-verify-smtp-mailer)  
- [4. Optional: Obelisk SIP Bridge Service](#4-optional-obelisk-sip-bridge-service)  
  - [4.1 What is Obelisk](#41-what-is-obelisk)  
  - [4.2 Configure SIP Provider and Call-In Settings](#42-configure-sip-provider-and-call-in-settings)  
  - [4.3 Enable Obelisk Service](#43-enable-obelisk-service)  
  - [4.4 Verify Obelisk Status](#44-verify-obelisk-status)  
- [5. Optional: TURN over TLS with Embedded TURN Server (LiveKit)](#5-optional-turn-over-tls-with-embedded-turn-server-livekit)  
  - [5.1 Default Embedded TURN Behavior](#51-default-embedded-turn-behavior)  
  - [5.2 Enabling TURN over TLS](#52-enabling-turn-over-tls)  
    - [5.2.1 Configuration in settings.yml](#521-configuration-in-settingsyml)  
    - [5.2.2 Deploy configuration changes](#522-deploy-configuration-changes)  
  - [5.3 Verification](#53-verification)
- [6. Optional: Configure the OpenTalk Outlook Add-in](#6-optional-configure-the-opentalk-outlook-add-in)  
  - [6.1 What is the OpenTalk Outlook Add-in](#61-what-is-the-opentalk-outlook-add-in)  
  - [6.2 Prerequisites](#62-prerequisites)  
  - [6.3 Enable the Service](#63-enable-the-service)  
  - [6.4 Add the Add-in to Microsoft Outlook](#64-add-the-add-in-to-microsoft-outlook)  
- [7. Updating OpenTalk Compose](#7-updating-opentalk-compose)  
  - [7.1 Overview](#71-overview)  
  - [7.2 Step 1: Create a Full Backup](#72-step-1-create-a-full-backup)  
  - [7.3 Step 2: Update Repository & Package](#73-step-2-update-repository--package)  
    - [7.3.1 Debian / Ubuntu](#731-debian--ubuntu)  
    - [7.3.2 RHEL-based](#732-rhel-based)  
    - [7.3.3 SUSE-based](#733-suse-based)  
  - [7.4 Step 3: Redeploy Services](#74-step-3-redeploy-services)  
  - [7.5 Verification & Rollback Notes](#75-verification--rollback-notes)

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
curl http://anfi.opentalk.run:7376/deb-experimental/public.gpg | gpg --yes --dearmor -o /usr/share/keyrings/openrepo-deb-experimental.gpg
echo "deb [arch=any signed-by=/usr/share/keyrings/openrepo-deb-experimental.gpg] http://anfi.opentalk.run:7376/deb-experimental/ stable main" > /etc/apt/sources.list.d/openrepo-deb-experimental.list
apt update
```

RHEL-based:

```bash
echo """
[rpm-experimental]
name=rpm-experimental
baseurl=http://anfi.opentalk.run:7376/rpm-experimental
enabled=1
repo_gpgcheck=1
gpgkey=http://anfi.opentalk.run:7376/rpm-experimental/public.gpg
""" > /etc/yum.repos.d/opentalk-compose-rpm-experimental.repo
```

SUSE-based:

```bash
echo """
[rpm-experimental]
name=rpm-experimental
baseurl=http://anfi.opentalk.run:7376/rpm-experimental
enabled=1
repo_gpgcheck=1
gpgkey=http://anfi.opentalk.run:7376/rpm-experimental/public.gpg
""" > /etc/zypp/repos.d/opentalk-compose-rpm-experimental.repo
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

Run the initial deployment:

```bash
otctl deploy
```

Initial run performs:

- Generate required secrets (if absent)
- Render `compose.yml` and service configs
- Prepare runtime directories and symlinks
- Start containers (equivalent to `docker compose up -d`)

> **Note:** Initial configuration and startup may take several minutes. Wait until completion before accessing the application.
> After initialization the web UI is available at: <https://example.com> (replace with your domain).

### 2.5 Optional: Enable Demo User

A demo user can be created for testing. Username: `demo`. The password is generated (or taken from settings) and shown with `--diff`.

Enable:

```bash
otctl deploy --diff --tags demo-user
```

Remove:

```bash
otctl deploy --tags demo-user -e postInstall_demoUser_state=absent
```

Regular user management is done via Keycloak. Refer the docs [Managing OpenTalk Users](./3-managing-opentalk-users-in-keycloak.md).

### 2.6 Optional: Adjust Configuration & Redeploy

Modify `/etc/opentalk-compose/settings.yml` to adapt domain, certificate handling, external services, or feature toggles. Maintain a valid YAML indentation.

Redeploy (idempotent):

```bash
otctl deploy
```

Only changed settings are applied; re-running is safe.

> **Tip:** `otctl` prints the underlying command (`[otctl] + <command>`) for transparency.

### 2.7 Check Service Status and Health

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
...
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
- Re-deploy config: `otctl deploy`
- Inspect raw compose: `otctl compose ps`, `otctl compose logs <service>`

### 2.8 Backups (Optional but Recommended)

Default backup root: `/var/opt/opentalk-compose/backups` (falls back to `/opt/opentalk-compose/backups`).

Core commands:

```bash
otctl backup all             # Full backup (databases + config + volumes)
otctl backup ls              # List backups
otctl backup dump            # All DB dumps
otctl backup dump controller # Controller DB only
otctl backup dump keycloak   # Keycloak DB only
otctl backup config          # Config snapshot
otctl backup volume          # Backup volumes of non-database containers 
otctl backup volume <name>   # Single volume
otctl backup clean           # Apply retention policy
```

Notes:

- Retention is controlled via Ansible tags (e.g. `backup_retention`).
- Volume names must match exactly (no automatic prefixing).
- Extra vars can be passed via `OTCTL_BACKUP_EXTRA_VARS` for advanced use.

### 2.9 Reference: `otctl` Overview

| Command | Purpose |
|---------|---------|
| otctl deploy | Render and (re)start stack |
| otctl status | Show service + health summary |
| otctl logs [svc] | Stream logs |
| otctl restart [svc...] | Restart services |
| otctl recreate [svc...] | Recreate containers |
| otctl backup … | Backup workflows |
| otctl version | Show versions |
| otctl compose … | Pass through to docker compose |
| otctl playbook … | Advanced Ansible usage |

Health column semantics (`otctl status`):

- healthy: Healthcheck passed
- unhealthy: Healthcheck failing
- starting: Healthcheck not yet passing
- n/a: No healthcheck defined

## 3. Optional: SMTP Mailer Service

### 3.1 What is the SMTP Mailer

The SMTP Mailer service sends transactional emails for OpenTalk (meeting invitations, notifications). It connects to a configured SMTP server and derives sender addresses and links from your `ot_domain` setting. The mailer requires valid credentials for your SMTP server; therefore it is disabled by default until configured.

For a full list of available SMTP Mailer configuration variables see the [Configuration Options Reference](5-configuration-options-reference.md#opentalk_composeopentalk_smtp_mailer).

### 3.2 Enable & Configure SMTP Mailer

Set the required options in `/etc/opentalk-compose/settings.yml` before deploying:

```yaml
compose_service_opentalk_smtp_mailer_enabled: true

mailer_smtp_smtp_server: "smtp://user%40example.com:PaSsw0Rd@mail.example.com:587"
mailer_smtp_from_name: "OpenTalk - {{ ot_domain }}"
mailer_smtp_from_email: "no-reply@{{ ot_domain }}"
mailer_frontend_base_url: "https://{{ ot_domain }}"
mailer_support_contact_phone: "+49 123 456789"
mailer_support_contact_mail: "support@{{ ot_domain }}"
```

Notes:

- You can reuse `{{ ot_domain }}` inside these smtp mailer options; it is substituted when templates render.
- The `mailer_smtp_smtp_server` value embeds credentials URL-encoded (e.g. `user%40example.com` for `user@example.com`). Ensure password escaping if it contains special characters (`:` `@` `/`).
- Use `smtp://` or `smtps://` (implicit TLS) depending on your provider. For STARTTLS on port 587 `smtp://` is typical. Implicit TLS typical uses port 465 `smtps://`
- Keep this file secured; credentials are sensitive.

Deploy after editing:

```bash
otctl deploy
```

### 3.3 Verify SMTP Mailer

Check service status:

```bash
otctl status
```

Look for:

```text
mailer        running   healthy
```

Review logs:

```bash
otctl logs mailer
```

Common verification steps:

1. Trigger an action that sends email (invite user to event).
2. Confirm log line showing successful SMTP connection / send.
3. Inspect mail headers for correct `From:` and links.

Troubleshooting:

- Authentication failures: Re-check URL-encoded username, password validity.
- TLS errors: Switch between `smtp://` + STARTTLS and `smtps://` depending on provider requirements.
- Missing emails: Verify provider spam/relay policies and `mailer_frontend_base_url` correctness for links.
- Support contacts: Ensure phone/mail fields appear in templates (template customization may require additional steps beyond quickstart).

## 4. Optional: Obelisk SIP Bridge Service

### 4.1 What is Obelisk

Obelisk is a SIP (Session Initiation Protocol) telephony bridge service that enables traditional phone systems and SIP clients to connect to OpenTalk video conferences. This allows participants to:

- Join OpenTalk meetings via traditional telephone
- Connect from SIP-enabled desk phones
- Provide dial-in access for participants without internet or video capabilities

For all Obelisk-related configuration variables refer to the [Configuration Options Reference](5-configuration-options-reference.md#opentalk_composeopentalk_obelisk).

Obelisk requires credentials for your SIP provider or PBX system, so it is disabled by default.

### 4.2 Configure SIP Provider and Call-In Settings

Before enabling Obelisk, you must configure both the OpenTalk Controller call-in settings and the SIP provider connection.

#### Controller Call-In Configuration

First, configure the OpenTalk Controller for telephone integration in `/etc/opentalk-compose/settings.yml`:

```yaml
# Enable call-in functionality in controller and set the dial-in number
controller_call_in_tel: "+49 123 456789"
controller_call_in_enable_phone_mapping: false
controller_call_in_default_country_code: DE
```

#### Basic SIP Configuration

Configure the connection to your SIP provider or PBX system:

```yaml
# SIP server configuration
obelisk_sip_username: "your_sip_username"
obelisk_sip_password: "your_sip_password" 
obelisk_sip_realm: "your_sip_domain"
obelisk_sip_registrar: "sip:your_sip_server.example.com"
```

#### Advanced SIP Options

```yaml
# Optional: Custom SIP ID and contact
obelisk_sip_id: "sip:opentalk@your_domain.com"
obelisk_sip_contact: "sip:opentalk@your_public_ip:5060"

# Optional: Outbound proxy
obelisk_sip_outbound_proxy: "sip:proxy.example.com:5060"

# Optional: STUN server for NAT traversal
obelisk_sip_stun_server: "stun:stun.example.com:3478"

# Optional: RTP port range customization
obelisk_sip_rtp_port_range_start: 40000
obelisk_sip_rtp_port_range_end: 49999

# Optional: Enable SRTP encryption
obelisk_sip_offer_srtp: true

# Optional: Bitrate limits
obelisk_sip_max_video_bitrate: 6000
```

#### Firewall Considerations

Ensure the following ports are accessible:

- `5060/udp` - SIP signaling (default)
- `40000-49999/udp` - RTP media (configurable range)

#### Example SIP Configuration for Sipgate

```yaml
# Sipgate SIP configuration
obelisk_sip_username: "username"
obelisk_sip_password: "******"
obelisk_sip_realm: "sipconnect.sipgate.de"
obelisk_sip_registrar: "sip:sipconnect.sipgate.de"
obelisk_sip_outbound_proxy: "sip:sipconnect.sipgate.de"
obelisk_sip_stun_server: "stun.sipgate.net:10000"
```

#### Example SIP Configuration for Asterisk (PBX)

```yaml
# Asterisk SIP configuration
obelisk_sip_username: "username"
obelisk_sip_password: "******"
obelisk_sip_realm: "asterisk"
obelisk_sip_registrar: "sip:asterisk.example.com"
obelisk_sip_stun_server: "stun.example.com:3478"
```

### 4.3 Enable Obelisk Service

> **Important:** Obelisk must be properly configured before it can be enabled and started. The service will fail to start without valid SIP provider configuration.

Edit `/etc/opentalk-compose/settings.yml` and set

```yaml
compose_service_opentalk_obelisk_enabled: true
```

#### Deploy the configuration

```bash
otctl deploy
```

### 4.4 Verify Obelisk Status

After configuration, verify the service is running:

```bash
otctl status
```

Expected output should include:

```text
SERVICE       STATE     HEALTH
...
obelisk       running   healthy
...
```

Check Obelisk logs for SIP registration status:

```bash
otctl logs obelisk
```

Look for successful SIP registration messages. If there are connection issues, verify:

1. SIP credentials are correct
2. Firewall ports are open (5060/udp, RTP range)
3. Network connectivity to SIP server
4. DNS resolution of SIP server hostname

For detailed configuration options, refer to the [Configuration Options Reference](5-configuration-options-reference.md#opentalk_composeopentalk_obelisk).

## 5. Optional: TURN over TLS with Embedded TURN Server (LiveKit)

Reliable WebRTC connectivity sometimes requires TURN relaying when direct peer-to-peer connections are blocked by NAT or restrictive firewalls. OpenTalk ships with an embedded TURN server provided by LiveKit. This section explains the default behavior, how to enable TURN over TLS, required DNS changes, configuration options, verification, troubleshooting, scaling considerations, and security recommendations. For deploying a separate (dedicated) TURN service with coturn, see the advanced guide: [Dedicated TURN Server (coturn)](./4-advanced-installation-scenarios.md#5-install-and-configure-a-dedicated-turn-server-coturn).

### 5.1 Default Embedded TURN Behavior

The embedded TURN server is part of the LiveKit deployment and is managed by OpenTalk Compose. Out-of-the-box (without extra settings) LiveKit exposes TURN (UDP) on port `3478` using the host's main domain (e.g. `example.com:3478`). This supports most scenarios. Enabling TURN over TLS (TCP/TLS on port `443` behind a proxy) improves connectivity behind very restrictive corporate firewalls.

### 5.2 Enabling TURN over TLS

To offer TURN over TLS you should create a dedicated domain record (recommended: `turn.<ot_domain>`). This allows the loadbalancer (Traefik) to distinguish normal HTTPS traffic from TURN TLS signaling and relay flows.
Add a DNS A/AAAA record that points to the same host as the main OpenTalk services.

Requirements when enabling TURN TLS:

- Subdomain (e.g. `turn.example.com`) must resolve to the OpenTalk host
- Custom certificate must cover the TURN subdomain for TLS, ACME / Let's Encrypt gets cert automatically (default)

#### 5.2.1 Configuration in settings.yml

Edit `/etc/opentalk-compose/settings.yml` and set or adjust the following variables.

```yaml
livekit_turn_embedded_external_tls: true
livekit_turn_embedded_domain: "turn.{{ ot_domain }}" # expands to turn.example.com
```

#### 5.2.2 Deploy configuration changes

```bash
otctl deploy
```

### 5.3 Verification

1. Open a meeting in a browser from a restrictive network.
2. Inspect ICE candidates via Chrome (`chrome://webrtc-internals`) or Firefox (`about:webrtc`).
3. Confirm presence of `relay` candidates referencing your TURN domain.
4. Check LiveKit logs: `otctl logs livekit-server` for successful TURN allocation entries.

## 6. Optional: Configure the OpenTalk Outlook Add-in

The  OpenTalk Outlook Add-in integrates OpenTalk scheduling and meeting join links directly into the Microsoft Outlook Calendar. The OpenTalk Outlook Add-in in needs to be configured via the web version of Microsoft Outlook. But once installed, the add-in is bound to your account and can be used with any device.

### 6.1 What is the OpenTalk Outlook Add-in

The add-in provides:

- Creation of OpenTalk meetings from Outlook
- Insertion of meeting join links into calendar events
- User authentication via the existing OpenTalk Keycloak realm (OIDC)

The add-in is provided by the OpenTalk instance via an endpoint, e.g. <https://example.com/outlook/manifest.xml>. This XML file is then added to Microsoft Outlook.

### 6.2 Prerequisites

- A working base installation (sections 1–2 completed)
- Valid HTTPS setup (Let's Encrypt or custom certs)
- At least one existing OpenTalk user account

### 6.3 Enable the Service

Set the OpenTalk Outlook Add-in service in `/etc/opentalk-compose/settings.yml` to `enable`:

```yaml
compose_service_opentalk_outlook_add_in_enabled: true
```

Then deploy:

```bash
otctl deploy --tags outlook_add_in
```

This renders and starts the `outlook_add_in` service (container) with the required Traefik routing labels.

Check status:

```bash
otctl status
```

### 6.4 Add the Add-in to Microsoft Outlook

Proceed with the following steps to add the OpenTalk Outlook Add-in in Outlook:

1. Login to your OpenTalk instance
2. Download the manifest from <https://{{ ot_domain }}/outlook/manifest.xml>
3. Visit <https://outlook.live.com/owa/?path=/options/manageapps>
4. Select Add-ins → My add-ins → Add from file
5. Add the downloaded manifest e.g. `opentalk-outlook.xml`

When creating a new calendar event, open the three-dot menu and choose OpenTalk. You are redirected to OpenTalk to grant the permissions for event creation. After authorization, any Outlook events where OpenTalk was selected via the three-dot menu are also created in OpenTalk.

## 7. Updating OpenTalk Compose

Keeping your OpenTalk Compose installation current ensures you receive security fixes, performance improvements, and new features. Updates are designed to be safe and idempotent. The recommended workflow mirrors the initial installation and consists of three main steps: create a full backup, update the package from the repository, and redeploy the stack.

> **Maintenance Window:** Perform updates during a low-usage window to avoid interrupting active meetings. A typical update (package + redeploy) usually completes within a few minutes.

### 7.1 Overview

High-level process:

1. Create a full backup (databases, config, volumes)
2. Refresh repository metadata and upgrade the `opentalk-compose` package
3. Run `otctl deploy` to render any updated templates and restart services if needed

### 7.2 Step 1: Create a Full Backup

Use the bundled backup facility before every update:

```bash
otctl backup all
otctl backup ls   # (optional) verify backup presence
```

Backups are stored under `/var/opt/opentalk-compose/backups`. Keep sufficient disk space free. For more granular commands see section 2.8.

### 7.3 Step 2: Update Repository & Package

Ensure your repository configuration is still present (see section 2.1). Then update according to your distribution family.

#### 7.3.1 Debian / Ubuntu

```bash
apt update
apt install --only-upgrade opentalk-compose
```

#### 7.3.2 RHEL-based

```bash
dnf makecache
dnf upgrade -y opentalk-compose
```

#### 7.3.3 SUSE-based

```bash
zypper refresh
zypper update -y opentalk-compose
```

### 7.4 Step 3: Redeploy Services

After upgrading the package, redeploy to apply any new templates, configuration changes, or image updates:

```bash
otctl deploy
```

The deploy is idempotent: unchanged services remain running; changed components are recreated or restarted automatically.

### 7.5 Verification & Rollback Notes

Verify versions and health:

```bash
otctl version
otctl status
```

Check logs of critical services (e.g. controller, keycloak, livekit-server) for startup issues:

```bash
otctl logs controller
otctl logs livekit-server
otctl logs keycloak
```

If problems occur:

- Re-run deploy: `otctl deploy`
- Compare configs: use previous config backup snapshot
- Restore from backup using the artifacts created in Step 1

> **Tip:** Maintain at least the last two successful full backups to facilitate rollback in rare edge cases.
