# README

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
