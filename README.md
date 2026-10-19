# OpenTalk

WIP

## Development tasks (xtask)

- **controller-xtask** — Generates and verifies the Diesel database `schema.rs`. Run it after changing DB migrations to regenerate the schema (`generate-db-schema`) or in CI to check it's up to date (`verify-db-schema`).
    - `cargo run --package controller-xtask -- generate-db-schema`
- **obelisk-xtask** — Generates Obelisk's German/English audio prompt tracks via `piper` + `ffmpeg`. Run it when prompt texts change to regenerate the WAV files (requires `PIPER_BIN`, `MODEL_DE`, `MODEL_EN`).
    - `cargo run --package obelisk-xtask`

## Docs

Documentation for all services can be found in the `docs` folder.
Docs are built and deployed using the separate [docs repository](https://git.opentalk.dev/opentalk/backend/docs).
You can test the docs in this repository with

```sh
docker compose -f compose.docs.yaml up
```

The docs will be available at [http://localhost:8080](http://localhost:8080).

The generated docs show a `404 - Not found` error on the root page.
This is because the root page gets generated in the docs repository, where it is possible to also link to the [user docs](https://git.opentalk.dev/opentalk/support).
