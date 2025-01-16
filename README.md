# README

## Standard OpenTalk single-host Installation

Copy the `env.sample` to `.env`.

```bash
cp env.sample .env
```

Add your domain to the `.env` file.

```bash
sed -i 's/OT_DOMAIN=""/OT_DOMAIN="livekit-test-anfi.opentalk.run"/g' .env
```

Run the cforge container once to initilaze the deployment envrinonment.

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
| 5060        | TCP      | SIP (Obelisk)                 |
| 50000-64534 | UDP      | RTP (Lifekit)                 |
| 7881        | TCP      | WebRTC ICE over TCP (livekit) |
