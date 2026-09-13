// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use barrel::backend::Pg;
use barrel::{types, Migration};

pub fn migration() -> String {
    let mut migr = Migration::new();

    migr.create_table("sip_configs", |table| {
        table.add_column("id", types::custom("BIGSERIAL").primary(true));

        // make the FK unique to express a one-to-one relation
        table.add_column(
            "room",
            types::custom("UUID REFERENCES rooms(uuid)").unique(true),
        );

        // A string with 10 numeric characters to identify a room
        table.add_column("sip_id", types::varchar(10).unique(true).nullable(false));

        // A string with 10 numeric characters
        table.add_column("password", types::varchar(10).nullable(false));

        table.add_column("enable_lobby", types::boolean());
    });

    migr.make::<Pg>()
}
