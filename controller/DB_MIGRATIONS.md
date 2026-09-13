<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# Requirements

- install the diesel cli tool with `cargo install diesel_cli --version ~2 --no-default-features --features="postgres"`
- make sure `rustfmt` is installed with `rustup component add rustfmt`

# How to change the schema

- add file `V<version_nr>__<name>.sql` under crates/controller/src/db/migrations/
- Run `cargo xtask generate-db-schema` to generate a new diesel schema in `crates/db-storage/src/db/schema.rs`.
  This creates a random database by default and deletes it afterwards.

See `cargo xtask generate-db-schema --help` for information what options are possible to not use default values
or specify a fixed database.
