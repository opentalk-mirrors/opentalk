// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use snafu::Snafu;
use url::Url;

use crate::settings_runtime::{MAX_ALIAS_SUFFIX_LENGTH, MIN_ALIAS_SUFFIX_LENGTH};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum SettingsError {
    #[snafu(display("Couldn't find a configuration file. Searched: \"{}\".", paths.join("\", \"")))]
    ConfigurationFileNotFound { paths: Vec<String> },

    #[snafu(display("Failed to read data as config: {}", source), context(false))]
    BuildConfig { source: config::ConfigError },

    #[snafu(display("Failed to apply configuration from {} or environment", file_path.to_string_lossy()))]
    DeserializeConfig {
        file_path: PathBuf,

        #[snafu(source(from(serde_path_to_error::Error<config::ConfigError>, Box::new)))]
        source: Box<serde_path_to_error::Error<config::ConfigError>>,
    },

    #[snafu(display("Given base URL is not a base: {}", url))]
    NotBaseUrl { url: Url },

    #[snafu(display(
        "Inconsistent configuration for OIDC and user search, check [keycloak], [endpoints], [oidc] and [user_search] sections"
    ))]
    InconsistentOidcAndUserSearchConfig,

    #[snafu(display(
        "Found a {conflicting_field} configuration value which is not allowed when a [oidc] section is configured"
    ))]
    OidcInvalidConfiguration { conflicting_field: &'static str },

    #[snafu(display(
        "Missing OIDC configuration. Either an [oidc] or a deprecated [keycloak] section must be present in the configuration"
    ))]
    OidcConfigurationMissing,

    #[snafu(display(
        "Found `user_search.users_find_behavior` value `from_user_search_backend`, but no user search backend is configured"
    ))]
    UsersFindBehaviorBackendMissing,

    #[snafu(display(
        "Missing `tokens_per_second` field in the `websocket_rate_limit` configuration"
    ))]
    RateLimitTokensPerSecondMissing,

    #[snafu(display(
        "Missing `token_bucket_size` field in the `websocket_rate_limit` configuration"
    ))]
    RateLimitTokenBucketSizeMissing,

    #[snafu(display("Missing `service_api_keys` field in the `http` configuration"))]
    HttpServiceApiKeysMissing,

    #[snafu(display("Missing mandatory roomserver module(s): \"{}\".", modules.join("\", \"")))]
    MandatoryModulesMissing { modules: Vec<String> },

    #[snafu(display(
        "Invalid room alias suffix length: {length}. Must be between {MIN_ALIAS_SUFFIX_LENGTH} and {MAX_ALIAS_SUFFIX_LENGTH}.",
    ))]
    InvalidRoomAliasSuffixLength { length: u8 },
}
