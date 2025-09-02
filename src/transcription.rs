// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::HashMap, str::FromStr, sync::Arc};

use anyhow::Result;
use audio_nodes::AudioConvert;
use ezk::{Frame, Source, SourceEvent};
use ezk_audio::{Format, RawAudio, RawAudioConfigRange, SampleRate};
use livekit::{
    RoomEvent,
    id::TrackSid,
    prelude::{Participant, RemoteTrackPublication},
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
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperState};

use crate::{
    AppState,
    audio::{CHANNELS, NativeAudioStreamSource, SAMPLE_RATE},
};

pub(crate) struct TranscriptionSession {
    app_state: Arc<AppState>,

    run: bool,

    room: Room,
    language: Option<String>,

    speakers: Vec<Participant>,

    whisper: Arc<Mutex<HashMap<TrackSid, Whisper>>>,
    track_to_participant: Arc<Mutex<HashMap<TrackSid, ParticipantId>>>,

    segments_rx: mpsc::Receiver<TranscriptionSegment>,
    segments_tx: mpsc::Sender<TranscriptionSegment>,
}

impl TranscriptionSession {
    pub(crate) async fn create(
        app_state: Arc<AppState>,
        room_info: StartTranscriptionRequestBody,
    ) -> Result<TranscriptionSession> {
        let room = app_state
            .opentalk_client
            .connect_recorder(room_info.room_id, room_info.breakout_room, true)
            .await?
            .into();

        let (segments_tx, segments_rx) = mpsc::channel(1000);

        Ok(TranscriptionSession {
            app_state,
            run: true,
            room,
            language: room_info.language,
            speakers: Vec::new(),
            whisper: Arc::new(Mutex::new(HashMap::new())),
            track_to_participant: Arc::new(Mutex::new(HashMap::new())),
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
            RoomEvent::ParticipantConnected(remote_participant) => {
                let publications = remote_participant.track_publications();
                let mut track_to_participant = self.track_to_participant.lock().await;
                for (track, _) in publications.iter() {
                    track_to_participant.insert(
                        track.clone(),
                        ParticipantId::from_str(remote_participant.identity().as_str())
                            .expect("ParticipantIdentity is a ParticipantId"),
                    );
                }
            }
            RoomEvent::ParticipantDisconnected(remote_participant) => {
                let publications = remote_participant.track_publications();
                let mut track_to_participant = self.track_to_participant.lock().await;
                for (track, _) in publications.iter() {
                    track_to_participant.remove(track);
                }
            }
            RoomEvent::Connected {
                participants_with_tracks,
            } => {
                let mut track_to_participant = self.track_to_participant.lock().await;
                for (remote_participant, publications) in participants_with_tracks {
                    for publication in publications {
                        track_to_participant.insert(
                            publication.sid(),
                            ParticipantId::from_str(remote_participant.identity().as_str())
                                .expect("ParticipantIdentity is a ParticipantId"),
                        );
                    }
                }
            }
            RoomEvent::ActiveSpeakersChanged { speakers } => {
                self.handle_speaker_change(speakers);
            }
            RoomEvent::TrackSubscribed {
                track: RemoteTrack::Audio(audio_track),
                publication,
                participant: _,
            } => self.handle_new_audio_track(audio_track, publication),
            _ => {}
        }
    }

    fn handle_new_audio_track(
        &mut self,
        audio_track: RemoteAudioTrack,
        publication: RemoteTrackPublication,
    ) {
        let language = self.language.clone();
        let whisper_context = self.app_state.whisper_context.clone();
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
                        .or_insert_with(|| {
                            Whisper::new(whisper_context.clone(), language.clone()).unwrap()
                        })
                        .extend_audio_frame(frame);
                }
            }
        });
    }

    fn handle_speaker_change(&mut self, speakers: Vec<Participant>) {
        log::trace!("speaker changed: {speakers:?}");

        for old_speaker in &self.speakers {
            // Still speaking, skip
            if speakers.iter().any(|p| p.sid() == old_speaker.sid()) {
                continue;
            }

            // Stopped speaking, do ai stuff
            for track in old_speaker.track_publications().keys() {
                let track = track.clone();
                let whisper = self.whisper.clone();
                let send_messages_tx = self.segments_tx.clone();
                let track_to_participant = self.track_to_participant.clone();

                tokio::task::spawn_blocking(move || {
                    let mut whisper = whisper.blocking_lock();
                    let Some(whisper) = whisper.get_mut(&track) else {
                        return;
                    };

                    let track_to_participant = track_to_participant.blocking_lock();
                    let Some(participant_id) = track_to_participant.get(&track) else {
                        return;
                    };

                    if let Ok(data) = whisper.transcribe_segment() {
                        let segment = TranscriptionSegment {
                            participant_id: *participant_id,
                            track_id: track.to_string(),
                            starts_at: whisper.start_point,
                            ends_at: Timestamp::now(),
                            text: data,
                        };

                        send_messages_tx.blocking_send(segment).unwrap();
                    }
                });
            }
        }

        self.speakers = speakers;
    }
}

struct Whisper {
    state: WhisperState,
    language: Option<String>,
    audio_samples: Vec<f32>,
    pub(crate) start_point: Timestamp,
}

impl Whisper {
    fn new(context: Arc<WhisperContext>, language: Option<String>) -> Result<Self> {
        let state = context.create_state()?;
        let audio_samples = Vec::new();

        Ok(Whisper {
            state,
            language,
            audio_samples,
            start_point: Timestamp::now(),
        })
    }
}

impl Whisper {
    fn extend_audio_frame(&mut self, frame: Frame<RawAudio>) {
        let samples = frame.into_data().samples.into_samples::<f32>();
        self.audio_samples.extend(samples);
    }

    fn transcribe_segment(&mut self) -> Result<String> {
        let sample_duration =
            self.audio_samples.len() as f32 / SAMPLE_RATE as f32 / CHANNELS as f32;

        if sample_duration < 1.0 {
            self.audio_samples
                .resize((SAMPLE_RATE * CHANNELS * 2) as usize, 0.0);
        }

        let mut segments = Vec::new();

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        params.set_language(self.language.as_deref());

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

        self.audio_samples.clear();

        Ok(segments.join("").trim().to_owned())
    }
}
