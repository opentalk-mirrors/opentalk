<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# Advanced Installation Scenarios

## 1 Certificate handling

### 1.1 Use own TLS certificate instead of Let's Encrypt

By default, OpenTalk obtains TLS certificates via Let's Encrypt. If you want to use an existing certificate already signed by a public CA for your domain, copy the full-chain certificate (server plus intermediates) and the private key into `/etc/opentalk-compose/ssl` and reference their file names in `/etc/opentalk-compose/settings.yml`. Ensure the certificate is PEM‑encoded (`.crt`) and the private key has restrictive permissions (`chmod 600`).

#### 1.1.1 Stop all OpenTalk related services

Check whether OpenTalk services are running, then shut down the stack.

```bash
otctl status
otctl down
```

#### 1.1.2 Copy TLS certificate files

Copy your TLS certificate and key to `/etc/opentalk-compose/ssl/`.

Ensure the following:

- The certificate is a full-chain certificate containing the server certificate and intermediate certificates
- The certificate is PEM-encoded and named with the `.crt` file extension.
- The file permission of the key is restrictive (chmod 600)

#### 1.1.3 Edit settings.yml

Edit `/etc/opentalk-compose/settings.yml` and add the configuration below. Replace the example certificate and key file names with your own while keeping the naming conventions described above.

```yaml
custom_certificates_enabled: true
custom_certificates_fullchain_cert_file: fullchain.crt
custom_certificates_private_key_file: privkey.key
```

#### 1.1.4 Generate OpenTalk configuration and start the services

```bash
otctl deploy
```

### 1.2 Custom CA and custom TLS certificates

If you operate your own internal Certification Authority (CA) and want OpenTalk services to trust the certificates it issues, provide the server full‑chain certificate (server plus intermediates) and the CA root certificate (plus any intermediate CA certificates). A separate root CA certificate file is required so it can be injected into the host system and container trust stores, completing the trust chain.

#### 1.2.1 Stop all OpenTalk related services

Check whether OpenTalk services are running, then shut down the stack.

```bash
otctl status
otctl down
```

#### 1.2.2 Copy TLS certificate files

Copy the certificate files to /etc/opentalk-compose/ssl.

Ensure the following:

- The certificate is a full-chain certificate containing the server certificate and intermediate certificates
- The certificate is PEM-encoded and named with the `.crt` file extension.
- The CA root certificate is a fullchain certificate containing the server certificate and intermediate certificates
- The CA root certificate is PEM-encoded and named with the `.crt` file extension.
- The file permission of the key is restrictive (chmod 600)

#### 1.2.3 Edit settings.yml

Edit `/etc/opentalk-compose/settings.yml` and add the configuration below. Replace the example certificate and key file names with your own while keeping the naming conventions described above.

```yaml
custom_certificates_enabled: true
custom_certificates_fullchain_cert_file: fullchain.crt
custom_certificates_private_key_file: privkey.key
custom_certificates_ca_root_cert_file: ca.crt
```

#### 1.2.4 Generate OpenTalk configuration and start the services

```bash
otctl deploy
```

> **Note:** Keycloak is a Java application and uses the `KC_TRUSTSTORE_PATHS` environment variable to add the custom CA to its JVM truststore. This is required for all outbound TLS connections from Keycloak, including Identity Provider (IdP) federation connections to other Keycloak instances secured with this CA. The system CA store update (`update-ca-certificates`) alone is not sufficient for Java applications, which maintain their own trust chain independently.

### 1.3. Embedded Root CA and self-signed certificate creation

OpenTalk can create a temporary (ephemeral) Root CA and use it to self‑sign TLS certificates for its services. This feature is intended for testing and non‑production scenarios.

> **Note:** The generated Root CA certificate must be imported and trusted on all client systems (browser / OS trust store)
> otherwise browsers will reject the self‑signed service certificates. Set `self_signed_certificates_generate: true` in
> `settings.yml` and specify the filenames for the generated full‑chain certificate, private key, and root CA certificate
> using the `custom_certificates_*` keys (e.g. `fullchain.crt`, `privkey.key`, `ca.crt`). The embedded temporary Root CA will
> create these files and they are automatically enabled in the OpenTalk configuration. It is recommended to keep
> `/etc/opentalk-compose/ssl` empty before proceeding to avoid conflicts with existing certificates.

#### 1.3.1 Stop all OpenTalk related services

Check whether OpenTalk services are running, then shut down the stack. Clean `/etc/opentalk-compose/ssl/` directory.

```bash
otctl status
otctl down
rm -rf /etc/opentalk-compose/ssl/*
```

#### 1.3.2 Edit settings.yml

Edit `/etc/opentalk-compose/settings.yml` and add the configuration below.

```yaml
self_signed_certificates_generate: true
custom_certificates_enabled: true
custom_certificates_fullchain_cert_file: fullchain.crt
custom_certificates_private_key_file: privkey.key
custom_certificates_ca_root_cert_file: ca.crt
```

#### 1.3.3 Generate OpenTalk configuration and start the services

```bash
otctl deploy
```

## 2 External Keycloak integration

If you want to use an existing Keycloak deployment or host Keycloak on a separate server, disable the embedded Keycloak service and configure the external instance. Ensure DNS and network connectivity (HTTPS and required ports and paths) from the OpenTalk host to Keycloak.

The following sections consider two options:

1. Option 1: Automated remote configuration: Use the provided configuration management tooling on the OpenTalk host to bootstrap and configure the remote Keycloak (realm, clients, roles).
2. Option 2: Manual realm configuration by importing a prepared OpenTalk realm export.

Select the approach that fits your operations. Provide valid administrator credentials for setup if you choose the automated option.

### 2.1 Automated configuration of a remote Keycloak instance

The OpenTalk Compose package includes automation tools to configure an OpenTalk realm in an existing Keycloak instance. This instance may run locally (default) or remotely as in this scenario.

Configuration is executed via the Keycloak Admin REST API. Ensure the OpenTalk host can reach the Keycloak base URL and required admin endpoints over HTTPS.

It is recommended to use a dedicated realm for OpenTalk. If the configured realm does not exist, it is created automatically.

> **Warning:** If you point to an existing realm, OpenTalk-specific clients, roles, and other resources are added to that realm, which might be undesirable.

> **Note:** If the external Keycloak is secured with a certificate issued by a custom CA, also configure `custom_certificates_ca_root_cert_file` in `settings.yml` (see [Section 1.2](#12-custom-ca-and-custom-tls-certificates)). This ensures all local OpenTalk services trust the custom CA for outbound HTTPS connections to Keycloak.

#### 2.1.1 Stop all OpenTalk related services

Check whether OpenTalk services are running, then shut down the stack.

```bash
otctl status
otctl down
```

#### 2.1.2 Adjust Keycloak Settings in settings.yml

Edit `/etc/opentalk-compose/settings.yml` and add the configuration below. The following example assumes the external Keycloak is reachable at `https://keycloak.example.com/auth`; replace the host with your own. The local Keycloak service will be disabled. Traefik sets Content Security Policies (CSP) by default. If the external Keycloak runs under a different domain, add that domain to the CSP (for example via `csp_allowed_extra_connect_src`).

```yaml
compose_service_keycloak_enabled: false

keycloak_base_url: https://keycloak.example.com/auth
keycloak_realm: opentalk
keycloak_admin_username: admin
keycloak_admin_password: *****

csp_allowed_extra_connect_src: https://keycloak.example.com
```

If you have a brand‑new vanilla Keycloak instance and you are still using the initial bootstrap admin user, you can **additionally** set `keycloak_bootstrap_admin_username` and `keycloak_bootstrap_admin_password`. The tooling will first authenticate against the Keycloak API using these bootstrap credentials, **create** the permanent admin account defined by `keycloak_admin_username` and `keycloak_admin_password`, then remove the bootstrap admin user.

```yaml
compose_service_keycloak_enabled: false

keycloak_base_url: https://keycloak.example.com/auth
keycloak_realm: opentalk
keycloak_admin_username: admin
keycloak_admin_password: *****
keycloak_bootstrap_admin_username: bootstrap_admin
keycloak_bootstrap_admin_password: ******

csp_allowed_extra_connect_src: https://keycloak.example.com
```

#### 2.1.3 Generate OpenTalk configuration and start the services

Execute the deployment. The tasks will configure the external Keycloak and local services and then start them.

```bash
otctl deploy
```

For testing you can provision a demo user by running the following task:

```bash
otctl deploy -t demo-user
```

To remove the demo user, run this command:

```bash
otctl deploy --tags demo-user -e keycloak_demo_user_state=absent
```

### 2.2 Option 2 - Manual configuration of a remote Keycloak instance

This section describes how to manually integrate an already existing, externally operated Keycloak instance. The embedded (local) Keycloak provided by OpenTalk will be disabled and a `JSON` file will be prepared to import.

> Minimum Keycloak Version: 24.3
>
> The provided example realm export and recommended client / role configuration assume Keycloak 24.3 or newer. If your external instance is older, upgrade Keycloak before importing the realm to avoid incompatibilities (client protocol mappers, admin console expectations, or OIDC endpoints).

It is assumed that:

- You run your own external Keycloak installation with administrator access.
- Network / DNS allows HTTPS access from the OpenTalk host to the external Keycloak OIDC endpoints.

> **Note:** If the external Keycloak is secured with a certificate issued by a custom CA, also configure `custom_certificates_ca_root_cert_file` in `settings.yml` (see [Section 1.2](#12-custom-ca-and-custom-tls-certificates)). This ensures all local OpenTalk services trust the custom CA for outbound HTTPS connections to Keycloak.

Stop all OpenTalk related services

Check if OpenTalk Services are running and shut down the stack.

#### 2.2.1 Stop all OpenTalk related services

Check whether OpenTalk services are running, then shut down the stack.

```bash
otctl status
otctl down
```

#### 2.2.2 Disable the local (embedded) Keycloak instance

Disable the local Keycloak service and configure the base URL of the external Keycloak service. Extend also the Content Security Policy (CSP) to include the external Keycloak host.

Edit `/etc/opentalk-compose/settings.yml` and set:

```yaml
compose_service_keycloak_enabled: false
keycloak_manage_enabled: false

# External Keycloak base URL (example – adjust to your environment)
keycloak_base_url: https://keycloak.example.com/auth
keycloak_realm: opentalk

# If the external domain differs from the OpenTalk domain, allow it in CSP
csp_allowed_extra_connect_src: https://keycloak.example.com
```

Save the file but do not start OpenTalk yet.

#### 2.2.3 Prepare the OpenTalk realm export

The OpenTalk package ships an example realm export with placeholder values located at:

`/usr/share/opentalk-compose/examples/export-opentalk-realm-and-users.json`

Before importing, generate fresh random client secrets using the provided script. Run on the OpenTalk host (pure file operation — no API calls). You also need to provide your OpenTalk frontend URL to set the redirect URLs for the frontend OIDC client.

```bash
/opt/opentalk-compose/bin/opentalk_keycloak_realm_export_sanitize.sh --frontend-url https://example.com \
  /usr/share/opentalk-compose/examples/export-opentalk-realm-and-users.json \
  ./opentalk_realm_export.json
```

Result:

- File `opentalk_realm_export.json` contains the sanitized realm export with new secrets and your frontend URL.
- The terminal prints the generated client secrets and the demo password. Copy the relevant secrets for later configuration (Controller / Recorder / Obelisk / Frontend / etc, if required).

You can transfer the generated realm export file (e.g. `opentalk_realm_export.json`) to the external Keycloak instance.

#### 2.2.4 Import the realm into the external Keycloak

Import the OpenTalk realm export into the external Keycloak. Refer to the official Keycloak documentation for importing a realm from a file: [Keycloak documentation](https://www.keycloak.org/server/importExport#_importing_a_realm_from_a_file).

In the external Keycloak (CLI import recommended):

Prerequisites:

- Sanitized realm export file e.g. `opentalk_realm_export.json` is available in the directory `/opt/keycloak/data/import` on the external Keycloak instance.
- You have admin credentials for the master realm.

A common method for the import is, for example:

```bash
bin/kc.sh start --import-realm
```

#### 2.2.5 Apply generated secrets to the OpenTalk secrets.yml file

Add the secrets printed during sanitization (only those needed by services) into `/etc/opentalk-compose/secrets/secrets.yml`:

```yaml
OIDC_CONTROLLER_CLIENT_SECRET: "<controller-secret-from-export>"
OIDC_RECORDER_CLIENT_SECRET: "<recorder-secret-from-export>"
OIDC_OBELISK_CLIENT_SECRET: "<obelisk-secret-from-export>"
...
```

If you use additional integrations (e.g. OutlookAddin) copy their secrets likewise.

#### 2.2.6 Start OpenTalk services

Generate configuration and start services (Keycloak will not start locally):

```bash
otctl deploy
```

#### 2.2.7 Common pitfalls

| Symptom | Cause | Resolution |
|---------|-------|-----------|
| 403 on OIDC discovery | Incorrect `keycloak_base_url` (wrong host) | Fix URL and redeploy. |
| CSP blocking | Domain or port missing in CSP | Add to `csp_allowed_extra_connect_src`. |
| Missing clients after import | Incomplete import or wrong file | Re-sanitize and re-import. |
| Login redirect loop | Realm name or redirect URIs mismatch domain | Align realm name & client redirect URIs. |

## 3 External OpenTalk Recorder-only nodes

Recording meetings can be resource intensive. Offloading the Recorder service to a dedicated host is appropriate if recordings are frequent or occur in parallel. Ideally that host provides compatible GPUs to enable hardware acceleration (see <https://docs.opentalk.eu/admin/recorder/hardware_acceleration/>). Some security precautions are required: the external Recorder host must reach RabbitMQ on port `5672/tcp`, which is currently only possible unencrypted. Ensure the RabbitMQ port is not publicly exposed (bind to a private interface or restrict via firewall). The Recorder should communicate with RabbitMQ over a private, trusted network. Confirm the necessary network infrastructure and firewall rules are in place before proceeding. See also the configuration reference for relevant variables: [Recorder service options](./5-configuration-options-reference.md#opentalk_composeopentalk_recorder) and [RabbitMQ options](./5-configuration-options-reference.md#opentalk_composerabbitmq).

### 3.1 Prepare the OpenTalk server configuration

Disable the recorder service from the OpenTalk Compose stack and adjust the configuration on the main server.

#### 3.1.1 Stop all OpenTalk related services

Check whether OpenTalk services are running, then shut down the stack.

```bash
otctl status
otctl down
```

#### 3.1.2 Edit settings.yml

Edit and add the following settings to `/etc/opentalk-compose/settings.yml`.

```yaml
# Disable the local Recorder service
compose_service_opentalk_recorder_enabled: false

# Expose RabbitMQ AMQP port ONLY on a private/internal IP (never 0.0.0.0)
rabbitmq_docker_ports:
  - "PRIVATE-IP:5672:5672"

# The Controller exposes the LiveKit API; set its service URL using value from ot_domain
controller_livekit_service_url: "https://{{ ot_domain }}/livekit"
```

#### 3.1.3 Deploy configuration changes on the OpenTalk server

```bash
otctl deploy
```

### 3.2 Set Up an OpenTalk Recorder-only Node

An OpenTalk recorder-only node can be provisioned easily as a standalone Docker Compose service. The steps below outline the procedure.

#### Hardware acceleration (optional)

> **Note:** If you have a compatible integrated GPU (e.g. Intel iGPU with VA-API) or dedicated Intel Arc graphics card, you can enable hardware acceleration by uncommenting the settings below. Specifically, the Intel Arc A310 Eco and Intel Arc A770 models have been tested for hardware acceleration. Ensure the appropriate drivers and device passthrough (e.g. `/dev/dri`) are exposed in the container before enabling the option in the OpenTalk Recorder configuration. Find more information about the [Recorder and hardware acceleration](https://docs.opentalk.eu/admin/recorder/hardware_acceleration/) in the OpenTalk documentation.

#### 3.2.1 Ensure Docker Engine is installed on the external Recorder host

Follow the official [Docker Engine installation](https://docs.docker.com/engine/install/) docs to install `docker` including the `compose` plugin on the recorder node.

#### 3.2.2 Docker Compose configuration for the external Recorder service

Create a directory for the Recorder's Docker Compose and configuration file e.g.:

```bash
mkdir /opt/{opentalk-recorder,opentalk-recorder/config}
```

Create a `compose.yml` file in `/opt/opentalk-recorder/`:

> **Note:** Adjust the OpenTalk Recorder image tag version to match the installed OpenTalk product version if necessary. Published versions are available at <https://docs.opentalk.eu/releases>

```yaml
services:
  recorder:
    image: registry.heinlein.group/opentalk/recorder
    container_name: opentalk-recorder
    restart: always
    environment:
      RUST_LOG: info
    #devices:
    #  - "/dev/dri:/dev/dri"
    #group_add:
    #  - "video"
    #cap_add:
    #  - CAP_PERFMON
    volumes:
      - ./config/recorder.toml:/etc/opentalk/recorder.toml:ro
```

#### 3.2.3 OpenTalk Recorder Service configuration

Create a `recorder.toml` file under `/opt/opentalk-recorder/config/` with the content below. Replace `example.com` with your OpenTalk domain (value from `ot_domain`) and substitute the placeholder secrets (`OIDC_RECORDER_CLIENT_SECRET`, `RABBITMQ_PASSWORD`, `PRIVATE-IP`) with the actual values. Find the secrets in `/etc/opentalk-compose/secrets/secrets.yml` on the OpenTalk Compose host.

```toml
[controller]
domain = "example.com/controller"
insecure = false

[auth]
issuer = "https://example.com/auth/realms/opentalk"
client_id = "Recorder"
client_secret = "OIDC_RECORDER_CLIENT_SECRET"

[rabbitmq]
uri = "amqp://ot:RABBITMQ_PASSWORD@PRIVATE-IP:5672/%2F"
queue = "opentalk_recorder"

# [recorder.hardware_acceleration]
# manufacturer = "intel"
# device = "/dev/dri/renderD129"

clock_format = "%d.%m.%y %X %Z"
```

#### 3.2.4 Run Recorder container

```bash
cd /opt/opentalk-recorder && docker compose up -d
```

Check the Recorder log. The log message shows whether the connection to RabbitMQ was established successfully.

```txt
:/opt/opentalk-recorder # docker compose logs recorder -f
opentalk-recorder  | [2025-10-01T08:00:00Z INFO  opentalk_recorder] RabbitMQ connection established successfully
```

Recordings should now occur on the external Recorder node.

## 4 External LiveKit Server integration

The following instructions assume you already have a running LiveKit server installation. To install an LiveKit, follow the official self-hosting guide: [https://docs.livekit.io/home/self-hosting/vm/](https://docs.livekit.io/home/self-hosting/vm/). The examples below assume the public DNS names `livekit.example.com` and `livekit-turn.example.com` resolves to your LiveKit host. Replace these hostnames with your own everywhere it appears.
Also ensure that your firewall configuration allows the required LiveKit Server ports and protocols (signaling, TURN, media). For details, see: <https://docs.livekit.io/home/self-hosting/ports-firewall/>

### 4.1 Gather information about the external LiveKit Server

Before integrating an external LiveKit server with OpenTalk, collect the following details from your LiveKit installation:

- **Public WebSocket URL** (e.g., `wss://livekit.example.com`)
- **Public Service URL** (e.g., `https://livekit.example.com`)
- **TURN/TLS address** (e.g., `livekit-turn.example.com:443`)
- **TURN/UDP address** (e.g., `livekit-turn.example.com:3478`)
- **API Key** (generated during LiveKit setup)
- **API Secret** (generated during LiveKit setup)

You will need these values to configure OpenTalk for external LiveKit integration. Adjust the examples in the following steps to match your environment.

### 4.2 Replace the embedded LiveKit server with an external one

The following steps describe how to disable the embedded LiveKit server and apply the necessary configuration changes to integrate an external LiveKit server with OpenTalk.

#### 4.2.1 Stop all OpenTalk related services

Check if OpenTalk Services are running and shut down the stack.

```bash
otctl status
otctl down
```

#### 4.2.2 Adjust the LiveKit related configuration in settings.yml

Edit `/etc/opentalk-compose/settings.yml` and update or add the following configurations

```yaml
# Disable the local LiveKit service
compose_service_livekit_server_enabled: false

# Set the public and service URLs for the external LiveKit server
controller_livekit_public_url: wss://livekit.example.com
controller_livekit_service_url: https://livekit.example.com

# Fill in the API key and secret with the values generated during the LiveKit installation
controller_livekit_api_key: "generated key"
controller_livekit_api_secret: "generated secret"

# Add the WebSocket address to the Content Security Policy Content Security Policy (CSP)
csp_allowed_extra_connect_src: "wss://livekit.example.com"
```

#### 4.2.3 Deploy changes and start OpenTalk Services

```bash
otctl deploy
```

After applying the configuration changes, the external LiveKit server should be integrated. Log in to the OpenTalk web frontend to test WebRTC functionality.

## 5. External TURN and STUN Server integration (Coturn)

### 5.1 Overview

A STUN and TURN server is essential for reliable WebRTC communication, especially for users behind NAT or restrictive firewalls. The open-source Coturn server provides these functionality. Consider switching to a separate Coturn installation if:

- The embedded LiveKit TURN server no longer meets your requirements.
- You need a dedicated server(s) for capacity or scaling.
- You want to self-host a STUN server instead of using a public one.
- You require greater configuration flexibility.

This section explains how to integrate an external Coturn server into OpenTalk. It assumes Coturn is already installed and configured according to the official documentation (<https://github.com/coturn/coturn>) and is reachable at the domain `turn.example.com`. Replace this domain with your own as needed.

Before proceeding, ensure:

- A DNS A/AAAA record `turn.example.com` exists and points to the public IP of the TURN/STUN server.
- A TLS certificate and private key for the `turn.example.com` domain are available.
- Open port 3478/udp and 443/tcp for TLS on your firewall.
- Ensure the TURN server is reachable from your LiveKit server and clients.
- Ensure LiveKit RTC port range is reachable from the TURN server (OpenTalk Compose default: 50000-65535/udp).

### 5.2 Minimal Coturn configuration

Configure the following settings on the Coturn server so LiveKit can authenticate successfully.

Edit `/etc/turnserver.conf` and set at least the following options (adjust values as required):

```ini
listening-port=3478
tls-listening-port=443
realm=opentalk
fingerprint
lt-cred-mech
userdb=/var/lib/turn/turndb
cert=/path/to/certs/fullchain.pem
pkey=/path/to/certs/privkey.pem
no-rfc5780
no-stun-backward-compatibility
response-origin-only-with-rfc5780

```

Restart coturn after editing:

```bash
systemctl restart coturn
systemctl enable coturn
```

Create a user for the TURN service and generate a random secret. It will be required below for the LiveKit configuration.

```bash
turnadmin -a --db /var/lib/turn/turndb -u livekit -r opentalk -p <secret>
```

> **Note:** You might see a message that appears misleading; the user is still created successfully.
>
> ```txt
> 0: : SQLite connection was closed.
> 0: : log file opened: /var/log/turn_1969035_2025-11-05.log
> ```
>
> You can verify that the user exists with the command `turnadmin -a --db /var/lib/turn/turndb -l`.

### 5.3 Add Coturn to the LiveKit Configuration

On the OpenTalk Compose host, edit `/etc/opentalk-compose/settings.yml` and add the configuration for the Coturn host:

```yaml
livekit_turn_embedded_enabled: false
livekit_rtc_stun_servers:
  - turn.example.com:443
livekit_rtc_turn_servers:
   - host: turn.example.com
     port: 443
     protocol: tls
     username: livekit
     credential: "<secret>"
   - host: turn.example.com
     port: 3478
     protocol: udp
     username: livekit
     credential: "<secret>"
```

#### 5.3.1 Deploy changes and start OpenTalk Services

```bash
otctl deploy
```

## 6 S3 Storage Provider

OpenTalk uses an S3-compatible object storage service for files such as recordings and PDF documents. With opentalk-compose 26.1.0 Garage is the default storage provider. 

Two providers are supported:

| Provider | Description |
|----------|-------------|
| **Garage** (default) | Lightweight, self-contained S3-compatible storage. No web UI — administration via CLI. |
| **MinIO** | object storage with a web console at `/adm/minio/`. MinIO is no longer maintained and the the repository is now officially archived. Currently, Minio support is maintained solely for backward compatibility |

Normally only one provider is active at a time, selected via the `s3_storage_provider` variable in `settings.yml`. During a migration both providers can run in parallel — see [Section 6.3](#63-migrating-between-minio-and-garage).

### 6.1 Choosing a provider

New installations default to **Garage** — no additional configuration is required.

To use **MinIO** instead, set:

```yaml
s3_storage_provider: minio
```

Then deploy:

```bash
otctl deploy
```

> **Backward compatibility:** Existing deployments that have `compose_service_minio_enabled: true` in their `settings.yml` will automatically continue using MinIO. You do not need to change your configuration; however, it is recommended to migrate to the new `s3_storage_provider` variable.

### 6.2 Garage administration

Garage does not provide a web UI. Use the Garage CLI inside the running container instead:

```bash
# List buckets
otctl compose exec garage /garage bucket list

# List API keys
otctl compose exec garage /garage key list

# Show key details
otctl compose exec garage /garage key info opentalk-key

# Show cluster status
otctl compose exec garage /garage status

# Show cluster layout
otctl compose exec garage /garage layout show
```

For more information, refer to the [Garage documentation](https://garagehq.deuxfleurs.fr/documentation/quick-start/).

### 6.3 Migrating between MinIO and Garage

You can migrate S3 data in either direction — MinIO → Garage or Garage → MinIO — using the `otctl migrate` command. The command automates the full procedure: it installs `rclone` if needed, copies all objects between the two backends, updates `settings.yml`, and optionally runs a deployment to stop the source service.

> **Important:** Back up your data before starting the migration.
> ```bash
> otctl backup
> ```

#### 6.3.1 Enable both providers in parallel
For a successful migration, both S3 providers must be running at the same time. Add the following to `/etc/opentalk-compose/settings.yml` and deploy:

**MinIO → Garage** (Garage is not yet running):
```yaml
compose_service_minio_enabled: true
compose_service_garage_enabled: true
```

**Garage → MinIO** (MinIO is not yet running):
```yaml
compose_service_garage_enabled: true
compose_service_minio_enabled: true
```

Then run `otctl deploy` to start both services. A warning will appear at the end of the deployment indicating that both providers are active and a migration should be performed.

> **Note:** `s3_storage_provider` controls which S3 endpoint the controller uses. Leave it pointing at the current active provider until the migration is complete.

#### 6.3.2 Run the migration

```bash
# Migrate from MinIO to Garage:
otctl migrate minio-to-garage

# Migrate from Garage to MinIO:
otctl migrate garage-to-minio
```

The command will:

1. Install `rclone` on the host if not already present (via `apt` or `dnf`).
2. Verify that both containers are running and healthy.
3. Copy all objects from the source bucket to the destination bucket.
4. Verify that the object counts match.
5. Update `settings.yml`: set `s3_storage_provider` to the destination and set `compose_service_<source>_enabled: false`.
6. Run `otctl deploy` automatically to stop the source service and reconfigure the controller.

To skip the automatic deploy after migration (e.g. to verify manually first):

```bash
otctl migrate minio-to-garage --no-deploy
```

#### 6.3.3 Verify

After migration, confirm the destination service is healthy and the controller is using the new backend:

```bash
# For Garage as destination:
otctl compose exec garage /garage bucket info opentalk

# For MinIO as destination:
otctl compose exec minio mc ls local/opentalk
```

#### 6.3.4 Clean up (optional)

Once you have verified the migration, remove the old docker volumes to free disk space:

```bash
# After migrating away from MinIO:
otctl volumes rm opentalk_minio

# After migrating away from Garage:
otctl volumes rm opentalk_garage_data opentalk_garage_meta
```
