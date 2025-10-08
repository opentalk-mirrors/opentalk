# Advanced Installation Scenarios

## Table of Contents

- [1 Use own TLS certificate instead of Let's Encrypt](#1-use-own-tls-certificate-instead-of-lets-encrypt)
  - [1.1 Optional: Stop all OpenTalk related services](#11-optional-stop-all-opentalk-related-services)
  - [1.2 Copy TLS certificate files](#12-copy-tls-certificate-files)
  - [1.3 Edit settings.yml](#13-edit-settingsyml)
  - [1.4 Generate OpenTalk configuration and start the services](#14-generate-opentalk-configuration-and-start-the-services)
- [2 Add Custom CA and custom TLS certificates](#2-add-custom-ca-and-custom-tls-certificates)
  - [2.1 Optional: Stop all OpenTalk related services](#21-optional-stop-all-opentalk-related-services)
  - [2.2 Copy TLS certificate files](#22-copy-tls-certificate-files)
  - [2.3 Edit settings.yml](#23-edit-settingsyml)
  - [2.4 Generate OpenTalk configuration and start the services](#24-generate-opentalk-configuration-and-start-the-services)
- [3 Use embedded temporary Root CA and create self-signed TLS certificates](#3-use-embedded-temporary-root-ca-and-create-self-signed-tls-certificates)
  - [3.1 Optional: Stop all OpenTalk related services](#31-optional-stop-all-opentalk-related-services)
  - [3.2 Edit settings.yml](#32-edit-settingsyml)
  - [3.3 Generate OpenTalk configuration and start the services](#33-generate-opentalk-configuration-and-start-the-services)
- [4 Configure External Keycloak](#4-configure-external-keycloak)
  - [4.1 Optional: Stop all OpenTalk related services](#41-optional-stop-all-opentalk-related-services)
  - [4.2 Adjust Keycloak Settings in settings.yml](#42-adjust-keycloak-settings-in-settingsyml)
  - [4.3 Start OpenTalk](#43-start-opentalk)
- [5 Configure External Recorder](#5-configure-external-recorder)
  - [5.1 Disable Recorder Service in opentalk-compose](#51-disable-recorder-service-in-opentalk-compose)
    - [5.1.1 Optional: Stop all OpenTalk related services](#511-optional-stop-all-opentalk-related-services)
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
    - [6.2.1 Optional: Stop all OpenTalk related services](#621-optional-stop-all-opentalk-related-services)
    - [6.2.2 Adjust Docker Compose profile](#622-adjust-docker-compose-profile)
    - [6.2.3 Adjust LiveKit Settings in settings.yml](#623-adjust-livekit-settings-in-settingsyml)
    - [6.2.4 Start OpenTalk Services](#624-start-opentalk-services)

## 1 Use own TLS certificate instead of Let's Encrypt

By default, OpenTalk obtains TLS certificates via Let's Encrypt. If you want to use an existing certificate already signed by a public CA for your domain, copy the fullchain certificate (server plus intermediates) and the private key into `/etc/opentalk-compose/ssl` and reference their file names in `/etc/opentalk-compose/settings.yml`. Ensure the certificate is PEM-encoded (`.crt`) and the private key has restrictive permissions (`chmod 600`).

### 1.1 Optional: Stop all OpenTalk related services

Check if OpenTalk Services are running and stop the stack.

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

### 2.1 Optional: Stop all OpenTalk related services

Check if OpenTalk Services are running and stop the stack.

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

### 3.1 Optional: Stop all OpenTalk related services

Check if OpenTalk Services are running and stop the stack. Clean `/etc/opentalk-compose/ssl/` directory.

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

## 4 Configure External Keycloak

If you want to use an existing Keycloak instance, disable the embedded one and configure the external instance. Ensure reachability and admin credentials. Prefer a dedicated realm.

### 4.1 Optional: Stop all OpenTalk related services

Check if OpenTalk Services are running and stop the stack.

```bash
otctl status
otctl down
```

### 4.2 Adjust Keycloak Settings in settings.yml

Example:

```yaml
Deployment:
  scenario:
    - external_keycloak

Controller:
  oidc:
    authority: https://keycloak.example.com/auth/realms/opentalk
  frontend:
    authority: https://keycloak.example.com/auth/realms/opentalk
  user_search:
    api_base_url: https://keycloak.example.com/auth/admin/realms/opentalk

Keycloak:
  api_base_url: https://keycloak.example.com/auth/admin/realms/opentalk
  base_url: https://keycloak.example.com/auth
  realm: opentalk
  auth:
    password: admin
    realm: master
    username: admin
  health_url: https://keycloak.example.com/auth/realms/master
  hostname: https://keycloak.example.com/auth

Obelisk:
  auth:
    issuer: https://keycloak.example.com/auth/realms/opentalk

Recorder:
  auth:
    issuer: https://keycloak.example.com/auth/realms/opentalk
```

### 4.3 Start OpenTalk

```bash
otctl deploy
```

Check logs:

```bash
otctl logs keycloak --tail 100 -f
```

## 5 Configure External Recorder

Recording meetings is resource-intensive. Offload to dedicated nodes if needed.

### 5.1 Disable Recorder Service in opentalk-compose

Remove the recorder service from the main stack and run it externally.

#### 5.1.1 Optional: Stop all OpenTalk related services

Check if OpenTalk Services are running and stop the stack.

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

#### 6.2.1 Optional: Stop all OpenTalk related services

Check if OpenTalk Services are running and stop the stack.

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
