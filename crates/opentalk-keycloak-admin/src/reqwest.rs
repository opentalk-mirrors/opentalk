// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{future::Future, ops::Deref, pin::Pin};

use openidconnect::{AsyncHttpClient, HttpClientError, HttpRequest, HttpResponse, http};

/// This is a temporary solution to use openidconnect with reqwest 0.13 until upstream provides
/// support for it themselves.
///
/// Circumvents the orphan rule in AsyncHTTPClient implementations
#[derive(Debug, Clone)]
pub struct ClientWrapper(pub reqwest::Client);

impl<'c> AsyncHttpClient<'c> for ClientWrapper {
    type Error = HttpClientError<reqwest::Error>;

    type Future =
        Pin<Box<dyn Future<Output = Result<HttpResponse, Self::Error>> + Send + Sync + 'c>>;

    fn call(&'c self, request: HttpRequest) -> Self::Future {
        Box::pin(async move {
            let response = self
                .0
                .execute(request.try_into().map_err(Box::new)?)
                .await
                .map_err(Box::new)?;

            let mut builder = http::Response::builder()
                .status(response.status())
                .version(response.version());

            for (name, value) in response.headers().iter() {
                builder = builder.header(name, value);
            }

            builder
                .body(response.bytes().await.map_err(Box::new)?.to_vec())
                .map_err(HttpClientError::Http)
        })
    }
}

impl From<reqwest::Client> for ClientWrapper {
    fn from(value: reqwest::Client) -> Self {
        Self(value)
    }
}

impl From<&reqwest::Client> for ClientWrapper {
    fn from(value: &reqwest::Client) -> Self {
        Self(value.clone())
    }
}

impl Deref for ClientWrapper {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
