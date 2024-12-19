// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use std::sync::Arc;

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

    let args = cli::parse_args();
    if !args.should_start() {
        return Ok(());
    }

    // livekit uses libwebrtc, which in turn uses libsrtp. This tells libwebrtc that we're gonna
    // initialize libsrtp. If this is not done, livekit silently fails to work properly.
    webrtc_sys::prohibit_libsrtp_initialization::ffi::ProhibitLibsrtpInitialization();

    // Initialize libsrtp
    srtp::ensure_init();

    let settings = Arc::new(settings::Settings::load("config.toml")?);

    media::PortPool::init(settings.clone());

    sip::run(settings).await?;

    log::info!("Obelisk exiting, bye!");

    Ok(())
}
