# README

## Standard OpenTalk single-host Installation

### Server preperation

For a minimum test setup, ensure as follows:

- CPU: 2 Cores
- RAM: 4 GiB
- Stogare: ~10GiB free disk space

- DNS Domain record, pointing to the pulic IP address
- Docker and Docker Compose are installed and logged into the OpenTalk Container registry `git.opentalk.dev:5050`.

#### Open Firewall ports

| port        | protocol | service                       |
|-------------|----------|-------------------------------|
| 80,443      | TCP      | http(s) (Traefik)             |
| 40000-49999 | UDP      | RTP (Obelisk)                 |
| 5060        | UDP      | SIP (Obelisk)                 |
| 50000-64535 | UDP      | RTP (Livekit)                 |
| 7881        | TCP      | WebRTC ICE over TCP (livekit) |

## Setup OpenTalk

Clone the ot-compose git repo

```bash
git clone ssh://git@git.opentalk.dev:222/opentalk/opentalk-devops/ot-compose.git
```

Copy the `env.sample` to `.env`.

```bash
cp env.sample .env
```

Define your domain with the variable `OT_DOMAIN` in the `.env` file.

```bash
sed -i 's/OT_DOMAIN=""/OT_DOMAIN="example.com"/g' .env
```

Run the cforge container once to initilaze the deployment environment.
It generates the nessecary secrets. Find them in the file `services/cforge/data/.secrets.yaml`, after the first run.

```bash
docker compose run --rm cforge
```

Start the OpenTalk components and the depending services.

```bash
docker compose up -d
```

## Postinstall tasks

Some configurations can only be made when speciffic services are up and running.

currently availabe postInstall tasks

| task      | descriptiom |
| ----------| ----------- |
| demoUser  | Creates a demo user in keycloak that can be used to test OpenTalk. The name is `demo` by default. |

Run all `postinstall` tasks:

```bash
docker compose run --rm -e "TAGS=postInstall" cforge
```

### Run a specific `postinstall` task

```bash
docker compose run --rm -e "TAGS=demoUser" cforge
```

### Web Interfaces of 3rd party services

Some 3rd party servicese provides web interfaces. Traefik creates routes to these HTTP endpoints.
You can access the web interfaces behind the paths as follows.

To pass the basic auth, find the **password** of the user `admin` in `services/cforge/data/.secrets.yaml` with the key `TRAEFIK_DASHBOARD_ADMIN_PASSWORD`.

#### Traefik dashboard

- https://`<example.com>`/adm/traefik/dashboard/

#### RabbitMQ Management Web Interface

- https://`<example.com>`/adm/rabbitmq/

#### MinIO Web Console

- https://`<example.com>`/adm/minio/

#### MailCrab Web Interface

- https://`<example.com>`/adm/mailcrab/
