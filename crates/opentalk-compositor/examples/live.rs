// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

fn main() -> anyhow::Result<()> {
    #[cfg(feature = "gstreamer")]
    example::main()?;

    Ok(())
}

#[cfg(feature = "gstreamer")]
mod example {
    use std::time::Duration;

    use opentalk_compositor::{
        create_token, EncoderType, Mixer, MixerParameters, SystemSink, WebMParameters, WebMSink,
    };
    use tokio::{
        select,
        signal::{
            ctrl_c,
            unix::{signal, SignalKind},
        },
        time::{interval_at, Instant},
    };

    #[tokio::main]
    pub(crate) async fn main() -> anyhow::Result<()> {
        env_logger::init();

        gstreamer::init()?;

        let main_loop = gstreamer::glib::MainLoop::new(None, false);
        std::thread::spawn({
            let main_loop = main_loop.clone();

            move || {
                main_loop.run();
            }
        });

        let livekit_url =
            std::env::var("LIVEKIT_URL").expect("Missing LIVEKIT_URL environment variable");
        let livekit_api_key =
            std::env::var("LIVEKIT_API_KEY").expect("Missing LIVEKIT_API_KEY environment variable");
        let livekit_api_secret = std::env::var("LIVEKIT_API_SECRET")
            .expect("Missing LIVEKIT_API_SECRET environment variable");
        let livekit_room =
            std::env::var("LIVEKIT_ROOM").expect("Missing LIVEKIT_ROOM environment variable");
        let target_fps = std::env::var("COMPOSITOR_FPS")
            .unwrap_or("25".to_string())
            .parse::<u16>()
            .unwrap_or(25);

        let mixer_parameters = MixerParameters {
            auto_subscribe: true,
            clock_format: Default::default(),
            livekit_url,
            livekit_token: create_token(
                &livekit_api_key,
                &livekit_api_secret,
                &livekit_room,
                "example",
            )
            .unwrap(),
            target_fps,
        };

        let mut mixer = Mixer::new(mixer_parameters).await.unwrap();

        mixer
            .link_gstreamer_sink("system", SystemSink::create().unwrap())
            .await
            .unwrap();

        let webmsink = WebMSink::create(&WebMParameters {
            encoder_type: EncoderType::CPU,
            chunk_size: None,
        })
        .unwrap();
        let mut receiver = webmsink.subscribe();
        mixer.link_gstreamer_sink("webm", webmsink).await.unwrap();

        tokio::spawn(async move {
            loop {
                println!("Got chunk {}", receiver.recv().await.unwrap().len());
            }
        });

        tokio::spawn(async move {
            let duration = Duration::from_secs(10);
            let mut interval = interval_at(Instant::now() + duration, duration);
            loop {
                select! {
                    _ = mixer.run() => {}
                    _ = interval.tick() => {
                        mixer.set_target_fps(5);
                    }
                }
            }
        });

        let mut sig_term = signal(SignalKind::terminate()).expect("can not setup SIGTERM handler");

        select! {
            _ = ctrl_c() => { log::info!("received Ctrl-C"); }
            _ = sig_term.recv() => { log::info!("received SIGTERM"); }
        }

        Ok(())
    }
}
