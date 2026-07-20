// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{str::FromStr, sync::Arc};

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
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::{
    AppState,
    audio_processing::{
        audio_source::AudioSource, transcription::TranscriptionStage,
        voice_activity_detection::VadStage,
    },
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
            livekit::Room::connect(&url, &creds.token, livekit::RoomOptions::default())
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

/// A running transcription session for a single OpenTalk room.
///
/// It bridges OpenTalk and LiveKit room events, spawns a transcription pipeline per audio track
/// and forwards the resulting [`TranscriptionSegment`]s back to the controller.
pub(crate) struct TranscriptionSession {
    app_state: Arc<AppState>,

    run: bool,

    room: Room,
    room_id: RoomId,
    livekit_room: LivekitRoom,

    language: Option<String>,

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
    ) -> Result<TranscriptionSession> {
        let breakout_id: Option<BreakoutId> = room_info
            .breakout_room
            .map(|breakout_id| BreakoutId::from(breakout_id));

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

        Ok(TranscriptionSession {
            app_state,
            run: true,
            room,
            livekit_room,
            room_id: room_info.room_id,
            language: room_info.language,
            segments_rx,
            segments_tx,
        })
    }

    /// Runs the session event loop until the room ends, a stop is requested or the service shuts
    /// down, then disconnects from both the LiveKit and the OpenTalk room.
    pub(crate) async fn run(mut self) -> Result<()> {
        self.room
            .transcription_service_api()
            .context("transcription service api not available")?
            .send_transcription_started()
            .await?;

        let mut shutdown_rx = self.app_state.shutdown.resubscribe();

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
                _ = shutdown_rx.wait_for_shutdown() => {
                    break;
                }
            }
        }

        if let Some(livekit_room) = self.livekit_room.room.take() {
            if let Err(e) = livekit_room.close().await {
                log::error!("Failed to disconnect from livekit room: {e:?}");
            }
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

        match event {
            RoomEvent::TrackSubscribed {
                track: RemoteTrack::Audio(audio_track),
                publication: _,
                participant,
            } => self.handle_new_audio_track(audio_track, participant),
            _ => {}
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

        let whisper_context = self.app_state.whisper_context.clone();
        let language = self.language.clone();
        let send_tx = self.segments_tx.clone();

        tokio::spawn(async move {
            let mut audio_source = match AudioSource::new(audio_track).await {
                Ok(source) => source,
                Err(err) => {
                    log::error!("Failed to create AudioSource: {err:?}");
                    return;
                }
            };
            let mut vad = match VadStage::new() {
                Ok(vad) => vad,
                Err(err) => {
                    log::error!("Failed to create VoiceActivityDetector: {err:?}");
                    return;
                }
            };
            let mut transcription = match TranscriptionStage::new(whisper_context, language) {
                Ok(transcription) => transcription,
                Err(err) => {
                    log::error!("Failed to create Transcription stage: {err:?}");
                    return;
                }
            };

            loop {
                let Some(samples) = audio_source.next_samples().await else {
                    break;
                };

                let Some(speech_segment) = vad.process(samples) else {
                    continue;
                };

                let started_at = speech_segment.started_at;
                match transcription.transcribe(speech_segment) {
                    Ok(text) if !text.is_empty() => {
                        let segment = TranscriptionSegment {
                            participant_id,
                            track_id: track_id.clone(),
                            starts_at: started_at,
                            ends_at: Timestamp::now(),
                            text,
                        };

                        if send_tx.send(segment).await.is_err() {
                            log::debug!("Segment channel closed, stopping audio pipeline");
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(e) => log::error!("Transcription failed: {e:?}"),
                }
            }
        });
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
