// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::{
    ClientId, ClientSecret, EndpointMaybeSet, EndpointNotSet, EndpointSet, IntrospectionUrl,
    IssuerUrl, core::CoreClient, url::Url,
};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, whatever};

use crate::Result;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdditionalProviderMetadata {
    pub introspection_endpoint: Option<Url>,
}

impl openidconnect::AdditionalProviderMetadata for AdditionalProviderMetadata {}

type ProviderMetadata = openidconnect::ProviderMetadata<
    AdditionalProviderMetadata,
    openidconnect::core::CoreAuthDisplay,
    openidconnect::core::CoreClientAuthMethod,
    openidconnect::core::CoreClaimName,
    openidconnect::core::CoreClaimType,
    openidconnect::core::CoreGrantType,
    openidconnect::core::CoreJweContentEncryptionAlgorithm,
    openidconnect::core::CoreJweKeyManagementAlgorithm,
    openidconnect::core::CoreJsonWebKey,
    openidconnect::core::CoreResponseMode,
    openidconnect::core::CoreResponseType,
    openidconnect::core::CoreSubjectIdentifierType,
>;

/// Contains all structures necessary to talk to a single configured OIDC Provider.
#[derive(Debug)]
pub struct ProviderClient {
    pub metadata: ProviderMetadata,
    pub client: CoreClient<
        // HasAuthUrl
        EndpointSet,
        // HasDeviceAuthUrl
        EndpointNotSet,
        // HasIntrospectionUrl
        EndpointSet,
        // HasRevocationUrl
        EndpointNotSet,
        // HasTokenUrl
        EndpointMaybeSet,
        // HasUserInfoUrl
        EndpointMaybeSet,
    >,
}

impl ProviderClient {
    /// Discover Provider information from given settings
    pub async fn discover(
        http_client: opentalk_keycloak_admin::reqwest::ClientWrapper,
        auth_base_url: Url,
        client_id: ClientId,
        client_secret: ClientSecret,
    ) -> Result<ProviderClient> {
        let metadata =
            ProviderMetadata::discover_async(IssuerUrl::from_url(auth_base_url), &http_client)
                .await
                .whatever_context("Failed to discover provider metadata")?;

        // Require the userinfo endpoint
        if metadata.userinfo_endpoint().is_none() {
            whatever!("OpenID Connect provider is missing the 'userinfo' endpoint");
        }

        // Require the introspection endpoint
        let Some(introspection_url) = metadata
            .additional_metadata()
            .introspection_endpoint
            .clone()
        else {
            whatever!("OpenID Connect provider is missing the 'introspection' endpoint");
        };

        let client = CoreClient::from_provider_metadata(
            metadata.clone(),
            client_id.clone(),
            Some(client_secret),
        );

        let client = client.set_introspection_url(IntrospectionUrl::from_url(introspection_url));

        Ok(ProviderClient { metadata, client })
    }
}
