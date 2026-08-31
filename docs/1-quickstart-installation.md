<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# Quickstart Guide to Self-Hosting OpenTalk

OpenTalk Compose is a set of tools and utilities for running the OpenTalk video conferencing solution on your own server. This quickstart guide covers the minimal steps required to get the necessary services up and running. It assumes a standard scenario with a publicly accessible server where all components run on a single host and SSL/TLS certificates are obtained via Let's Encrypt.
For customized setups or special conditions, refer to the OpenTalk Administration Guide. The essentials are explained here.

## 1. Preparations

### 1.1 Minimal System Requirements

The target server should meet at least:

- CPU: 2 cores
- RAM: 4 GB
- Storage: 20 GB
- Network: Public network interface
- DNS: A domain record pointing to the public interface

### 1.2 Supported Operating Systems

Releases are provided as DEB and RPM packages. The packages are built for the x86-64 architecture and not tied to a specific Linux distribution.

Recommended distributions:

| Distributions using `.deb`                 | Distributions using `.rpm`  |
|--------------------------------------------|-----------------------------|
| Debian 12 "Bookworm"                       | RHEL 9                      |
| Debian 13 "Trixie"                         | RHEL 10                     |
| Ubuntu 22.04 LTS "Jammy Jellyfish"         | AlmaLinux 9                 |
| Ubuntu 24.04 LTS "Noble Numbat"            | AlmaLinux 10                |
| Ubuntu 26.04 LTS "Resolute Raccoon"        | Rocky Linux 9               |
|                                            | Rocky Linux 10              |
|                                            | openSUSE Leap 15            |
|                                            | openSUSE Leap 16            |
|                                            | SLES 15                     |
|                                            | SLES 16                     |

### 1.3 Firewall Configuration

Ensure these ports are reachable in a standard deployment:

- 80, 443/tcp (HTTP, HTTPS)
- 40000-49999/udp (RTP Obelisk)
- 5060/udp (SIP Obelisk)
- 50000-65535/udp (RTP LiveKit)
- 7881/tcp (WebRTC ICE over TCP, LiveKit)
- 3478/udp (LiveKit TURN)
- 5349/tcp (LiveKit TLS TURN - optional)

If the system itself has an external firewall is setup to block incoming connection.
Additionally to the public ports you have to allow the internal services to access Livekit.

Therefore you need to find your the docker opentalk network, e.g. 172.18.0.0/24
and allow at least the ports: TCP/7880, UDP/50000-65535.

```bash
# controller, recorder obelisk -> livekit websocket
iptables -A INPUT -p tcp -s 172.18.0.0/24 --dport 7880 -j ACCEPT
# recorder obelisk -> livekit RTP
iptables -A INPUT -p udp -s 172.18.0.0/24 --dport 50000:65535 -j ACCEPT
```

### 1.4 DNS Domain Record

Create a DNS record pointing to the server's public IPv4 and IPv6 addresses. This is the domain used to access the web application (e.g. <https://example.com>).

### 1.5 Docker Requirement

Install Docker and the Docker Compose plugin before installing `opentalk-compose`. If not installed, follow the official Docker documentation.

### 1.6 Automatic Address Discovery

Livekit uses per default a google STUN server to infer its own public accessible IP address,
while convenient this can changed by:
 - either using another STUN server. There is a section in the `settings.yml` to tweak:
```
  livekit_rtc_stun_servers:
   - "stun.YOUR-PROVIDER:STUN-PORT"

```
 - or disable `livekit_rtc_use_external_ip` and configure
    `livekit_rtc_node_ip` instead to the public IP of the livekit-service.

## 2. Installation Procedure

### 2.1 Add the Package Repository and install `opentalk-compose`

=== "Debian / Ubuntu"

  For Debian 12, 13 and Ubuntu 24.04/26.04 based systems.

    ```bash
    # Download and install the GPG public key
    curl -fsSL \
      https://repo.heinlein.group/repo/opentalk-compose/opentalk-compose_sign.asc | \
      sudo tee /etc/apt/trusted.gpg.d/opentalk-archive-keyring.asc > /dev/null

    # Add the OpenTalk Compose repository
    echo "deb [arch=amd64] \
      https://repo.heinlein.group/repo/opentalk-compose/deb/stable/ all main" | \
      sudo tee /etc/apt/sources.list.d/opentalk-compose.list

    # Update package index
    sudo apt update

    # Install package opentalk-compose
    sudo apt install opentalk-compose
    ```


=== "RHEL / Rocky / AlmaLinux"

    For RHEL 9, 10 and compatible distributions (Rocky Linux, AlmaLinux, etc.).

    ```bash
    # Download and install the GPG public key
    sudo rpm --import \
      https://repo.heinlein.group/repo/opentalk-compose/opentalk-compose_sign.asc

    # Add the OpenTalk Compose repository
    sudo tee /etc/yum.repos.d/opentalk-stable.repo <<EOF
    [opentalk-compose]
    name=OpenTalk Compose Repository - Stable Releases
    baseurl=https://repo.heinlein.group/repo/opentalk-compose/rpm/stable/
    enabled=1
    gpgcheck=1
    gpgkey=https://repo.heinlein.group/repo/opentalk-compose/opentalk-compose_sign.asc
    EOF

    # Update repository metadata
    sudo dnf makecache

    # Install package opentalk-compose
    sudo dnf install opentalk-compose
    ```

=== "SLES / openSUSE"

    For SUSE Linux Enterprise 15, 16 and openSUSE Leap based distributions.

    ```bash
    # Download and install the GPG public key
    sudo rpm --import \
      https://repo.heinlein.group/repo/opentalk-compose/opentalk-compose_sign.asc

    # Add the OpenTalk Compose (stable) repository
    sudo tee /etc/zypp/repos.d/opentalk-stable.repo <<EOF
    [opentalk-compose-stable]
    name=OpenTalk Compose Repository - Stable Releases
    baseurl=https://repo.heinlein.group/repo/opentalk-compose/rpm/stable/
    enabled=1
    gpgcheck=1
    gpgkey=https://repo.heinlein.group/repo/opentalk-compose/opentalk-compose_sign.asc
    EOF

    # Refresh repositories and install OpenTalk Compose
    sudo zypper refresh

    # Install package opentalk-compose
    sudo zypper install opentalk-compose
    ```

### 2.2 Initialize the Deployment Environment

Add your Domain to the `ot_domain` key in `/etc/opentalk-compose/settings.yml`

```yaml
ot_domain: example.com
```

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

### 2.3 Optional: Enable Demo User

A demo user can be created for testing. The user name is `demo`. The password is generated (or taken from `settings.yml`).

Enable:

```bash
otctl deploy --diff --tags demo-user
```

Find the password for `demo` in `/etc/opentalk-compose/secrets/secrets.yml`.

```bash
grep DEMO_USER_PASSWORD /etc/opentalk-compose/secrets/secrets.yml
```

Remove:

```bash
otctl deploy --tags demo-user -e keycloak_demo_user_state=absent
```

Regular user management is done via Keycloak. Refer the docs [Managing OpenTalk Users](./3-managing-opentalk-users-in-keycloak.md).

### 2.4 Optional: Adjust Configuration & Redeploy

Modify `/etc/opentalk-compose/settings.yml` to adapt domain, certificate handling, external services, or feature toggles. Maintain a valid YAML indentation.

Redeploy (idempotent):

```bash
otctl deploy
```

Only changed settings are applied; re-running is safe.

> **Tip:** `otctl` prints the underlying command (`[otctl] + <command>`) for transparency.

### 2.5 Check Service Status and Health

```bash
otctl status
```

Example output varies depending on which optional services are enabled. On a standard single-host deployment, you should see the core services such as:

```text
SERVICE       STATE     HEALTH
controller    running   healthy
controller-db running   healthy
etherpad      running   healthy
garage        running   healthy
keycloak      running   healthy
keycloak-db   running   healthy
livekit-server running  healthy
ndt           running   healthy
rabbitmq      running   healthy
recorder      running   healthy
spacedeck     running   healthy
traefik       running   healthy
web-frontend  running   healthy
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

### 2.6 Backups (Optional but Recommended)

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

### 2.7 Reference: `otctl` Overview

| Command | Purpose |
|---------|---------|
| otctl deploy | Render and (re)start stack |
| otctl status | Show service + health summary |
| otctl logs \[svc...\] | Stream logs |
| otctl restart \[svc...\] | Restart services |
| otctl recreate \[svc...\] | Recreate containers |
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

Reliable WebRTC connectivity sometimes requires TURN relaying when direct peer-to-peer connections are blocked by NAT or restrictive firewalls. OpenTalk ships with an embedded TURN server provided by LiveKit. This section explains the default behavior, how to enable TURN over TLS, required DNS changes, configuration options, verification, troubleshooting, scaling considerations, and security recommendations. For deploying a separate (dedicated) TURN service with coturn, see the advanced guide: [Dedicated TURN Server (coturn)](./4-advanced-installation-scenarios.md#5-external-turn-and-stun-server-integration-coturn).

### 5.1 Default Embedded TURN Behavior

The embedded TURN server is part of the LiveKit deployment and is managed by OpenTalk Compose.
Out-of-the-box (without extra settings) LiveKit exposes TURN (UDP) on port `3478` using the host's main domain (e.g. `example.com:3478`).
While this helps in some scenarios, if only TCP port 80 (HTTP) and 443(HTTPS) are available to the client, this will not bi sufficient.

### 5.2 Enabling TURN over TLS

Enabling TURN over TLS (TCP/TLS on port `443` behind a proxy) improves connectivity behind very restrictive corporate firewalls.
To offer TURN over TLS you should create a dedicated domain record (recommended: `turn.<ot_domain>`).
This allows the loadbalancer (Traefik) to distinguish normal HTTPS traffic from TURN TLS signaling and relay flows.
Add a DNS A/AAAA record that points to the same host as the main OpenTalk services.

Requirements when enabling TURN TLS:

- Subdomain (e.g. `turn.<ot_domain>`) must resolve to the OpenTalk host
- Custom certificate must cover the TURN subdomain for TLS, ACME / Let's Encrypt gets cert automatically (default)
- Connections to `turn.<ot_domain>:443 (TCP)` need to be forwarded to the livekit service on port 5349 (TCP) by the loadbalancer

#### 5.2.1 Configuration in settings.yml

Edit `/etc/opentalk-compose/settings.yml` and set or adjust the following variables.

```yaml
livekit_turn_embedded_tls_enabled: true
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
3. Check for package-provided config update files and merge new options
4. Run `otctl deploy` to render any updated templates and restart services if needed

### 7.2 Step 1: Create a Full Backup

Use the bundled backup facility before every update:

```bash
otctl backup all
otctl backup ls   # (optional) verify backup presence
```

Backups are stored under `/var/opt/opentalk-compose/backups`. Keep sufficient disk space free. For more granular commands see section 2.8.

If the release includes a PostgreSQL major upgrade (for example controller-db/keycloak-db image major bump), create dedicated database dumps before deploying:

```bash
otctl backup dump controller
otctl backup dump keycloak
```

Use the backup workflow from section 2.6 for restore planning.

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

### 7.4 Step 3: Merge new configuration options from package updates

After package upgrade, check whether the package manager kept your existing settings file and wrote an updated template alongside it:

```bash
ls -l /etc/opentalk-compose/settings.yml.dpkg-dist /etc/opentalk-compose/settings.yml.rpmnew
```

If one of these files exists, compare it with your active configuration and merge relevant new options into `/etc/opentalk-compose/settings.yml`.

- Debian/Ubuntu commonly uses `/etc/opentalk-compose/settings.yml.dpkg-dist`
- RPM-based systems commonly use `/etc/opentalk-compose/settings.yml.rpmnew`

Pay special attention to newly introduced storage options. If you are moving between S3 backends (MinIO/Garage), follow the dedicated migration workflow in section 6.3 before or during the upgrade window.

### 7.5 Step 4: Redeploy Services

After upgrading the package, redeploy to apply any new templates, configuration changes, or image updates:

```bash
otctl deploy
```

The deploy is idempotent: unchanged services remain running; changed components are recreated or restarted automatically.

### 7.6 Verification & Rollback Notes

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
