// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use barrel::backend::Pg;
use barrel::{types, Migration};

pub fn migration() -> String {
    let mut migr = Migration::new();

    migr.create_table("users", |table| {
        table.add_column("id", types::custom("BIGSERIAL").primary(true));
        table.add_column("oidc_uuid", types::uuid());
        table.add_column("email", types::varchar(255).unique(true).nullable(false));
        table.add_column("title", types::varchar(255).nullable(false));
        table.add_column("firstname", types::varchar(255).nullable(false));
        table.add_column("lastname", types::varchar(255).nullable(false));
        table.add_column("id_token_exp", types::custom("BIGINT").nullable(false));
        table.add_column("theme", types::varchar(255));
        table.add_column("language", types::varchar(35).nullable(false));
    });

    migr.create_table("groups", |table| {
        table.add_column("id", types::varchar(255).primary(true));
    });

    migr.create_table("user_groups", |table| {
        table.add_column("user_id", types::custom("BIGINT REFERENCES users(id)"));
        table.add_column(
            "group_id",
            types::custom("VARCHAR(255) REFERENCES groups(id)"),
        );
        table.inject_custom("PRIMARY KEY (user_id, group_id)");
    });

    migr.create_table("rooms", |table| {
        table.add_column("id", types::custom("BIGSERIAL").primary(true));
        table.add_column("uuid", types::uuid());
        table.add_column("owner", types::custom("BIGINT REFERENCES users(id)"));
        table.add_column("password", types::varchar(255));
        table.add_column("wait_for_moderator", types::boolean().nullable(false));
        table.add_column("listen_only", types::boolean().nullable(false));
    });

    migr.create_table("legal_votes", |table| {
        table.add_column("id", types::uuid().primary(true));
        table.add_column("initiator", types::custom("BIGINT REFERENCES users(id)"));
        table.add_column("protocol", types::custom("JSONB"));
    });

    migr.make::<Pg>()
}
