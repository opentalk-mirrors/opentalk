// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_database::DatabaseError;
use opentalk_db_storage as db;
use opentalk_inventory::{RoomInvite, RoomInviteInventory};
use opentalk_types_common::{
    rooms::{RoomId, invite_codes::InviteCode},
    time::Timestamp,
};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Error, Result, error::DatabaseSnafu};

#[async_trait::async_trait]
impl RoomInviteInventory for DatabaseConnection {
    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_room_invite(&mut self, invite_code: InviteCode) -> Result<RoomInvite> {
        match db::queries::invites::get_room_invite(&mut self.inner, invite_code).await {
            Ok(invite) => Ok(invite.into()),
            Err(DatabaseError::NotFound) => Err(Error::NotFound),
            Err(err) => Err(err).context(DatabaseSnafu)?,
        }
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_room_invites_with_room_inactive_or_expired_before(
        &mut self,
        expired_before: Timestamp,
    ) -> Result<Vec<(InviteCode, RoomId)>> {
        Ok(db::queries::invites::get_inactive_or_expired_before_invite(
            &mut self.inner,
            expired_before.into(),
        )
        .await
        .context(DatabaseSnafu)?)
    }
}
