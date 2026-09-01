// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    str::FromStr,
    sync::Arc,
    time::{Instant, SystemTime},
};

use anyhow::{Context, Result, bail};
use livekit::{
    RoomEvent,
    prelude::RemoteParticipant,
    track::{RemoteAudioTrack, RemoteTrack},
};
use opentalk_client_signaling::{
    JoinedLobby, OpenTalkEvent, Room,
    opentalk_roomserver_types::{
        breakout::{breakout_id::BreakoutId, event::BreakoutEvent},
        room_kind::RoomKind,
    },
    opentalk_roomserver_types_livekit::Credentials as LivekitCredentials,
    opentalk_roomserver_types_transcription::{
        segment::TranscriptionSegment, service::command::TranscriptionServiceCommand,
    },
    opentalk_types::{
        common::{rooms::RoomId, time::Timestamp, users::DisplayName},
        signaling::ParticipantId,
    },
};
use opentalk_transcription_web_api::v1::TranscriptionTarget;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver},
    watch,
};
use whisper_rs::WhisperContext;

use crate::{
    AppState,
    audio_processing::{
        self,
        audio_source::AudioSource,
        transcription::TranscriptionStage,
        voice_activity_detection::{VadConfig, VadStage},
    },
    settings::Settings,
};

const DISPLAY_NAME: &str = "transcription";

/// A connection to a LiveKit room together with its event stream.
///
/// On drop, any still-open room is closed in a detached task.
pub(crate) struct LivekitRoom {
    room: Option<livekit::Room>,
    event_stream: UnboundedReceiver<RoomEvent>,
}

impl LivekitRoom {
    pub(crate) async fn connect(creds: &LivekitCredentials) -> Result<Self> {
        let url = creds
            .service_url
            .clone()
            .unwrap_or(creds.public_url.clone());

        let url = url.trim_end_matches('/');

        let (room, event_stream) =
            livekit::Room::connect(url, &creds.token, livekit::RoomOptions::default())
                .await
                .context("failed to connect to livekit room")?;

        Ok(Self {
            room: Some(room),
            event_stream,
        })
    }

    pub(crate) async fn recv(&mut self) -> Option<RoomEvent> {
        self.event_stream.recv().await
    }
}

impl Drop for LivekitRoom {
    fn drop(&mut self) {
        let Some(room) = self.room.take() else {
            // Room was already taken/dropped, nothing to do
            return;
        };

        tokio::spawn(async move {
            if let Err(err) = room.close().await {
                log::error!("Failed to disconnect from livekit room: {err:?}");
            }
        });
    }
}

#[derive(Debug)]
pub enum SessionState {
    Run,
    Exit,
}

/// The handle for a [`TranscriptionSession`]
#[derive(Debug, Clone)]
pub(crate) struct TranscriptionSessionHandle {
    state_sender: watch::Sender<SessionState>,
}

impl TranscriptionSessionHandle {
    /// Signal the transcription session to exit
    pub(crate) fn exit_task(&self) {
        // can only fail if the receiver is dropped -> 'successful' exit
        let _ = self.state_sender.send(SessionState::Exit);
    }

    /// Check if the associated session is finished
    pub(crate) fn is_finished(&self) -> bool {
        self.state_sender.is_closed()
    }

    /// Completes when the transcription session has exited
    pub(crate) async fn closed(&self) {
        self.state_sender.closed().await
    }
}

/// A running transcription session for a single OpenTalk room.
///
/// It bridges OpenTalk and LiveKit room events, spawns a transcription pipeline per audio track
/// and forwards the resulting [`TranscriptionSegment`]s back to the controller.
pub(crate) struct TranscriptionSession {
    app_state: Arc<AppState>,
    state_receiver: watch::Receiver<SessionState>,

    run: bool,

    room: Room,
    room_id: RoomId,
    livekit_room: LivekitRoom,

    language: watch::Sender<Option<String>>,

    segments_rx: mpsc::Receiver<TranscriptionSegment>,
    segments_tx: mpsc::Sender<TranscriptionSegment>,
}

impl TranscriptionSession {
    /// Connects to the controller and the LiveKit room for the given target.
    ///
    /// When a breakout room is targeted, the session moves into it before returning.
    pub(crate) async fn create(
        app_state: Arc<AppState>,
        room_info: TranscriptionTarget,
    ) -> Result<(TranscriptionSession, TranscriptionSessionHandle)> {
        let breakout_id: Option<BreakoutId> = room_info.breakout_room.map(BreakoutId::from);

        let JoinedLobby {
            mut room,
            can_enter,
            display_name,
        } = app_state
            .opentalk_client
            .connect_transcription(room_info.room_id, breakout_id)
            .await?;

        if !can_enter {
            bail!("not allowed to enter the room");
        }

        let display_name = display_name
            .is_none()
            .then(|| DisplayName::from_str_lossy(DISPLAY_NAME));
        room.enter_room(display_name).await?;

        if let Some(breakout_id) = room_info.breakout_room {
            move_to_breakout_room(&mut room, breakout_id.into()).await?;
        }

        let credentials = room
            .livekit_credentials()
            .context("no livekit credentials available")?;
        let livekit_room = LivekitRoom::connect(credentials).await?;

        let (segments_tx, segments_rx) = mpsc::channel(1000);
        let (state_sender, state_receiver) = watch::channel(SessionState::Run);

        Ok((
            TranscriptionSession {
                app_state,
                state_receiver,
                run: true,
                room,
                livekit_room,
                room_id: room_info.room_id,
                language: watch::channel(room_info.language).0,
                segments_rx,
                segments_tx,
            },
            TranscriptionSessionHandle { state_sender },
        ))
    }

    /// Runs the session event loop until the room ends, a stop is requested or the service shuts
    /// down, then disconnects from both the LiveKit and the OpenTalk room.
    pub(crate) async fn run(mut self) -> Result<()> {
        let language = self.language.borrow().clone();

        self.room
            .transcription_service_api()
            .context("transcription service api not available")?
            .send_transcription_started(language)
            .await?;

        let mut shutdown_rx = self.app_state.shutdown.resubscribe();
        let mut state_receiver = self.state_receiver.clone();

        while self.run {
            tokio::select! {
                event = self.room.recv() => {
                    let event = match event {
                        Err(error) => {
                            log::error!("Received event error: {error:?}");
                            continue;
                        },
                        Ok(event) => event,
                    };
                    self.handle_opentalk_event(event).await?;
                }
                livekit_event = self.livekit_room.recv() => {
                    match livekit_event {
                        Some(event) => self.handle_livekit_event(event).await?,
                        None => {
                            log::info!("LiveKit event stream closed, stopping transcription");
                            self.run = false;
                        }
                    }
                }
                Some(segment) = self.segments_rx.recv() => {
                    log::debug!("Sending segment {segment:?}");

                    self.room
                        .transcription_service_api()
                        .context("transcription service api not available")?
                        .send_transcription_segment(segment)
                        .await?;
                }
                res = state_receiver.changed() => {
                    if matches!(*state_receiver.borrow_and_update(), SessionState::Exit) || res.is_err() {
                        break;
                    }
                }
                _ = shutdown_rx.wait_for_shutdown() => {
                    break;
                }
            }
        }

        if let Some(livekit_room) = self.livekit_room.room.take()
            && let Err(e) = livekit_room.close().await
        {
            log::error!("Failed to disconnect from livekit room: {e:?}");
        }

        self.room.close().await?;

        Ok(())
    }

    async fn handle_opentalk_event(&mut self, event: OpenTalkEvent) -> Result<()> {
        match event {
            OpenTalkEvent::ParticipantLeft(..) => {
                if self.room.active_participants().is_empty() {
                    log::info!("No active participants left in the room, stopping transcription");
                    self.run = false;
                }
            }
            OpenTalkEvent::MovedToWaitingRoom => {
                log::error!("Moved to waiting room, exiting room: {}", self.room_id);
                self.run = false;
            }
            OpenTalkEvent::Breakout(breakout_event) => {
                if let Err(err) = self.handle_breakout_event(breakout_event).await {
                    log::warn!("Handling breakout event failed with {err:?}");
                    return Err(err);
                }
            }
            OpenTalkEvent::LiveKit(_)
            | OpenTalkEvent::WaitingRoomAccepted
            | OpenTalkEvent::Recording(_)
            | OpenTalkEvent::ParticipantJoined(_)
            | OpenTalkEvent::ParticipantUpdated {
                previous: _,
                updated: _,
            }
            | OpenTalkEvent::RecordingService(_)
            | OpenTalkEvent::EntryPermissionChanged { can_enter: _ }
            | OpenTalkEvent::DisplayNameAssigned(_) => {}
            OpenTalkEvent::Transcription(..) => (),
            OpenTalkEvent::TranscriptionService(transcription_service_command) => {
                match transcription_service_command {
                    TranscriptionServiceCommand::Stop => {
                        log::debug!("Got stop event");
                        self.room
                            .transcription_service_api()
                            .context("transcription service api not available")?
                            .send_transcription_stopped()
                            .await
                            .context("failed to send transcription stopped event")?;

                        self.run = false;
                    }
                    TranscriptionServiceCommand::ChangeLanguage { language } => {
                        let changed = self.language.send_if_modified(|current_language| {
                            if *current_language == language {
                                return false;
                            }

                            *current_language = language.clone();

                            true
                        });

                        if changed {
                            self.room
                                .transcription_service_api()
                                .context("transcription service api not available")?
                                .send_transcription_updated(language)
                                .await
                                .context("Failed to send transcription updated event")?;
                        }
                    }
                }
            }
            OpenTalkEvent::Disconnected(disconnect_reason) => {
                log::debug!("Got disconnected event: {disconnect_reason:?}");
                self.run = false;
            }
        }

        Ok(())
    }

    async fn handle_livekit_event(&mut self, event: RoomEvent) -> Result<()> {
        log::trace!("livekit event: {event:?}");

        if let RoomEvent::TrackSubscribed {
            track: RemoteTrack::Audio(audio_track),
            publication: _,
            participant,
        } = event
        {
            self.handle_new_audio_track(audio_track, participant)
        }

        Ok(())
    }

    async fn handle_breakout_event(&mut self, breakout_event: BreakoutEvent) -> Result<()> {
        match breakout_event {
            BreakoutEvent::ParticipantSwitchedRoom { .. } => {
                // nothing to do, participant tracks are added through livekit events and removed through track state
            }
            BreakoutEvent::Closed => {
                if self.room.breakout_api().current_room() != RoomKind::Main {
                    log::info!("Breakout room was closed, stopping transcription");
                    self.run = false;
                }
            }
            BreakoutEvent::Error(breakout_error) => {
                log::error!("Received unexpected error from breakout module: {breakout_error:?}");
            }

            BreakoutEvent::Started { .. }
            | BreakoutEvent::SwitchedRoom { .. }
            | BreakoutEvent::CloseNotice { .. }
            | BreakoutEvent::Closing { .. } => (),
        }

        Ok(())
    }

    fn handle_new_audio_track(
        &mut self,
        audio_track: RemoteAudioTrack,
        participant: RemoteParticipant,
    ) {
        let identity = participant.identity();
        let Some((participant_id, ..)) = identity.as_str().split_once(':') else {
            log::warn!(
                "Got new audio track from invalid participant identity {}",
                participant.identity()
            );
            return;
        };

        let participant_id = match ParticipantId::from_str(participant_id) {
            Ok(participant_id) => participant_id,
            Err(err) => {
                log::warn!(
                    "Got new audio track from participant with unparseable id {participant_id:?}: {err:?}"
                );
                return;
            }
        };
        let track_id = audio_track.sid().to_string();

        let settings = self.app_state.settings.clone();
        let whisper_context = self.app_state.whisper_context.clone();
        let language = self.language.subscribe();
        let send_tx = self.segments_tx.clone();

        tokio::spawn(transcribe_audio_track_task(
            audio_track,
            participant_id,
            track_id,
            settings,
            whisper_context,
            language,
            send_tx,
        ));
    }
}

async fn transcribe_audio_track_task(
    audio_track: RemoteAudioTrack,
    participant_id: ParticipantId,
    track_id: String,
    settings: Arc<Settings>,
    whisper_context: Arc<WhisperContext>,
    mut language: watch::Receiver<Option<String>>,
    segments_tx: mpsc::Sender<TranscriptionSegment>,
) {
    let mut audio_source = match AudioSource::new(audio_track).await {
        Ok(source) => source,
        Err(err) => {
            log::error!("Failed to create AudioSource: {err:?}");
            return;
        }
    };
    let mut vad = match VadStage::new(VadConfig::default()) {
        Ok(vad) => vad,
        Err(err) => {
            log::error!("Failed to create VoiceActivityDetector: {err:?}");
            return;
        }
    };
    let mut transcription =
        match TranscriptionStage::new(whisper_context.clone(), language.borrow().clone()) {
            Ok(transcription) => transcription,
            Err(err) => {
                log::error!("Failed to create Transcription stage: {err:?}");
                return;
            }
        };

    loop {
        enum Event {
            LanguageChanged(Option<String>),
            SamplesReceived(Option<Vec<f32>>),
        }

        let event = tokio::select! {
            _ = language.changed() => Event::LanguageChanged(language.borrow_and_update().clone()),
            samples = audio_source.next_samples() => Event::SamplesReceived(samples),
        };

        match event {
            Event::LanguageChanged(language) => {
                match TranscriptionStage::new(whisper_context.clone(), language.clone()) {
                    Ok(t) => transcription = t,
                    Err(err) => log::warn!(
                        "Failed to re-create transcription stage with new language {language:?}, {err:?}"
                    ),
                }
            }
            Event::SamplesReceived(samples) => {
                let Some(samples) = samples else {
                    break;
                };

                let Some(mut speech_segment) = vad.process(samples) else {
                    continue;
                };

                // Convert Instant to DateTime UTC using SystemTime
                let instant_now = Instant::now();
                let system_now = SystemTime::now();
                let starts_at = (system_now - (instant_now - speech_segment.started_at)).into();

                match transcription.transcribe(&mut speech_segment) {
                    Ok(transcript) => {
                        if let Some(dump_dir) = &settings.transcription.dump_dir {
                            audio_processing::dump::dump(
                                &speech_segment,
                                &transcript.text,
                                dump_dir,
                            );
                        }

                        if transcript.text.is_empty() {
                            continue;
                        }

                        let segment = TranscriptionSegment {
                            participant_id,
                            track_id: track_id.clone(),
                            starts_at,
                            ends_at: Timestamp::now(),
                            text: transcript.text,
                        };

                        if segments_tx.send(segment).await.is_err() {
                            log::debug!("Segment channel closed, stopping audio pipeline");
                            break;
                        }
                    }
                    Err(e) => log::error!("Transcription failed: {e:?}"),
                }
            }
        }
    }
}

async fn move_to_breakout_room(room_state: &mut Room, breakout_target: BreakoutId) -> Result<()> {
    log::info!("Moving to breakout room {breakout_target}");
    let Some(breakout_config) = room_state.breakout_api().breakout_config() else {
        return Err(anyhow::anyhow!(
            "Received breakout target {breakout_target} but there are no breakout rooms configured"
        ));
    };

    if !breakout_config
        .rooms
        .iter()
        .any(|room| room.id == breakout_target)
    {
        return Err(anyhow::anyhow!(
            "Got breakout target {breakout_target} but it was not found in the room's breakout configuration"
        ));
    }

    room_state
        .breakout_api()
        .switch_room(RoomKind::Breakout(breakout_target))
        .await?;

    log::info!("Entering breakout room, waiting for breakout switch event...");
    loop {
        match room_state.recv().await? {
            OpenTalkEvent::MovedToWaitingRoom => {
                bail!("Recorder was moved to the waiting room, cannot start recording")
            }
            OpenTalkEvent::Breakout(breakout_event) => match breakout_event {
                BreakoutEvent::SwitchedRoom { .. } => {
                    log::info!("Received breakout switched room event, recording can be started");
                    return Ok(());
                }
                BreakoutEvent::Closing { .. } | BreakoutEvent::Closed => {
                    bail!("Targeted breakout room was closed, cannot start recording")
                }
                BreakoutEvent::Error(breakout_error) => {
                    bail!("Error in breakout module: {breakout_error:?}")
                }
                _ => (),
            },
            OpenTalkEvent::Disconnected(disconnect_reason) => {
                bail!("Disconnected from room with reason: {disconnect_reason:?}")
            }
            event => log::trace!("Received event while waiting for breakout switch: {event:?}"),
        }
    }
}
