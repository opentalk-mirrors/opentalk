# Advanced Installation Scenarios

## Table of Contents

- [1 Use own TLS certificate instead of Let's Encrypt](#1-use-own-tls-certificate-instead-of-lets-encrypt)
  - [1.1 Stop all OpenTalk related services](#11-stop-all-opentalk-related-services)
  - [1.2 Copy TLS certificate files](#12-copy-tls-certificate-files)
  - [1.3 Edit settings.yml](#13-edit-settingsyml)
  - [1.4 Generate OpenTalk configuration and start the services](#14-generate-opentalk-configuration-and-start-the-services)
- [2 Add Custom CA and custom TLS certificates](#2-add-custom-ca-and-custom-tls-certificates)
  - [2.1 Stop all OpenTalk related services](#21-stop-all-opentalk-related-services)
  - [2.2 Copy TLS certificate files](#22-copy-tls-certificate-files)
  - [2.3 Edit settings.yml](#23-edit-settingsyml)
  - [2.4 Generate OpenTalk configuration and start the services](#24-generate-opentalk-configuration-and-start-the-services)
- [3 Use embedded temporary Root CA and create self-signed TLS certificates](#3-use-embedded-temporary-root-ca-and-create-self-signed-tls-certificates)
  - [3.1 Stop all OpenTalk related services](#31-stop-all-opentalk-related-services)
  - [3.2 Edit settings.yml](#32-edit-settingsyml)
  - [3.3 Generate OpenTalk configuration and start the services](#33-generate-opentalk-configuration-and-start-the-services)
- [4 Configure and integrate an external Keycloak service](#4-configure-and-integrate-an-external-keycloak-service)
  - [4.1 Option 1 - Automated configuration of a remote keycloak instance](#41-option-1---automated-configuration-of-a-remote-keycloak-instance)
    - [4.1.1 Stop all OpenTalk related services](#411-stop-all-opentalk-related-services)
    - [4.1.2 Adjust Keycloak Settings in settings.yml](#412-adjust-keycloak-settings-in-settingsyml)
    - [4.1.3 Generate OpenTalk configuration and start the services](#413-generate-opentalk-configuration-and-start-the-services)
  - [4.2 Option 2 - Manual configuration of a remote keycloak instance](#42-option-2---manual-configuration-of-a-remote-keycloak-instance)
- [5 Configure External Recorder](#5-configure-external-recorder)
  - [5.1 Disable Recorder Service in opentalk-compose](#51-disable-recorder-service-in-opentalk-compose)
    - [5.1.1 Stop all OpenTalk related services](#511-stop-all-opentalk-related-services)
    - [5.1.2 Adjust Docker Compose profile](#512-adjust-docker-compose-profile)
  - [5.2 Expose RabbitMQ AMQP port in settings.yml](#52-expose-rabbitmq-amqp-port-in-settingsyml)
  - [5.3 Set Up an OpenTalk Recorder-only Node](#53-set-up-an-opentalk-recorder-only-node)
    - [5.3.1 Ensure docker engine is installed](#531-ensure-docker-engine-is-installed)
    - [5.3.2 Create the recorder configuration](#532-create-the-recorder-configuration)
    - [5.3.3 Create a Docker Compose file for the recorder service](#533-create-a-docker-compose-file-for-the-recorder-service)
    - [5.3.4 Run Recorder container](#534-run-recorder-container)
- [6 Configure External LiveKit Server](#6-configure-external-livekit-server)
  - [6.1 Install LiveKit Server](#61-install-livekit-server)
  - [6.2 Disable LiveKit in opentalk-compose](#62-disable-livekit-in-opentalk-compose)
    - [6.2.1 Stop all OpenTalk related services](#621-stop-all-opentalk-related-services)
    - [6.2.2 Adjust Docker Compose profile](#622-adjust-docker-compose-profile)
    - [6.2.3 Adjust LiveKit Settings in settings.yml](#623-adjust-livekit-settings-in-settingsyml)
    - [6.2.4 Start OpenTalk Services](#624-start-opentalk-services)

## 1 Use own TLS certificate instead of Let's Encrypt

By default, OpenTalk obtains TLS certificates via Let's Encrypt. If you want to use an existing certificate already signed by a public CA for your domain, copy the fullchain certificate (server plus intermediates) and the private key into `/etc/opentalk-compose/ssl` and reference their file names in `/etc/opentalk-compose/settings.yml`. Ensure the certificate is PEM-encoded (`.crt`) and the private key has restrictive permissions (`chmod 600`).

### 1.1 Stop all OpenTalk related services

Check if OpenTalk Services are running and shut down the stack.

```bash
otctl status
otctl down
```

### 1.2 Copy TLS certificate files

Copy your TLS certificate and key to `/etc/opentalk-compose/ssl/`.

Ensure the following:

- The certificate is a fullchain certificate containing the server certificate and intermediate certificates
- The certificate is PEM-encoded and named with the `.crt` file extensinon.
- The file permission of the key is restrictive (chmod 600)

### 1.3 Edit settings.yml

Edit `/etc/opentalk-compose/settings.yml` and add the configuration below. Replace the example certificate and key file names with your own while keeping the naming conventions described above.

```yaml
custom_certificates_enabled: true
custom_certificates_fullchain_cert_file: fullchain.crt
custom_certificates_private_key_file: privkey.key
```

### 1.4 Generate OpenTalk configuration and start the services

```bash
otctl deploy
```

## 2 Add Custom CA and custom TLS certificates

If you operate your own internal Certification Authority (CA) and want OpenTalk services to trust the certificates it issues, provide the server full-chain certificate (server plus intermediates) and the CA root certificate (and any intermediate CA certificates). A separate root CA certificate file is required so it can be injected into the host system and container trust stores, completing the trust chain.

### 2.1 Stop all OpenTalk related services

Check if OpenTalk Services are running and shut down the stack.

```bash
otctl status
otctl down
```

### 2.2 Copy TLS certificate files

Copy the certificate files to /etc/opentalk-compose/ssl.

Ensure the following:

- The certificate is a fullchain certificate containing the server certificate and intermediate certificates
- The certificate is PEM-encoded and named with the `.crt` file extensinon.
- The CA root certificate is a fullchain certificate containing the server certificate and intermediate certificates
- The CA root certificate is PEM-encoded and named with the `.crt` file extensinon.
- The file permission of the key is restrictive (chmod 600)

### 2.3 Edit settings.yml

Edit `/etc/opentalk-compose/settings.yml` and add the configuration below. Replace the example certificate and key file names with your own while keeping the naming conventions described above.

```yaml
custom_certificates_enabled: true
custom_certificates_fullchain_cert_file: fullchain.crt
custom_certificates_private_key_file: privkey.key
custom_certificates_ca_root_cert_file: ca.crt
```

### 2.4 Generate OpenTalk configuration and start the services

```bash
otctl deploy
```

## 3. Use embedded temporary Root CA and create self-signed TLS certificates

OpenTalk can create a temporary (ephemeral) Root CA and use it to self‑sign TLS certificates for its services. This feature is intended for testing and non‑production scenarios.

**Note:** The generated Root CA certificate must be imported and trusted on all client systems (browser / OS trust store); otherwise browsers will reject the self‑signed service certificates.

Usage: Set `self_signed_certificates_generate: true` in `settings.yml` and specify the filenames for the generated full‑chain certificate, private key, and root CA certificate using the `custom_certificates_* keys` (e.g. fullchain.crt, privkey.key, ca.crt). The embedded temporary Root CA will create these files and they are automatically enabled in the OpenTalk configuration. It is recommended to keep `/etc/opentalk-compose/ssl` empty before proceeding to avoid conflicts with existing certificates.

### 3.1 Stop all OpenTalk related services

Check if OpenTalk Services are running and shut down the stack. Clean `/etc/opentalk-compose/ssl/` directory.

```bash
otctl status
otctl down
rm -rf /etc/opentalk-compose/ssl/*
```

### 3.2 Edit settings.yml

Edit `/etc/opentalk-compose/settings.yml` and add the configuration below.

```yaml
self_signed_certificates_generate: true
custom_certificates_enabled: true
custom_certificates_fullchain_cert_file: fullchain.crt
custom_certificates_private_key_file: privkey.key
custom_certificates_ca_root_cert_file: ca.crt
```

### 3.3 Generate OpenTalk configuration and start the services

```bash
otctl deploy
```

## 4 Configure and integrate an external Keycloak service

If you want to use an existing Keycloak deployment or host Keycloak on a separate server, disable the embedded Keycloak service and configure the external instance. Ensure DNS and network connectivity (HTTPS and required ports and paths) from the OpenTalk host to Keycloak.

The following sections consider two options:

1. Option 1: Automated remote configuration: Use the provided configuration management tooling on the OpenTalk host to bootstrap and configure the remote Keycloak (realm, clients, roles).
2. Option 2: Manual realm configuration by importing an prepared OpenTalk realm export.

Select the approach that fits your operations. Provide valid administrator credentials for setup if you choose the automated option.

### 4.1 Option 1 - Automated configuration of a remote keycloak instance

The OpenTalk Compose package includes automation tools to configure a existing Keycloak instance for OpenTalk. This instance may run locally (default) or remotely as in this scenario.

Configuration is executed via the Keycloak Admin REST API. Ensure the OpenTalk host can reach the Keycloak base URL and required admin endpoints over HTTPS.

It is recommended to use a dedicated realm for OpenTalk. If the configured realm does not exist, it is created automatically. **WARNING**: If you point to an existing realm, OpenTalk-specific clients, roles, and other resources are added to that realm, which might be undesirable.

#### 4.1.1 Stop all OpenTalk related services

Check if OpenTalk Services are running and shut down the stack.

```bash
otctl status
otctl down
```

#### 4.1.2 Adjust Keycloak Settings in settings.yml

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

#### 4.1.3 Generate OpenTalk configuration and start the services

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
otctl deploy --tags demo-user -e postInstall_demoUser_state=absent
```

### 4.2 Option 2 - Manual configuration of a remote keycloak instance

This section describes how to manually integrate an already existing, externally operated Keycloak instance. The embedded (local) Keycloak provided by OpenTalk is disabled and a prepared example realm is imported.

It is assumed that:

- You run your own external Keycloak installation with administrator access.
- Network / DNS allows HTTPS access from the OpenTalk host to the external Keycloak OIDC endpoints.

Stop all OpenTalk related services

Check if OpenTalk Services are running and shut down the stack.

#### 4.2.1 Stop all OpenTalk related services

Check if OpenTalk Services are running and shut down the stack.

```bash
otctl status
otctl down
```

#### 4.2.2 Disable the local (embedded) Keycloak instance

Disable the local Keycloak service and configure the base URL of the external Keycloak service. Extend also the Content Security Policy (CSP) to include the external Keycloak host.

Edit `/etc/opentalk-compose/settings.yml` and set:

```yaml
compose_service_keycloak_enabled: false

# External Keycloak base URL (example – adjust to your environment)
keycloak_base_url: https://keycloak.example.com/auth
keycloak_realm: opentalk

# If the external domain differs from the OpenTalk domain, allow it in CSP
csp_allowed_extra_connect_src: https://keycloak.example.com
```

Save the file but do not start OpenTalk yet.

#### 4.2.3 Prepare the example realm export

The OpenTalk package ships an example realm export with placeholder values located at:

`/usr/share/opentalk-compose/examples/export-opentalk-realm-and-users.json`

Before importing, generate fresh random client secrets and the demo user password using the provided script. Run on the OpenTalk host (pure file operation — no API calls):

```bash
/opt/opentalk-compose/bin/opentalk_keycloak_realm_export_sanitize.sh \
  /usr/share/opentalk-compose/examples/export-opentalk-realm-and-users.json \
  ./opentalk_realm_export.json
```

Result:

- File `opentalk_realm_export.json` contains the sanitized realm export.
- The terminal prints the generated client secrets and the demo password. Copy the relevant secrets for later configuration (Controller / Recorder / Obelisk / Frontend, if required).

You can transfer the generated realm export file (e.g. `opentalk_realm_export.json`) to the external Keycloak instance.

Use `--use-placeholder` to keep `CHANGE_ME` values (documentation only). This is NOT recommended for real deployments.

#### 4.2.4 Import the realm into the external Keycloak

Import the OpenTalk realm export into the external Keycloak. Refer to the official Keycloak documentation for importing a realm from a file: [Keycloak documentation](https://www.keycloak.org/server/importExport#_importing_a_realm_from_a_file).

In the external Keycloak (CLI import recommended):

Prerequisites:

- Sanitized realm export file e.g. `opentalk_realm_export.json` is available in the directory `/opt/keycloak/data/import` on the external Keycloak instance.
- You have admin credentials for the master realm.
- Base URL below uses legacy /auth path (adapt if your Keycloak version omits /auth).

The most common method for the import is, for example:

```bash
bin/kc.sh start --import-realm 
```

#### 4.2.5 Apply secrets to the OpenTalk configuration

Add the secrets printed during sanitization (only those needed by services) into `/etc/opentalk-compose/secrets/secrets.yml`:

```yaml
OIDC_CONTROLLER_CLIENT_SECRET: "<controller-secret-from-export>"
OIDC_RECORDER_CLIENT_SECRET: "<recorder-secret-from-export>"
OIDC_OBELISK_CLIENT_SECRET: "<obelisk-secret-from-export>"
```

If you use additional integrations (e.g. OutlookAddin) copy their secrets likewise.

#### 4.2.6 Start OpenTalk services

Generate configuration and start services (Keycloak will not start locally):

```bash
otctl deploy
```

#### 4.2.7 Validation

1. Check OIDC discovery:

```bash
otctl logs controller --tail 100 | grep -i oidc
```

1. Test browser login – redirect to the external Keycloak must work.
1. If CSP errors occur: inspect response headers and ensure the external domain is present in `csp_allowed_extra_connect_src` (include port if not 443).

#### 4.2.8 Common pitfalls

| Symptom | Cause | Resolution |
|---------|-------|-----------|
| 403 on OIDC discovery | Incorrect `keycloak_base_url` (wrong host) | Fix URL and redeploy. |
| CSP blocking | Domain or port missing in CSP | Add to `csp_allowed_extra_connect_src`. |
| Missing clients after import | Incomplete import or wrong file | Re-sanitize and re-import. |
| Login redirect loop | Realm name or redirect URIs mismatch domain | Align realm name & client redirect URIs. |


## 5 Configure External Recorder

Recording meetings is resource-intensive. Offload to dedicated nodes if needed.

### 5.1 Disable Recorder Service in opentalk-compose

Remove the recorder service from the main stack and run it externally.

#### 5.1.1 Stop all OpenTalk related services

Check if OpenTalk Services are running and shut down the stack.

```bash
otctl status
otctl down
```

#### 5.1.2 Adjust Docker Compose profile

Edit `config/default.env` and exclude the recorder (example):

```bash
COMPOSE_PROFILES=controller,webapp,mailer,pad,spacedeck,keycloak,keycloak-db,minio,traefik,livekit,rabbitmq,controller-db,ofelia
```

### 5.2 Expose RabbitMQ AMQP port in settings.yml

```yaml
RabbitMQ:
  exposed_ports:
    - "10.1.2.3:5672:5672"
```

### 5.3 Set Up an OpenTalk Recorder-only Node

#### 5.3.1 Ensure docker engine is installed

Follow official Docker Engine installation docs.

#### 5.3.2 Create the recorder configuration

`recorder.toml`:

```toml
[controller]
domain = "example.com/controller"
insecure = false

[auth]
issuer = "https://example.com/auth/realms/opentalk"
client_id = "Recorder"
client_secret = "OIDC_RECORDER_CLIENT_SECRET"

[rabbitmq]
uri = "amqp://ot:RABBITMQ_PASSWORD@internal-IP:5672/%2F"
queue = "opentalk_recorder"

# [recorder.hardware_acceleration]
# manufacturer = "intel"
# device = "/dev/dri/renderD129"

clock_format = "%d.%m.%y %X %Z"
```

#### 5.3.3 Create a Docker Compose file for the recorder service

`compose.yml`:

```yaml
services:
  recorder:
    image: registry.opencode.de/opentalk/recorder:v0.15.1
    container_name: opentalk-recorder
    restart: unless-stopped
    volumes:
      - ./recorder.toml:/etc/opentalk/recorder.toml:ro
```

#### 5.3.4 Run Recorder container

```bash
otctl deploy
```

## 6 Configure External LiveKit Server

### 6.1 Install LiveKit Server

Install per official docs. Assume DNS `livekit.example.com`.

### 6.2 Disable LiveKit in opentalk-compose

#### 6.2.1 Stop all OpenTalk related services

Check if OpenTalk Services are running and shut down the stack.

```bash
otctl status
otctl down
```

#### 6.2.2 Adjust Docker Compose profile

Exclude LiveKit:

```bash
COMPOSE_PROFILES=controller,webapp,recorder,mailer,pad,spacedeck,keycloak,keycloak-db,minio,traefik,rabbitmq,controller-db,ofelia
```

#### 6.2.3 Adjust LiveKit Settings in settings.yml

```yaml
Deployment:
  scenario:
    - external_livekit

Controller:
  livekit:
    public_url: "wss://livekit.example.com"
    service_url: "https://livekit.example.com"
    api_key: "controller"
    api_secret: "LIVEKIT_API_SECRET"
```

#### 6.2.4 Start OpenTalk Services

```bash
otctl deploy
```
