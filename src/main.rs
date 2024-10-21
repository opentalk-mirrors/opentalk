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
        "openh264",
        "vpx",
        "videobox",
        "udp",
        "rtpmanager",
        "rtp",
        "mulaw",
        "level",
        "debug",
        "alaw",
        "libav",
        "webrtc",
        "srtp",
        "dtls",
        "nice",
        "volume",
        "videotestsrc",
        "videoconvertscale",
        "pango",
        "opus",
        "compositor",
        "audiotestsrc",
        "audioresample",
        "audiomixer",
        "audioconvert",
        "app",
        "coreelements",
        "staticelements",
    ];

    let failed_plugins = required
        .into_iter()
        .filter(|n| {
            registry
                .find_plugin(n)
                .and_then(|p| p.load().ok())
                .is_none()
        })
        .collect::<Vec<_>>();

    if !failed_plugins.is_empty() {
        bail!("Failed to load GStreamer plugins [{}], try to start the application with 'GST_DEBUG=1' if the plugins are installed correctly.", failed_plugins.join(", "));
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

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if !is_x86_feature_detected!("avx") {
        bail!("Missing CPU feature 'avx'");
    }

    gst::init()?;
    gstrsinter::plugin_register_static()?;
    media::init_custom_rtpdtmfdepay()?;

    check_plugins()?;

    env_logger::init();

    let settings = Arc::new(settings::Settings::load("config.toml")?);

    media::port_pool::PortPool::init(settings.clone());

    // Run a MainLoop on a separate thread so gstreamer bus watches work
    let main_loop = gst::glib::MainLoop::new(None, false);
    std::thread::spawn({
        let main_loop = main_loop.clone();

        move || {
            main_loop.run();
        }
    });

    sip::run(settings).await?;

    log::info!("Obelisk exiting, bye!");

    main_loop.quit();

    Ok(())
}
