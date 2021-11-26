use anyhow::{bail, Result};
use gst::glib;
use std::sync::Arc;
use std::thread;

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
        "app",
        "audiomixer",
        "dtmf",
        "audioconvert",
        "audioresample",
        "opus",
    ];

    let missing = required
        .iter()
        .filter(|n| registry.find_plugin(n).is_none())
        .collect::<Vec<_>>();

    if !missing.is_empty() {
        bail!("Missing plugins: {:?}", missing);
    } else {
        Ok(())
    }
}

fn main() -> Result<()> {
    gst::init()?;

    check_plugins()?;

    env_logger::init();

    // Let glib run on the main thread
    let main_loop = glib::MainLoop::new(None, false);

    let settings = settings::Settings::load("config.toml")?;

    // Spawn a new thread for the tokio runtime
    let main_loop_clone = main_loop.clone();
    thread::spawn(move || {
        let res = tokio::runtime::Runtime::new()
            .map_err(Into::into)
            .and_then(|rt| rt.block_on(sip::run(main_loop_clone, Arc::new(settings))));

        if let Err(e) = res {
            eprintln!("{:?}", e);
            std::process::exit(-1);
        }
    });

    // Run until main_loop.quit() is called in the sip task
    main_loop.run();

    Ok(())
}
