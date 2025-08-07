# Advanced Installation Scenarios

## Table of Contents

- [Use own TLS certificate instead of Let's Encrypt](#use-own-tls-certificate-instead-of-lets-encrypt)
  - [1. Stop all OpenTalk related services](#1-stop-all-opentalk-related-services)
  - [2. Copy TLS certificate files to /opt/opentalk-compose/config/ssl](#2-copy-tls-certificate-files-to-opt-opentalk-compose-config-ssl)
  - [3. Edit /opt/opentalk-compose/config/settings.yml](#3-edit-opt-opentalk-compose-config-settingsyml)
  - [4. Start OpenTalk](#4-start-opentalk)
- [Add Custom CA and custom TLS certificates](#add-custom-ca-and-custom-tls-certificates)
  - [1. Stop all OpenTalk related services](#1-stop-all-opentalk-related-services-1)
  - [2. Copy TLS certificate files to /opt/opentalk-compose/config/ssl](#2-copy-tls-certificate-files-to-opt-opentalk-compose-config-ssl-1)
  - [3. Edit /opt/opentalk-compose/config/settings.yml](#3-edit-opt-opentalk-compose-config-settingsyml-1)
  - [4. Initialize your OpenTalk Setup again](#4-initialize-your-opentalk-setup-again)
  - [5. Start OpenTalk](#5-start-opentalk)
- [Configure External Keycloak](#configure-external-keycloak)
  - [1. Stop all services](#1-stop-all-services)
  - [2. Adjust Keycloak Settings in settings.yml](#2-adjust-keycloak-settings-in-settingsyml)
  - [3. Start OpenTalk](#3-start-opentalk)
- [Configure External Recorder](#configure-external-recorder)
  - [1. Disable Recorder Service in opentalk-compose](#1-disable-recorder-service-in-opentalk-compose)
    - [1.1 Stop all services](#11-stop-all-services)
    - [1.2 Adjust the Docker Compose profile in config/default.env](#12-adjust-the-docker-compose-profile-in-config-defaultenv)
  - [2. Expose RabbitMQ AMQP port in settings.yml](#2-expose-rabbitmq-amqp-port-in-settingsyml)
  - [3. Set Up an OpenTalk Recorder-only Node](#3-set-up-an-opentalk-recorder-only-node)
    - [3.1 Ensure docker engine is installed](#31-ensure-docker-engine-is-installed)
    - [3.2 Create the recorder configuration](#32-create-the-recorder-configuration)
    - [3.3 Create a Docker Compose file for the recorder service](#33-create-a-docker-compose-file-for-the-recorder-service)
    - [3.4 Run Recorder container](#34-run-recorder-container)
- [Configure External LiveKit Server](#configure-external-livekit-server)
  - [1. Install LiveKit Server](#1-install-livekit-server)
  - [2. Disable LiveKit in opentalk-compose](#2-disable-livekit-in-opentalk-compose)
    - [2.1 Stop all services](#21-stop-all-services)
    - [2.2 Adjust the Docker Compose profile in config/default.env](#22-adjust-the-docker-compose-profile-in-config-defaultenv)
    - [2.3 Adjust LiveKit Settings in settings.yml](#23-adjust-livekit-settings-in-settingsyml)
    - [2.4 Start OpenTalk Services](#24-start-opentalk-services)

## Use own TLS certificate instead of Let's Encrypt

If you do not want to use Let's Encrypt and already have a certificate for your DNS domain that was signed by an official CA, place the certificate files in the directory `/opt/opentalk-compose/config/ssl` and configure them in the `settings.yml` file.

### 1. Stop all OpenTalk related services

Check if Docker Compose services are running in the OpenTalk context and stop them.

### 2. Copy TLS certificate files to /opt/opentalk-compose/config/ssl

- server fullchain certifikate
- server TLS certificate key

### 3. Edit `/opt/opentalk-compose/config/settings.yml`

Edit the `settings.yml` file and add the following configuration block, replacing the certificate and key file names with your own. Make sure to keep correct YAML indentation and syntax.

```yaml
CustomCertificates:
  enabled: true
  ssl_certificate_fullchain_file: my-server_fullchain.crt
  ssl_certificate_key_file: my-server.key
```

### 4. Start OpenTalk

```bash
cd /opt/opentalk-compose && docker compose up -d
```

## Add Custom CA and custom TLS certificates

If you operate your own certification authority (CA) and want your services to use it, the challenge is to make your custom CA available inside the containers and accessible to the services so that it is recognized as trusted. Start by following the steps in [Add TLS certificate](#add-tls-certificate), but also ensure that the root certificate of your CA is placed in the `/opt/opentalk-compose/config/` directory.

### 1. Stop all OpenTalk related services

Check if Docker Compose services are running in the OpenTalk context and stop them.

### 2. Copy TLS certificate files to /opt/opentalk-compose/config/ssl

- server fullchain certifikate
- server TLS certificate key
- RootCA TLS certificate

### 3. Edit `/opt/opentalk-compose/config/settings.yml`

Edit the `settings.yml` file and add the following configuration block, replacing the certificate and key file names with your own. Make sure to keep correct YAML indentation and syntax.

```yaml
CustomCA:
  enabled: false
  ssl_ca_file: my-ca.crt
CustomCertificates:
  enabled: true
  ssl_ca_file: my-ca.crt
  ssl_certificate_fullchain_file: my-server_fullchain.crt
  ssl_certificate_key_file: my-server.key
```

### 4. Initialize your OpenTalk Setup again

Damit die zertifikate und key beim starten der container verfügbar sind muss die docker compose configuration angepasst werden. Dies wird über die override files in config/service-config/<service name> erreicht. Diese anpssung erfolgt in der initialisierungs phase. 

```bash
cforge init
```

### 5. Start OpenTalk

```bash
cd /opt/opentalk-compose && docker compose up -d
```

## Configure External Keycloak

If you want to use an existing Keycloak instance for the OpenTalk installation, proceed as follows. In this case, the Keycloak service in `opentalk-compose` is disabled and the external instance is configured instead. For this step, ensure that the external Keycloak instance is reachable by the OpenTalk server and an account with administrator privileges is available. Decide in advance whether you want to use an existing realm for OpenTalk or create a new realm in the external Keycloak during the setup process.

Because realm settings are adjusted to OpenTalk’s requirements during setup, it is recommended to use a dedicated realm for OpenTalk or let the setup process create one. If you intend to use an existing realm that is also used by other applications outside the OpenTalk context, we recommend disabling Keycloak management in opentalk-compose and managing the settings and resources required by OpenTalk manually or via your organization’s configuration management.

### 1. Stop all services

If OpenTalk is already running and you want to switch Keycloak to an external instance, stop all services first. For a new installation, you can configure the external Keycloak after initialization, see [2.5 Edit Settings in settings.yml](./1-quickstart-installation.md#25-optional-edit-settings-in-settingsyml) in the Quickstart Guide and skip this step.

```bash
cd /opt/opentalk-compose && docker compose down
```

### 2. Adjust Keycloak Settings in `settings.yml`

The following configuration shows a typical example of integrating an existing Keycloak into the OpenTalk setup. Your infrastructure may differ from this example; see the [Configuration Option Reference](./5-configuration-options-reference.md#keycloak) to select the appropriate parameters for your environment and to validate or override the defaults as needed.

In the example below, we assume the following:

- The external Keycloak is reachable at the DNS domain `keycloak.example.com`.
- A new realm named `opentalk` will be created in the external Keycloak for OpenTalk.
- The Keycloak `master` realm is available and contains an administrator account named `admin`. This account is required to create the new `opentalk` realm.
- The Keycloak base URL is `https://keycloak.example.com/auth`.

**Add to `settings.yml` or override the defaults:**

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

### 3. Start OpenTalk

On startup, cforge runs and creates or updates the Keycloak realm, the required OIDC clients, and other settings needed by OpenTalk.

```bash
cd /opt/opentalk-compose && docker compose up -d
```

You can check the opentalk-compose configuration management:

```bash
cd /opt/opentalk-compose && docker logs cforge --tail 100 -f 
```

## Configure External Recorder

Recording meetings is resource-intensive for a typical server. If recordings are frequent, consider running dedicated systems with a suitable GPU to enable hardware acceleration. 

> Caution: A separate recorder server must communicate with the controller over a trusted private network. Currently, the controller only supports the AMQP protocol for communication with RabbitMQ, which provides no transport encryption. For this purpose, a dedicated private IP address should be provisioned.

The following describes how to set up an external recording server and integrate it into your OpenTalk Installation.

### 1. Disable Recorder Service in `opentalk-compose`

### 1.1 Stop all services

If OpenTalk is already running and you want to disable the Recorder service, stop all services first. If you are in the initialization phase, you can skip this step.

```bash
cd /opt/opentalk-compose && docker compose down
```

### 1.2 Adjust the Docker Compose profile in `config/default.env`

If not already done, follow the OpenTalk [installation procedure](1-quickstart-installation.md#2-installation-procedure) from the Quickstart Guide up to [2.3 Set the Server's DNS Domain Record](./1-quickstart-installation.md#23-set-the-servers-dns-domain-record), and additionally adjust the `COMPOSE_PROFILES` environment variable.

By default, the combination of services to start is controlled by the values defined in the `COMPOSE_PROFILES` variable. There are predefined service collections named `standard`, `core`, and `full`. See [Docker Compose profiles](./3-docker-compose-profiles.md) to choose the profiles that fit your needs.

Alternatively, explicitly list all services you want to start and exclude the recorder, for example:

**config/default.env e.g.:**

```bash
COMPOSE_PROFILES=controller,webapp,mailer,pad,spacedeck,keycloak,keycloak-db,minio,traefik,livekit,rabbitmq,controller-db,ofelia
```

### 2. Expose RabbitMQ AMQP port in  `settings.yml`

Replace the IP address with the appropriate host IP for your environment in `/opt/opentalk-compose/config/settings.yml`. Consider that maybe a firewall needs to allow inbound TCP 5672 from the recorder node(s).

```yaml
RabbitMQ:
  exposed_ports:
    - "10.1.2.3:5672:5672"
``` 

Then continue the [installation procedure](1-quickstart-installation.md#2-installation-procedure) as usual. If OpenTalk is already installed, it is sufficient to just start the application stack.

### 3. Set Up an OpenTalk Recorder-only Node

#### 3.1 Ensure docker engine is installed

Install the Docker Engine following the official Docker documentation: https://docs.docker.com/engine/install/debian/#install-using-the-repository

#### 3.2 Create the recorder configuration

Create a directory, for example /opt/opentalk-recorder, and place the recorder service configuration file recorder.toml there.

Retrieve the client_secret for the Recorder OIDC client and the RabbitMQ password from the OpenTalk server. They are stored in `/opt/opentalk-compose/config/secrets/secrets.yaml` under `OIDC_RECORDER_CLIENT_SECRET` and `RABBITMQ_PASSWORD`. Use these values to replace them in the `recorder.toml` example, and adjust the domain, issuer, and internal IP to match your environment.

If your recorder server has an Intel Arc GPU, you can enable hardware acceleration in the `[recorder.hardware_acceleration]` section.

**recorder.toml:**

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
# Enables Hardware Acceleration for Intel GPUs
#[recorder.hardware_acceleration]
#manufacturer = "intel"
#device = "/dev/dri/renderD129"
clock_format = "%d.%m.%y %X %Z"
```

#### 3.3 Create a Docker Compose file for the recorder service

Place the compose file, also to e.g. `/opt/opentalk-compose/`.

**compose.yml:**

```yaml
services:
  recorder:
    image: registry.opencode.de/opentalk/recorder:v0.15.1
    container_name: opentalk-recorder
    restart: unless-stopped
    volumes:
      - ./recorder.toml:/etc/opentalk/recorder.toml:ro
```

#### 3.4 Run Recorder container

```yaml
docker compose up -d
```

## Configure External LiveKit Server

### 1. Install LiveKit Server

Provision a separate server and install LiveKit Server as described in the [official documentation](https://docs.livekit.io/home/self-hosting/vm/)

In the following, we assume the following example environment:

- a separate, publicly accessible server
- a DNS record `livekit.example.com` that points to the server’s public IP address
- ports 80/tcp, 443/tcp, 7880/tcp, 7881/tcp, 7882/tcp, and 50000-64535/udp are publicly reachable

### 2. Disable LiveKit in `opentalk-compose`

### 2.1 Stop all services

If OpenTalk is already running and you want to disable the LiveKit service, stop all services first. If you are in the initialization phase, you can skip this step.

```bash
cd /opt/opentalk-compose && docker compose down
```

### 2.2 Adjust the Docker Compose profile in `config/default.env`

By default, the combination of services to start is controlled by the values defined in the `COMPOSE_PROFILES` variable. There are predefined service collections named `standard`, `core`, and `full`. See [Docker Compose profiles](./3-docker-compose-profiles.md) to choose the profiles that fit your needs.

Alternatively, explicitly list all services you want to start and exclude LiveKit, for example:

**config/default.env e.g.:**

```bash
COMPOSE_PROFILES=controller,webapp,recorder,mailer,pad,spacedeck,keycloak,keycloak-db,minio,traefik,rabbitmq,controller-db,ofelia
```

### 2.3 Adjust LiveKit Settings in `settings.yml`

Find the Deployment and Controller sections in `/opt/opentalk-compose/config/settings.yml` and update them to reference your external LiveKit service. The `LIVEKIT_API_SECRET` variable in `/opt/opentalk-compose/config/secrets/secrets.yaml` must match the secret configured on the LiveKit server. Either copy the secret from LiveKit into `LIVEKIT_API_SECRET`, or update the key on the LiveKit server to the value from `LIVEKIT_API_SECRET`. They must be identical to avoid authentication errors.

**settings.yml:**

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

### 2.4 Start OpenTalk Services

Then continue the [installation procedure](1-quickstart-installation.md#2-installation-procedure) as usual. If OpenTalk is already installed, it is sufficient to just start the application stack.