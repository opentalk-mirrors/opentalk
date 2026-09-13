// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::SharingLinkType;

/// Options for creating a share link via [`Client::create_link`].
///
/// [`Client::create_link`]: crate::Client::create_link
#[derive(Debug, Clone, Default)]
pub struct CreateLinkOptions {
    /// The type of the link. Defaults to [`SharingLinkType::default`].
    pub link_type: SharingLinkType,
    /// The point in time at which the link expires.
    pub expiration: Option<DateTime<Utc>>,
    /// A password protecting the link.
    pub password: Option<String>,
    /// A display name for the link.
    pub display_name: Option<String>,
    /// Whether the link is a quick link.
    pub quick_link: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateLinkBody {
    #[serde(rename = "type")]
    link_type: SharingLinkType,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration_date_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    #[serde(
        rename = "@libre.graph.quickLink",
        skip_serializing_if = "std::ops::Not::not"
    )]
    quick_link: bool,
}

impl From<CreateLinkOptions> for CreateLinkBody {
    fn from(options: CreateLinkOptions) -> Self {
        let CreateLinkOptions {
            link_type,
            expiration,
            password,
            display_name,
            quick_link,
        } = options;
        Self {
            link_type,
            expiration_date_time: expiration,
            password,
            display_name,
            quick_link,
        }
    }
}
