// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use audio_nodes::AudioConvert;
use ezk::{Frame, Source, SourceEvent};
use ezk_audio::{Format, RawAudio, RawAudioConfigRange, SampleRate};
use livekit::{
    RoomEvent,
    id::{ParticipantSid, TrackSid},
    prelude::{Participant, RemoteTrackPublication},
    track::{RemoteAudioTrack, RemoteTrack},
    webrtc::audio_stream::native::NativeAudioStream,
};
use openidconnect::{ResourceOwnerPassword, ResourceOwnerUsername};
use opentalk_client_signaling::{
    AuthConfig, ControllerConfig, Event, OpenTalkClient, Room,
    config::{ClientConfig, DirectAccessCredentialsAuthConfig},
    types::signaling::chat,
};
use tokio::sync::{Mutex, mpsc};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperState};

use crate::{
    AppState, StartRequestBody,
    audio::{CHANNELS, NativeAudioStreamSource, SAMPLE_RATE},
};

pub(crate) struct TranscriptionSession {
    service_context: Arc<AppState>,

    room: Room,

    speakers: Vec<Participant>,

    whisper: Arc<Mutex<HashMap<TrackSid, Whisper>>>,
    send_messages_rx: mpsc::Receiver<(ParticipantSid, String)>,
    send_messages_tx: mpsc::Sender<(ParticipantSid, String)>,
}

impl TranscriptionSession {
    pub(crate) async fn create(
        service_context: Arc<AppState>,
        room_info: StartRequestBody,
    ) -> Result<TranscriptionSession> {
        let client = OpenTalkClient::create(ClientConfig {
            auth: AuthConfig::DirectAccess(DirectAccessCredentialsAuthConfig {
                issuer: service_context.settings.auth.issuer.clone(),
                client_id: service_context.settings.auth.client_id.clone(),
                client_secret: service_context.settings.auth.client_secret.clone(),
                username: ResourceOwnerUsername::new(String::from("bob")),
                password: ResourceOwnerPassword::new(String::from("bob")),
            }),
            controller: ControllerConfig {
                domain: service_context.settings.controller.domain.clone(),
                insecure: service_context.settings.controller.insecure,
            },
        })
        .await?;
        let room = client
            .connect_as_user(
                "transcription".parse()?,
                room_info.room_id,
                room_info.breakout_room,
                true,
            )
            .await?
            .into();

        let (send_messages_tx, send_messages_rx) = mpsc::channel(1000);

        Ok(Self {
            service_context,
            room,
            speakers: Vec::new(),
            whisper: Arc::new(Mutex::new(HashMap::new())),
            send_messages_rx,
            send_messages_tx,
        })
    }

    pub(crate) async fn run(&mut self) -> Result<()> {
        let mut shutdown_rx = self.service_context.shutdown.resubscribe();

        loop {
            tokio::select! {
                event = self.room.recv() => {
                    let event = match event {
                        Err(error) => {
                            log::error!("Received event error: {error:?}");
                            continue;
                        },
                        Ok(event) => event,
                    };
                    match event {
                        Event::OpenTalk(event) => log::trace!("Received OpenTalk event: {event:?}"),
                        Event::LiveKit(event) => self.handle_livekit_event(event).await,
                    }
                }
                Some((participant, message)) = self.send_messages_rx.recv() => {
                    log::debug!("send message {message}");
                    self
                        .room
                        .chat_api()
                        .unwrap()
                        .send_message(
                            format!("[{participant}] said: {message}"),
                            chat::Scope::Global
                        ).await?;
                }
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_livekit_event(&mut self, event: RoomEvent) {
        log::trace!("livekit event: {event:?}");

        match event {
            RoomEvent::ActiveSpeakersChanged { speakers } => {
                self.handle_speaker_change(speakers).await;
            }
            RoomEvent::TrackSubscribed {
                track: RemoteTrack::Audio(audio_track),
                publication,
                participant: _,
            } => self.handle_new_audio_track(audio_track, publication).await,
            _ => {}
        }
    }

    async fn handle_new_audio_track(
        &mut self,
        audio_track: RemoteAudioTrack,
        publication: RemoteTrackPublication,
    ) {
        let whisper_context = self.service_context.whisper_context.clone();
        let whisper = self.whisper.clone();

        tokio::spawn(async move {
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
                    channels: ezk::ValueRange::Value(ezk_audio::Channels::NotPositioned(CHANNELS)),
                    format: ezk::ValueRange::Value(Format::F32),
                }])
                .await
                .unwrap();

            while let Ok(source_event) = audio_convert.next_event().await {
                if let SourceEvent::Frame(frame) = source_event {
                    whisper
                        .lock()
                        .await
                        .entry(publication.sid())
                        .or_insert_with(|| Whisper::new(whisper_context.clone()).unwrap())
                        .extend_audio_frame(frame);
                }
            }
        });
    }

    async fn handle_speaker_change(&mut self, speakers: Vec<Participant>) {
        log::trace!("speaker changed: {speakers:?}");

        for old_speaker in &self.speakers {
            // Still speaking, skip
            if speakers.iter().any(|p| p.sid() == old_speaker.sid()) {
                continue;
            }

            // Stopped speaking, do ai stuff
            for track in old_speaker.track_publications().keys() {
                let participant = old_speaker.sid().clone();
                let track = track.clone();
                let whisper = self.whisper.clone();
                let send_messages_tx = self.send_messages_tx.clone();

                tokio::task::spawn_blocking(move || {
                    let mut whisper = whisper.blocking_lock();
                    let Some(whisper) = whisper.get_mut(&track) else {
                        return;
                    };

                    if let Ok(data) = whisper.do_ai_and_clear() {
                        send_messages_tx.blocking_send((participant, data)).unwrap();
                    }
                });
            }
        }

        self.speakers = speakers;
    }
}

struct Whisper {
    state: WhisperState,
    audio_samples: Vec<f32>,
}

impl Whisper {
    fn new(context: Arc<WhisperContext>) -> Result<Self> {
        let state = context.create_state()?;
        let audio_samples = Vec::new();

        Ok(Self {
            state,
            audio_samples,
        })
    }
}

impl Whisper {
    fn extend_audio_frame(&mut self, frame: Frame<RawAudio>) {
        let samples = frame.into_data().samples.into_samples::<f32>();
        self.audio_samples.extend(samples);
    }

    fn do_ai_and_clear(&mut self) -> Result<String> {
        let sample_duration =
            self.audio_samples.len() as f32 / SAMPLE_RATE as f32 / CHANNELS as f32;
        if sample_duration < 1.0 {
            self.audio_samples
                .resize((SAMPLE_RATE * CHANNELS * 2) as usize, 0.0);
        }

        let result = self.do_ai();
        self.clear_audio_frame();
        Ok(result?.unwrap())
    }

    fn do_ai(&mut self) -> Result<Option<String>> {
        let sample_duration =
            self.audio_samples.len() as f32 / SAMPLE_RATE as f32 / CHANNELS as f32;

        if sample_duration < 1.0 {
            return Ok(None);
        }

        let mut segments = Vec::new();

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        params.set_language(Some("de"));

        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        self.state.full(params, &self.audio_samples[..])?;

        let num_segments = self.state.full_n_segments()?;
        for i in 0..num_segments {
            let segment = self.state.full_get_segment_text(i)?;
            segments.push(segment);
        }

        Ok(Some(segments.join("").trim().to_owned()))
    }

    fn clear_audio_frame(&mut self) {
        self.audio_samples.clear();
    }
}
