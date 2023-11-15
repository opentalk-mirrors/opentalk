// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{bail, Result};
use std::sync::Arc;

mod http;
mod media;
mod settings;
mod signaling;
mod sip;
mod websocket;

fn check_plugins() -> Result<()> {
    let registry = gst::Registry::get();

    let required = [
        "webrtc",
        "udp",
        "rtp",
        "alaw",
        "mulaw",
        "app",
        "audiomixer",
        "dtmf",
        "audioconvert",
        "audioresample",
        "opus",
        "libav",
    ];

    let missing = required
        .iter()
        .filter(|n| registry.find_plugin(n).is_none())
        .collect::<Vec<_>>();

    if !missing.is_empty() {
        bail!("Missing gstreamer plugins: {:?}", missing);
    } else {
        Ok(())
    }
}

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

    gst::init()?;
    media::init_custom_rtpdtmfdepay()?;

    check_plugins()?;

    env_logger::init();

    let settings = settings::Settings::load("config.toml")?;

    media::port_pool::PortPool::init(
        settings.sip.rtp_port_range.start,
        settings.sip.rtp_port_range.end,
    );

    // Run a MainLoop on a separate thread so gstreamer bus watches work
    let main_loop = gst::glib::MainLoop::new(None, false);
    std::thread::spawn({
        let main_loop = main_loop.clone();

        move || {
            main_loop.run();
        }
    });

    sip::run(Arc::new(settings)).await?;

    log::info!("Obelisk exiting, bye!");

    main_loop.quit();

    Ok(())
}
