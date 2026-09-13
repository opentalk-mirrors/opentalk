// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashSet;

use chrono::NaiveDate;
use log::warn;
use reqwest::StatusCode;
use serde::Serialize;

use crate::{
    types::{OcsShareData, OcsShareResponse, ShareResponse},
    Client, Error, Result, SharePermission, ShareType,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct Parameters {
    path: String,
    #[serde(with = "crate::utils::share_type")]
    share_type: ShareType,
    #[serde(skip_serializing_if = "Option::is_none")]
    public_upload: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(
        with = "crate::utils::optional_share_permissions",
        skip_serializing_if = "Option::is_none"
    )]
    permissions: Option<HashSet<SharePermission>>,
    #[serde(
        with = "crate::utils::optional_naive_date",
        skip_serializing_if = "Option::is_none"
    )]
    expire_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
}

#[must_use]
pub struct ShareCreator {
    client: Client,
    parameters: Parameters,
}

impl ShareCreator {
    pub(crate) fn new(client: Client, path: String, share_type: ShareType) -> Self {
        Self {
            client,
            parameters: Parameters {
                path,
                share_type,
                ..Default::default()
            },
        }
    }

    pub fn public_upload(mut self, public_upload: bool) -> Self {
        self.parameters.public_upload = Some(public_upload);
        self
    }

    pub fn password<P: Into<String>>(mut self, password: P) -> Self {
        self.parameters.password = Some(password.into());
        self
    }

    pub fn permission(mut self, permission: SharePermission) -> Self {
        self.parameters
            .permissions
            .get_or_insert(Default::default())
            .insert(permission);
        self
    }

    pub fn expire_date(mut self, expire_date: NaiveDate) -> Self {
        self.parameters.expire_date = Some(expire_date);
        self
    }

    pub fn note<N: Into<String>>(mut self, note: N) -> Self {
        self.parameters.note = Some(note.into());
        self
    }

    pub fn label<L: Into<String>>(mut self, label: L) -> Self {
        self.parameters.label = Some(label.into());
        self
    }

    pub async fn send(self) -> Result<OcsShareResponse<OcsShareData>> {
        let Self { client, parameters } = self;

        let url = client.share_api_base_url()?.join("shares")?;
        let request = client
            .inner
            .http_client
            .post(url)
            .basic_auth(&client.inner.username, Some(&client.inner.password))
            .json(&parameters);
        let response = request.send().await?;

        match response.status() {
            StatusCode::CONTINUE | StatusCode::OK => {}
            StatusCode::BAD_REQUEST => {
                // 400
                return Err(Error::UnknownShareType);
            }
            StatusCode::UNAUTHORIZED => {
                // 401
                return Err(Error::Unauthorized);
            }
            StatusCode::FORBIDDEN => {
                // 403
                return Err(Error::PublicUploadDisabledByAdmin);
            }
            StatusCode::NOT_FOUND => {
                // 404
                return Err(Error::FileCouldNotBeShared);
            }
            status_code => {
                warn!("Received unexpected status code {status_code} from NextCloud server.");
                match response.text().await {
                    Ok(text) => {
                        warn!("Response for unexpected status code {status_code}:\n{text}");
                    }
                    Err(e) => {
                        warn!("Error retrieving body from NextCloud: {e}");
                    }
                }
                return Err(Error::UnexpectedStatusCode { status_code });
            }
        }
        let response: ShareResponse<OcsShareData> = response.json().await?;
        Ok(response.ocs)
    }
}
