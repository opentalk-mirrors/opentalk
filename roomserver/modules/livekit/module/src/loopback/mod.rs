// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use futures::{StreamExt as _, stream};
use livekit_api::services::room::{RoomClient, UpdateParticipantOptions};
use livekit_protocol::{ParticipantInfo, ParticipantPermission, TrackSource};
use opentalk_roomserver_types::connection_id::ConnectionId;
use opentalk_types_signaling::ParticipantId;

pub use crate::loopback::{
    create_room::create_room, microphone_restriction::update_restricted_microphones,
    mute::mute_participants, revoke_token::revoke_token,
    screen_share_permissions::set_screenshare_permissions,
};
use crate::{LIVEKIT_MEDIA_SOURCES, PARALLEL_UPDATES};

mod create_room;
mod microphone_restriction;
mod mute;
mod revoke_token;
mod screen_share_permissions;

pub enum LiveKitLoopback {
    RoomCreated,
    RoomRemoved,
    ProxySocketClosed,

    /// Note that the token identities were removed
    NoteRevokedTokens {
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        token_identities: BTreeSet<String>,
    },

    ScreenSharePermissionsUpdated {
        sender: ParticipantId,
        participants: BTreeSet<ParticipantId>,
        grant: bool,
    },
}

/// Update all provided participants.
///
/// If `grant` is `true`, the provided `source_numbers` will be allowed to be published.
async fn update_participants_permission(
    livekit_client: &RoomClient,
    participants: Vec<ParticipantInfo>,
    source_numbers: &[i32],
    grant: bool,
    room: &str,
) {
    stream::iter(participants)
        .map(|participant| {
            update_single_participant_permission(
                livekit_client,
                participant,
                source_numbers,
                grant,
                room,
            )
        })
        .buffer_unordered(PARALLEL_UPDATES)
        .collect::<Vec<_>>()
        .await;
}

#[tracing::instrument(skip(livekit_client, participant), level = "debug", fields(livekit_participant_id=participant.identity))]
async fn update_single_participant_permission(
    livekit_client: &RoomClient,
    participant: ParticipantInfo,
    source_numbers: &[i32],
    grant: bool,
    room: &str,
) {
    let mut can_publish_sources = participant.permission.map_or_else(
        || {
            LIVEKIT_MEDIA_SOURCES
                .map(|s: TrackSource| s as i32)
                .to_vec()
        },
        |p| p.can_publish_sources,
    );

    for source_number in source_numbers {
        update_publish_sources(&mut can_publish_sources, *source_number, grant);
    }

    if let Err(e) = livekit_client
        .update_participant(
            room,
            &participant.identity,
            UpdateParticipantOptions {
                permission: Some(ParticipantPermission {
                    can_subscribe: true,
                    can_publish: true,
                    can_publish_data: false,
                    can_publish_sources,
                    hidden: false,
                    can_update_metadata: false,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await
    {
        tracing::error!(
            livekit.participant = participant.identity,
            room = room,
            "Failed to update participant, {e}",
        );
    }
    tracing::trace!("participant permissions updated");
}

fn update_publish_sources(can_publish_sources: &mut Vec<i32>, source: i32, grant: bool) {
    if grant {
        if !can_publish_sources.contains(&source) {
            can_publish_sources.push(source);
        }
    } else {
        can_publish_sources.retain(|&x| x != source);
    }
}
