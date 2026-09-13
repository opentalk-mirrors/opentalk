// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::time::Duration;

use anyhow::Context;
use livekit::{
    RoomEvent,
    options::TrackPublishOptions,
    track::{LocalAudioTrack, LocalTrack, TrackSource},
    webrtc::{
        audio_source::native::NativeAudioSource,
        prelude::{AudioFrame, AudioSourceOptions, RtcAudioSource},
    },
};
use opentalk_roomserver_crypto_provider::ensure_crypto_provider;
use opentalk_roomserver_types_livekit::LiveKitSettings;
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt as _,
    core::{IntoContainerPort, WaitFor, logs::consumer::logging_consumer::LoggingConsumer},
    runners::AsyncRunner as _,
};
use tokio::sync::mpsc::UnboundedReceiver;
use url::Url;

pub const LIVEKIT_PORT: u16 = 7880;
pub const LIVEKIT_KEY: &str = "devkey";
pub const LIVEKIT_SECRET: &str = "devsecret";

fn livekit_config(port: u16, key: &str, secret: &str) -> Vec<u8> {
    let config_str = format!(
        r"
---
port: {port}
rtc:
    tcp_port: 7881
    # udp_port: 7882
    port_range_start: 50000
    port_range_end: 60000
    use_external_ip: false
keys:
    {key}: {secret}
logging:
    json: false
    level: info
",
    );
    config_str.into_bytes()
}

#[derive(Debug, Default)]
pub struct ContainerGuard {
    _inner: Option<ContainerAsync<GenericImage>>,
}

impl From<ContainerAsync<GenericImage>> for ContainerGuard {
    fn from(value: ContainerAsync<GenericImage>) -> Self {
        Self {
            _inner: Some(value),
        }
    }
}

pub async fn create_livekit_container() -> (ContainerGuard, LiveKitSettings) {
    ensure_crypto_provider();

    let livekit_container = GenericImage::new("livekit/livekit-server", "latest")
        .with_exposed_port(LIVEKIT_PORT.tcp())
        .with_wait_for(WaitFor::message_on_stderr("starting LiveKit server"))
        .with_network("bridge")
        .with_copy_to(
            "/livekit.yaml",
            livekit_config(LIVEKIT_PORT, LIVEKIT_KEY, LIVEKIT_SECRET),
        )
        .with_cmd([
            "--config",
            "/livekit.yaml",
            "--dev",
            "--node-ip",
            "127.0.0.1",
        ])
        .with_log_consumer(LoggingConsumer::new())
        .start()
        .await
        .unwrap();
    let host = livekit_container.get_host().await.unwrap().to_string();
    let host_port = livekit_container
        .get_host_port_ipv4(LIVEKIT_PORT)
        .await
        .unwrap();
    let url = Url::parse(&format!("http://{host}:{host_port}")).unwrap();

    let settings = LiveKitSettings {
        api_key: LIVEKIT_KEY.to_string(),
        api_secret: LIVEKIT_SECRET.to_string(),
        public_url: url.to_string(),
        service_url: url,
    };

    (livekit_container.into(), settings)
}

// since all integration tests load this module separately, it will be flagged
// as unused in some of them.
pub async fn publish_audio(
    room: &livekit::Room,
    room_events: &mut UnboundedReceiver<RoomEvent>,
) -> anyhow::Result<LocalAudioTrack> {
    const SAMPLE_RATE: u32 = 48000;
    const NUM_CHANNELS: u32 = 2;
    const SAMPLES_PER_CHANNEL: u32 = 480;

    tracing::info!("Try publishing audio");

    let source =
        NativeAudioSource::new(AudioSourceOptions::default(), SAMPLE_RATE, NUM_CHANNELS, 10);

    let audio_track =
        LocalAudioTrack::create_audio_track("file", RtcAudioSource::Native(source.clone()));
    // The LiveKit server does not respond when the user is not allowed to publish audio, instead
    // the LiveKit SDK waits for a 10 second timeout, which slows down the tests. Assume that if the
    // publish does not complete in 1 second, it has failed.
    tokio::time::timeout(
        Duration::from_secs(1),
        room.local_participant().publish_track(
            LocalTrack::Audio(audio_track.clone()),
            TrackPublishOptions {
                source: TrackSource::Microphone,
                ..Default::default()
            },
        ),
    )
    .await
    .context("Publishing track timed out. The user might not be allowed to publish audio.")?
    .context("Publishing track failed")?;

    tracing::info!("Try capture audio frame");
    let audio_frame = AudioFrame::new(SAMPLE_RATE, NUM_CHANNELS, SAMPLES_PER_CHANNEL);
    source.capture_frame(&audio_frame).await?;
    tracing::info!("Audio frame captured");

    loop {
        let event = tokio::time::timeout(Duration::from_secs(1), room_events.recv())
            .await?
            .context("Failed to receive track published event")?;

        tracing::info!("Received LiveKit event: {:?}", event);
        if let RoomEvent::LocalTrackPublished { track, .. } = event
            && track.sid() == audio_track.sid()
        {
            break;
        }
    }

    tokio::time::timeout(Duration::from_secs(1), room_events.recv())
        .await
        .context("Failed to receive participant update")?;

    Ok(audio_track)
}
