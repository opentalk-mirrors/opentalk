// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage as db;
use opentalk_inventory::{
    self as inventory, EventEmailInvite, EventInvite, EventInviteInventory, NewEventEmailInvite,
    NewEventInvite, UpdateEventEmailInvite, UpdateEventInvite, User,
};
use opentalk_types_common::{
    events::{EventId, invites::EventInviteStatus},
    pagination::{ItemCount, Page, PageSize},
    rooms::RoomId,
    users::UserId,
};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result, error::DatabaseSnafu};

#[async_trait::async_trait]
impl EventInviteInventory for DatabaseConnection {
    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn try_create_event_email_invite(
        &mut self,
        invite: NewEventEmailInvite,
    ) -> Result<Option<EventEmailInvite>> {
        Ok(
            db::queries::events::try_create_event_email_invite(&mut self.inner, invite.into())
                .await
                .context(DatabaseSnafu)?
                .map(Into::into),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn try_create_event_invite(
        &mut self,
        invite: NewEventInvite,
    ) -> Result<Option<EventInvite>> {
        Ok(
            db::queries::events::try_create_event_invite(&mut self.inner, invite.into())
                .await
                .context(DatabaseSnafu)?
                .map(Into::into),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_event_invites_paginated(
        &mut self,
        event_id: EventId,
        per_page: PageSize,
        page: Page,
        filter_by_status: Option<EventInviteStatus>,
    ) -> Result<(Vec<(EventInvite, User)>, ItemCount)> {
        let (items, overall) = db::queries::events::get_event_invites_paginated(
            &mut self.inner,
            event_id,
            per_page,
            page,
            filter_by_status,
        )
        .await
        .context(DatabaseSnafu)?;
        Ok((
            items
                .into_iter()
                .map(|(invite, user)| (invite.into(), user.into()))
                .collect(),
            overall,
        ))
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_event_email_invites_paginated(
        &mut self,
        event_id: EventId,
        per_page: PageSize,
        page: Page,
    ) -> Result<(Vec<EventEmailInvite>, ItemCount)> {
        let (invites, overall) = db::queries::events::get_event_email_invites_paginated(
            &mut self.inner,
            event_id,
            per_page,
            page,
        )
        .await
        .context(DatabaseSnafu)?;
        Ok((invites.into_iter().map(Into::into).collect(), overall))
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_event_invite_for_user_and_room(
        &mut self,
        user_id: UserId,
        room_id: RoomId,
    ) -> Result<Option<EventInvite>> {
        Ok(db::queries::events::get_event_invite_for_user_and_room(
            &mut self.inner,
            user_id,
            room_id,
        )
        .await
        .context(DatabaseSnafu)?
        .map(Into::into))
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_event_user_invites_for_events(
        &mut self,
        events: &[&inventory::Event],
    ) -> Result<Vec<Vec<(EventInvite, User)>>> {
        let events = events
            .iter()
            .cloned()
            .map(|e| e.clone().into())
            .collect::<Vec<db::tables::events::Event>>();

        let events = events.iter().collect::<Vec<&db::tables::events::Event>>();

        let invites =
            db::queries::events::get_event_user_invites_for_events(&mut self.inner, &events)
                .await
                .context(DatabaseSnafu)?;
        Ok(invites
            .into_iter()
            .map(|items| {
                items
                    .into_iter()
                    .map(|(invite, user)| (invite.into(), user.into()))
                    .collect()
            })
            .collect())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_event_email_invites_for_events(
        &mut self,
        events: &[&inventory::Event],
    ) -> Result<Vec<Vec<EventEmailInvite>>> {
        let events = events
            .iter()
            .cloned()
            .map(|e| e.clone().into())
            .collect::<Vec<db::tables::events::Event>>();
        let events = events.iter().collect::<Vec<&_>>();
        Ok(
            db::queries::events::get_event_email_invites_for_events(&mut self.inner, &events)
                .await
                .context(DatabaseSnafu)?
                .into_iter()
                .map(|v| v.into_iter().map(Into::into).collect())
                .collect(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_invites_pending_for_user(&mut self, user_id: UserId) -> Result<Vec<EventInvite>> {
        Ok(
            db::queries::events::get_invites_pending_for_user(&mut self.inner, user_id)
                .await
                .context(DatabaseSnafu)?
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn delete_event_invite_by_invitee(
        &mut self,
        event_id: EventId,
        user_id: UserId,
    ) -> Result<EventInvite> {
        Ok(
            db::queries::events::delete_event_invite_by_invitee(&mut self.inner, event_id, user_id)
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn delete_event_invite_by_email(
        &mut self,
        event_id: EventId,
        email: &str,
    ) -> Result<EventEmailInvite> {
        Ok(db::queries::events::delete_event_email_invite_by_email(
            &mut self.inner,
            &event_id,
            email,
        )
        .await
        .context(DatabaseSnafu)?
        .into())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn update_event_user_invite(
        &mut self,
        event_id: EventId,
        user_id: UserId,
        event_invite: UpdateEventInvite,
    ) -> Result<EventInvite> {
        Ok(db::queries::events::update_event_invite(
            &mut self.inner,
            event_id,
            user_id,
            event_invite.into(),
        )
        .await
        .context(DatabaseSnafu)?
        .into())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn update_event_email_invite(
        &mut self,
        event_id: EventId,
        email: &str,
        event_invite: UpdateEventEmailInvite,
    ) -> Result<EventEmailInvite> {
        Ok(db::queries::events::update_event_email_invite(
            &mut self.inner,
            email,
            event_id,
            event_invite.into(),
        )
        .await
        .context(DatabaseSnafu)?
        .into())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn migrate_event_email_invites_to_user_invites(
        &mut self,
        user: User,
    ) -> Result<Vec<(EventId, RoomId)>> {
        Ok(
            db::queries::events::migrate_event_email_invites_to_user_invites(
                &mut self.inner,
                &user.into(),
            )
            .await
            .context(DatabaseSnafu)?,
        )
    }
}
