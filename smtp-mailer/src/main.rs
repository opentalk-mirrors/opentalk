// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{path::PathBuf, process::exit};

use build_info::BuildInfo;
use clap::{Parser, Subcommand};
use log::LevelFilter;
use opentalk_smtp_mailer::{
    preview::{OutputVariant, TemplateVariant, preview_send_mail},
    run, settings,
};
use opentalk_types_common::users::Language;
use opentalk_version::InfoArgs;
use service_probe_client::is_ready;
use url::Url;

mod license;

#[derive(Parser, Debug)]
#[clap(author, about = env!("CARGO_PKG_DESCRIPTION"), long_about = None)]
pub(crate) struct Args {
    /// Path of the configuration file.
    ///
    /// If present, exactly this config file will be used.
    ///
    /// If absent, `smtp-mailer` looks for a config file in these locations and uses the first one that is found:
    ///
    /// - `config.toml` in the current directory (deprecated, for backwards compatiblity only)
    /// - `smtp-mailer.toml` in the current directory
    /// - `<XDG_CONFIG_HOME>/opentalk/smtp-mailer.toml` (where `XDG_CONFIG_HOME` is usually `~/.config`)
    /// - `/etc/opentalk/smtp-mailer.toml`
    #[clap(short, long, verbatim_doc_comment)]
    config: Option<PathBuf>,

    #[clap(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    info: InfoArgs,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Return a preview of an email
    Preview {
        /// Output type
        #[clap(value_enum)]
        type_: OutputVariant,
        /// Template to preview
        #[clap(value_enum)]
        template: TemplateVariant,
        /// Language Code
        #[clap(value_parser = parse_language)]
        language: Language,
    },
    PreviewSend {
        /// Template to preview
        #[clap(value_enum)]
        template: TemplateVariant,
        /// To Email
        to: String,
        /// Delay between invite and cancellation in seconds
        #[clap(default_value_t = 15)]
        cancellation_delay: u64,
    },
    /// Return the readiness state
    Health {
        /// The monitoring endpoint can be provided optionally
        endpoint: Option<Url>,
    },
}

fn parse_language(s: &str) -> anyhow::Result<Language> {
    s.parse()
        .map_err(|e| anyhow::anyhow!("Invalid language. {}", e))
}

opentalk_version::build_info!();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.info.should_print() {
        let build_info = BuildInfo::with_license(license::LICENSE.to_owned());
        if let Some(text) = build_info.format(&args.info) {
            println!("{text}");
        }

        return Ok(());
    }

    const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Warn;

    env_logger::Builder::new()
        .filter_level(DEFAULT_LOG_LEVEL)
        .filter_module("html5ever", DEFAULT_LOG_LEVEL)
        .filter_module("selectors", DEFAULT_LOG_LEVEL)
        .parse_default_env()
        .init();

    let settings = if let Some(settings_path) = &args.config {
        settings::Settings::load_from_path(settings_path)?
    } else {
        settings::Settings::load_from_standard_paths()?
    };

    if let Some(Commands::Health { endpoint }) = args.command {
        let Some(monitoring_endpoint) = endpoint.or_else(|| {
            settings.monitoring.as_ref().map(|monitoring_settings| {
                format!(
                    "http://{}:{}",
                    monitoring_settings.addr, monitoring_settings.port
                )
                .parse()
                .expect("valid endpoint can be built from monitoring settings")
            })
        }) else {
            log::warn!("Monitoring not configured and no url endpoint parameter given");
            exit(1);
        };
        return match is_ready(&monitoring_endpoint).await {
            Ok(is_ready) => match is_ready {
                true => {
                    log::info!("READY");
                    Ok(())
                }
                false => {
                    log::info!("Not Ready");
                    exit(1)
                }
            },
            Err(err) => {
                log::error!("Err: {}", err);
                exit(-1)
            }
        };
    } else if let Some(Commands::Preview {
        type_,
        language,
        template,
    }) = &args.command
    {
        let example_mail = template.render_template(&settings, *type_, language.clone())?;
        print!("{example_mail}");

        exit(0);
    } else if let Some(Commands::PreviewSend {
        template,
        to,
        cancellation_delay,
    }) = &args.command
    {
        preview_send_mail(&settings, *template, to.clone(), *cancellation_delay).await?;

        exit(0);
    }

    run(settings).await?;
    Ok(())
}
