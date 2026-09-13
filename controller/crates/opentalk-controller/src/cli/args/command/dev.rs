// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::HashMap, path::Path};

use clap::Subcommand;
use opentalk_controller_core::load_settings_provider;
use opentalk_controller_settings::Settings;
use opentalk_database::{DatabaseError, Db};
use opentalk_db_storage as db;
use snafu::ResultExt as _;

use crate::Result;

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// Print all invite links for the rooms owned by the user with the given email
    Invites {
        /// Email address of the user whose room invite links should be printed
        email: String,
    },
}

impl Command {
    pub async fn exec(self, optional_config_path: Option<&Path>) -> Result<()> {
        let settings = load_settings_provider(optional_config_path)?.get();
        match self {
            Command::Invites { email } => print_invites(&settings, &email).await,
        }
        .whatever_context("Dev command failed")
    }
}

/// Implementation of the `opentalk-controller dev invites <email>` command
async fn print_invites(settings: &Settings, email: &str) -> Result<(), DatabaseError> {
    let db = Db::connect(&settings.database)?;
    let mut conn = db.get_conn().await?;

    let rooms_with_creator = db::queries::rooms::get_all_rooms_with_creator(&mut conn).await?;
    let user_room_ids: HashMap<_, _> = rooms_with_creator
        .into_iter()
        .filter(|(_, creator)| creator.email == email)
        .map(|(room, _)| (room.id, room))
        .collect();

    if user_room_ids.is_empty() {
        println!("No rooms found for user with email {email}");
        return Ok(());
    }

    let invites = db::queries::invites::get_all_invites(&mut conn).await?;
    let base_url = &settings.frontend.base_url;

    let mut found = false;
    for invite in invites {
        if !user_room_ids.contains_key(&invite.room) {
            continue;
        }
        found = true;
        match base_url.join(&format!("invite/{}", invite.id)) {
            Ok(link) => println!("room {}: {link}", invite.room),
            Err(_) => println!("room {}: invite/{}", invite.room, invite.id),
        }
    }

    if !found {
        println!("No invite links found for the rooms owned by user with email {email}");
    }

    Ok(())
}
