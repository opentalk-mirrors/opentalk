// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Body for the `DELETE /events/{event_id}/shared_folder` endpoint
#[derive(Default, Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct DeleteEventsQuery {
    /// Flag to force delete the reference if the deletion at the external services fails
    #[cfg_attr(feature = "serde", serde(default))]
    pub force_delete_reference_if_external_services_fail: bool,

    /// Flag to disable email notification
    #[cfg_attr(feature = "serde", serde(default))]
    pub suppress_email_notification: bool,
}
