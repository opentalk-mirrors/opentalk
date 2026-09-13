// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::{Identifiable, Queryable};
use opentalk_database::{DatabaseError, Result};
use opentalk_inventory as inventory;
use opentalk_types_common::{
    self as types,
    rooms::RoomId,
    streaming::{
        StreamingKey, StreamingKind, StreamingTarget, StreamingTargetId, StreamingTargetKind,
    },
};
use snafu::{Report, Snafu};
use url::Url;

use crate::{schema::room_streaming_targets, tables::rooms::Room};

#[derive(Debug, Queryable, Identifiable, Associations, Insertable)]
#[diesel(belongs_to(Room, foreign_key = room_id))]
#[diesel(table_name = room_streaming_targets)]
pub struct RoomStreamingTarget {
    pub id: StreamingTargetId,
    pub room_id: RoomId,
    pub name: String,
    pub kind: StreamingKind,
    pub streaming_endpoint: String,
    pub streaming_key: StreamingKey,
    pub public_url: String,
}

impl From<inventory::RoomStreamingTargetRecord> for RoomStreamingTarget {
    fn from(
        inventory::RoomStreamingTargetRecord {
            id,
            room_id,
            name,
            kind,
            streaming_endpoint,
            streaming_key,
            public_url,
        }: inventory::RoomStreamingTargetRecord,
    ) -> Self {
        Self {
            id,
            room_id,
            name,
            kind,
            streaming_endpoint,
            streaming_key,
            public_url,
        }
    }
}

impl From<RoomStreamingTarget> for inventory::RoomStreamingTargetRecord {
    fn from(
        RoomStreamingTarget {
            id,
            room_id,
            name,
            kind,
            streaming_endpoint,
            streaming_key,
            public_url,
        }: RoomStreamingTarget,
    ) -> Self {
        Self {
            id,
            room_id,
            name,
            kind,
            streaming_endpoint,
            streaming_key,
            public_url,
        }
    }
}

impl TryFrom<RoomStreamingTarget> for types::streaming::RoomStreamingTarget {
    type Error = DatabaseError;

    fn try_from(record: RoomStreamingTarget) -> Result<Self> {
        let kind = match record.kind {
            StreamingKind::Custom => StreamingTargetKind::Custom {
                streaming_endpoint: Url::parse(&record.streaming_endpoint).map_err(|err| {
                    tracing::warn!(
                        "Failed to parse streaming endpoint: {}",
                        Report::from_error(err)
                    );
                    DatabaseError::Custom {
                        message: "Inconsistent data".to_string(),
                    }
                })?,
                streaming_key: record.streaming_key,
                public_url: Url::parse(&record.public_url).map_err(|err| {
                    tracing::warn!(
                        "Invalid public url entry in db: {}",
                        Report::from_error(err)
                    );
                    DatabaseError::Custom {
                        message: "Inconsistent data".to_string(),
                    }
                })?,
            },
        };

        Ok(Self {
            id: record.id,
            streaming_target: StreamingTarget {
                name: record.name,
                kind,
            },
        })
    }
}

#[derive(Debug, Snafu)]
pub enum StreamTargetConversionError {
    #[snafu(display("Parsing the url failed, because {target} is not a valid URL"))]
    WrongUrl { target: String },
}
