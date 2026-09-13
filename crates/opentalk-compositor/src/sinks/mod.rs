// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{fmt::Debug, future::Future, pin::Pin};

use anyhow::Result;
use ezk::Frame;
use ezk_audio::RawAudio;
use futures::future::BoxFuture;
use serde::Deserialize;

#[cfg(feature = "gstreamer")]
mod rtmp;
#[cfg(feature = "gstreamer")]
mod system;
#[cfg(feature = "gstreamer")]
mod webm;

#[cfg(feature = "gstreamer")]
pub use rtmp::*;
#[cfg(feature = "gstreamer")]
pub use system::*;
#[cfg(feature = "gstreamer")]
pub use webm::*;

#[derive(Debug, Clone, Deserialize)]
pub enum EncoderType {
    CPU,
    VAAPI,
}

// TODO: This will be fixed later on
#[allow(clippy::missing_errors_doc)]
pub trait Sink: Send + Debug {
    fn on_audio_frame(&mut self, frame: Frame<RawAudio>) -> BoxFuture<'_, Result<()>>;

    fn on_video_frame(&mut self, buffer: &[u8]) -> Result<()>;

    fn close(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}
