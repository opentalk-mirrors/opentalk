// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use std::{process::exit, sync::Arc};

use crate::cli::{Args, Commands, print_info};
use clap::Parser;
use service_probe_client::is_ready;

mod cli;
mod http;
mod media;
mod settings;
mod signaling;
mod sip;
mod websocket;

#[tokio::main]
async fn main() -> Result<()> {
    match std::env::args().next() {
        Some(s) if s.contains("k3k-obelisk") => {
            use owo_colors::OwoColorize as _;
            anstream::eprintln!(
                "{}: It appears you're using the deprecated `k3k-obelisk` executable, \
                you should be using the `opentalk-obelisk` executable instead. \
                The `k3k-obelisk` executable will be removed in a future release.",
                "DEPRECATION WARNING".yellow().bold(),
            );
        }
        _ => {}
    }

    env_logger::init();

    let args = Args::parse();
    if args.info.should_print() {
        print_info(&args.info);
        return Ok(());
    }

    let settings = Arc::new(settings::Settings::load(args.config.as_ref())?);
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
            Ok(false) => {
                log::info!("Not Ready");
                exit(1)
            }
            Ok(true) => {
                log::info!("READY");
                Ok(())
            }
            Err(err) => {
                log::error!("Err: {err}");
                exit(-1)
            }
        };
    }

    // livekit uses libwebrtc, which in turn uses libsrtp. This tells libwebrtc that we're gonna
    // initialize libsrtp. If this is not done, livekit silently fails to work properly.
    webrtc_sys::prohibit_libsrtp_initialization::ffi::ProhibitLibsrtpInitialization();

    // Initialize libsrtp
    srtp::ensure_init();

    media::PortPool::init(settings.clone());

    sip::run(settings).await?;

    log::info!("Obelisk exiting, bye!");

    Ok(())
}
