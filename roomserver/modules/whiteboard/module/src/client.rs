// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::marker::Unpin;

use anyhow::bail;
use bytes::Bytes;
use futures::Stream;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

const API_TOKEN_HEADER: &str = "x-spacedeck-api-token";

#[derive(Clone)]
/// The client for the spacedeck API.
pub(crate) struct SpacedeckClient {
    /// The reqwest client.
    client: Client,
    /// The base url of the spacedeck instance.
    pub(crate) base_url: Url,
    /// Token for API requests.
    api_token: String,
}

/// Request body for POST /api/spaces.
#[derive(Debug, Serialize)]
struct PostSpacesRequest<'s> {
    artifacts: Vec<&'s str>,
    name: &'s str,
    parent_space_id: Option<&'s str>,
    space_type: &'s str,
}

/// Response body for POST /api/spaces.
///
/// A lot of irrelevant fields are omitted.
#[derive(Debug, Deserialize)]
pub(crate) struct PostSpacesResponse {
    #[serde(rename = "_id")]
    pub(crate) id: String,
    pub(crate) edit_hash: String,
    pub(crate) edit_slug: String,
}

/// Response body for GET /spaces/{id}/pdf.
#[derive(Debug, Deserialize)]
pub(crate) struct GetPdfResponse {
    pub(crate) url: String,
}

impl SpacedeckClient {
    /// Create a new spacedeck client.
    pub(crate) fn new(base_url: Url, api_key: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_token: api_key,
        }
    }

    // Create a new space.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn create_space(
        &self,
        name: &str,
        parent_id: Option<&str>,
    ) -> anyhow::Result<PostSpacesResponse> {
        let url = self.base_url.join("api/spaces")?;

        let body = PostSpacesRequest {
            artifacts: vec![],
            name,
            parent_space_id: parent_id,
            space_type: "space",
        };

        let response = self
            .client
            .post(url)
            .header(API_TOKEN_HEADER, &self.api_token)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            bail!("Failed to create space, status code: {}", response.status())
        }

        let response = response.json().await?;

        Ok(response)
    }

    /// Generates the current whiteboard as PDF document.
    ///
    /// Returns the URL to the document.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn get_pdf(&self, id: &str) -> anyhow::Result<Url> {
        let url = self.base_url.join(&format!("api/spaces/{id}/pdf"))?;

        let response = self
            .client
            .get(url)
            .header(API_TOKEN_HEADER, &self.api_token)
            .send()
            .await?;

        if !response.status().is_success() {
            bail!(
                "Failed to get space `{id}` as PDF document, status code: {}",
                response.status()
            )
        }

        let response: GetPdfResponse = response.json().await?;

        let url = self.base_url.join(response.url.as_str())?;

        Ok(url)
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn download_pdf(
        &self,
        pdf_url: Url,
    ) -> anyhow::Result<impl Stream<Item = reqwest::Result<Bytes>> + Unpin + use<>> {
        let response = self.client.get(pdf_url).send().await?;

        if !response.status().is_success() {
            bail!(
                "Failed to get binary pdf data, status code: {}",
                response.status()
            )
        }

        Ok(response.bytes_stream())
    }

    /// Delete the space with the provided `id`.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn delete_space(&self, id: &str) -> anyhow::Result<()> {
        let url = self.base_url.join(&format!("api/spaces/{id}"))?;

        let response = self
            .client
            .delete(url)
            .header(API_TOKEN_HEADER, &self.api_token)
            .send()
            .await?;

        if !response.status().is_success() {
            bail!(
                "Failed to delete space `{}`, status code: {}",
                id,
                response.status()
            )
        }

        Ok(())
    }
}
