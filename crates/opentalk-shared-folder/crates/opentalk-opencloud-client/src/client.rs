// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use futures::{stream, StreamExt as _, TryStreamExt as _};
use log::warn;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use reqwest::{Response, StatusCode};
use url::Url;

use crate::{
    link_creator::CreateLinkBody,
    types::{
        CreatedShareLink, Drive, DriveItem, DriveItemList, DriveList, OcsCapabilitiesEnvelope,
        PasswordPolicy, Permission,
    },
    CreateLinkOptions, DriveId, Error, ItemId, LinkUpdater, PermissionId, Result,
};

/// Characters that must be percent-encoded in a single path segment.
///
/// In addition to the control characters and the usual unsafe characters this
/// also encodes `/` (so segment boundaries are preserved), `%` (to avoid
/// double-encoding) and `$` (which appears in OpenCloud drive identifiers and
/// must be sent as `%24`).
const PATH_SEGMENT_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}')
    .add(b'/')
    .add(b'%')
    .add(b'$')
    .add(b'\\');

fn encode_segment(segment: &str) -> String {
    utf8_percent_encode(segment, PATH_SEGMENT_ENCODE_SET).to_string()
}

fn encode_path(path: &str) -> String {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .map(encode_segment)
        .collect::<Vec<_>>()
        .join("/")
}

/// Maps the status code of a response to an [`Error`] for the common cases,
/// returning the response unchanged on success.
pub(crate) async fn error_for_status(response: Response) -> Result<Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    match status {
        StatusCode::UNAUTHORIZED => Err(Error::Unauthorized),
        StatusCode::FORBIDDEN => Err(Error::Forbidden),
        StatusCode::BAD_REQUEST => {
            let message = response.text().await.unwrap_or_default();
            Err(Error::BadRequest { message })
        }
        status_code => {
            match response.text().await {
                Ok(text) => warn!("Unexpected status code {status_code} from OpenCloud:\n{text}"),
                Err(e) => warn!("Error retrieving body from OpenCloud: {e}"),
            }
            Err(Error::UnexpectedStatusCode { status_code })
        }
    }
}

#[derive(Clone)]
pub struct Client {
    pub(crate) inner: Arc<ClientRef>,
}

pub(crate) struct ClientRef {
    pub(crate) http_client: reqwest::Client,
    pub(crate) base_url: Url,
    pub(crate) username: String,
    pub(crate) password: String,
}

impl Client {
    /// Creates a new client for the OpenCloud instance at `base_url`,
    /// authenticating with HTTP Basic auth using `username` and `password`
    /// (the password may be an app token).
    pub fn new(base_url: Url, username: String, password: String) -> Result<Self> {
        let mut http_headers = reqwest::header::HeaderMap::new();
        http_headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        Ok(Self {
            inner: Arc::new(ClientRef {
                http_client: reqwest::ClientBuilder::new()
                    .default_headers(http_headers)
                    .build()?,
                base_url,
                username,
                password,
            }),
        })
    }

    /// Lists the drives (spaces) the authenticated user has access to.
    pub async fn get_drives(&self) -> Result<Vec<Drive>> {
        let url = self.graph_url("v1.0", "me/drives")?;
        let response = self
            .inner
            .http_client
            .get(url)
            .basic_auth(&self.inner.username, Some(&self.inner.password))
            .send()
            .await?;
        let response = error_for_status(response).await?;
        let list: DriveList = response.json().await?;
        Ok(list.value)
    }

    /// Returns the personal drive (space) of the authenticated user.
    pub async fn get_personal_drive(&self) -> Result<Drive> {
        let url = self.graph_url("v1.0", "me/drive")?;
        let response = self
            .inner
            .http_client
            .get(url)
            .basic_auth(&self.inner.username, Some(&self.inner.password))
            .send()
            .await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(Error::PersonalDriveNotFound);
        }
        let response = error_for_status(response).await?;
        let drive: Drive = response.json().await?;
        Ok(drive)
    }

    /// Returns the password policy that share-link passwords must conform to,
    /// queried from the OCS capabilities endpoint.
    pub async fn get_password_policy(&self) -> Result<PasswordPolicy> {
        let url = self.capabilities_url()?;
        let response = self
            .inner
            .http_client
            .get(url)
            .basic_auth(&self.inner.username, Some(&self.inner.password))
            .send()
            .await?;
        let response = error_for_status(response).await?;
        let envelope: OcsCapabilitiesEnvelope = response.json().await?;
        Ok(envelope
            .ocs
            .data
            .capabilities
            .password_policy
            .unwrap_or_default())
    }

    /// Generates a random password that conforms to the server's password
    /// policy, suitable for protecting a share link.
    pub async fn generate_password(&self) -> Result<String> {
        let policy = self.get_password_policy().await?;
        crate::password::generate(&policy)
    }

    /// Creates a folder at `path` inside the drive `drive_id`.
    ///
    /// Intermediate folders are created as needed. Folders that already exist
    /// are left untouched.
    pub async fn create_folder(&self, drive_id: &DriveId, path: &str) -> Result<()> {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = String::new();
        for segment in segments {
            if !current.is_empty() {
                current.push('/');
            }
            current.push_str(segment);
            self.mkcol(drive_id, &current).await?;
        }
        Ok(())
    }

    /// Deletes the folder (or file) at `path` inside the drive `drive_id`.
    pub async fn delete_folder(&self, drive_id: &DriveId, path: &str) -> Result<()> {
        let url = self.dav_url(drive_id, path)?;
        let response = self
            .inner
            .http_client
            .delete(url)
            .basic_auth(&self.inner.username, Some(&self.inner.password))
            .send()
            .await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(Error::FolderNotFound {
                path: path.to_owned(),
            });
        }
        error_for_status(response).await?;
        Ok(())
    }

    /// Resolves a `path` inside the drive `drive_id` to the corresponding
    /// [`DriveItem`].
    ///
    /// An empty path resolves to the drive root, whose item id equals the
    /// drive id.
    pub async fn resolve_item(&self, drive_id: &DriveId, path: &str) -> Result<DriveItem> {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        let root = DriveItem {
            id: ItemId::from(drive_id.as_str().to_owned()),
            name: String::new(),
        };
        stream::iter(segments)
            .map(Ok)
            .try_fold(root, |acc, segment| async move {
                let children = self.list_children(drive_id, &acc.id).await?;
                children
                    .into_iter()
                    .find(|child| child.name == segment)
                    .ok_or_else(|| Error::ItemNotFound {
                        path: path.to_owned(),
                    })
            })
            .await
    }

    /// Creates a share link for the folder at `path` inside the drive
    /// `drive_id`.
    pub async fn create_link(
        &self,
        drive_id: &DriveId,
        path: &str,
        options: CreateLinkOptions,
    ) -> Result<CreatedShareLink> {
        let item = self.resolve_item(drive_id, path).await?;
        let url = self.create_link_url(drive_id, &item.id)?;
        let body = CreateLinkBody::from(options);
        let response = self
            .inner
            .http_client
            .post(url)
            .basic_auth(&self.inner.username, Some(&self.inner.password))
            .json(&body)
            .send()
            .await?;
        let response = error_for_status(response).await?;
        let permission: Permission = response.json().await?;
        Ok(CreatedShareLink {
            item_id: item.id,
            permission,
        })
    }

    /// Starts updating an existing share link identified by `permission_id` on
    /// the item `item_id` inside the drive `drive_id`.
    pub fn update_link(
        &self,
        drive_id: DriveId,
        item_id: ItemId,
        permission_id: PermissionId,
    ) -> LinkUpdater {
        LinkUpdater::new(self.clone(), drive_id, item_id, permission_id)
    }

    /// Deletes the share link identified by `permission_id` on the item
    /// `item_id` inside the drive `drive_id`.
    pub async fn delete_link(
        &self,
        drive_id: &DriveId,
        item_id: &ItemId,
        permission_id: &PermissionId,
    ) -> Result<()> {
        let url = self.permission_url(drive_id, item_id, permission_id)?;
        let response = self
            .inner
            .http_client
            .delete(url)
            .basic_auth(&self.inner.username, Some(&self.inner.password))
            .send()
            .await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(Error::PermissionNotFound {
                permission_id: permission_id.clone(),
            });
        }
        error_for_status(response).await?;
        Ok(())
    }

    async fn list_children(&self, drive_id: &DriveId, item_id: &ItemId) -> Result<Vec<DriveItem>> {
        let url = self.graph_url(
            "v1.0",
            &format!(
                "drives/{}/items/{}/children",
                encode_segment(drive_id.as_str()),
                encode_segment(item_id.as_str())
            ),
        )?;
        let response = self
            .inner
            .http_client
            .get(url)
            .basic_auth(&self.inner.username, Some(&self.inner.password))
            .send()
            .await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(Error::ItemNotFound {
                path: item_id.as_str().to_owned(),
            });
        }
        let response = error_for_status(response).await?;
        let list: DriveItemList = response.json().await?;
        Ok(list.value)
    }

    async fn mkcol(&self, drive_id: &DriveId, path: &str) -> Result<()> {
        let url = self.dav_url(drive_id, path)?;
        let method = reqwest::Method::from_bytes(b"MKCOL").expect("MKCOL is a valid HTTP method");
        let response = self
            .inner
            .http_client
            .request(method, url)
            .basic_auth(&self.inner.username, Some(&self.inner.password))
            .send()
            .await?;
        match response.status() {
            // The folder already exists, which we treat as success.
            StatusCode::METHOD_NOT_ALLOWED => Ok(()),
            status if status.is_success() => Ok(()),
            _ => {
                error_for_status(response).await?;
                Ok(())
            }
        }
    }

    fn base_url_str(&self) -> &str {
        self.inner.base_url.as_str().trim_end_matches('/')
    }

    fn graph_url(&self, version: &str, suffix: &str) -> Result<Url> {
        let base = self.base_url_str();
        Ok(Url::parse(&format!("{base}/graph/{version}/{suffix}"))?)
    }

    fn capabilities_url(&self) -> Result<Url> {
        let base = self.base_url_str();
        Ok(Url::parse(&format!(
            "{base}/ocs/v1.php/cloud/capabilities?format=json"
        ))?)
    }

    fn dav_url(&self, drive_id: &DriveId, path: &str) -> Result<Url> {
        let base = self.base_url_str();
        let drive = encode_segment(drive_id.as_str());
        let encoded_path = encode_path(path);
        Ok(Url::parse(&format!(
            "{base}/dav/spaces/{drive}/{encoded_path}"
        ))?)
    }

    pub(crate) fn create_link_url(&self, drive_id: &DriveId, item_id: &ItemId) -> Result<Url> {
        self.graph_url(
            "v1beta1",
            &format!(
                "drives/{}/items/{}/createLink",
                encode_segment(drive_id.as_str()),
                encode_segment(item_id.as_str())
            ),
        )
    }

    pub(crate) fn permission_url(
        &self,
        drive_id: &DriveId,
        item_id: &ItemId,
        permission_id: &PermissionId,
    ) -> Result<Url> {
        self.graph_url(
            "v1beta1",
            &format!(
                "drives/{}/items/{}/permissions/{}",
                encode_segment(drive_id.as_str()),
                encode_segment(item_id.as_str()),
                encode_segment(permission_id.as_str())
            ),
        )
    }

    pub(crate) fn set_password_url(
        &self,
        drive_id: &DriveId,
        item_id: &ItemId,
        permission_id: &PermissionId,
    ) -> Result<Url> {
        self.graph_url(
            "v1beta1",
            &format!(
                "drives/{}/items/{}/permissions/{}/setPassword",
                encode_segment(drive_id.as_str()),
                encode_segment(item_id.as_str()),
                encode_segment(permission_id.as_str())
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::{encode_path, encode_segment, Client};
    use crate::DriveId;

    #[test]
    fn encodes_dollar_in_segment() {
        assert_eq!(
            encode_segment("storage-users-1$0000-0000"),
            "storage-users-1%240000-0000"
        );
    }

    #[test]
    fn encodes_path_segments_individually() {
        assert_eq!(encode_path("/My Folder/Sub"), "My%20Folder/Sub");
    }

    #[test]
    fn builds_dav_url() {
        let client = Client::new(
            Url::parse("https://cloud.example.com").unwrap(),
            "user".to_owned(),
            "password".to_owned(),
        )
        .unwrap();
        let drive = DriveId::from("storage-users-1$abc".to_owned());
        let url = client.dav_url(&drive, "Meeting/Notes").unwrap();
        assert_eq!(
            url.as_str(),
            "https://cloud.example.com/dav/spaces/storage-users-1%24abc/Meeting/Notes"
        );
    }

    #[test]
    fn builds_create_link_url() {
        let client = Client::new(
            Url::parse("https://cloud.example.com/").unwrap(),
            "user".to_owned(),
            "password".to_owned(),
        )
        .unwrap();
        let drive = DriveId::from("storage-users-1$abc".to_owned());
        let item = crate::ItemId::from("storage-users-1$abc!item".to_owned());
        let url = client.create_link_url(&drive, &item).unwrap();
        assert_eq!(
            url.as_str(),
            "https://cloud.example.com/graph/v1beta1/drives/storage-users-1%24abc/items/storage-users-1%24abc!item/createLink"
        );
    }
}
