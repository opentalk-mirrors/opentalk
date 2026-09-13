// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![allow(clippy::module_name_repetitions)]

use std::sync::Mutex as StdMutex;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc, time::Instant};

use anyhow::{bail, Context, Result};
use audio::{audio_mixer_task, NativeAudioStreamSource, Silence};
use ezk::nodes::{Access, AccessHandle};
use ezk_audio_nodes::{AudioConvert, AudioMixer};
use ezk_image::{ColorInfo, ColorPrimaries, ColorSpace, ColorTransfer, YuvColorInfo};
use futures::StreamExt;
use livekit::webrtc::prelude::I420Buffer;
use livekit::{
    prelude::*,
    webrtc::{audio_stream::native::NativeAudioStream, video_stream::native::NativeVideoStream},
};
use livekit_api::access_token::{AccessToken, AccessTokenError, VideoGrants};
use tokio::sync::watch;
use tokio::time::timeout;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use video::{VideoPipeline, VideoStreamCommand};

pub mod audio;
pub mod font;
pub mod image;
pub mod sinks;
pub mod video;

#[cfg(feature = "gstreamer")]
pub mod debug;
#[cfg(feature = "gstreamer")]
pub mod elements;
#[cfg(feature = "gstreamer")]
pub mod gst_with_context;
#[cfg(feature = "gstreamer")]
pub mod gstreamer;
#[cfg(feature = "gstreamer")]
pub mod pipeline_watched;

#[cfg(feature = "gstreamer")]
pub use gst_with_context::*;
pub use sinks::*;

pub use livekit;

#[macro_use]
extern crate log;

pub use livekit::id::ParticipantIdentity;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ClockFormat(String);

impl Default for ClockFormat {
    fn default() -> Self {
        Self(String::from("%x %X %Z"))
    }
}

impl AsRef<str> for ClockFormat {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub const WIDTH: usize = 1920;
pub const HEIGHT: usize = 1080;
pub const FRAMES_PER_SECOND: usize = 25;

/// The amount of pixels for borders
pub(crate) const BORDER: usize = 4;

pub const I420_COLOR: ColorInfo = ColorInfo::YUV(YuvColorInfo {
    transfer: ColorTransfer::Linear,
    primaries: ColorPrimaries::BT709,
    space: ColorSpace::BT709,
    full_range: false,
});

pub const PADDING: usize = 16;
pub const OFFSET_TOP: usize = 40;

pub struct Mixer {
    #[cfg(feature = "gstreamer")]
    start: Instant,

    auto_subscribe: bool,

    sinks: Arc<Mutex<HashMap<String, Box<dyn Sink>>>>,

    room: Option<Room>,
    // LiveKitRoom events
    room_events: mpsc::UnboundedReceiver<RoomEvent>,

    http_client: reqwest::Client,

    #[allow(clippy::type_complexity)]
    external_room_event_handler: Vec<Box<dyn FnMut(&RoomEvent) + Send>>,

    // Shared Data for Audio and Video Mixer
    shared: Arc<StdMutex<Shared>>,

    // Audio
    audio_mixer_handle: Arc<parking_lot::Mutex<AccessHandle<AudioMixer>>>,
    audio_mixer_task: JoinHandle<()>,

    // Video
    video_stream_tx: mpsc::UnboundedSender<VideoStreamCommand>,
    video_task: Option<JoinHandle<()>>,

    shutdown_tx: watch::Sender<WantShutdown>,
}

enum WantShutdown {
    Yes,
    No,
}

#[derive(Debug, Clone)]
struct SpeakingState {
    is_speaking: bool,
    last_event: Instant,
}

#[derive(Debug)]
struct Shared {
    participants: HashMap<ParticipantIdentity, Participant>,
    speakers: HashMap<ParticipantIdentity, SpeakingState>,

    clock_format: ClockFormat,
    event_title: Option<String>,

    render_frames: bool,
}

// FIXME
impl std::fmt::Debug for Mixer {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Participant {
    display_name: String,
    // Optional avatar to show instead of the placeholder image
    avatar: Option<Arc<I420Buffer>>,
}

pub struct MixerParameters {
    pub auto_subscribe: bool,
    pub clock_format: ClockFormat,
    pub livekit_url: String,
    pub livekit_token: String,
    pub target_fps: u16,
}

impl Mixer {
    // TODO: This will be fixed later on
    #[allow(clippy::missing_errors_doc)]
    // RoomOptions in future livekits will have #[non_exhaustive] making the struct initilization impossible
    #[allow(clippy::field_reassign_with_default)]
    pub async fn new(parameters: MixerParameters) -> Result<Self> {
        #[cfg(feature = "gstreamer")]
        {
            use anyhow::Context;

            elements::register_all().context("Unable to register all custom GStreamer Elements")?;
        }

        let mut room_options = RoomOptions::default();
        room_options.auto_subscribe = false;

        // Livekit uses the path_segment API from the `url` crate to build their signaling path, but they don't call the
        // `pop` method on the path segments, which results in a double slash in the path if we do not trim the trailing
        // slash here.
        let livekit_url = parameters.livekit_url.trim_end_matches('/');

        let (room, room_events) = Box::pin(Room::connect(
            livekit_url,
            &parameters.livekit_token,
            room_options,
        ))
        .await?;

        let shared = Arc::new(StdMutex::new(Shared {
            participants: HashMap::default(),
            speakers: HashMap::new(),
            clock_format: parameters.clock_format,
            event_title: None,
            render_frames: true,
        }));

        #[cfg(feature = "gstreamer")]
        let start = Instant::now();

        // Initialize Audio Mixer
        let (access, audio_mixer_handle) =
            Access::new(AudioMixer::new(AudioConvert::new(Silence::default())));
        let audio_mixer_handle = Arc::new(parking_lot::Mutex::new(audio_mixer_handle));
        let sinks = Arc::new(Mutex::new(HashMap::default()));
        let audio_mixer_task = tokio::spawn(audio_mixer_task(access, sinks.clone()));

        // Initialize Video Mixer
        let (shutdown_tx, shutdown_rx) = watch::channel(WantShutdown::No);
        let (video_stream_tx, video_task) = VideoPipeline::create(
            sinks.clone(),
            shared.clone(),
            shutdown_rx,
            parameters.target_fps,
        )
        .context("Failed to create video pipeline")?;

        let mixer = Mixer {
            #[cfg(feature = "gstreamer")]
            start,
            auto_subscribe: parameters.auto_subscribe,
            sinks,
            room: Some(room),
            room_events,
            http_client: reqwest::Client::new(),
            external_room_event_handler: vec![],
            shared,
            audio_mixer_handle,
            audio_mixer_task,
            video_stream_tx,
            video_task: Some(video_task),
            shutdown_tx,
        };

        Ok(mixer)
    }

    pub fn add_livekit_event_handler<F>(&mut self, event_handler: F)
    where
        F: FnMut(&RoomEvent) + Send + 'static,
    {
        self.external_room_event_handler
            .push(Box::new(event_handler));
    }

    /// Returns the local livekit participant which can be used to publish additional tracks
    ///
    /// # Panics
    ///
    /// Panics when called after calling [`Mixer::close`]
    #[must_use]
    pub fn local_participant(&self) -> LocalParticipant {
        let room = self
            .room
            .as_ref()
            .expect("must not call any function after calling close");

        room.local_participant()
    }

    /// Run the compositor event loop
    ///
    /// Returns once the client was disconnected from livekit
    ///
    /// This function is cancel safe
    ///
    /// # Panics
    ///
    /// Panics when called after calling [`Mixer::close`]
    pub async fn run(&mut self) -> DisconnectReason {
        let room = self
            .room
            .as_ref()
            .expect("must not call any function after calling close");

        if self.auto_subscribe {
            for participant in room.remote_participants().values() {
                self.add_participant(&participant.identity(), participant.name());
            }
        }

        while let Some(event) = self.room_events.recv().await {
            for handler in &mut self.external_room_event_handler {
                handler(&event);
            }

            if let Some(disconnect_reason) = self.handle_livekit_event(event) {
                return disconnect_reason;
            }
        }

        DisconnectReason::UnknownReason
    }

    /// Sets the target fps of this [`Mixer`].
    ///
    /// # Panics
    ///
    /// Panics if the background render thread has exited
    pub fn set_target_fps(&mut self, target_fps: u16) {
        self.video_stream_tx
            .send(VideoStreamCommand::SetTargetFps(target_fps))
            .expect("unable to send set target fps event");
    }

    fn handle_livekit_event(&mut self, event: RoomEvent) -> Option<DisconnectReason> {
        log::debug!("LiveKit event received: {event:?}");
        match event {
            RoomEvent::TrackSubscribed {
                track,
                publication: _,
                participant,
            } => match track {
                RemoteTrack::Audio(audio_track) => {
                    self.add_audio_track(participant.identity(), audio_track);
                }
                RemoteTrack::Video(video_track) => {
                    self.add_video_track(participant, video_track);
                }
            },
            RoomEvent::TrackUnsubscribed {
                track,
                publication: _,
                participant: _,
            } => {
                self.video_stream_tx
                    .send(VideoStreamCommand::RemoveTrack(track.sid()))
                    .expect("unable to send video stream remove track event");
            }
            RoomEvent::ActiveSpeakersChanged { speakers } => {
                self.handle_active_speakers_changed(speakers);
            }
            RoomEvent::TrackMuted {
                participant: _,
                publication,
            } => {
                self.video_stream_tx
                    .send(VideoStreamCommand::Mute(publication.sid()))
                    .expect("unable to send video stream mute event");
            }
            RoomEvent::TrackUnmuted {
                participant: _,
                publication,
            } => {
                self.video_stream_tx
                    .send(VideoStreamCommand::Unmute(publication.sid()))
                    .expect("unable to send video stream unmute event");
            }
            RoomEvent::TrackPublished {
                publication,
                participant,
            } => {
                let shared = self.shared.lock().unwrap();

                if !shared.render_frames && matches!(publication.kind(), TrackKind::Video) {
                    // do not subscribe video while not rendering frames
                    return None;
                }

                if shared.participants.contains_key(&participant.identity()) {
                    publication.set_subscribed(true);
                }
            }
            RoomEvent::TrackUnpublished {
                publication,
                participant: _,
            } => {
                let track = publication.track()?;

                self.video_stream_tx
                    .send(VideoStreamCommand::RemoveTrack(track.sid()))
                    .expect("unable to send video stream remove track event");
            }
            RoomEvent::ParticipantConnected(participant) => {
                if self.auto_subscribe {
                    self.add_participant(&participant.identity(), participant.name());
                }
            }
            RoomEvent::ParticipantDisconnected(participant) => {
                self.remove_participant(&participant.identity());
            }
            RoomEvent::Disconnected { reason } => return Some(reason.into()),
            _ => {}
        }

        None
    }

    /// Changes the active Speaker for this [`Mixer`].
    ///
    /// # Panics
    ///
    /// Panics if the [`Shared`] lock couldn't be acquired.
    fn handle_active_speakers_changed(&mut self, speakers: Vec<livekit::participant::Participant>) {
        let shared = &mut *self.shared.lock().unwrap();

        for state in shared.speakers.values_mut() {
            state.is_speaking = false;
        }

        for participant in speakers {
            shared.speakers.insert(
                participant.identity(),
                SpeakingState {
                    is_speaking: true,
                    last_event: Instant::now(),
                },
            );
        }
    }

    // TODO: This will be fixed later on
    #[allow(clippy::missing_errors_doc)]
    #[cfg(feature = "gstreamer")]
    pub async fn link_gstreamer_sink(
        &mut self,
        name: &str,
        sink: impl gstreamer::GStreamerSink,
    ) -> Result<()> {
        trace!("link sink, name: {name}, sink: {sink:?}");

        let mut sinks = self.sinks.lock().await;
        if sinks.contains_key(name) {
            bail!("a stream with the name '{name}' already exists");
        }

        let active_sink = gstreamer::GStreamerActiveSink::new(self.start, name, sink)?;

        sinks.insert(name.to_owned(), Box::new(active_sink));

        Ok(())
    }

    // TODO: This will be fixed later on
    #[allow(clippy::missing_errors_doc)]
    pub async fn link_sink(&mut self, name: &str, sink: Box<dyn Sink>) -> Result<()> {
        trace!("link sink, name: {name}, sink: {sink:?}");

        let mut sinks = self.sinks.lock().await;
        if sinks.contains_key(name) {
            bail!("a stream with the name '{name}' already exists");
        }

        sinks.insert(name.to_owned(), sink);

        Ok(())
    }

    pub async fn release_sink(&mut self, name: &String) {
        trace!("release_sink {name}");

        let sink = self.sinks.lock().await.remove(name);

        if let Some(mut sink) = sink {
            close_sink(name, sink.as_mut()).await;
        }
    }

    /// Sets the event title of this [`Mixer`].
    ///
    /// # Panics
    ///
    /// Panics if the lock for the [`Shared`] object could not be acquired.
    pub fn set_event_title(&mut self, title: String) {
        self.shared.lock().unwrap().event_title = Some(title);
    }

    fn add_audio_track(
        &mut self,
        participant_identity: ParticipantIdentity,
        audio_track: RemoteAudioTrack,
    ) {
        let rtc_track = audio_track.rtc_track();
        self.audio_mixer_handle
            .lock()
            .access_no_wait(move |audio_mixer| {
                audio_mixer.add_source(AudioConvert::new(NativeAudioStreamSource {
                    stream: NativeAudioStream::new(rtc_track, 48_000, 2),
                    timestamp: 0,
                }));
            });

        self.video_stream_tx
            .send(VideoStreamCommand::AddAudioTrack((
                participant_identity,
                audio_track,
            )))
            .expect("unable to send add event to video_stream_tx");
    }

    fn add_video_track(&mut self, participant: RemoteParticipant, video_track: RemoteVideoTrack) {
        self.video_stream_tx
            .send(VideoStreamCommand::AddVideoTrack((
                participant.identity(),
                video_track.clone(),
                Box::pin(
                    NativeVideoStream::new(video_track.rtc_track()).map(move |frame| {
                        (
                            participant.identity(),
                            video_track.sid(),
                            frame.buffer.to_i420(),
                        )
                    }),
                ),
            )))
            .expect("unable to send add event to video_stream_tx");
    }

    /// Adds a Participant to the [`Shared`] list.
    ///
    /// # Panics
    ///
    /// Panics if the [`Shared`] object lock couldn't be acquired.
    pub fn add_participant(&mut self, identity: &ParticipantIdentity, display_name: String) {
        log::debug!("Add participant {identity:?}");

        self.shared
            .lock()
            .unwrap()
            .participants
            .entry(identity.clone())
            .and_modify(|p| p.display_name.clone_from(&display_name))
            .or_insert_with(|| Participant {
                display_name,
                avatar: None,
            });

        if let Some(remote_participant) = self
            .room
            .as_ref()
            .expect("room exists")
            .remote_participants()
            .get(identity)
        {
            for (_track_sid, track_publication) in remote_participant.track_publications() {
                track_publication.set_subscribed(true);
            }
        }
    }

    /// Set a image http url for the participant
    ///
    /// The image is loaded asynchronously and used when the participant has no camera or screenshare enabled
    ///
    /// # Panics
    ///
    /// Panics if the [`Shared`] object lock couldn't be acquired.
    pub fn set_participant_avatar_url(
        &mut self,
        identity: &ParticipantIdentity,
        avatar_url: String,
    ) {
        async fn load_avatar_from_url(
            http_client: reqwest::Client,
            shared: Arc<StdMutex<Shared>>,
            identity: ParticipantIdentity,
            avatar_url: &str,
        ) -> Result<()> {
            let response = http_client
                .get(avatar_url)
                .send()
                .await
                .context("Failed to send a HTTP request")?;

            if !response.status().is_success() {
                bail!("Got unexpected {} response", response.status());
            }

            let bytes = response
                .bytes()
                .await
                .context("Failed to receive HTTP response")?;

            let avatar = ::image::load_from_memory(&bytes)
                .context("Failed to load image from HTTP response")?;

            let mut shared = shared.lock().unwrap();

            if let Some(participant) = shared.participants.get_mut(&identity) {
                participant.avatar = Some(Arc::new(
                    video::placeholder::avatar_to_placeholder(&avatar)
                        .context("Failed to convert received avatar to I420Buffer")?,
                ));
            }

            Ok(())
        }

        // Return early if the participant doesn't exist
        if !self
            .shared
            .lock()
            .unwrap()
            .participants
            .contains_key(identity)
        {
            return;
        }

        let http_client = self.http_client.clone();
        let shared = self.shared.clone();
        let identity = identity.clone();

        tokio::spawn(async move {
            if let Err(e) = load_avatar_from_url(http_client, shared, identity, &avatar_url).await {
                log::warn!("Failed to fetch avatar from url={avatar_url}, error={e:?}");
            }
        });
    }

    /// # Panics
    ///
    /// This can fail if the event could not be send to internal the channel.
    pub fn remove_participant(&mut self, identity: &ParticipantIdentity) {
        log::debug!("Remove participant {identity:?}");

        self.video_stream_tx
            .send(VideoStreamCommand::RemoveParticipant(identity.to_owned()))
            .expect("unable to send add remove event to video_stream_tx");
    }

    /// Sets the video support of this [`Mixer`].
    ///
    /// # Panics
    ///
    /// Panics if the [`Shared`] lock couldn't be acquired.
    pub fn set_video_support(&mut self, enabled: bool) {
        let mut shared = self.shared.lock().unwrap();

        shared.render_frames = enabled;

        // set subscription state of all participant publications
        for (pid, remote_participant) in self
            .room
            .as_ref()
            .expect("room exists")
            .remote_participants()
        {
            if !shared.participants.contains_key(&pid) {
                // not tracking this participant, skip
                continue;
            }

            for publication in remote_participant.track_publications().into_values() {
                if matches!(publication.kind(), TrackKind::Video) {
                    publication.set_subscribed(enabled);
                }
            }
        }
    }

    pub async fn close(&mut self) {
        let Some(room) = self.room.take() else {
            return;
        };

        self.audio_mixer_task.abort();

        close_mixer(
            self.shutdown_tx.clone(),
            self.video_task.take(),
            self.sinks.clone(),
            room,
        )
        .await;
    }
}

impl Drop for Mixer {
    fn drop(&mut self) {
        let Some(room) = self.room.take() else {
            // already shut down
            return;
        };

        self.audio_mixer_task.abort();

        let video_task = self.video_task.take();

        tokio::spawn(close_mixer(
            self.shutdown_tx.clone(),
            video_task,
            self.sinks.clone(),
            room,
        ));
    }
}

async fn close_mixer(
    shutdown_tx: watch::Sender<WantShutdown>,
    video_task: Option<JoinHandle<()>>,
    sinks: Arc<Mutex<HashMap<String, Box<dyn Sink + 'static>>>>,
    livekit_room: Room,
) {
    if let Err(e) = livekit_room.close().await {
        log::warn!("Failed to close livekit room, {e:?}");
    }

    log::debug!("Send shutdown to all tasks");

    shutdown_tx.send_replace(WantShutdown::Yes);

    if let Some(video_task) = video_task {
        if !video_task.is_finished() {
            log::debug!("Waiting for video task to be finished");

            match timeout(Duration::from_secs(8), video_task).await {
                Ok(Ok(())) => {
                    log::debug!("Video task has completed");
                }
                Ok(Err(err)) => {
                    log::warn!("Failed to wait for video task, {err:?}");
                }
                Err(_) => {
                    log::warn!("Timeout while waiting for video task");
                }
            }
        }
    }

    log::debug!("Drop all active sinks");
    let mut sinks = sinks.lock().await;

    for (name, mut sink) in sinks.drain() {
        close_sink(&name, sink.as_mut()).await;
    }
}

async fn close_sink(name: &str, sink: &mut dyn Sink) {
    log::debug!("Closing sink {name:?}");

    if let Err(err) = sink.close().await {
        log::warn!("Failed to close sink {name}, {err:?}");
    }
}

/// Create a livekit token
///
/// # Errors
///
/// If the given strings are empty this function will fail
pub fn create_token(
    api_key: &str,
    api_secret: &str,
    room: &str,
    name: &str,
) -> Result<String, AccessTokenError> {
    AccessToken::with_api_key(api_key, api_secret)
        .with_identity(uuid::Uuid::new_v4().to_string().as_str())
        .with_name(name)
        .with_grants(VideoGrants {
            room_join: true,
            room: room.to_string(),
            hidden: false,
            recorder: true,
            ..Default::default()
        })
        .to_jwt()
}

#[cfg(test)]
mod tests {

    //! Providing an url with a trailing slash to livekit will result in an invalid livekit url with two slashes in the
    //! path. These tests ensure that we are aware of breaking changes in their api in regards to the signaling url.
    use std::net::SocketAddr;

    use axum::{routing::get, Router};
    use livekit_api::signal_client::{SignalError, SignalOptions};
    use reqwest::StatusCode;

    async fn spawn_mock_livekit_server() -> SocketAddr {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let mock_livekit = Router::new()
                .route("/livekit/rtc", get(|| async { StatusCode::OK }))
                .route("/livekit/rtc/validate", get(|| async { StatusCode::OK }))
                .into_make_service();

            axum::serve(listener, mock_livekit).await.unwrap();
        });

        addr
    }

    async fn connect_livekit_to_url(url: &str) -> Result<(), SignalError> {
        livekit_api::signal_client::SignalClient::connect(
            url,
            "doesn't matter",
            SignalOptions::default(),
            None,
        )
        .await
        .map(|_| ())
    }

    #[tokio::test]
    async fn livekit_url_regression_test() {
        let addr = spawn_mock_livekit_server().await;
        let url = format!("http://{addr}/livekit");
        let result = connect_livekit_to_url(&url).await;

        match result {
            Ok(_) => unreachable!(),
            Err(e) => {
                match e {
                    SignalError::Handshake { status } if status == StatusCode::OK => {
                        // success case, the livekit path was correct and we got a websocket error
                        return;
                    }
                    other => {
                        panic!("Expected a handshake error, but got a different error: {other:?}")
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn livekit_url_trailing_slash() {
        let addr = spawn_mock_livekit_server().await;
        let url = format!("http://{addr}/livekit/");
        let result = connect_livekit_to_url(&url).await;

        match result {
            Ok(_) => unreachable!(),
            Err(e) => match e {
                SignalError::Client(status, ..) => {
                    assert_eq!(status, StatusCode::NOT_FOUND);
                    return;
                }
                other => {
                    panic!("Expected a not found error, but got a different error: {other:?}")
                }
            },
        }
    }
}
