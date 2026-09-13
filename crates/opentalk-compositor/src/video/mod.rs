// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    cmp::Reverse,
    collections::HashMap,
    future::poll_fn,
    pin::Pin,
    sync::Arc,
    task::Poll,
    time::{Duration, Instant},
};

use std::sync::Mutex as StdMutex;

use anyhow::{Context, Result};
use chrono::Local;
use ezk_image::{
    resize::{FilterType, ResizeAlg, Resizer},
    Cropped, Image, ImageRef, PixelFormat, Window,
};
use futures::{stream::SelectAll, FutureExt, Stream, StreamExt};
use image::DynamicImage;
use livekit::{
    id::{ParticipantIdentity, TrackSid},
    track::{RemoteAudioTrack, RemoteVideoTrack, TrackSource},
    webrtc::video_frame::{I420Buffer, VideoBuffer},
};
use tokio::{
    sync::{mpsc, watch, Mutex},
    task::JoinHandle,
    time::{interval_at, Interval, MissedTickBehavior},
};

use crate::{
    font::{DrawText, SimpleText, TextBox},
    image::{blend_yuv, I420BufferImageRef, I420Image, Point},
    Participant, Shared, Sink, SpeakingState, WantShutdown, BORDER, HEIGHT, I420_COLOR, OFFSET_TOP,
    PADDING, WIDTH,
};

pub(crate) mod placeholder;
pub(crate) mod svg;

pub(crate) type FrameInfo = (ParticipantIdentity, TrackSid, I420Buffer);

pub(crate) type VideoStream = Pin<Box<dyn Stream<Item = FrameInfo> + Send>>;

pub(crate) type NewVideoStream = (ParticipantIdentity, RemoteVideoTrack, VideoStream);

pub(crate) enum VideoStreamCommand {
    AddVideoTrack(NewVideoStream),
    AddAudioTrack((ParticipantIdentity, RemoteAudioTrack)),
    RemoveParticipant(ParticipantIdentity),
    RemoveTrack(TrackSid),
    Mute(TrackSid),
    Unmute(TrackSid),
    SetTargetFps(u16),
}

pub(crate) struct VideoPipeline {
    pub(crate) base_image: Vec<u8>,
    pub(crate) resizer: Resizer,
    pub(crate) resize_staging_buffer: Image<Vec<u8>>,

    // Participant placerholder image when there's no video frames to display
    placeholder_image: I420Buffer,

    mic_off_image: Image<Vec<u8>>,
    cam_off_image: Image<Vec<u8>>,

    pub(crate) sinks: Arc<Mutex<HashMap<String, Box<dyn Sink>>>>,
    pub(crate) shared: Arc<StdMutex<Shared>>,

    pub(crate) commands_rx: mpsc::UnboundedReceiver<VideoStreamCommand>,
    pub(crate) video_sources: SelectAll<VideoStream>,
    pub(crate) video_frames: HashMap<TrackSid, I420Buffer>,
    pub(crate) tracks: HashMap<TrackSid, TrackData>,

    pub(crate) target_fps: u16,
}

#[derive(Debug, Clone)]
pub(crate) struct TrackData {
    participant_identity: ParticipantIdentity,
    source: TrackSource,
    is_muted: bool,
}

impl VideoPipeline {
    pub(crate) fn create(
        sinks: Arc<Mutex<HashMap<String, Box<dyn Sink>>>>,
        shared: Arc<StdMutex<Shared>>,
        shutdown_channel: watch::Receiver<WantShutdown>,
        target_fps: u16,
    ) -> Result<(mpsc::UnboundedSender<VideoStreamCommand>, JoinHandle<()>)> {
        let background_image =
            image::load_from_memory(include_bytes!("../../assets/background.png")).unwrap();
        let logo_image =
            image::load_from_memory(include_bytes!("../../assets/logo_gradient.png")).unwrap();

        // Convert the background image
        let background_image = if background_image.width() != WIDTH as u32
            || background_image.height() != HEIGHT as u32
        {
            background_image
                .resize(
                    WIDTH as u32,
                    HEIGHT as u32,
                    image::imageops::FilterType::Triangle,
                )
                .to_rgb8()
                .into_raw()
        } else {
            background_image.to_rgb8().into_raw()
        };

        let background_image = Image::from_buffer(
            PixelFormat::RGB,
            &background_image[..],
            None,
            WIDTH,
            HEIGHT,
            I420_COLOR,
        )
        .context("Failed to create background_image")?;

        let mut base_image = vec![0u8; PixelFormat::I420.buffer_size(WIDTH, HEIGHT)];
        ezk_image::convert_multi_thread(
            &background_image,
            &mut Image::from_buffer(
                PixelFormat::I420,
                &mut base_image[..],
                None,
                WIDTH,
                HEIGHT,
                I420_COLOR,
            )
            .context("Failed to create base image")?,
        )?;

        // Render the logo once into the base image
        let mut base_image_i420 = I420Image::try_from(&mut base_image, Point::new(WIDTH, HEIGHT))?;
        render_image(
            logo_image.width() as usize,
            logo_image.height() as usize,
            PADDING,
            PADDING,
            &logo_image,
            &mut base_image_i420,
        );

        let (video_streams_tx, video_streams_rx) = mpsc::unbounded_channel();
        let task = tokio::spawn(
            VideoPipeline {
                base_image,
                resizer: Resizer::new(ResizeAlg::Interpolation(FilterType::Bilinear)),
                resize_staging_buffer: Image::blank(PixelFormat::I420, WIDTH, HEIGHT, I420_COLOR),
                placeholder_image: placeholder::load_placeholder_image()?,
                mic_off_image: svg::load(include_bytes!("../../assets/mic-off.svg"))?,
                cam_off_image: svg::load(include_bytes!("../../assets/camera-off.svg"))?,
                sinks,
                shared,
                commands_rx: video_streams_rx,
                video_sources: SelectAll::default(),
                video_frames: HashMap::default(),
                tracks: HashMap::default(),
                target_fps,
            }
            .run(shutdown_channel),
        );

        Ok((video_streams_tx, task))
    }

    fn handle_new_video_frame(&mut self, rdy: FrameInfo) {
        let (participant_sid, track_sid, video_frame) = rdy;

        let participant_exists = self
            .shared
            .lock()
            .unwrap()
            .participants
            .contains_key(&participant_sid);
        let Some(track_data) = self.tracks.get(&track_sid) else {
            return;
        };
        if !participant_exists || track_data.is_muted {
            return;
        }

        self.video_frames.insert(track_sid, video_frame);
    }

    pub(crate) async fn run(mut self, mut shutdown_channel: watch::Receiver<WantShutdown>) {
        let mut frame_counter = 0u64;
        let mut now = Instant::now();
        let target_frame_interval = 1000 / self.target_fps;
        let mut rerender_interval =
            tokio::time::interval(Duration::from_millis(target_frame_interval.into()));
        rerender_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut last_frame = now;

        loop {
            tokio::select! {
                () = shutdown_channel.wait_for(|v| matches!(v, WantShutdown::Yes)).map(|_| ()) => {
                    log::debug!("Shutdown received for VideoPipeline");
                    return;
                }
                current_frame = rerender_interval.tick() => {
                    if !self.shared.lock().unwrap().render_frames {
                        continue;
                    }

                    // drain video sources to ensure we are up to date
                    // before rerendering
                    poll_fn(|cx| loop {
                        match self.video_sources.poll_next_unpin(cx) {
                            Poll::Ready(Some(rdy)) => self.handle_new_video_frame(rdy),
                            Poll::Ready(None) | Poll::Pending => return Poll::Ready(()),
                        }
                    })
                    .await;


                    frame_counter += 1;
                    last_frame = current_frame.into();

                    // Move self into a blocking threadpool to avoid locking up the tokio runtime while compositing the video

                    self = tokio::task::spawn_blocking(move || {
                        if let Err(err) = self.rerender_frame() {
                            log::error!("Rerender frame failed: {err:?}");
                        }

                        self
                    }).await.expect("unable to spawn rerender_frame task");
                }
                Some(command) = self.commands_rx.recv() => {
                    self.handle_command(command, last_frame, &mut rerender_interval);
                }
                Some(frame_info) = self.video_sources.next() => self.handle_new_video_frame(frame_info),
            }
            let as_secs = now.elapsed().as_secs_f64();
            if as_secs >= 1.0 {
                log::trace!("FPS: {}", (frame_counter as f64 / as_secs));
                now = Instant::now();
                frame_counter = 0;
            }
        }
    }
    fn handle_command(
        &mut self,
        command: VideoStreamCommand,
        last_frame: Instant,
        rerender_interval: &mut Interval,
    ) {
        match command {
            VideoStreamCommand::AddVideoTrack((participant_identity, video_track, stream)) => {
                self.tracks.insert(
                    video_track.sid(),
                    TrackData {
                        participant_identity,
                        source: video_track.source(),
                        is_muted: video_track.is_muted(),
                    },
                );
                self.video_sources.push(stream);
            }
            VideoStreamCommand::AddAudioTrack((participant_identity, audio_track)) => {
                self.tracks.insert(
                    audio_track.sid(),
                    TrackData {
                        participant_identity,
                        source: audio_track.source(),
                        is_muted: audio_track.is_muted(),
                    },
                );
            }
            VideoStreamCommand::RemoveParticipant(participant_identity) => {
                self.shared
                    .lock()
                    .unwrap()
                    .participants
                    .remove(&participant_identity);
                let tracks = self.tracks.clone().into_iter().filter(|(_, track_data)| {
                    track_data.participant_identity == participant_identity
                });

                for (track_sid, _) in tracks {
                    self.video_frames.remove(&track_sid);
                    self.tracks.remove(&track_sid);
                }
            }
            VideoStreamCommand::RemoveTrack(track_sid) => {
                self.video_frames.remove(&track_sid);
                self.tracks.remove(&track_sid);
            }
            VideoStreamCommand::Mute(track_sid) => {
                if let Some(track) = self.tracks.get_mut(&track_sid) {
                    track.is_muted = true;
                }
                self.video_frames.remove(&track_sid);
            }
            VideoStreamCommand::Unmute(track_sid) => {
                if let Some(track) = self.tracks.get_mut(&track_sid) {
                    track.is_muted = false;
                }
            }
            VideoStreamCommand::SetTargetFps(target_fps) => {
                self.target_fps = target_fps;
                let delta = Duration::from_secs_f64(1. / f64::from(self.target_fps));
                *rerender_interval = interval_at((last_frame + delta).into(), delta);
                rerender_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            }
        }
    }

    #[allow(clippy::many_single_char_names, clippy::too_many_lines)]
    fn rerender_frame(&mut self) -> Result<()> {
        let mut base_image_buf = self.base_image.clone();
        let mut base_image = I420Image::try_from(&mut base_image_buf, Point::new(WIDTH, HEIGHT))?;

        let shared = self.shared.lock().unwrap();

        let clock_format = shared.clock_format.clone();
        let event_title = shared.event_title.clone();
        let participants_to_show = get_participants_to_show(
            &self.tracks,
            &self.video_frames,
            &shared.participants,
            &shared.speakers,
        );

        drop(shared);

        // ==== Render Event Title ====

        if let Some(event_title) = &event_title {
            let event_title_text = SimpleText::new(32.0, event_title);
            event_title_text.draw(
                Point::new(
                    (WIDTH - event_title_text.width() as usize) / 2,
                    OFFSET_TOP - event_title_text.height() as usize / 2,
                ),
                &mut base_image,
            );
        }

        // ==== Render Datetime ====

        let text = &Local::now().format(&clock_format.0).to_string();
        let date_time_text = SimpleText::new(32.0, text);

        date_time_text.draw(
            Point::new(
                WIDTH - date_time_text.width() as usize - PADDING,
                OFFSET_TOP - date_time_text.height() as usize / 2,
            ),
            &mut base_image,
        );

        // ==== Render All Participants  ====

        let participants_to_show_len = participants_to_show.len();
        for (pos, participant_to_show) in participants_to_show.into_iter().take(8).enumerate() {
            let i420_video = if let Some((i420_video, _)) = participant_to_show.i420_video {
                i420_video
            } else if let Some(avatar) = &participant_to_show.participant.avatar {
                avatar
            } else {
                &self.placeholder_image
            };

            // Resize image to fit
            let mut window = calculate_speaker_view(
                pos,
                participants_to_show_len.min(8),
                WIDTH,
                HEIGHT,
                PADDING,
            );

            let original_aspect_ratio = i420_video.width() as f32 / i420_video.height() as f32;

            let w = (window.width as f32).min(window.height as f32 / (1. / original_aspect_ratio));
            let h = (window.height as f32).min(window.width as f32 / original_aspect_ratio);

            window.x += ((window.width as f32 - w) / 2.) as usize;
            window.y += ((window.height as f32 - h) / 2.) as usize;

            window.width = make_even(w as usize);
            window.height = make_even(h as usize);

            let mut resize_staging = Cropped::new(
                &mut self.resize_staging_buffer,
                Window {
                    x: 0,
                    y: 0,
                    width: window.width,
                    height: window.height,
                },
            )?;

            self.resizer
                .resize(&I420BufferImageRef(i420_video), &mut resize_staging)?;

            // ====== Render speaker overlay ======
            let overlay = Window {
                x: window.x.saturating_sub(BORDER),
                y: window.y.saturating_sub(BORDER),
                width: window.width + BORDER,
                height: window.height + BORDER,
            };

            let col = if participant_to_show.is_speaking {
                YuvColor::rgb_to_yuv(209, 229, 69)
            } else {
                YuvColor::rgb_to_yuv(32, 67, 79)
            };

            horizontal_line(&mut base_image, overlay, &col, BORDER, 0);
            horizontal_line(&mut base_image, overlay, &col, BORDER, overlay.height);

            vertical_line(&mut base_image, overlay, &col, WIDTH, BORDER, 0);
            vertical_line(&mut base_image, overlay, &col, WIDTH, BORDER, overlay.width);

            // Copy image into buffer
            ezk_image::copy(&resize_staging, &mut Cropped::new(&mut base_image, window)?)?;

            // ==== Render Participant Name ====

            let simple_text = SimpleText::new(24.0, &participant_to_show.participant.display_name);
            let text_box = TextBox::new(simple_text);
            text_box.draw(
                Point::new(
                    window.x + (window.width / 2) - text_box.width() as usize / 2,
                    window.y + window.height - text_box.height() as usize,
                ),
                &mut base_image,
            );

            // Render mute icons
            if let Some((_, track_source)) = participant_to_show.i420_video {
                let source = match track_source {
                    TrackSource::Camera => TrackSource::Microphone,
                    TrackSource::Screenshare => {
                        let participant_has_camera = self.tracks.values().any(|t| {
                            t.participant_identity == participant_to_show.participant_identity
                                && t.source == TrackSource::Camera
                                && !t.is_muted
                        });

                        // TODO: if we ever use screenshare audio we can return TrackSource::ScreenShare audio here instead
                        if participant_has_camera {
                            continue;
                        }

                        TrackSource::Microphone
                    }
                    _ => continue,
                };

                let participant_has_audio = self.tracks.values().any(|t| {
                    t.participant_identity == participant_to_show.participant_identity
                        && t.source == source
                        && !t.is_muted
                });

                if !participant_has_audio {
                    let window = Window {
                        x: window.x + 10,
                        y: window.y + 10,
                        width: self.mic_off_image.width(),
                        height: self.mic_off_image.height(),
                    };

                    ezk_image::copy(
                        &self.mic_off_image,
                        &mut Cropped::new(&mut base_image, window)?,
                    )?;
                }
            } else {
                // We're not rendering a any video tracks for this participant, always render the camera off icon
                let window = Window {
                    x: window.x + 10,
                    y: window.y + 10,
                    width: self.cam_off_image.width(),
                    height: self.cam_off_image.height(),
                };

                ezk_image::copy(
                    &self.cam_off_image,
                    &mut Cropped::new(&mut base_image, window)?,
                )?;

                // If there's not any audio for this participant of any kind - render the mic off icon on the current tile
                let has_any_audio = self.tracks.values().any(|track_data| {
                    track_data.participant_identity == participant_to_show.participant_identity
                        && (track_data.source == TrackSource::Microphone
                            || track_data.source == TrackSource::ScreenshareAudio)
                        && !track_data.is_muted
                });

                if !has_any_audio {
                    let window = Window {
                        x: window.x + 46,
                        y: window.y,
                        width: self.mic_off_image.width(),
                        height: self.mic_off_image.height(),
                    };

                    ezk_image::copy(
                        &self.mic_off_image,
                        &mut Cropped::new(&mut base_image, window)?,
                    )?;
                }
            }
        }

        // ==== push image into GStreamer pipeline ====

        for sink in self.sinks.blocking_lock().values_mut() {
            if let Err(err) = sink.on_video_frame(&base_image_buf) {
                log::error!("Unable to push video: {err:?}");
            }
        }

        Ok(())
    }
}

fn vertical_line(
    base_image_i420: &mut I420Image<'_>,
    overlay: Window,
    col: &YuvColor,
    width: usize,
    border: usize,
    x_offset: usize,
) {
    let stride = width / border;

    for y in base_image_i420
        .get_luma_range(overlay.x + x_offset, overlay.y, width * overlay.height)
        .chunks_mut(border)
        .step_by(stride)
    {
        y.fill(col.y as u8);
    }
    for u in base_image_i420
        .get_chroma_u_range(overlay.x + x_offset, overlay.y, width * overlay.height / 2)
        .chunks_mut(border / 2)
        .step_by(stride)
    {
        u.fill(col.u as u8);
    }
    for v in base_image_i420
        .get_chroma_v_range(overlay.x + x_offset, overlay.y, width * overlay.height / 2)
        .chunks_mut(border / 2)
        .step_by(stride)
    {
        v.fill(col.v as u8);
    }
}

fn horizontal_line(
    base_image_i420: &mut I420Image<'_>,
    overlay: Window,
    col: &YuvColor,
    border: usize,
    y_offset: usize,
) {
    for i in 0..border {
        base_image_i420
            .get_luma_range(overlay.x, overlay.y + y_offset + i, overlay.width + border)
            .fill(col.y as u8);
    }
    for i in 0..border {
        base_image_i420
            .get_chroma_u_range(overlay.x, overlay.y + y_offset + i, overlay.width + border)
            .fill(col.u as u8);
    }
    for i in 0..border {
        base_image_i420
            .get_chroma_v_range(overlay.x, overlay.y + y_offset + i, overlay.width + border)
            .fill(col.v as u8);
    }
}

struct ParticipantToShow<'a> {
    participant: Participant,
    participant_identity: ParticipantIdentity,
    i420_video: Option<(&'a I420Buffer, TrackSource)>,
    is_speaking: bool,
}

fn get_participants_to_show<'a>(
    tracks: &'a HashMap<TrackSid, TrackData>,
    video_frames: &'a HashMap<TrackSid, I420Buffer>,
    participants: &HashMap<ParticipantIdentity, Participant>,
    speakers: &HashMap<ParticipantIdentity, SpeakingState>,
) -> Vec<ParticipantToShow<'a>> {
    let mut participant_sort_items = participants
        .iter()
        .map(|(identity, participant)| {
            let has_screenshare = tracks.values().any(|t| {
                t.participant_identity == *identity && t.source == TrackSource::Screenshare
            });

            let speaking_state = speakers.get(identity);

            (
                identity.clone(),
                participant.clone(),
                has_screenshare,
                speaking_state,
            )
        })
        .collect::<Vec<_>>();

    participant_sort_items.sort_by_key(|(_, _, has_screenshare, speaking_state)| {
        Reverse((
            *has_screenshare,
            speaking_state.map(|state| state.is_speaking),
            speaking_state.map(|state| state.last_event),
        ))
    });

    participant_sort_items
        .into_iter()
        .flat_map(
            |(identity, participant, _has_screenshare, speaking_state)| {
                let mut tracks = tracks
                    .iter()
                    .filter(|(_track_id, track_data)| {
                        track_data.participant_identity == identity
                            && (track_data.source == TrackSource::Camera
                                || track_data.source == TrackSource::Screenshare)
                            && !track_data.is_muted
                    })
                    .collect::<Vec<_>>();

                if tracks.is_empty() {
                    vec![ParticipantToShow {
                        participant,
                        participant_identity: identity,
                        i420_video: None,
                        is_speaking: speaking_state.is_some_and(|state| state.is_speaking),
                    }]
                } else {
                    tracks.sort_by_key(|(_, track_data)| Reverse(track_data.source as u32));

                    tracks
                        .into_iter()
                        .map(|(track_id, track_data)| ParticipantToShow {
                            participant: participant.clone(),
                            participant_identity: identity.clone(),
                            i420_video: video_frames.get(track_id).map(|f| (f, track_data.source)),
                            is_speaking: speaking_state.is_some_and(|state| state.is_speaking),
                        })
                        .collect::<Vec<_>>()
                }
            },
        )
        .collect()
}

#[derive(Debug)]
struct YuvColor {
    y: f32,
    u: f32,
    v: f32,
}

impl YuvColor {
    pub fn rgb_to_yuv(r: u8, g: u8, b: u8) -> Self {
        let (r, g, b) = (f32::from(r), f32::from(g), f32::from(b));

        Self {
            y: r * 0.2126 + g * 0.7152 + b * 0.0722,
            u: r * -0.114_572_1 + g * -0.385_427_9 + b * 0.5 + 128.,
            v: r * 0.5 + g * -0.454_152_9 + b * -0.045_847_09 + 128.,
        }
    }
}

#[allow(clippy::many_single_char_names)]
fn render_image(
    width: usize,
    height: usize,
    offset_x: usize,
    offset_y: usize,
    logo_image: &DynamicImage,
    base_image: &mut I420Image<'_>,
) {
    let logo_image = logo_image.to_rgba8();

    for y in 0..height {
        for x in 0..width {
            let [r, g, b, a] = logo_image.get_pixel(x as u32, y as u32).0;

            let as_yuv = YuvColor::rgb_to_yuv(r, g, b);

            let x = x + offset_x;
            let y = y + offset_y;

            let yu = base_image.get_luma(x, y);
            *yu = blend_yuv(*yu, f32::from(a) / 255., as_yuv.y);

            let u = base_image.get_chroma_u(x, y);
            *u = blend_yuv(*u, f32::from(a) / 255., as_yuv.u);

            let v = base_image.get_chroma_v(x, y);
            *v = blend_yuv(*v, f32::from(a) / 255., as_yuv.v);
        }
    }
}

fn calculate_speaker_view(
    pos: usize,
    max_visibles: usize,
    canvas_width: usize,
    canvas_height: usize,
    padding: usize,
) -> Window {
    assert!(pos < max_visibles,);

    if max_visibles <= 2 {
        let width = (canvas_width - (max_visibles + 1) * padding) / max_visibles;
        let height = canvas_height - OFFSET_TOP - 2 * padding;

        return Window {
            x: (pos + 1) * padding + pos * width,
            y: OFFSET_TOP + padding,
            width: make_even(width),
            height: make_even(height),
        };
    }

    let canvas_width = canvas_width as f32;
    let canvas_height = canvas_height as f32;
    let padding = padding as f32;

    let main_width = (canvas_width - 3. * padding) / 4. * 3.;
    let main_height = (canvas_height - OFFSET_TOP as f32 - 2. * padding) / 4. * 3.;

    if pos == 0 {
        return Window {
            x: padding as usize,
            y: OFFSET_TOP + padding as usize,
            // The first video takes 3/4 width all other take 1/4 width
            width: make_even(main_width as usize),
            height: make_even(main_height as usize),
        };
    }

    let height = (main_height - 2. * padding) / 3.;

    if let 1..=4 = pos {
        let pos = pos as f32;
        let x = main_width + 2. * padding;
        let y = OFFSET_TOP as f32 + (pos * padding) + ((pos - 1.) * height);

        return Window {
            x: x as usize,
            y: y as usize,
            width: make_even((main_width / 3.) as usize),
            height: make_even(height as usize),
        };
    }

    if let 5..=7 = pos {
        let pos = (pos - 4) as f32;
        let width = (main_width - 2. * padding) / 3.;

        let x = (pos - 1.) * width + pos * padding;
        let y = main_height + OFFSET_TOP as f32 + 2. * padding;

        return Window {
            x: x as usize,
            y: y as usize,
            width: make_even(width as usize),
            height: make_even(height as usize),
        };
    }

    unreachable!("speaker layout only supports a maximum of 8 positions")
}

fn make_even(i: usize) -> usize {
    i - (i & 1)
}
