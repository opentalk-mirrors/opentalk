// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::HashMap,
    net::IpAddr,
    path::{Path, PathBuf},
    str::FromStr,
};

use config::{Config, ConfigError, Environment, File, FileFormat};
use dirs::config_dir;
use itertools::Itertools as _;
use opentalk_types_common::users::Language;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Deserializer, Serialize, de};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub rabbit_mq: RabbitMqConfig,

    #[serde(default)]
    pub smtp: SmtpConfig,

    #[serde(default)]
    pub templates: Templates,

    #[serde(default)]
    pub template_builder: TemplateBuilder,

    #[serde(default)]
    pub frontend: Frontend,

    #[serde(default)]
    pub languages: Languages,

    #[serde(default)]
    pub monitoring: Option<MonitoringSettings>,

    pub support_contact: Option<SupportContact>,

    #[serde(default)]
    pub tracing: Option<opentalk_tracing::TracingSettings>,
}

impl Settings {
    /// Creates a new Settings instance from the provided TOML file.
    /// Specific fields can be set or overwritten with environment variables (See struct level docs
    /// for more details).
    pub fn load_from_path(file_path: &Path) -> Result<Settings, ConfigError> {
        let config = Config::builder()
            .add_source(File::from(file_path).format(FileFormat::Toml))
            // MAILER is the prefix used up until v0.17 and kept for compatibility reasons
            .add_source(
                Environment::with_prefix("MAILER")
                    .prefix_separator("_")
                    .separator("__"),
            )
            // The prefix that is following the same convention as other services
            .add_source(
                Environment::with_prefix("OPENTALK_MAIL")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;

        let mut warn_unknown_key = Self::warn_unused_key;
        let ignored_deserializer = serde_ignored::Deserializer::new(config, &mut warn_unknown_key);
        let settings = serde_path_to_error::deserialize(ignored_deserializer)
            .map_err(|e| ConfigError::Message(e.to_string()))?;

        log::info!("Settings loaded from \"{}\".", file_path.to_string_lossy());

        Ok(settings)
    }

    fn warn_unused_key(path: serde_ignored::Path) {
        log::warn!("Unknown configuration key: {path}");
    }

    pub fn load_from_standard_paths() -> Result<Settings, ConfigError> {
        let paths = Self::build_standard_search_paths();
        for ConfigSearchPath { path, deprecated } in &paths {
            if path.exists() {
                if *deprecated {
                    let supported_paths = paths
                        .iter()
                        .filter_map(ConfigSearchPath::display_non_deprecated)
                        .join(", ");
                    log::warn!(
                        "You're using the deprecated configuration path \"{}\", please use one of these instead: {}.",
                        path.to_string_lossy(),
                        supported_paths
                    );
                }
                return Settings::load_from_path(path);
            }
        }

        Err(ConfigError::Message(format!(
            "Couldn't find a configuration file. Searched: {}.",
            paths
                .iter()
                .map(|ConfigSearchPath { path, .. }| format!("\"{}\"", path.to_string_lossy()))
                .join(", ")
        )))
    }

    fn build_standard_search_paths() -> Vec<ConfigSearchPath> {
        let mut paths = vec![];

        paths.push(ConfigSearchPath {
            path: "config.toml".into(),
            deprecated: true,
        });

        paths.push(ConfigSearchPath {
            path: "smtp-mailer.toml".into(),
            deprecated: false,
        });

        if let Some(config_dir) = config_dir() {
            paths.push(ConfigSearchPath {
                path: config_dir.join("opentalk/smtp-mailer.toml"),
                deprecated: false,
            });
        }

        paths.push(ConfigSearchPath {
            path: "/etc/opentalk/smtp-mailer.toml".into(),
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

/// RabbitMQ Config containing the url to connect to
/// and the queue_name for incoming mail tasks
#[derive(Debug, Clone, Deserialize)]
pub struct RabbitMqConfig {
    #[serde(default = "rabbitmq_default_url")]
    pub url: String,
    #[serde(default = "rabbitmq_default_queue_name")]
    pub queue_name: String,
    #[serde(default = "rabbitmq_default_task_processing_timeout_seconds")]
    pub task_processing_timeout_seconds: u64,
}

impl Default for RabbitMqConfig {
    fn default() -> Self {
        Self {
            url: rabbitmq_default_url(),
            queue_name: rabbitmq_default_queue_name(),
            task_processing_timeout_seconds: rabbitmq_default_task_processing_timeout_seconds(),
        }
    }
}

fn rabbitmq_default_url() -> String {
    "amqp://guest:guest@localhost:5672".to_owned()
}

fn rabbitmq_default_queue_name() -> String {
    "opentalk_mailer".to_owned()
}

const fn rabbitmq_default_task_processing_timeout_seconds() -> u64 {
    60 * 20
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonitoringSettings {
    #[serde(default = "default_monitoring_port")]
    pub port: u16,
    #[serde(default = "default_monitoring_addr")]
    pub addr: IpAddr,
}

impl Default for MonitoringSettings {
    fn default() -> Self {
        Self {
            port: default_monitoring_port(),
            addr: default_monitoring_addr(),
        }
    }
}

fn default_monitoring_port() -> u16 {
    11411
}

fn default_monitoring_addr() -> IpAddr {
    [0, 0, 0, 0].into()
}

/// A SMTP URI type
///
/// # Examples
/// SMTP Cleartext: smtp://user:pass@mailserver.example.org:1234?disable_starttls=true
/// SMTP with StartTLS: smtp://user:pass@mailserver.example.org:1234
/// SMTP with implicit TLS: smtps://user:pass@mailserver.example.org:1234
#[derive(Debug, Clone)]
pub struct SmtpUri(url::Url);
impl FromStr for SmtpUri {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match url::Url::parse(s) {
            Ok(res) => match res.scheme() {
                "smtp" | "smtps" => Ok(SmtpUri(res)),
                _ => Err(anyhow::anyhow!("Invalid smtp scheme")),
            },
            Err(_) => Err(anyhow::anyhow!("Invalid smtp scheme")),
        }
    }
}

impl SmtpUri {
    pub fn disable_starttls(&self) -> bool {
        self.0
            .query_pairs()
            .any(|x| x.0 == "disable_starttls" && x.1 == "true")
    }

    pub fn port(&self) -> Option<u16> {
        self.0.port()
    }

    pub fn host(&self) -> Option<&str> {
        self.0.host_str()
    }

    pub fn scheme(&self) -> &str {
        self.0.scheme()
    }

    pub fn credentials(&self) -> anyhow::Result<(Option<String>, Option<String>)> {
        let username = self.0.username();
        let res = if username.is_empty() {
            (
                None,
                self.0
                    .password()
                    .map(percent_decode_str)
                    .map(|x| x.decode_utf8())
                    .transpose()?
                    .map(|x| x.into_owned()),
            )
        } else {
            (
                Some(percent_decode_str(username).decode_utf8()?.into_owned()),
                self.0
                    .password()
                    .map(percent_decode_str)
                    .map(|x| x.decode_utf8())
                    .transpose()?
                    .map(|x| x.into_owned()),
            )
        };

        Ok(res)
    }
}

fn smtp_uri_deserializer<'de, D>(d: D) -> Result<SmtpUri, D::Error>
where
    D: Deserializer<'de>,
{
    let string: String = Deserialize::deserialize(d)?;
    SmtpUri::from_str(&string).map_err(de::Error::custom)
}

/// SMTP Config
///
/// Configure with which SMTP server mails should be send.
#[derive(Debug, Clone, Deserialize)]
pub struct SmtpConfig {
    #[serde(
        deserialize_with = "smtp_uri_deserializer",
        default = "smtp_default_server"
    )]
    pub smtp_server: SmtpUri,
    #[serde(default = "smtp_default_from_name")]
    pub from_name: String,
    #[serde(default = "smtp_default_from_email")]
    pub from_email: String,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            smtp_server: smtp_default_server(),
            from_name: smtp_default_from_name(),
            from_email: smtp_default_from_email(),
        }
    }
}

fn smtp_default_server() -> SmtpUri {
    SmtpUri::from_str("smtp://localhost:25").unwrap()
}

fn smtp_default_from_name() -> String {
    "OpenTalk".to_string()
}

fn smtp_default_from_email() -> String {
    "no-reply@example.org".to_string()
}

/// MailTemplate Config
///
/// Paths to a set of txt and html templates for a specific mail task
#[derive(Debug, Clone, Deserialize)]
struct MailTemplate {
    txt: PathBuf,
    html: PathBuf,
}

/// Mail Templates config
///
/// Uses MailTemplate to point to sets of templates for specific mail tasks
#[derive(Debug, Clone, Deserialize)]
pub struct Templates {
    #[serde(default = "template_default_registered_invite")]
    registered_invite: MailTemplate,
    #[serde(default = "template_default_unregistered_invite")]
    unregistered_invite: MailTemplate,
    #[serde(default = "template_default_external_invite")]
    external_invite: MailTemplate,
    //
    #[serde(default = "template_default_event_update")]
    registered_event_update: MailTemplate,
    #[serde(default = "template_default_unregistered_event_update")]
    unregistered_event_update: MailTemplate,
    #[serde(default = "template_default_external_event_update")]
    external_event_update: MailTemplate,
    //
    #[serde(default = "template_default_event_cancellation")]
    registered_event_cancellation: MailTemplate,
    #[serde(default = "template_default_unregistered_event_cancellation")]
    unregistered_event_cancellation: MailTemplate,
    #[serde(default = "template_default_external_event_cancellation")]
    external_event_cancellation: MailTemplate,
    //
    #[serde(default = "template_default_uninvite")]
    registered_uninvite: MailTemplate,
    #[serde(default = "template_default_unregistered_uninvite")]
    unregistered_uninvite: MailTemplate,
    #[serde(default = "template_default_external_uninvite")]
    external_uninvite: MailTemplate,
}

impl Templates {
    pub fn iter(&self) -> impl Iterator<Item = (&Path, &str)> {
        vec![
            (self.registered_invite.txt.as_ref(), "registered_invite.txt"),
            (
                self.registered_invite.html.as_ref(),
                "registered_invite.html",
            ),
            (
                self.unregistered_invite.txt.as_ref(),
                "unregistered_invite.txt",
            ),
            (
                self.unregistered_invite.html.as_ref(),
                "unregistered_invite.html",
            ),
            (self.external_invite.txt.as_ref(), "external_invite.txt"),
            (self.external_invite.html.as_ref(), "external_invite.html"),
            //
            (
                self.registered_event_update.txt.as_ref(),
                "registered_event_update.txt",
            ),
            (
                self.registered_event_update.html.as_ref(),
                "registered_event_update.html",
            ),
            (
                self.unregistered_event_update.txt.as_ref(),
                "unregistered_event_update.txt",
            ),
            (
                self.unregistered_event_update.html.as_ref(),
                "unregistered_event_update.html",
            ),
            (
                self.external_event_update.txt.as_ref(),
                "external_event_update.txt",
            ),
            (
                self.external_event_update.html.as_ref(),
                "external_event_update.html",
            ),
            //
            (
                self.registered_event_cancellation.txt.as_ref(),
                "registered_event_cancellation.txt",
            ),
            (
                self.registered_event_cancellation.html.as_ref(),
                "registered_event_cancellation.html",
            ),
            (
                self.unregistered_event_cancellation.txt.as_ref(),
                "unregistered_event_cancellation.txt",
            ),
            (
                self.unregistered_event_cancellation.html.as_ref(),
                "unregistered_event_cancellation.html",
            ),
            (
                self.external_event_cancellation.txt.as_ref(),
                "external_event_cancellation.txt",
            ),
            (
                self.external_event_cancellation.html.as_ref(),
                "external_event_cancellation.html",
            ),
            //
            (
                self.registered_uninvite.txt.as_ref(),
                "registered_uninvite.txt",
            ),
            (
                self.registered_uninvite.html.as_ref(),
                "registered_uninvite.html",
            ),
            (
                self.unregistered_uninvite.txt.as_ref(),
                "unregistered_uninvite.txt",
            ),
            (
                self.unregistered_uninvite.html.as_ref(),
                "unregistered_uninvite.html",
            ),
            (self.external_uninvite.txt.as_ref(), "external_uninvite.txt"),
            (
                self.external_uninvite.html.as_ref(),
                "external_uninvite.html",
            ),
        ]
        .into_iter()
    }
}

impl Default for Templates {
    fn default() -> Self {
        Self {
            registered_invite: template_default_registered_invite(),
            unregistered_invite: template_default_unregistered_invite(),
            external_invite: template_default_external_invite(),
            //
            registered_event_update: template_default_event_update(),
            unregistered_event_update: template_default_unregistered_event_update(),
            external_event_update: template_default_external_event_update(),
            //
            registered_event_cancellation: template_default_event_cancellation(),
            unregistered_event_cancellation: template_default_unregistered_event_cancellation(),
            external_event_cancellation: template_default_external_event_cancellation(),
            //
            registered_uninvite: template_default_uninvite(),
            unregistered_uninvite: template_default_unregistered_uninvite(),
            external_uninvite: template_default_external_uninvite(),
        }
    }
}

fn resource_template_path(relative_path: &Path) -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(|s| {
            Path::new(&s)
                .join("resources/templates")
                .join(relative_path)
        })
        .unwrap_or_else(|_| Path::new("resources/templates").join(relative_path))
}

fn template_default_registered_invite() -> MailTemplate {
    MailTemplate {
        txt: resource_template_path(Path::new("registered_invite.txt")),
        html: resource_template_path(Path::new("registered_invite.html")),
    }
}

fn template_default_unregistered_invite() -> MailTemplate {
    MailTemplate {
        txt: resource_template_path(Path::new("unregistered_invite.txt")),
        html: resource_template_path(Path::new("unregistered_invite.html")),
    }
}

fn template_default_external_invite() -> MailTemplate {
    MailTemplate {
        txt: resource_template_path(Path::new("external_invite.txt")),
        html: resource_template_path(Path::new("external_invite.html")),
    }
}

fn template_default_event_update() -> MailTemplate {
    MailTemplate {
        txt: resource_template_path(Path::new("registered_event_update.txt")),
        html: resource_template_path(Path::new("registered_event_update.html")),
    }
}

fn template_default_unregistered_event_update() -> MailTemplate {
    MailTemplate {
        txt: resource_template_path(Path::new("unregistered_event_update.txt")),
        html: resource_template_path(Path::new("unregistered_event_update.html")),
    }
}

fn template_default_external_event_update() -> MailTemplate {
    MailTemplate {
        txt: resource_template_path(Path::new("external_event_update.txt")),
        html: resource_template_path(Path::new("external_event_update.html")),
    }
}

fn template_default_event_cancellation() -> MailTemplate {
    MailTemplate {
        txt: resource_template_path(Path::new("registered_event_cancellation.txt")),
        html: resource_template_path(Path::new("registered_event_cancellation.html")),
    }
}

fn template_default_unregistered_event_cancellation() -> MailTemplate {
    MailTemplate {
        txt: resource_template_path(Path::new("unregistered_event_cancellation.txt")),
        html: resource_template_path(Path::new("unregistered_event_cancellation.html")),
    }
}

fn template_default_external_event_cancellation() -> MailTemplate {
    MailTemplate {
        txt: resource_template_path(Path::new("external_event_cancellation.txt")),
        html: resource_template_path(Path::new("external_event_cancellation.html")),
    }
}

fn template_default_uninvite() -> MailTemplate {
    MailTemplate {
        txt: resource_template_path(Path::new("registered_uninvite.txt")),
        html: resource_template_path(Path::new("registered_uninvite.html")),
    }
}

fn template_default_unregistered_uninvite() -> MailTemplate {
    MailTemplate {
        txt: resource_template_path(Path::new("unregistered_uninvite.txt")),
        html: resource_template_path(Path::new("unregistered_uninvite.html")),
    }
}

fn template_default_external_uninvite() -> MailTemplate {
    MailTemplate {
        txt: resource_template_path(Path::new("external_uninvite.txt")),
        html: resource_template_path(Path::new("external_uninvite.html")),
    }
}

/// Language config
///
/// Currently only setting the default_language (a fallback) is supported
#[derive(Debug, Clone, Deserialize)]
pub struct Frontend {
    #[serde(default = "frontend_default_base_url")]
    pub base_url: String,
    #[serde(default = "frontend_default_data_protection_url")]
    pub data_protection_url: String,
    /// Quick guide URLs keyed by the language of the email
    ///
    /// Languages that are not configured fall back to the built-in default URL.
    #[serde(default = "frontend_default_quick_guide_urls")]
    pub quick_guide_urls: HashMap<Language, String>,
}

impl Frontend {
    /// Returns the quick guide URL for the given (negotiated) email language
    pub fn quick_guide_url(&self, language: &Language) -> &str {
        self.quick_guide_urls
            .get(language)
            .map(String::as_str)
            .unwrap_or_else(|| default_quick_guide_url(language))
    }
}

impl Default for Frontend {
    fn default() -> Self {
        Self {
            base_url: frontend_default_base_url(),
            data_protection_url: frontend_default_data_protection_url(),
            quick_guide_urls: frontend_default_quick_guide_urls(),
        }
    }
}

fn frontend_default_base_url() -> String {
    "https://opentalk.example.org".into()
}

fn frontend_default_data_protection_url() -> String {
    "https://opentalk.example.org/dataprotection".into()
}

const DEFAULT_QUICK_GUIDE_URL_EN: &str = "https://opentalk.eu/en/quick-manual";
const DEFAULT_QUICK_GUIDE_URL_DE: &str = "https://opentalk.eu/de/kurzanleitung";

fn default_quick_guide_url(language: &Language) -> &'static str {
    match language.0.language.as_str() {
        "de" => DEFAULT_QUICK_GUIDE_URL_DE,
        _ => DEFAULT_QUICK_GUIDE_URL_EN,
    }
}

fn frontend_default_quick_guide_urls() -> HashMap<Language, String> {
    ["en-US", "de-DE"]
        .into_iter()
        .map(|language| {
            let language = Language::from_str(language).expect("valid language identifier");
            let url = default_quick_guide_url(&language).to_owned();
            (language, url)
        })
        .collect()
}

/// Template Builder Config
///
/// Some information is currently not present in the mail tasks, e.g. baseURLs
/// This can be configured here.
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateBuilder {
    #[serde(default = "email_default_dashboard_event_link_builder")]
    pub dashboard_event_link_builder: String,
    /// {invite_id} will be replaced
    #[serde(default = "email_default_join_link_builder")]
    pub join_link_builder: String,
}

impl Default for TemplateBuilder {
    fn default() -> Self {
        Self {
            dashboard_event_link_builder: email_default_dashboard_event_link_builder(),
            join_link_builder: email_default_join_link_builder(),
        }
    }
}

fn email_default_dashboard_event_link_builder() -> String {
    "{base_url}/dashboard/meetings/{event_id}".into()
}

fn email_default_join_link_builder() -> String {
    "{base_url}/room/{room_id}".into()
}

/// Language config
///
/// Currently only setting the default_language (a fallback) is supported
#[derive(Debug, Clone, Deserialize)]
pub struct Languages {
    #[serde(default = "languages_default_default_language")]
    pub default_language: Language,
}

impl Default for Languages {
    fn default() -> Self {
        Self {
            default_language: languages_default_default_language(),
        }
    }
}

fn languages_default_default_language() -> Language {
    Language::from_str("en-US").expect("en-US is a valid language")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportContact {
    pub phone: String,
    pub mail: String,
}

#[cfg(test)]
mod test {
    use std::{collections::BTreeMap, env};

    use serial_test::serial;

    use super::*;

    #[test]
    fn frontend_default_quick_guide_urls() {
        let frontend = Frontend::default();

        assert_eq!(
            frontend.quick_guide_url(&Language::from_str("en-US").unwrap()),
            "https://opentalk.eu/en/quick-manual"
        );
        assert_eq!(
            frontend.quick_guide_url(&Language::from_str("de-DE").unwrap()),
            "https://opentalk.eu/de/kurzanleitung"
        );
    }

    #[test]
    fn frontend_configured_quick_guide_urls() -> Result<(), ConfigError> {
        let settings: Settings = Config::builder()
            .add_source(File::from_str(
                r#"
                [frontend.quick_guide_urls]
                "de-DE" = "https://example.org/de/quick-guide"
                "#,
                FileFormat::Toml,
            ))
            .build()?
            .try_deserialize()?;

        assert_eq!(
            settings
                .frontend
                .quick_guide_url(&Language::from_str("de-DE").unwrap()),
            "https://example.org/de/quick-guide"
        );
        // Languages missing in the configuration fall back to the built-in default
        assert_eq!(
            settings
                .frontend
                .quick_guide_url(&Language::from_str("en-US").unwrap()),
            "https://opentalk.eu/en/quick-manual"
        );

        Ok(())
    }
    /// Test loading the settings overwritten by env vars.
    ///
    /// This test sets and reads environment variables which is inherently unsafe.
    /// Therefore it is marked as `#[serial]` so that it doesn't interfere with any other
    /// tests that might run in parallel.
    ///
    /// Once the test is finished, all variables are restored.
    #[test]
    #[serial]
    fn settings_env_vars_overwrite_config() -> Result<(), ConfigError> {
        // backup current environment variables
        let backup_vars = backup_env_variables();

        // perform the test which modifies the env variables
        let result = settings_env_vars_overwrite_config_inner();

        // restore the environment variables from the backup
        unsafe {
            restore_env_variables(backup_vars);
        }

        result
    }

    fn settings_env_vars_overwrite_config_inner() -> Result<(), ConfigError> {
        // Sanity check
        let settings = Settings::load_from_path(Path::new("./example/smtp-mailer.toml"))?;
        let support_contact = settings.support_contact.unwrap();

        assert_eq!(support_contact.phone, "+49123321123".to_string());
        assert_eq!(support_contact.mail, "support@example.com".to_string());

        // Set environment variables to overwrite default config file
        let env_support_phone = "+49777666555".to_string();
        let env_support_mail = "support@newexample.com".to_string();
        unsafe {
            env::set_var("OPENTALK_MAIL_SUPPORT_CONTACT__PHONE", &env_support_phone);
            env::set_var("OPENTALK_MAIL_SUPPORT_CONTACT__MAIL", &env_support_mail);
        }

        let settings = Settings::load_from_path(Path::new("./example/smtp-mailer.toml"))?;
        let support_contact = settings.support_contact.unwrap();

        assert_eq!(support_contact.phone, env_support_phone);
        assert_eq!(support_contact.mail, env_support_mail);

        Ok(())
    }

    fn backup_env_variables() -> BTreeMap<String, String> {
        env::vars().collect()
    }

    unsafe fn restore_env_variables(backup_vars: BTreeMap<String, String>) {
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
