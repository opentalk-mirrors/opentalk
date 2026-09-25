// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{process::exit, sync::Arc};

use anyhow::Result;
use clap::Parser;
use service_probe_client::is_ready;

use crate::cli::{Args, Commands, print_info};

mod cli;
mod media;
mod settings;
mod signaling;
mod sip;
mod trace;

#[tokio::main]
async fn main() -> Result<()> {
    ensure_crypto_provider();
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

    let args = Args::parse();
    if args.info.should_print() {
        print_info(&args.info);
        return Ok(());
    }

    let settings = Arc::new(settings::Settings::load(args.config.as_ref())?);

    let tracer_provider = trace::init(settings.tracing.as_ref())?;

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

    let result = sip::run(settings).await;

    log::info!("Obelisk exiting, bye!");

    opentalk_tracing::shutdown(tracer_provider);

    result
}

/// `rustls` and `jsonwebtoken` depend on a `CryptoProvider` being configured.
/// If no provider was explicitly configured, a provider will be derived from
/// the enabled features. Since there are many crates that depend on rustls and
/// `jsonwebtoken`, we don't have complete control over the enabled features.
/// If the configuration via feature is ambiguous these crates will panic.
///
/// Here we ensure that these crates are explicitly configured.
fn ensure_crypto_provider() {
    rustls::crypto::CryptoProvider::install_default(rustls::crypto::aws_lc_rs::default_provider())
        .expect("valid default crypto provider expected");

    // This should not be necessary, but we add it anyways to prevent future code
    // changes to introduce a panic.
    // jsonwebtoken is used without any crypto (dangerous, insecure decode).
    jsonwebtoken::crypto::CryptoProvider::install_default(
        &jsonwebtoken::crypto::aws_lc::DEFAULT_PROVIDER,
    )
    .expect("valid default crypto provider expected");
}
