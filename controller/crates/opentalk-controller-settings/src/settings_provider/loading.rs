// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use snafu::ResultExt as _;

use super::SettingsProvider;
use crate::{Result, SettingsRaw, settings_error::DeserializeConfigSnafu};

impl SettingsProvider {
    pub(super) fn load_raw(file_path: &Path) -> Result<SettingsRaw> {
        use config::{Config, Environment, File, FileFormat};

        let config = Config::builder()
            .add_source(File::from(file_path).format(FileFormat::Toml))
            .add_source(
                Environment::with_prefix("OPENTALK_CTRL")
                    .prefix_separator("_")
                    .separator("__")
                    .list_separator(",")
                    .with_list_parse_key("http.cors.allowed_origin")
                    .with_list_parse_key("http.service_api_keys")
                    .with_list_parse_key("logging.default_directives")
                    .with_list_parse_key("etcd.urls")
                    .try_parsing(true),
            )
            .build()?;

        let mut warn_unknown_key = Self::warn_unused_key;
        let ignored_deserializer = serde_ignored::Deserializer::new(config, &mut warn_unknown_key);
        let settings_raw: SettingsRaw = serde_path_to_error::deserialize(ignored_deserializer)
            .context(DeserializeConfigSnafu {
                file_path: file_path.to_owned(),
            })?;
        Self::warn_about_deprecated_items(&settings_raw);

        Ok(settings_raw)
    }

    fn warn_unused_key(path: serde_ignored::Path) {
        use owo_colors::OwoColorize as _;

        let path = path.to_string();
        match path.as_str() {
            "reports" => {
                anstream::eprintln!(
                    "{}: Found an obsolete {reports} configuration section.\n\
                     \t{}: This section is deprecated and will be reintroduced in a different form in the future.",
                    "DEPRECATION WARNING".yellow().bold(),
                    "NOTE".green(),
                    reports = "reports".bold(),
                );
            }
            "room_server" => {
                anstream::eprintln!(
                    "{}: Found an obsolete {room_server} (janus) configuration section.\n\
                     \t{}: This section is no longer needed, please remove it and add a {livekit} section instead.",
                    "DEPRECATION WARNING".yellow().bold(),
                    "NOTE".green(),
                    room_server = "room_server".bold(),
                    livekit = "livekit".bold(),
                );
            }
            "websocket_rate_limit" => {
                anstream::eprintln!(
                    "{}: Found an obsolete {old_section} configuration section.\n\
                     \t{}: Use the {new_section} section instead.",
                    "DEPRECATION WARNING".yellow().bold(),
                    "NOTE".green(),
                    old_section = "websocket_rate_limit".bold(),
                    new_section = "roomserver.websocket_rate_limit".bold(),
                );
            }
            _ => {
                anstream::eprintln!(
                    "{}: Unknown configuration key {}",
                    "WARNING".yellow().bold(),
                    path.bold(),
                );
            }
        }
    }

    fn warn_about_deprecated_items(raw: &SettingsRaw) {
        use owo_colors::OwoColorize as _;

        if raw.keycloak.is_some() {
            anstream::eprintln!(
                "{}: Found an obsolete {keycloak} (oidc) configuration section.\n\
                 {}: This section is deprecated, please replace it with the newly introduced {oidc} and {user_search} sections.",
                "DEPRECATION WARNING".yellow().bold(),
                "NOTE".green(),
                keycloak = "keycloak".bold(),
                oidc = "oidc".bold(),
                user_search = "user_search".bold(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use opentalk_service_auth::{ApiKey, service::ApiKeys};
    use pretty_assertions::assert_eq;
    use serial_test::serial;

    use super::*;
    use crate::settings_provider::tests::{backup_env_variables, restore_env_variables};

    /// Test whether settings are properly overwritten by environment variables.
    ///
    /// This test sets and reads environment variables which is inherently unsafe.
    /// Therefore it is marked as `#[serial]` so that it doesn't interfere with any other
    /// tests that might run in parallel.
    ///
    /// Once the test is finished, all variables are restored.
    #[test]
    #[serial]
    fn serial_test_settings_env_vars_overwrite_config() -> Result<()> {
        // backup current environment variables
        let backup_vars = backup_env_variables();

        // perform the test which modifies the env variables
        let result = settings_env_ars_overwrite_config_inner();

        // restore the environment variables from the backup
        unsafe {
            restore_env_variables(backup_vars);
        }

        result
    }

    fn settings_env_ars_overwrite_config_inner() -> Result<()> {
        // TODO(w.rabl) Do we really have to remove these env vars first? See this discussion:
        // https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1671#note_237420
        unsafe {
            env::remove_var("OPENTALK_CTRL_DATABASE__URL");
            env::remove_var("OPENTALK_CTRL_HTTP__PORT");
            env::remove_var("OPENTALK_CTRL_DEFAULTS__SCREEN_SHARE_REQUIRES_PERMISSION");
        }

        // Sanity check
        let settings = SettingsProvider::load_raw(Path::new("../../example/controller.toml"))?;

        assert_eq!(
            settings.database.url,
            "postgres://postgres:password123@localhost:5432/opentalk"
        );
        assert_eq!(
            settings.http.unwrap().service_api_keys,
            Some(ApiKeys::new(vec![ApiKey::new("controller", "secret")]))
        );

        // Set environment variables to overwrite default config file
        let env_db_url = "postgres://envtest:password@localhost:5432/opentalk".to_string();
        let env_http_port: u16 = 8000;

        unsafe {
            env::set_var("OPENTALK_CTRL_DATABASE__URL", &env_db_url);
            env::set_var("OPENTALK_CTRL_HTTP__PORT", env_http_port.to_string());
        }

        let settings = SettingsProvider::load_raw(Path::new("../../example/controller.toml"))?;

        assert_eq!(settings.database.url, env_db_url);
        assert_eq!(settings.http.as_ref().unwrap().port, Some(env_http_port));

        Ok(())
    }
}
