// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{HashMap, VecDeque},
    str::FromStr,
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use audio_nodes::AudioConvert;
use ezk::{Source, SourceEvent};
use ezk_audio::{Format, RawAudioConfigRange, SampleRate};
use livekit::{
    RoomEvent,
    id::{ParticipantIdentity, TrackSid},
    prelude::RemoteParticipant,
    track::{RemoteAudioTrack, RemoteTrack},
    webrtc::audio_stream::native::NativeAudioStream,
};
use opentalk_client_signaling::{
    OpenTalkEvent, Room,
    opentalk_roomserver_types::{
        breakout::{breakout_id::BreakoutId, event::BreakoutEvent},
        room_kind::RoomKind,
    },
    opentalk_roomserver_types_livekit::Credentials as LivekitCredentials,
    opentalk_roomserver_types_transcription::{
        segment::TranscriptionSegment, service::command::TranscriptionServiceCommand,
    },
    types::{
        common::{rooms::RoomId, time::Timestamp},
        signaling::ParticipantId,
    },
};
use opentalk_transcription_web_api::v1::TranscriptionTarget;
use tokio::sync::{
    Mutex,
    mpsc::{self, UnboundedReceiver},
};
use voice_activity_detector::VoiceActivityDetector;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperState};

use crate::{
    AppState,
    audio::{CHANNELS, NativeAudioStreamSource, SAMPLE_RATE},
    voice_activity_detection::{
        MINIMUM_SAMPLES, POST_BUFFER_LENGTH, SAMPLE_SIZE, SilenceBuffer, detect_speech,
        sample_block_per_duration,
    },
};

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
        let room = self.room.take().expect("livekit room must be some");

        tokio::spawn(async move {
            if let Err(err) = room.close().await {
                log::error!("Failed to disconnect from livekit room: {err:?}");
            }
        });
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub(crate) struct TranscriptionParticipant {
    pub(crate) identity: ParticipantIdentity,
    pub(crate) track_sid: TrackSid,
}

pub(crate) struct TranscriptionSession {
    app_state: Arc<AppState>,

    run: bool,

    room: Room,
    room_id: RoomId,
    livekit_room: LivekitRoom,

    language: Option<String>,

    transcription: Arc<Mutex<HashMap<TranscriptionParticipant, TranscriptionContext>>>,

    segments_rx: mpsc::Receiver<TranscriptionSegment>,
    segments_tx: mpsc::Sender<TranscriptionSegment>,
}

impl TranscriptionSession {
    pub(crate) async fn create(
        app_state: Arc<AppState>,
        room_info: TranscriptionTarget,
    ) -> Result<TranscriptionSession> {
        let breakout_id: Option<BreakoutId> = room_info
            .breakout_room
            .map(|breakout_id| BreakoutId::from(breakout_id));

        let mut room: Room = app_state
            .opentalk_client
            .connect_transcription(room_info.room_id, breakout_id)
            .await?
            .into();

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
            transcription: Arc::new(Mutex::new(HashMap::new())),
            segments_rx,
            segments_tx,
        })
    }

    pub(crate) async fn run(&mut self) -> Result<()> {
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
                    self.handle_livekit_event(livekit_event.unwrap()).await?;
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
            | OpenTalkEvent::RecordingService(_) => {}
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
            } => {
                self.transcription.lock().await.insert(
                    TranscriptionParticipant {
                        identity: participant.identity().clone(),
                        track_sid: audio_track.sid().clone(),
                    },
                    TranscriptionContext::new(
                        self.app_state.whisper_context.clone(),
                        self.language.clone(),
                    )?,
                );

                log::debug!("Added new participant: {}", participant.identity());

                self.handle_new_audio_track(audio_track, participant);
            }
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
        let language = self.language.clone();
        let whisper_context = self.app_state.whisper_context.clone();

        let transcription = self.transcription.clone();

        let identity = participant.identity();
        let Some((participant_id, ..)) = identity.as_str().split_once(':') else {
            log::warn!(
                "Got new audio track from invalid participant identity {}",
                participant.identity()
            );
            return;
        };

        let participant_id = ParticipantId::from_str(participant_id).unwrap(); // TODO: result return type

        let transcription_participant = TranscriptionParticipant {
            identity: participant.identity().clone(),
            track_sid: audio_track.sid().clone(),
        };

        let send_messages_tx = self.segments_tx.clone();

        tokio::spawn({
            async move {
                let mut audio_convert = AudioConvert::new(NativeAudioStreamSource {
                    stream: NativeAudioStream::new(
                        audio_track.rtc_track(),
                        SAMPLE_RATE as i32,
                        CHANNELS as i32,
                    ),
                    timestamp: 0,
                });

                audio_convert
                    .negotiate_config(vec![RawAudioConfigRange {
                        sample_rate: ezk::ValueRange::Value(SampleRate(SAMPLE_RATE)),
                        channels: ezk::ValueRange::Value(ezk_audio::Channels::NotPositioned(
                            CHANNELS,
                        )),
                        format: ezk::ValueRange::Value(Format::F32),
                    }])
                    .await
                    .unwrap();

                let mut silence_buffer = SilenceBuffer::Silence;
                loop {
                    match audio_convert.next_event().await {
                        Ok(SourceEvent::Frame(frame)) => {
                            let samples = frame.into_data().samples.into_samples::<f32>();
                            log::trace!(
                                "new raw audio frame with length: {} received.",
                                samples.len()
                            );
                            on_new_frame(
                                &whisper_context,
                                &transcription,
                                language.clone(),
                                samples,
                                &transcription_participant,
                                &mut silence_buffer,
                                &send_messages_tx,
                                &participant_id,
                            )
                            .await;
                        }
                        Ok(source_event) => {
                            log::warn!("Received event: {source_event:?}");
                        }
                        Err(err) => {
                            log::error!("Received error on event retrieval: {err:?}");
                        }
                    }
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

async fn on_new_frame(
    whisper_context: &Arc<WhisperContext>,
    transcription: &Arc<Mutex<HashMap<TranscriptionParticipant, TranscriptionContext>>>,
    language: Option<String>,
    samples: Vec<f32>,
    transcription_participant: &TranscriptionParticipant,
    silence_buffer: &mut SilenceBuffer,
    send_messages_tx: &mpsc::Sender<TranscriptionSegment>,
    participant_id: &ParticipantId,
) {
    let mut whisper_guard = transcription.lock().await;

    let track_id = whisper_guard.iter().find_map(|(participant, _context)| {
        if participant == transcription_participant {
            Some(participant.track_sid.to_string())
        } else {
            None
        }
    });

    let Some(track_id) = track_id else {
        log::error!("Failed to resolve track sid for participant: {transcription_participant:?}");
        return;
    };

    let whisper_entry = whisper_guard
        .entry(transcription_participant.clone())
        .or_insert_with(|| TranscriptionContext::new(whisper_context.clone(), language).unwrap());
    whisper_entry.extend_audio_frame(&samples);

    let detected = detect_speech(whisper_entry).await;
    if detected {
        *silence_buffer = SilenceBuffer::HadActivity(Vec::with_capacity(
            sample_block_per_duration(SAMPLE_RATE, CHANNELS, POST_BUFFER_LENGTH),
        ));
    } else if let SilenceBuffer::HadActivity(activity) = silence_buffer {
        activity.extend(&samples);
    }

    let SilenceBuffer::HadActivity(silence) = silence_buffer else {
        return;
    };

    if silence.len() < sample_block_per_duration(SAMPLE_RATE, CHANNELS, POST_BUFFER_LENGTH) {
        return;
    }

    log::debug!(
        "silence buffer exceeded minimum size of: {}",
        sample_block_per_duration(SAMPLE_RATE, CHANNELS, POST_BUFFER_LENGTH)
    );
    whisper_entry.samples.extend(silence.iter());
    *silence_buffer = SilenceBuffer::Silence;

    if let Ok(data) = whisper_entry.transcribe_segment() {
        let segment = TranscriptionSegment {
            participant_id: *participant_id,
            track_id,
            starts_at: whisper_entry.start_point,
            ends_at: Timestamp::now(),
            text: data,
        };

        send_messages_tx.send(segment).await.unwrap();
    }
}

pub(crate) struct TranscriptionContext {
    state: WhisperState,
    language: Option<String>,
    pub(crate) audio_queue: VecDeque<f32>,
    pub(crate) samples: Vec<f32>,
    pub(crate) voice_activity_detector: VoiceActivityDetector,
    pub(crate) start_point: Timestamp,
}

impl TranscriptionContext {
    pub(crate) fn new(context: Arc<WhisperContext>, language: Option<String>) -> Result<Self> {
        let state = context.create_state()?;
        let audio_samples = VecDeque::with_capacity(SAMPLE_SIZE * 2);
        let voice_activity_detector = VoiceActivityDetector::builder()
            .sample_rate(SAMPLE_RATE)
            .chunk_size(SAMPLE_SIZE)
            .build()?;

        Ok(TranscriptionContext {
            state,
            language,
            audio_queue: audio_samples,
            samples: (0..MINIMUM_SAMPLES).map(|_| 0.0).collect(),
            voice_activity_detector,
            start_point: Timestamp::now(),
        })
    }

    pub(crate) fn extend_audio_frame(&mut self, samples: &Vec<f32>) {
        self.audio_queue.extend(samples);
    }

    pub(crate) fn transcribe_segment(&mut self) -> Result<String> {
        let sample_duration = self.samples.len() as f32 / SAMPLE_RATE as f32 / CHANNELS as f32;

        if sample_duration < 1.0 {
            self.samples
                .resize((SAMPLE_RATE * CHANNELS * 2) as usize, 0.0);
        }

        let mut segments = Vec::new();

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        params.set_language(self.language.as_deref());

        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        self.state.full(params, &self.samples[..])?;

        let num_segments = self.state.full_n_segments();
        for i in 0..num_segments {
            let segment = self
                .state
                .get_segment(i)
                .with_context(|| format!("Couldn't get segment #{i}"))?;

            let segmment_string = segment
                .to_str_lossy()
                .with_context(|| format!("Failed to get text for segment #{i}"))?;

            if !segmment_string.is_empty() {
                segments.push(segmment_string);
            }
        }

        self.samples.clear();

        Ok(segments.join("").trim().to_owned())
    }
}
