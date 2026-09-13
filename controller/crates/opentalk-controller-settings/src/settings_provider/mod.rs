// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use arc_swap::ArcSwap;
use dirs::config_dir;
use itertools::Itertools as _;
use owo_colors::OwoColorize as _;

use crate::{Result, Settings, SettingsRaw, settings_error::ConfigurationFileNotFoundSnafu};

mod loading;

/// A struct for loading and holding the runtime settings.
#[derive(Debug, Clone)]
pub struct SettingsProvider {
    settings: Arc<ArcSwap<Settings>>,
}

impl SettingsProvider {
    /// Load the settings from a TOML file.
    ///
    /// This will succeed in case the file could be loaded successfully.
    /// Environment variables in the `OPENTALK_CTRL_*` pattern are considered
    /// and will override the settings found in the file.
    fn load_from_path(file_path: &Path) -> Result<Self> {
        let settings_raw = Self::load_raw(file_path)?;
        Self::new_raw(settings_raw)
    }

    /// Create a new [`SettingsProvider`] with settings that are already loaded.
    pub(crate) fn new_raw(settings_raw: SettingsRaw) -> Result<Self> {
        let settings = Settings::try_from(settings_raw)?;
        Ok(Self {
            settings: Arc::new(ArcSwap::new(Arc::new(settings))),
        })
    }

    /// Get an `[Arc]` holding the current raw settings.
    ///
    /// The returned settings will remain unchanged even if the settings are
    /// reloaded by the [`SettingsProvider`]. A new [`Arc`] will be created
    /// internally by the `reload` function. This allows consistent use of a
    /// "snapshot" inside a function by calling `get_raw` once, and then using
    /// the returned value.
    pub fn get(&self) -> Arc<Settings> {
        self.settings.load_full().clone()
    }

    /// Reload the settings from a TOML file.
    ///
    /// Not all settings are used, as most of the settings are not reloadable
    /// while the controller is running.
    ///
    /// This will succeed in case the file could be loaded successfully.
    /// Environment variables in the `OPENTALK_CTRL_*` pattern are considered
    /// and will override the settings found in the file.
    ///
    /// If loading the settings fails, an error is returned from this function
    /// and stored configuration will remain unchanged.
    ///
    /// Any "snapshots" handed out to callers by the `get` function remain
    /// unchanged, so wherever these are used, the values will not change.
    /// Because an `Arc` was given to these callers, the value will be freed
    /// once the last reference to it has been dropped.
    fn reload_from_path(&self, file_path: &Path) -> Result<()> {
        let settings_raw = Self::load_raw(file_path)?;

        let mut current_settings = (*self.settings.load_full()).clone();

        current_settings.try_reload_from(settings_raw)?;

        // replace the shared settings with the modified ones
        self.settings.store(Arc::new(current_settings));

        Ok(())
    }

    fn reload_from_standard_paths(&self) -> Result<()> {
        self.reload_from_path(&Self::select_standard_path()?)
    }

    pub fn load_from_path_or_standard_paths(file_path: Option<&Path>) -> Result<Self> {
        if let Some(file_path) = file_path {
            return Self::load_from_path(file_path);
        }
        Self::load_from_standard_paths()
    }

    pub fn reload_from_path_or_standard_paths(&self, file_path: Option<&Path>) -> Result<()> {
        if let Some(file_path) = file_path {
            return self.reload_from_path(file_path);
        }
        self.reload_from_standard_paths()
    }

    fn load_from_standard_paths() -> Result<Self> {
        Self::load_from_path(&Self::select_standard_path()?)
    }

    fn select_standard_path() -> Result<PathBuf> {
        let paths = Self::build_search_search_paths();
        for ConfigSearchPath { path, deprecated } in &paths {
            if path.exists() {
                if *deprecated {
                    let supported_paths = paths
                        .iter()
                        .filter_map(ConfigSearchPath::display_non_deprecated)
                        .join(", ");
                    anstream::eprintln!(
                        "{}: You're using the deprecated configuration path \"{}\", please use one of these instead: {}.",
                        "DEPRECATION WARNING".yellow().bold(),
                        path.to_string_lossy(),
                        supported_paths
                    );
                }
                return Ok(path.to_path_buf());
            }
        }

        let paths: Vec<String> = paths
            .iter()
            .map(|ConfigSearchPath { path, .. }| path.to_string_lossy().to_string())
            .collect();
        ConfigurationFileNotFoundSnafu { paths }.fail()
    }

    fn build_search_search_paths() -> Vec<ConfigSearchPath> {
        let mut paths = vec![];

        paths.push(ConfigSearchPath {
            path: "config.toml".into(),
            deprecated: true,
        });

        paths.push(ConfigSearchPath {
            path: "controller.toml".into(),
            deprecated: false,
        });

        if let Some(config_dir) = config_dir() {
            paths.push(ConfigSearchPath {
                path: config_dir.join("opentalk/controller.toml"),
                deprecated: false,
            });
        }

        paths.push(ConfigSearchPath {
            path: "/etc/opentalk/controller.toml".into(),
            deprecated: false,
        });

        paths
    }
}

struct ConfigSearchPath {
    path: PathBuf,
    deprecated: bool,
}

impl ConfigSearchPath {
    fn display_non_deprecated(&self) -> Option<String> {
        if self.deprecated {
            return None;
        }
        Some(format!("\"{}\"", self.path.to_string_lossy()))
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, env, fs::File, io::Write as _, path::Path};

    use pretty_assertions::{assert_eq, assert_matches, assert_ne};
    use serial_test::serial;
    use tempfile::tempdir;

    use super::SettingsProvider;
    use crate::{
        SettingsError, settings_file::SETTINGS_RAW_MINIMAL_CONFIG_TOML,
        settings_runtime::settings::minimal_example,
    };

    #[test]
    fn load_example_toml() -> Result<(), SettingsError> {
        SettingsProvider::load_from_path(Path::new("../../example/controller.toml"))?;
        Ok(())
    }

    #[test]
    fn load_minimal() {
        let tempdir = tempdir().unwrap();

        let path = tempdir.path().join("controller.toml");

        {
            let mut file = File::create(&path).unwrap();
            writeln!(file, "{SETTINGS_RAW_MINIMAL_CONFIG_TOML}")
                .expect("temp file should be writable");
        }

        let settings_provider =
            SettingsProvider::load_from_path(&path).expect("valid configuration expected");

        assert_eq!(&(*settings_provider.get()), &minimal_example());
    }

    #[test]
    fn load_invalid() {
        let tempdir = tempdir().unwrap();

        let path = tempdir.path().join("controller.toml");

        {
            // Create an empty file which will result in an invalid definition
            let _file = File::create(&path).unwrap();
        }

        assert_matches!(
            SettingsProvider::load_from_path(&path),
            Err(SettingsError::DeserializeConfig {
                file_path: _,
                source: _
            })
        );
    }

    /// Test reloading the settings.
    ///
    /// This test sets and reads environment variables which is inherently unsafe.
    /// Therefore it is marked as `#[serial]` so that it doesn't interfere with any other
    /// tests that might run in parallel.
    ///
    /// Once the test is finished, all variables are restored.
    #[test]
    #[serial]
    fn serial_test_reload() {
        // backup current environment variables
        let backup_vars = backup_env_variables();

        // perform the test which modifies the env variables
        reload_inner();

        // restore the environment variables from the backup
        unsafe {
            restore_env_variables(backup_vars);
        }
    }

    fn reload_inner() {
        // TODO(w.rabl) Why exactly do we have to remove these env vars first? See according discussion:
        // https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1671#note_237430
        unsafe {
            env::remove_var("OPENTALK_CTRL_DATABASE__URL");
            env::remove_var("OPENTALK_CTRL_HTTP__PORT");
            env::remove_var("OPENTALK_CTRL_DEFAULTS__USER_LANGUAGE");
            env::remove_var("OPENTALK_CTRL_DEFAULTS__SCREEN_SHARE_REQUIRES_PERMISSION");
        }

        let tempdir = tempdir().unwrap();

        let modified_path = tempdir.path().join("controller_modified.toml");
        let minimal_path = tempdir.path().join("controller_minimal.toml");

        {
            let mut file = File::create(&modified_path).unwrap();
            writeln!(
                file,
                r#"
                {SETTINGS_RAW_MINIMAL_CONFIG_TOML}

                [call_in]
                tel = "+55667788"
                enable_phone_mapping = true
                default_country_code = "DE"
                "#
            )
            .expect("temp file should be writable");
        }
        {
            let mut file = File::create(&minimal_path).unwrap();
            writeln!(file, "{SETTINGS_RAW_MINIMAL_CONFIG_TOML}")
                .expect("temp file should be writable");
        }

        let settings_provider =
            SettingsProvider::load_from_path(&modified_path).expect("valid configuration expected");

        assert_ne!(&(*settings_provider.get()), &minimal_example());

        settings_provider
            .reload_from_path(&minimal_path)
            .expect("reload is expected to succeed");

        assert_eq!(&(*settings_provider.get()), &minimal_example());
    }

    #[test]
    fn reload_invalid() {
        let tempdir = tempdir().unwrap();

        let invalid_path = tempdir.path().join("controller_invalid.toml");
        let minimal_path = tempdir.path().join("controller_minimal.toml");

        {
            // Create an empty file which will result in an invalid definition
            let _file = File::create(&invalid_path).unwrap();
        }
        {
            let mut file = File::create(&minimal_path).unwrap();
            writeln!(file, "{SETTINGS_RAW_MINIMAL_CONFIG_TOML}")
                .expect("temp file should be writable");
        }

        let settings_provider =
            SettingsProvider::load_from_path(&minimal_path).expect("valid configuration expected");

        assert_eq!(&(*settings_provider.get()), &minimal_example());

        assert_matches!(
            settings_provider.reload_from_path(&invalid_path),
            Err(SettingsError::DeserializeConfig {
                file_path: _,
                source: _
            })
        );

        assert_eq!(&(*settings_provider.get()), &minimal_example());
    }

    pub(super) fn backup_env_variables() -> BTreeMap<String, String> {
        env::vars().collect()
    }

    pub(super) unsafe fn restore_env_variables(backup_vars: BTreeMap<String, String>) {
        {
            for (k, _) in env::vars() {
                if !backup_vars.contains_key(&k) {
                    unsafe { env::remove_var(&k) };
                }
            }
            for (k, v) in backup_vars {
                unsafe { env::set_var(k, v) };
            }
        }
    }
}
