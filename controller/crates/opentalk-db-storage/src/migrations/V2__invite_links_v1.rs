// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::migrations::type_polyfills::datetime;
use barrel::backend::Pg;
use barrel::{types, Migration};

pub fn migration() -> String {
    let mut migr = Migration::new();

    migr.create_table("invites", |table| {
        table.add_column("id", types::custom("BIGSERIAL").primary(true));

        table.add_column("uuid", types::uuid().nullable(false).indexed(true));

        table.add_column("created", datetime().nullable(false));
        table.add_column("created_by", types::custom("BIGINT REFERENCES users(id)"));

        table.add_column("updated", datetime().nullable(false));
        table.add_column("updated_by", types::custom("BIGINT REFERENCES users(id)"));

        // Use the rooms(uuid) field here as this allows us to return the room_id without getting it from the database in a 2nd query.
        table.add_column("room", types::custom("UUID REFERENCES rooms(uuid)"));

        table.add_column("active", types::boolean());
        table.add_column("expiration", datetime().nullable(true));
    });

    migr.make::<Pg>()
}
