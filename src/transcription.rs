// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{HashMap, VecDeque},
    str::FromStr,
    sync::{Arc, atomic::AtomicU32},
};

use anyhow::Result;
use audio_nodes::AudioConvert;
use ezk::{Source, SourceEvent};
use ezk_audio::{Format, RawAudioConfigRange, SampleRate};
use livekit::{
    RoomEvent,
    id::TrackSid,
    prelude::{RemoteParticipant, RemoteTrackPublication},
    track::{RemoteAudioTrack, RemoteTrack},
    webrtc::audio_stream::native::NativeAudioStream,
};
use opentalk_client_signaling::{
    Event, OpenTalkEvent, OpenTalkTranscriptionServiceEvent, Room, TranscriptionSegment,
    types::{
        common::time::Timestamp,
        signaling::{ParticipantId, transcription::http::StartTranscriptionRequestBody},
    },
};
use tokio::sync::{Mutex, mpsc};
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

pub(crate) struct TranscriptionSession {
    app_state: Arc<AppState>,

    run: bool,

    room: Room,
    language: Option<String>,

    transcription: Arc<Mutex<HashMap<TrackSid, TranscriptionContext>>>,

    segments_rx: mpsc::Receiver<TranscriptionSegment>,
    segments_tx: mpsc::Sender<TranscriptionSegment>,
}

impl TranscriptionSession {
    pub(crate) async fn create(
        app_state: Arc<AppState>,
        room_info: StartTranscriptionRequestBody,
    ) -> Result<TranscriptionSession> {
        let mut room: Room = app_state
            .opentalk_client
            .connect_recorder(room_info.room_id, room_info.breakout_room, true)
            .await?
            .into();

        room.transcription_service_api().send_started().await?;

        let (segments_tx, segments_rx) = mpsc::channel(1000);

        Ok(TranscriptionSession {
            app_state,
            run: true,
            room,
            language: room_info.language,
            transcription: Arc::new(Mutex::new(HashMap::new())),
            segments_rx,
            segments_tx,
        })
    }

    pub(crate) async fn run(&mut self) -> Result<()> {
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
                    match event {
                        Event::OpenTalk(event) => self.handle_opentalk_event(event),
                        Event::LiveKit(event) => self.handle_livekit_event(event).await,
                    }
                }
                Some(segment) = self.segments_rx.recv() => {
                    log::debug!("Sending segment {segment:?}");

                    self.room.transcription_service_api().send_segment(segment).await?;
                }
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }

        self.room.transcription_service_api().send_stopped().await?;

        Ok(())
    }

    fn handle_opentalk_event(&mut self, event: OpenTalkEvent) {
        match event {
            OpenTalkEvent::TranscriptionService(event) => match event {
                OpenTalkTranscriptionServiceEvent::Stop => {
                    log::debug!("Got stop event");
                    self.run = false;
                }
            },
            OpenTalkEvent::Disconnected(..) => {
                log::debug!("Got disconnected event");

                self.run = false
            }
            _ => {}
        }
    }

    async fn handle_livekit_event(&mut self, event: RoomEvent) {
        log::trace!("livekit event: {event:?}");

        match event {
            RoomEvent::TrackSubscribed {
                track: RemoteTrack::Audio(audio_track),
                publication,
                participant,
            } => self.handle_new_audio_track(audio_track, publication, participant),
            _ => {}
        }
    }

    fn handle_new_audio_track(
        &mut self,
        audio_track: RemoteAudioTrack,
        publication: RemoteTrackPublication,
        participant: RemoteParticipant,
    ) {
        let language = self.language.clone();
        let whisper_context = self.app_state.whisper_context.clone();
        let transcription = self.transcription.clone();

        let Ok(participant_id) = ParticipantId::from_str(participant.identity().as_str()) else {
            log::warn!(
                "Got new audio track from invalid participant identity {}",
                participant.identity()
            );
            return;
        };

        let send_messages_tx = self.segments_tx.clone();

        tokio::spawn({
            let publication = publication.clone();
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
                                &publication,
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

async fn on_new_frame(
    whisper_context: &Arc<WhisperContext>,
    transcription: &Mutex<HashMap<TrackSid, TranscriptionContext>>,
    language: Option<String>,
    samples: Vec<f32>,
    publication: &RemoteTrackPublication,
    silence_buffer: &mut SilenceBuffer,
    send_messages_tx: &mpsc::Sender<TranscriptionSegment>,
    participant_id: &ParticipantId,
) {
    let mut whisper_guard = transcription.lock().await;
    let whisper_entry = whisper_guard
        .entry(publication.sid())
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
            track_id: publication.sid().to_string(),
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
}

impl TranscriptionContext {
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

        let num_segments = self.state.full_n_segments()?;
        for i in 0..num_segments {
            let segment = self.state.full_get_segment_text(i)?;
            segments.push(segment);
        }

        self.samples.clear();

        Ok(segments.join("").trim().to_owned())
    }
}
