# README

## Standard OpenTalk single-host Installation

Clone the ot-compose git repo

```bash
git clone ssh://git@git.opentalk.dev:222/opentalk/opentalk-devops/ot-compose.git
```

Copy the `env.sample` to `.env`.

```bash
cp env.sample .env
```

Add your domain to the `.env` file.

```bash
sed -i 's/OT_DOMAIN=""/OT_DOMAIN="example.com"/g' .env
```

Run the cforge container once to initilaze the deployment environment.

```bash
docker compose run --rm cforge
```

Start the OpenTalk components and the depending services.

```bash
docker compose up -d
```

### Open Firewall ports

| port        | protocol | service                       |
|-------------|----------|-------------------------------|
| 80,443      | TCP      | http(s) (Traefik)             |
| 40000-49999 | UDP      | RTP (Obelisk)                 |
| 5060        | UDP      | SIP (Obelisk)                 |
| 50000-64535 | UDP      | RTP (Lifekit)                 |
| 7881        | TCP      | WebRTC ICE over TCP (livekit) |


### Create OpenTalk User in keycloak

1. Use the Keycloak bootstap account to login to the master realm. The User is `admin` by default. Find the generated password in `cat services/cforge/data/.secrets.yaml`.

2. Switch to the `opentalk` realm.

3. Create a new regular user and credentials.
