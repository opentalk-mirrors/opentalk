// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub mod v1;

/// Versioned Mail Task Protocol
#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "version")
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum MailTask {
    #[cfg_attr(feature = "serde", serde(rename = "1"))]
    V1(v1::Message),
}

#[cfg(feature = "client")]
impl MailTask {
    /// Creates an invite MailTask for a registered invitee
    pub fn registered_event_invite<E, I, U>(inviter: I, event: E, invitee: U) -> MailTask
    where
        I: Into<v1::RegisteredUser>,
        E: Into<v1::Event>,
        U: Into<v1::RegisteredUser>,
    {
        Self::V1(v1::Message::RegisteredEventInvite(
            v1::RegisteredEventInvite {
                invitee: invitee.into(),
                event: event.into(),
                inviter: inviter.into(),
            },
        ))
    }

    /// Creates an invite MailTask for an unregistered invitee
    pub fn unregistered_event_invite<E, I, U>(inviter: I, event: E, invitee: U) -> MailTask
    where
        I: Into<v1::RegisteredUser>,
        E: Into<v1::Event>,
        U: Into<v1::UnregisteredUser>,
    {
        Self::V1(v1::Message::UnregisteredEventInvite(
            v1::UnregisteredEventInvite {
                invitee: invitee.into(),
                event: event.into(),
                inviter: inviter.into(),
            },
        ))
    }

    /// Creates an invite MailTask for an external invitee
    pub fn external_event_invite<E, I, U>(inviter: I, event: E, invitee: U) -> MailTask
    where
        I: Into<v1::RegisteredUser>,
        E: Into<v1::Event>,
        U: Into<v1::ExternalUser>,
    {
        Self::V1(v1::Message::ExternalEventInvite(v1::ExternalEventInvite {
            invitee: invitee.into(),
            event: event.into(),
            inviter: inviter.into(),
        }))
    }

    /// Creates an event update MailTask for a registered invitee
    pub fn registered_event_update<E, EE, I, U>(
        inviter: I,
        event: E,
        event_exception: Option<EE>,
        invitee: U,
    ) -> MailTask
    where
        I: Into<v1::RegisteredUser>,
        E: Into<v1::Event>,
        EE: Into<v1::EventException>,
        U: Into<v1::RegisteredUser>,
    {
        Self::V1(v1::Message::RegisteredEventUpdate(
            v1::RegisteredEventUpdate {
                invitee: invitee.into(),
                event: event.into(),
                event_exception: event_exception.map(Into::into),
                inviter: inviter.into(),
            },
        ))
    }

    /// Creates an event update MailTask for an unregistered invitee
    pub fn unregistered_event_update<E, EE, I, U>(
        inviter: I,
        event: E,
        event_exception: Option<EE>,
        invitee: U,
    ) -> MailTask
    where
        I: Into<v1::RegisteredUser>,
        E: Into<v1::Event>,
        EE: Into<v1::EventException>,
        U: Into<v1::UnregisteredUser>,
    {
        Self::V1(v1::Message::UnregisteredEventUpdate(
            v1::UnregisteredEventUpdate {
                invitee: invitee.into(),
                event: event.into(),
                event_exception: event_exception.map(Into::into),
                inviter: inviter.into(),
            },
        ))
    }

    /// Creates an event update MailTask for an external invitee
    pub fn external_event_update<E, EE, I, U>(
        inviter: I,
        event: E,
        event_exception: Option<EE>,
        invitee: U,
    ) -> MailTask
    where
        I: Into<v1::RegisteredUser>,
        E: Into<v1::Event>,
        EE: Into<v1::EventException>,
        U: Into<v1::ExternalUser>,
    {
        Self::V1(v1::Message::ExternalEventUpdate(v1::ExternalEventUpdate {
            invitee: invitee.into(),
            event: event.into(),
            event_exception: event_exception.map(Into::into),
            inviter: inviter.into(),
        }))
    }

    /// Creates an event cancellation MailTask for a registered invitee
    pub fn registered_event_cancellation<E, I, U>(inviter: I, event: E, invitee: U) -> MailTask
    where
        I: Into<v1::RegisteredUser>,
        E: Into<v1::Event>,
        U: Into<v1::RegisteredUser>,
    {
        Self::V1(v1::Message::RegisteredEventCancellation(
            v1::RegisteredEventCancellation {
                invitee: invitee.into(),
                event: event.into(),
                inviter: inviter.into(),
            },
        ))
    }

    /// Creates an event cancellation MailTask for an unregistered invitee
    pub fn unregistered_event_cancellation<E, I, U>(inviter: I, event: E, invitee: U) -> MailTask
    where
        I: Into<v1::RegisteredUser>,
        E: Into<v1::Event>,
        U: Into<v1::UnregisteredUser>,
    {
        Self::V1(v1::Message::UnregisteredEventCancellation(
            v1::UnregisteredEventCancellation {
                invitee: invitee.into(),
                event: event.into(),
                inviter: inviter.into(),
            },
        ))
    }

    /// Creates an event cancellation MailTask for an external invitee
    pub fn external_event_cancellation<E, I, U>(inviter: I, event: E, invitee: U) -> MailTask
    where
        I: Into<v1::RegisteredUser>,
        E: Into<v1::Event>,
        U: Into<v1::ExternalUser>,
    {
        Self::V1(v1::Message::ExternalEventCancellation(
            v1::ExternalEventCancellation {
                invitee: invitee.into(),
                event: event.into(),
                inviter: inviter.into(),
            },
        ))
    }

    /// Creates an event uninvite MailTask for a registered invitee
    pub fn registered_event_uninvite<E, I, U>(inviter: I, event: E, invitee: U) -> MailTask
    where
        I: Into<v1::RegisteredUser>,
        E: Into<v1::Event>,
        U: Into<v1::RegisteredUser>,
    {
        Self::V1(v1::Message::RegisteredEventUninvite(
            v1::RegisteredEventUninvite {
                invitee: invitee.into(),
                event: event.into(),
                inviter: inviter.into(),
            },
        ))
    }

    /// Creates an event uninvite MailTask for an unregistered invitee
    pub fn unregistered_event_uninvite<E, I, U>(inviter: I, event: E, invitee: U) -> MailTask
    where
        I: Into<v1::RegisteredUser>,
        E: Into<v1::Event>,
        U: Into<v1::UnregisteredUser>,
    {
        Self::V1(v1::Message::UnregisteredEventUninvite(
            v1::UnregisteredEventUninvite {
                invitee: invitee.into(),
                event: event.into(),
                inviter: inviter.into(),
            },
        ))
    }

    /// Creates an event uninvite MailTask for an external invitee
    pub fn external_event_uninvite<E, I, U>(inviter: I, event: E, invitee: U) -> MailTask
    where
        I: Into<v1::RegisteredUser>,
        E: Into<v1::Event>,
        U: Into<v1::ExternalUser>,
    {
        Self::V1(v1::Message::ExternalEventUninvite(
            v1::ExternalEventUninvite {
                invitee: invitee.into(),
                event: event.into(),
                inviter: inviter.into(),
            },
        ))
    }

    pub fn as_kind_str(&self) -> &'static str {
        match self {
            MailTask::V1(message) => match message {
                // Invites
                v1::Message::RegisteredEventInvite(_) => "registered_invite",
                v1::Message::UnregisteredEventInvite(_) => "unregistered_invite",
                v1::Message::ExternalEventInvite(_) => "external_invite",
                // Updates
                v1::Message::RegisteredEventUpdate(_) => "registered_update",
                v1::Message::UnregisteredEventUpdate(_) => "unregistered_update",
                v1::Message::ExternalEventUpdate(_) => "external_update",
                // Cancellations
                v1::Message::RegisteredEventCancellation(_) => "registered_cancellation",
                v1::Message::UnregisteredEventCancellation(_) => "unregistered_cancellation",
                v1::Message::ExternalEventCancellation(_) => "external_cancellation",
                // Uninvites
                v1::Message::RegisteredEventUninvite(_) => "registered_uninvite",
                v1::Message::UnregisteredEventUninvite(_) => "unregistered_uninvite",
                v1::Message::ExternalEventUninvite(_) => "external_uninvite",
            },
        }
    }
}

#[cfg(feature = "client")]
impl From<String> for v1::ExternalUser {
    fn from(email: String) -> Self {
        Self {
            email: email.into(),
        }
    }
}

#[cfg(feature = "client")]
impl
    From<(
        chrono::DateTime<chrono::Utc>,
        opentalk_types_common::time::TimeZone,
    )> for v1::Time
{
    fn from(
        (time, timezone): (
            chrono::DateTime<chrono::Utc>,
            opentalk_types_common::time::TimeZone,
        ),
    ) -> Self {
        v1::Time {
            time,
            timezone: timezone.to_string(),
        }
    }
}
