# README

## Setup OpenTalk local Dev environment

Clone the opentalk-compose git repo and change to the root directory of the repo.
e.g

```bash
git clone ssh://git@git.opentalk.dev:222/opentalk/opentalk-devops/opentalk-compose.git && cd opentalk-compose
```

Copy the `config/default.env.sampl` to `config/default.env`.

```bash
cp config/default.env.sample config/default.env
```

Run the cforge container once to initilaze the local dev environment.
It generates the nessecary secrets. Find them in the file `config/secrets/secrets.yaml`, after the first run.

```bash
docker compose run --rm cforge --dev init
```

Start the OpenTalk components and the depending services.

```bash
docker compose up -d
```

## Postinstall tasks

Some configurations can only be made when speciffic services are up and running.

### Create a demo User

For testing purposes there is a task `demoUser` to quckly create a test account.
With the `--diff` option the credentials are printed to stdout. The default User name is `demo`.

```bash
docker compose run --rm cforge --diff task demoUser
```

### Run a specific task

Use the `--help` option with `task` to get informations about available tasks.

```
docker compose run --rm cforge task --help
usage: cforge task [--timestamp TIMESTAMP] [--help] [task_name]

positional arguments:
  task_name             Specify a post-install task. Use 'help' to see available tasks.

options:
  --timestamp TIMESTAMP
                        Specify a timestamp (YYYYMMDDHHMMSSZ), required for restore tasks
  --help, -h            Show this help message and exit

Available Post-Install Tasks:

                        User Management                         
┏━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Task Name       ┃ Description                                                        ┃
┡━━━━━━━━━━━━━━━━━╇━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ demoUser        │ Creates a user in Keycloak for testing purposes.                   │
│ outlookClient   │ Creates a OIDC client in Keycloak for the OpenTalk Outlook Add-in. │
│ selfSignedCerts │ Creates self-signed certificates for the OpenTalk.                 │
└─────────────────┴────────────────────────────────────────────────────────────────────┘
                                  Backup                                   
┏━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Task Name                ┃ Description                                  ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━╇━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ createBackup             │ Creates full backup of all services          │
│ createBackupConfig       │ Creates backup of the config directory       │
│ createBackupControllerDB │ Creates backup of the Controller database    │
│ createBackupKeycloakDB   │ Creates backup of the Keycloak database      │
│ createBackupTraefik      │ Creates backup of the Traefik data directory │
│ createBackupMinio        │ Creates backup of MinIO bucket               │
└──────────────────────────┴──────────────────────────────────────────────┘
                                                  Restore                                                  
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Task Name                 ┃ Description                                                                 ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━╇━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ restoreBackup             │ Restore OpenTalk from a given timestamp. A full backup archive is required. │
│ restoreBackupConfig       │ Restores the config directory from a given timestamp.                       │
│ restoreBackupControllerDB │ Restores Controller database from a given timestamp.                        │
│ restoreBackupKeycloakDB   │ Restores Keycloak database from a given timestamp.                          │
│ restoreBackupTraefik      │ Restores Traefik data directory from a given timestamp.                     │
│ restoreBackupMinio        │ Restores MinIO bucket from a given timestamp.                               │
└───────────────────────────┴─────────────────────────────────────────────────────────────────────────────┘
```

## Login to OpenTalk webapp

By default the web interface on a local dev environment is accessible at [http://localhost:8080](http://localhost:8080).

### Web Interfaces of 3rd party services

Some 3rd party servicese provides web interfaces. Traefik creates routes to these HTTP endpoints.
You can access the web interfaces behind the paths as follows.

#### Traefik dashboard

- [http://localhost:8080/adm/traefik/dashboard/](http://localhost:8080/adm/traefik/dashboard/)

#### Keycloak dashboard

- [http://localhost:8080/auth/](http://localhost:8080/auth/)

#### RabbitMQ Management Web Interface

- [http://localhost:8080/adm/rabbitmq/](http://localhost:8080/adm/rabbitmq/)

#### MinIO Web Console

- [http://localhost:8080/adm/minio/](http://localhost:8080/adm/minio/)

#### MailCrab Web Interface

- [http://localhost:8080/adm/mailcrab/](http://localhost:8080/adm/mailcrab/)

## Configure and `apply` service configurations

To configure specific services you can edit the `config/setting.yaml` or `config/settings.override.yaml` files.
Ensure the YAML structure when uncommenting or overrriding default values.

Apply config changes with

```
docker compose run --rm cforge --diff apply
```

## Merge new config options

When ansible reconizes new config options in role opentalk_config/defaults/main.yml it notices it with a warning message:

> ⚠️ WARNING ⚠️  There are new configuration options available, that are not yet in settings.yml!
New keys found:
Traefik.log
Please check and manually merge settings.yml.unmerged into settings.yml.
And delete settings.yml.unmerged afterwards."

You can adjust `settings.yml.unmerged` to your needs and/or apply the diff file to add new config options.

```bash
cd /opt/opentalk
# check config diff
diff -u config/settings.yml config/settings.yml.unmerged

# add new options to settings.yml
diff -u config/settings.yml config/settings.yml.unmerged | patch config/settings.yml \
&& rm -i config/settings.yml.unmerged
```

## Deployment scenario: external keycloak

Ensure that no opentalk-compose deployment is already running or shut down alls services:

```bash
docker compose down
```

### local keycloak test setup

- A local Keycloak running in docker

the opentalk theme is mountend into the container under `/opt/keycloak/providers/`. Keycloak is listen on e.g. 0.0.0.0:9999

```bash
KC_OT_THEME_PATH=/home/anfi/git/opentalk/devops/opentalk-compose/services/3rd_party/keycloak/
docker run -d --name keycloak_dev -p 9999:8080 \
        -e KC_BOOTSTRAP_ADMIN_USERNAME=admin \
        -e KC_BOOTSTRAP_ADMIN_PASSWORD=admin \
        -e KC_HEALTH_ENABLED=true \
        -e KC_HTTP_RELATIVE_PATH=/auth \
        -v ${KC_OT_THEME_PATH}/keycloak-ot-theme.jar:/opt/keycloak/providers/keycloak-ot-theme.jar \
        quay.io/keycloak/keycloak:latest \
        start-dev
```

- Set `COMPOSE_PROFILES` without keycloak

```bash
COMPOSE_PROFILES=controller,webapp,recorder,mailer,pad,spacedeck,livekit,minio,traefik,rabbitmq,controller-db,ofelia
```

- initialize opentalk-compose with `dev` environment

```bash
docker compose run --rm cforge --dev init
```

### Adjust settings.yml with the target keycloak

```yaml
Deployment:
  environment: dev
  scenario:
    - external_keycloak

Controller:
  oidc:
    authority: http://host.docker.internal:9999/auth/realms/opentalk
  frontend:
    authority: http://host.docker.internal:9999/auth/realms/opentalk
  user_search:
    api_base_url: http://host.docker.internal:9999/auth/admin/realms/opentalk

Keycloak:
  api_base_url: http://host.docker.internal:9999/auth/admin/realms/opentalk
  base_url: http://host.docker.internal:9999/auth
  auth:
    password: admin
    realm: master
    username: admin
  health_url: http://host.docker.internal:9999/auth/realms/master
  hostname: http://host.docker.internal:9999/auth

Obelisk:
  auth:
    issuer: http://host.docker.internal:9999/auth/realms/opentalk

Recorder:
  auth:
    issuer http://host.docker.internal:9999/auth/realms/opentalk
```

### Run services without keycloak

```bash
docker compose up -d
```

## Deployment scenario: external Livekit server

Ensure that no opentalk-compose deployment is already running or shut down alls services:

```bash
docker compose down
```

### local livekit test setup

- A local livekit server

Livekit is listening on e.g. on ports 7880,7881,7882 by default. Ensure no port conflicts with running containers.

```bash
SECRET=$(pwgen -sB 12 1); echo $SECRET; livekit-server --dev --bind 0.0.0.0 --keys "controller: $SECRET"
```

- Set `COMPOSE_PROFILES` without livekit

```bash
COMPOSE_PROFILES=controller,webapp,recorder,mailer,pad,spacedeck,keycloak,keycloak-db,minio,traefik,rabbitmq,controller-db,ofelia
```

- initialize opentalk-compose with `dev` environment

```bash
docker compose run --rm cforge --diff --dev init
```

### Adjust settings.yml with the target livekit service

```yaml
Deployment:
  environment: dev
  scenario:
    - external_livekit

Controller:
  livekit:
    api_key: controller
    api_secret: yourSecret
```

### Run services without livekit

```bash
docker compose up -d
```

## Deployment scenario: external recorder

Ensure that no opentalk-compose deployment is already running or shut down alls services:

```bash
docker compose down
```

### local recorder test setup

- Set `COMPOSE_PROFILES` without recorder

```bash
COMPOSE_PROFILES=controller,webapp,mailer,pad,spacedeck,keycloak,keycloak-db,minio,traefik,livekit,rabbitmq,controller-db,ofelia
```

- initialize opentalk-compose with `dev` environment

```bash
docker compose run --rm cforge --diff --dev init
```

### Adjust settings.yml with external_recorder as deplyoment target

```yaml
Deployment:
  environment: dev
  scenario:
    - external_recorder
```

### Run services without recorder

```bash
docker compose up -d
docker compose up -d rabbitmq  --force-recreate # recreate rabbitmq because exposed port was changed 
```

- A local recorder service

create a minimal recorder config e.g. `config/ext_recorder.toml`

```toml
[auth]
issuer = "http://host.docker.internal:9999/auth/realms/opentalk"
client_id = "Recorder"
client_secret = "<RecorderOIDCSecret>"
[controller]
domain = "host.docker.internal:11311"
insecure = true
[rabbitmq]
uri = "amqp://ot:<rabbitmqSecret>@host.docker.internal/%2F"
queue = "opentalk_recorder"
```

#### Run Recorder container

```bash
docker run --add-host="host.docker.internal:host-gateway" \
  -v ./config/ext_recorder.toml:/etc/opentalk/recorder.toml \
  -e RUST_LOG=debug \
  git.opentalk.dev:5050/opentalk/backend/services/recorder:v0.15.1
```

## Default server installation procedure

### Debian

1. Install docker and docker compose.
Follow the [official docker install documentation](https://docs.docker.com/engine/install/debian/#install-using-the-repository).

2. add the OpenTalk deb repository

```bash
apt update && apt install -y curl gnupg
curl http://anfi.opentalk.run:7376/deb-testing/public.gpg | gpg --yes --dearmor -o /usr/share/keyrings/openrepo-deb-testing.gpg
echo "deb [arch=any signed-by=/usr/share/keyrings/openrepo-deb-testing.gpg] http://anfi.opentalk.run:7376/deb-testing/ stable main" > /etc/apt/sources.list.d/openrepo-deb-testing.list
```

3. Install the `opentalk-compose` package

```bash
apt update && apt install opentalk-compose -y
```

4. Set `OT_DOMAIN` in `/opt/opentalk-compose/config/default.env`

```bash
sed -i 's/^OT_DOMAIN=.*/OT_DOMAIN="example.com"/' /opt/opentalk-compose/config/default.env
```

5. Generate default OpenTalk config

```bash
cforge init
```

6. Optional: Edit settings `/opt/opentalk-compose/config/settings.yml`

7. Run OpenTalk

```bash
cd /opt/opentalk-compose && docker compose up -d
```

8. Optional: Enable demo user

```bash
cforge --diff task demoUser
```

Refer `man opentalk-compose` for more information.

## Server installation procedure with Custom CA

Run "Default server installation procedure" steps 1-5.

Copy SSL certificate files to /opt/opentalk-compose/config/ssl
- CA certificate
- server fullchain certifikate
- server SSL certificate key

Edit `/opt/opentalk-compose/config/` 

```
CustomCertificates:
  enabled: true
  ssl_ca_file: my-ca.crt
  ssl_certificate_fullchain_file: my-server_fullchain.crt
  ssl_certificate_key_file: my-server.key
```