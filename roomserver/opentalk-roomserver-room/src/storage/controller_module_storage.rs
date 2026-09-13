// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::Context;
use async_trait::async_trait;
use opentalk_roomserver_signaling::storage::module_resources::{
    Error, provider::ModuleResourceProvider,
};
use opentalk_service_auth::ApiKey;
use opentalk_types_api_internal::module_resources::{
    ModuleResource, ModuleResourceFilter, ModuleResourceOperation, NewModuleResource,
    PatchModuleResourceBody,
};
use reqwest::{Response, header::CONTENT_TYPE};
use serde::de::DeserializeOwned;
use url::Url;

const MAX_BODY_PREVIEW_LENGTH: usize = 80;

#[derive(Debug)]
pub struct ControllerModuleStorage {
    base_url: Url,
    api_key: ApiKey,
    client: reqwest::Client,
}

impl ControllerModuleStorage {
    pub fn new(base_url: Url, api_key: ApiKey) -> Self {
        Self {
            base_url,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    fn module_resource_endpoint(&self) -> anyhow::Result<Url> {
        self.base_url
            .join("/internal/module_resources")
            .context("Invalid URL")
    }
}

#[async_trait]
impl ModuleResourceProvider for ControllerModuleStorage {
    async fn create(&self, resource: NewModuleResource) -> Result<ModuleResource, Error> {
        let url = self.module_resource_endpoint()?;

        let new_resource_body =
            serde_json::to_vec(&resource).context("Failed to serialize POST request body")?;

        let token = self
            .api_key
            .generate_jwt()
            .context("Failed to generate auth header")?;

        let response = self
            .client
            .post(url)
            .body(new_resource_body)
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(token)
            .send()
            .await
            .context("Failed to send POST module resource request")?;

        let module_resource = parse_response_body(response)
            .await
            .context("Failed to create module resource")?;

        Ok(module_resource)
    }

    async fn get(&self, filter: ModuleResourceFilter) -> Result<Vec<ModuleResource>, Error> {
        let url = self.module_resource_endpoint()?;

        let request_body =
            serde_json::to_vec(&filter).context("Failed to serialize GET request body")?;

        let token = self
            .api_key
            .generate_jwt()
            .context("Failed to generate auth header")?;

        let response = self
            .client
            .get(url)
            .body(request_body)
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(token)
            .send()
            .await
            .context("Failed to send GET module resource request")?;

        let module_resources = parse_response_body(response)
            .await
            .context("Failed to get module resource")?;

        Ok(module_resources)
    }

    async fn patch(
        &self,
        filter: ModuleResourceFilter,
        operations: Vec<ModuleResourceOperation>,
    ) -> Result<Vec<ModuleResource>, Error> {
        let url = self.module_resource_endpoint()?;

        let patch_request = PatchModuleResourceBody {
            filter,
            patch_operations: operations,
        };

        let request_body =
            serde_json::to_vec(&patch_request).context("Failed to serialize PATCH request body")?;

        let token = self
            .api_key
            .generate_jwt()
            .context("Failed to generate auth header")?;

        let response = self
            .client
            .patch(url)
            .body(request_body)
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(token)
            .send()
            .await
            .context("Failed to send PATCH module resource request")?;

        let module_resources = parse_response_body(response)
            .await
            .context("Failed to update module resource")?;

        Ok(module_resources)
    }

    async fn delete(&self, filter: ModuleResourceFilter) -> Result<Vec<ModuleResource>, Error> {
        let url = self.module_resource_endpoint()?;

        let request_body =
            serde_json::to_vec(&filter).context("Failed to serialize DELETE request body")?;

        let token = self
            .api_key
            .generate_jwt()
            .context("Failed to generate auth header")?;

        let response = self
            .client
            .delete(url)
            .body(request_body)
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(token)
            .send()
            .await
            .context("Failed to send DELETE module resource request")?;

        let deleted_resources = parse_response_body(response).await?;

        Ok(deleted_resources)
    }
}

async fn parse_response_body<T: DeserializeOwned>(response: Response) -> anyhow::Result<T> {
    let status = response.status();
    let body = response
        .bytes()
        .await
        .context("Failed to read response body")?;

    if !status.is_success() {
        let error_body = match std::str::from_utf8(&body) {
            Ok(s) if s.len() > MAX_BODY_PREVIEW_LENGTH => {
                format!("{}...[truncated]", &s[..MAX_BODY_PREVIEW_LENGTH])
            }
            Ok(s) => s.to_string(),
            Err(_) => "<non-utf8>".to_string(),
        };

        return Err(anyhow::anyhow!(
            "Received non-success response with status: `{}` Body: `{}`",
            status,
            error_body,
        ));
    }

    let success_body: T = serde_json::from_slice(&body).with_context(|| {
        format!(
            "Failed to parse upload response body: {:?}",
            std::str::from_utf8(&body).unwrap_or("<non-utf8>"),
        )
    })?;

    Ok(success_body)
}
