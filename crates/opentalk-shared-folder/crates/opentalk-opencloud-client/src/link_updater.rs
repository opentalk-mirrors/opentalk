// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use reqwest::{RequestBuilder, StatusCode};
use serde::Serialize;

use crate::{
    client::error_for_status, types::Permission, Client, DriveId, Error, ItemId, PermissionId,
    Result, SharingLinkType,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkTypeUpdate {
    link: LinkTypeField,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkTypeField {
    #[serde(rename = "type")]
    link_type: SharingLinkType,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExpirationUpdate {
    expiration_date_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SetPasswordBody {
    password: String,
}

/// Builder for updating an existing share link, created via
/// [`Client::update_link`].
///
/// Each method performs a single request and consumes the updater.
///
/// [`Client::update_link`]: crate::Client::update_link
#[must_use]
pub struct LinkUpdater {
    client: Client,
    drive_id: DriveId,
    item_id: ItemId,
    permission_id: PermissionId,
}

impl LinkUpdater {
    pub(crate) fn new(
        client: Client,
        drive_id: DriveId,
        item_id: ItemId,
        permission_id: PermissionId,
    ) -> Self {
        Self {
            client,
            drive_id,
            item_id,
            permission_id,
        }
    }

    /// Changes the type of the link.
    pub async fn link_type(self, link_type: SharingLinkType) -> Result<Permission> {
        let url = self
            .client
            .permission_url(&self.drive_id, &self.item_id, &self.permission_id)?;
        let request = self
            .client
            .inner
            .http_client
            .patch(url)
            .basic_auth(
                &self.client.inner.username,
                Some(&self.client.inner.password),
            )
            .json(&LinkTypeUpdate {
                link: LinkTypeField { link_type },
            });
        self.execute(request).await
    }

    /// Sets or clears the expiration date of the link. Passing `None` removes
    /// the expiration.
    pub async fn expiration(self, expiration: Option<DateTime<Utc>>) -> Result<Permission> {
        let url = self
            .client
            .permission_url(&self.drive_id, &self.item_id, &self.permission_id)?;
        let request = self
            .client
            .inner
            .http_client
            .patch(url)
            .basic_auth(
                &self.client.inner.username,
                Some(&self.client.inner.password),
            )
            .json(&ExpirationUpdate {
                expiration_date_time: expiration,
            });
        self.execute(request).await
    }

    /// Sets the password protecting the link.
    pub async fn password<P: Into<String>>(self, password: P) -> Result<Permission> {
        let url =
            self.client
                .set_password_url(&self.drive_id, &self.item_id, &self.permission_id)?;
        let request = self
            .client
            .inner
            .http_client
            .post(url)
            .basic_auth(
                &self.client.inner.username,
                Some(&self.client.inner.password),
            )
            .json(&SetPasswordBody {
                password: password.into(),
            });
        self.execute(request).await
    }

    async fn execute(self, request: RequestBuilder) -> Result<Permission> {
        let response = request.send().await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(Error::PermissionNotFound {
                permission_id: self.permission_id,
            });
        }
        let response = error_for_status(response).await?;
        Ok(response.json().await?)
    }
}
