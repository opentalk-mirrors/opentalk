// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use barrel::backend::Pg;
use barrel::{types, Migration};

pub fn migration() -> String {
    let mut migr = Migration::new();

    migr.change_table("legal_votes", |table| {
        table.add_column(
            "room_id",
            types::custom("UUID REFERENCES rooms(uuid) ON DELETE CASCADE").nullable(true),
        );
    });

    migr.make::<Pg>()
}
