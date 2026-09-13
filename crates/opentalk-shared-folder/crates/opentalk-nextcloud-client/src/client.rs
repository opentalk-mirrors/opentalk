// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use log::warn;
use reqwest::StatusCode;
use reqwest_dav as dav;
use url::Url;

use crate::{
    types::{OcsPassword, ShareResponse},
    Error, Result, ShareCreator, ShareId, ShareType, ShareUpdater,
};

#[derive(Clone)]
pub struct Client {
    pub(crate) inner: Arc<ClientRef>,
}

pub(crate) struct ClientRef {
    pub(crate) dav_client: dav::Client,
    pub(crate) http_client: reqwest::Client,
    pub(crate) base_url: Url,
    pub(crate) username: String,
    pub(crate) password: String,
}

impl Client {
    pub fn new(base_url: Url, username: String, password: String) -> Result<Self> {
        let dav_url = base_url.join("remote.php/dav")?;
        let mut http_headers = reqwest::header::HeaderMap::new();
        http_headers.insert(
            "OCS-APIRequest",
            reqwest::header::HeaderValue::from_static("true"),
        );
        http_headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        Ok(Self {
            inner: Arc::new(ClientRef {
                dav_client: dav::ClientBuilder::new()
                    .set_host(dav_url.to_string())
                    .set_auth(dav::Auth::Basic(username.clone(), password.clone()))
                    .build()?,
                http_client: reqwest::ClientBuilder::new()
                    .default_headers(http_headers)
                    .build()?,
                base_url,
                username,
                password,
            }),
        })
    }

    pub async fn create_folder(&self, path: &str) -> Result<()> {
        self.inner.dav_client.mkcol(path).await?;
        Ok(())
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        if let Err(e) = self.inner.dav_client.delete(path).await {
            return Err(Error::FileNotFound {
                file_path: path.to_owned(),
                source: e,
            });
        }

        Ok(())
    }

    pub fn create_share(&self, path: &str, share_type: ShareType) -> ShareCreator {
        ShareCreator::new(self.clone(), path.to_string(), share_type)
    }

    pub fn update_share(&self, id: ShareId) -> ShareUpdater {
        ShareUpdater::new(self.clone(), id)
    }

    pub async fn delete_share(&self, share_id: ShareId) -> Result<()> {
        let url = self
            .share_api_base_url()?
            .join("shares/")?
            .join(share_id.as_str())?;

        let request = self
            .inner
            .http_client
            .delete(url)
            .basic_auth(&self.inner.username, Some(&self.inner.password));
        let response = request.send().await?;

        match response.status() {
            StatusCode::CONTINUE | StatusCode::OK => {}
            StatusCode::UNAUTHORIZED => {
                // 401
                return Err(Error::Unauthorized);
            }
            StatusCode::NOT_FOUND => {
                // 404
                return Err(Error::ShareNotFound { share_id });
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
        Ok(())
    }

    pub async fn generate_password(&self) -> Result<String> {
        let url = self.password_policy_base_url()?.join("generate")?;

        let request = self
            .inner
            .http_client
            .get(url)
            .basic_auth(&self.inner.username, Some(&self.inner.password));

        let response = request.send().await?;

        match response.status() {
            StatusCode::CONTINUE | StatusCode::OK => {}
            StatusCode::UNAUTHORIZED => {
                // 401
                return Err(Error::Unauthorized);
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

        let response: ShareResponse<OcsPassword> = response.json().await?;

        Ok(response.ocs.data.password)
    }

    pub(crate) fn password_policy_base_url(&self) -> Result<Url> {
        Ok(self
            .inner
            .base_url
            .join("ocs/v2.php/apps/password_policy/api/v1/")?)
    }

    pub(crate) fn share_api_base_url(&self) -> Result<Url> {
        Ok(self
            .inner
            .base_url
            .join("ocs/v2.php/apps/files_sharing/api/v1/")?)
    }
}
