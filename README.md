# OpenTalk

WIP

## Development tasks (xtask)

- **controller-xtask** — Generates and verifies the Diesel database `schema.rs`. Run it after changing DB migrations to regenerate the schema (`generate-db-schema`) or in CI to check it's up to date (`verify-db-schema`).
    - `cargo run --package controller-xtask -- generate-db-schema`
- **obelisk-xtask** — Generates Obelisk's German/English audio prompt tracks via `piper` + `ffmpeg`. Run it when prompt texts change to regenerate the WAV files (requires `PIPER_BIN`, `MODEL_DE`, `MODEL_EN`).
    - `cargo run --package obelisk-xtask`
