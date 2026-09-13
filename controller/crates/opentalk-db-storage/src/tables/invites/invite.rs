// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::{Identifiable, Queryable};
use opentalk_inventory as inventory;
use opentalk_types_common::{
    rooms::{RoomId, invite_codes::InviteCode},
    users::UserId,
};

use crate::{
    schema::invites,
    tables::{invites::InviteCodeSerialId, users::User},
};

/// Diesel invites struct
///
/// Represents an invite in the database
#[derive(Debug, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(User, foreign_key = created_by))]
#[diesel(table_name = invites)]
pub struct Invite {
    pub id: InviteCode,
    pub id_serial: InviteCodeSerialId,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_by: UserId,
    pub updated_at: DateTime<Utc>,
    pub room: RoomId,
    pub active: bool,
    pub expiration: Option<DateTime<Utc>>,
}

impl From<Invite> for inventory::RoomInvite {
    fn from(
        Invite {
            id,
            id_serial,
            created_by,
            created_at,
            updated_by,
            updated_at,
            room,
            active,
            expiration,
        }: Invite,
    ) -> Self {
        Self {
            invite_code: id,
            id_serial: id_serial.into(),
            created_by,
            created_at: created_at.into(),
            updated_by,
            updated_at: updated_at.into(),
            room,
            active,
            expiration: expiration.map(Into::into),
        }
    }
}

impl From<inventory::RoomInvite> for Invite {
    fn from(
        inventory::RoomInvite {
            invite_code,
            id_serial,
            created_by,
            created_at,
            updated_by,
            updated_at,
            room,
            active,
            expiration,
        }: inventory::RoomInvite,
    ) -> Self {
        Self {
            id: invite_code,
            id_serial: id_serial.into(),
            created_by,
            created_at: created_at.into(),
            updated_by,
            updated_at: updated_at.into(),
            room,
            active,
            expiration: expiration.map(Into::into),
        }
    }
}
